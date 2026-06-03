use alloc::string::String;
use alloc::vec::Vec;

use crate::resource_primitives::type_is_owner_token;
use crate::span::Span;
use crate::types::TypeId;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{AggregateKind, OwnerState, OwnerStorageExtent, Place, PlaceProjection};
use super::owner_alias::{aliased_owner_descendant_entries, resolve_owner_alias_place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_transfer::{move_owner_state_out, transfer_owner_state};
use super::place_utils::{
    construct_aggregate_field_place, place_suffix_after_prefix, place_with_suffix, places_overlap,
    replace_place_prefix, should_track,
};
use super::report::{ResourceOwnerDiagnostic, ResourceOwnerOperation};
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn construct_owner_fields(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        output: &Place,
        kind: &AggregateKind,
        inputs: &[Place],
        span: Span,
    ) {
        let region_token_extent_ok = self.region_token_construct_extent_requirement_holds(
            owners,
            raw_aliases,
            output,
            inputs,
            span,
        );
        for (index, input) in inputs.iter().enumerate() {
            let field = construct_aggregate_field_place(output, kind, index, input);
            raw_aliases.copy_scalar_facts_if_tracked(input, &field);
            self.transfer_owner(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                input,
                &field,
                ResourceOwnerOperation::ConstructInput,
                span,
            );
            raw_views.copy_non_owning(input, &field);
            if matches!(kind, AggregateKind::Enum { .. }) && owners.state(&field).is_none() {
                owners.set_state(&field, OwnerState::NoFreeObligation);
            }
        }
        if region_token_extent_ok && self.region_token_construct_kind(output) && inputs.len() >= 2 {
            let raw = construct_aggregate_field_place(output, kind, 0, &inputs[0]);
            owners.set_live_extent(&raw, OwnerStorageExtent::RegionTokenSize);
        }
    }

    fn region_token_construct_extent_requirement_holds(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        inputs: &[Place],
        span: Span,
    ) -> bool {
        if !self.region_token_construct_kind(output) {
            return true;
        }
        let [raw, size, ..] = inputs else {
            return true;
        };
        if self.ensure_owner_extent_matches_argument(
            owners,
            raw_aliases,
            raw,
            size,
            ResourceOwnerOperation::ConstructInput,
            span,
        ) {
            true
        } else {
            self.push_extent_unavailable(
                owners,
                raw_aliases,
                raw,
                ResourceOwnerOperation::ConstructInput,
                span,
            );
            false
        }
    }

    fn region_token_construct_kind(&self, output: &Place) -> bool {
        type_is_owner_token(self.types, output.ty)
    }

    pub(super) fn report_overwritten_owners(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        target: &Place,
        value: &Place,
        span: Span,
    ) {
        for entry in owners.live_entries_under(target) {
            if places_overlap(&entry.place, value) {
                continue;
            }
            if replacement_preserves_live_storage(owners, raw_aliases, target, value, &entry) {
                continue;
            }
            match entry.state {
                OwnerState::Live { storage, .. } => {
                    self.diagnostics.push(ResourceOwnerDiagnostic::OwnerLeaked {
                        function: String::from(self.function),
                        place: entry.place.clone(),
                        storage,
                        span,
                    });
                }
                OwnerState::MaybeFreed { .. } => {
                    self.diagnostics
                        .push(ResourceOwnerDiagnostic::OwnerMaybeLeaked {
                            function: String::from(self.function),
                            place: entry.place.clone(),
                            span,
                        });
                }
                OwnerState::Reserved { .. } => {
                    self.diagnostics
                        .push(ResourceOwnerDiagnostic::OwnerMaybeLeaked {
                            function: String::from(self.function),
                            place: entry.place.clone(),
                            span,
                        });
                }
                OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed => {}
            }
            owners.set_state(&entry.place, OwnerState::Moved);
            storage_origins.clear(&entry.place);
        }
    }

    pub(super) fn transfer_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        source: &Place,
        target: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) {
        self.transfer_owner_with_raw_view_policy(
            owners,
            raw_aliases,
            raw_views,
            storage_origins,
            source,
            target,
            operation,
            span,
            false,
        );
    }

    pub(super) fn transfer_owner_from_summary_effect(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        source: &Place,
        target: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) {
        self.transfer_owner_with_raw_view_policy(
            owners,
            raw_aliases,
            raw_views,
            storage_origins,
            source,
            target,
            operation,
            span,
            true,
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn transfer_owner_with_raw_view_policy(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        source: &Place,
        target: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
        resolve_non_owning_raw_views: bool,
    ) {
        if source == target || !should_track(source) {
            return;
        }
        let resolved_source = if !resolve_non_owning_raw_views && raw_views.contains_under(source) {
            source.clone()
        } else {
            resolve_owner_alias_place(owners, raw_aliases, source)
        };
        let descendants = owners.descendant_entries(&resolved_source);
        let aliased_descendants =
            aliased_owner_descendant_entries(owners, raw_aliases, &resolved_source);
        match owners.state(&resolved_source) {
            Some(state @ OwnerState::Live { .. }) | Some(state @ OwnerState::MaybeFreed { .. }) => {
                transfer_owner_state(
                    owners,
                    raw_aliases,
                    storage_origins,
                    &resolved_source,
                    source,
                    target,
                    state,
                );
            }
            Some(OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed) => {
                let state = owners
                    .state(&resolved_source)
                    .unwrap_or(OwnerState::NoFreeObligation);
                self.push_unavailable(operation, &resolved_source, state, span);
            }
            Some(OwnerState::NoFreeObligation) | None => {}
        }
        if matches!(
            owners.state(&resolved_source),
            Some(OwnerState::NoFreeObligation) | None
        ) {
            storage_origins.move_origin(source, target);
        }
        for entry in descendants {
            if raw_views.contains(&entry.place) {
                continue;
            }
            let Some(target_place) = replace_place_prefix(&entry.place, &resolved_source, target)
            else {
                continue;
            };
            match entry.state {
                state @ OwnerState::Live { .. } | state @ OwnerState::MaybeFreed { .. } => {
                    transfer_owner_state(
                        owners,
                        raw_aliases,
                        storage_origins,
                        &entry.place,
                        &entry.place,
                        &target_place,
                        state,
                    );
                }
                OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed => {
                    self.push_unavailable(operation, &entry.place, entry.state, span);
                }
                OwnerState::NoFreeObligation => {
                    if should_track(&target_place) {
                        owners.set_state(&target_place, OwnerState::NoFreeObligation);
                    }
                }
            }
        }
        for aliased in aliased_descendants {
            let source_place =
                place_with_suffix(&resolved_source, &aliased.suffix, aliased.entry.place.ty);
            if raw_views.contains(&source_place) {
                continue;
            }
            let target_place = place_with_suffix(target, &aliased.suffix, aliased.entry.place.ty);
            match aliased.entry.state {
                state @ OwnerState::Live { .. } | state @ OwnerState::MaybeFreed { .. } => {
                    transfer_owner_state(
                        owners,
                        raw_aliases,
                        storage_origins,
                        &aliased.entry.place,
                        &aliased.entry.place,
                        &target_place,
                        state,
                    );
                }
                OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed => {
                    self.push_unavailable(
                        operation,
                        &aliased.entry.place,
                        aliased.entry.state,
                        span,
                    );
                }
                OwnerState::NoFreeObligation => {
                    if should_track(&target_place) {
                        owners.set_state(&target_place, OwnerState::NoFreeObligation);
                    }
                }
            }
        }
    }

    pub(super) fn move_owner_out(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        place: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) {
        if !should_track(place) {
            return;
        }
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        let descendants = owners.descendant_entries(&resolved_place);
        let aliased_descendants =
            aliased_owner_descendant_entries(owners, raw_aliases, &resolved_place);
        match owners.state(&resolved_place) {
            Some(OwnerState::Live { .. } | OwnerState::MaybeFreed { .. }) => {
                move_owner_state_out(owners, raw_aliases, storage_origins, &resolved_place);
                raw_aliases.clear(place);
                raw_aliases.clear(&resolved_place);
            }
            Some(OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed) => {
                let state = owners
                    .state(&resolved_place)
                    .unwrap_or(OwnerState::NoFreeObligation);
                self.push_unavailable(operation, &resolved_place, state, span);
            }
            Some(OwnerState::NoFreeObligation) | None => {}
        }
        for entry in descendants {
            match entry.state {
                OwnerState::Live { .. } | OwnerState::MaybeFreed { .. } => {
                    move_owner_state_out(owners, raw_aliases, storage_origins, &entry.place);
                    raw_aliases.clear(&entry.place);
                }
                OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed => {
                    self.push_unavailable(operation, &entry.place, entry.state, span);
                }
                OwnerState::NoFreeObligation => {}
            }
        }
        for aliased in aliased_descendants {
            match aliased.entry.state {
                OwnerState::Live { .. } | OwnerState::MaybeFreed { .. } => {
                    move_owner_state_out(
                        owners,
                        raw_aliases,
                        storage_origins,
                        &aliased.entry.place,
                    );
                    raw_aliases.clear(&aliased.entry.place);
                }
                OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed => {
                    self.push_unavailable(
                        operation,
                        &aliased.entry.place,
                        aliased.entry.state,
                        span,
                    );
                }
                OwnerState::NoFreeObligation => {}
            }
        }
    }

    pub(super) fn move_return_storage_origin_owners_out(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        value: &Place,
        span: Span,
    ) {
        let mut sources = storage_origins
            .entries_under(value)
            .into_iter()
            .filter_map(|entry| {
                let source = storage_origins
                    .origin_source(&entry.place)
                    .unwrap_or(entry.place);
                (!places_overlap(&source, value)).then_some(source)
            })
            .collect::<Vec<_>>();
        sources.sort_by_key(|source| source.projections.len());

        let mut moved_sources: Vec<Place> = Vec::new();
        for source in sources {
            if moved_sources
                .iter()
                .any(|moved| places_overlap(moved, &source))
            {
                continue;
            }
            if !self.has_transferable_owner(owners, raw_aliases, &source) {
                continue;
            }
            self.move_owner_out(
                owners,
                raw_aliases,
                storage_origins,
                &source,
                ResourceOwnerOperation::ReturnValue,
                span,
            );
            moved_sources.push(source);
        }
    }

    pub(super) fn push_live_owner_diagnostics(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &StorageOriginTable,
        span: Span,
    ) {
        for entry in owners.live_entries() {
            if self.owner_escapes_through_parameter_raw_cell(
                raw_aliases,
                raw_views,
                storage_origins,
                &entry.place,
            ) {
                continue;
            }
            match entry.state {
                OwnerState::Live { storage, .. } => {
                    self.diagnostics.push(ResourceOwnerDiagnostic::OwnerLeaked {
                        function: String::from(self.function),
                        place: entry.place,
                        storage,
                        span,
                    });
                }
                OwnerState::MaybeFreed { .. } => {
                    self.diagnostics
                        .push(ResourceOwnerDiagnostic::OwnerMaybeLeaked {
                            function: String::from(self.function),
                            place: entry.place,
                            span,
                        });
                }
                OwnerState::Reserved { .. } => {
                    self.diagnostics
                        .push(ResourceOwnerDiagnostic::OwnerMaybeLeaked {
                            function: String::from(self.function),
                            place: entry.place,
                            span,
                        });
                }
                OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed => {}
            }
        }
    }

    fn owner_escapes_through_parameter_raw_cell(
        &self,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &StorageOriginTable,
        place: &Place,
    ) -> bool {
        let Some(address) = raw_cell_address_prefix(place, self.types.i32()) else {
            return false;
        };
        if raw_views.contains_non_owning(&address)
            || raw_views.contains_non_owning_projection(&address)
        {
            return true;
        }
        self.raw_address_reaches_parameter(raw_aliases, storage_origins, &address)
    }

    fn raw_address_reaches_parameter(
        &self,
        raw_aliases: &RawCellAddressAliases,
        storage_origins: &StorageOriginTable,
        address: &Place,
    ) -> bool {
        let mut candidates = raw_aliases.raw_address_aliases_for_value(address);
        if let Some(origin) = storage_origins.origin_source(address) {
            candidates.push(origin);
        }
        candidates
            .iter()
            .any(|candidate| self.place_reaches_parameter(candidate))
    }

    fn place_reaches_parameter(&self, place: &Place) -> bool {
        self.params.iter().any(|param| {
            place_suffix_after_prefix(place, &param.place).is_some()
                || place_suffix_after_prefix(&param.place, place).is_some()
        })
    }

    pub(super) fn push_unavailable(
        &mut self,
        operation: ResourceOwnerOperation,
        place: &Place,
        state: OwnerState,
        span: Span,
    ) {
        self.diagnostics
            .push(ResourceOwnerDiagnostic::OwnerUnavailable {
                function: String::from(self.function),
                operation,
                place: place.clone(),
                state,
                span,
            });
    }
}

fn raw_cell_address_prefix(place: &Place, raw_address_ty: TypeId) -> Option<Place> {
    let deref_index = place
        .projections
        .iter()
        .position(|projection| matches!(projection, PlaceProjection::Deref))?;
    let mut address = place.clone();
    address.projections.truncate(deref_index);
    address.ty = raw_address_ty;
    Some(address)
}

fn replacement_preserves_live_storage(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    target: &Place,
    value: &Place,
    overwritten: &super::model::OwnerStateEntry,
) -> bool {
    let Some(suffix) = place_suffix_after_prefix(&overwritten.place, target) else {
        return false;
    };
    let replacement = place_with_suffix(value, &suffix, overwritten.place.ty);
    let resolved_replacement = resolve_owner_alias_place(owners, raw_aliases, &replacement);
    matches!(
        (&overwritten.state, owners.state(&resolved_replacement)),
        (
            OwnerState::Live {
                storage: overwritten_storage,
                ..
            },
            Some(OwnerState::Live {
                storage: replacement_storage,
                ..
            }),
        ) if *overwritten_storage == replacement_storage
    )
}
