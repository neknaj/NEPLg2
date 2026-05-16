use alloc::string::String;

use crate::resource_primitives::compiler_memory_type_from_constructor_name;
use crate::source_map::CompilerMemoryType;
use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{AggregateKind, OwnerState, OwnerStorageExtent, Place};
use super::owner_alias::{aliased_owner_descendant_entries, resolve_owner_alias_place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_transfer::{free_owner_state, move_owner_state_out, transfer_owner_state};
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
            kind,
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
        if region_token_extent_ok && region_token_construct_kind(kind) && inputs.len() >= 2 {
            let raw = construct_aggregate_field_place(output, kind, 0, &inputs[0]);
            owners.set_live_extent(&raw, OwnerStorageExtent::RegionTokenSize);
        }
    }

    fn region_token_construct_extent_requirement_holds(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        kind: &AggregateKind,
        inputs: &[Place],
        span: Span,
    ) -> bool {
        if !region_token_construct_kind(kind) {
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

    pub(super) fn release_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        place: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        if !should_track(place) {
            return false;
        }
        if self.place_is_non_owning_raw_address_view(owners, raw_aliases, raw_views, place) {
            self.push_unavailable(operation, place, OwnerState::NoFreeObligation, span);
            return false;
        }
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        match owners.state(&resolved_place) {
            Some(OwnerState::Live { .. }) => {
                free_owner_state(owners, raw_aliases, storage_origins, &resolved_place);
                true
            }
            Some(state) => {
                self.push_unavailable(operation, &resolved_place, state, span);
                false
            }
            None => {
                if self.storage_origin_expects_owned(storage_origins, raw_aliases, place) {
                    self.push_unavailable(operation, place, OwnerState::NoFreeObligation, span);
                }
                false
            }
        }
    }

    pub(super) fn release_owner_with_extent(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        place: &Place,
        expected_extent: &Place,
        operation: ResourceOwnerOperation,
        extent_operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        if !self.ensure_owner_extent_matches_argument(
            owners,
            raw_aliases,
            place,
            expected_extent,
            extent_operation,
            span,
        ) {
            self.push_extent_unavailable(owners, raw_aliases, place, extent_operation, span);
            return false;
        }
        self.release_owner(
            owners,
            raw_aliases,
            raw_views,
            storage_origins,
            place,
            operation,
            span,
        )
    }

    pub(super) fn ensure_owner_available_with_extent(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &StorageOriginTable,
        place: &Place,
        expected_extent: &Place,
        operation: ResourceOwnerOperation,
        extent_operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        if !self.ensure_owner_extent_matches_argument(
            owners,
            raw_aliases,
            place,
            expected_extent,
            extent_operation,
            span,
        ) {
            self.push_extent_unavailable(owners, raw_aliases, place, extent_operation, span);
            return false;
        }
        self.ensure_owner_available(
            owners,
            raw_aliases,
            raw_views,
            storage_origins,
            place,
            operation,
            span,
        )
    }

    pub(super) fn ensure_owner_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &StorageOriginTable,
        place: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        if !should_track(place) {
            return false;
        }
        if self.place_is_non_owning_raw_address_view(owners, raw_aliases, raw_views, place) {
            self.push_unavailable(operation, place, OwnerState::NoFreeObligation, span);
            return false;
        }
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        match owners.state(&resolved_place) {
            Some(OwnerState::Live { .. }) => true,
            Some(state) => {
                self.push_unavailable(operation, &resolved_place, state, span);
                false
            }
            None => {
                if self.storage_origin_expects_owned(storage_origins, raw_aliases, place) {
                    self.push_unavailable(operation, place, OwnerState::NoFreeObligation, span);
                }
                false
            }
        }
    }

    pub(super) fn storage_origin_expects_owned(
        &self,
        storage_origins: &StorageOriginTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
    ) -> bool {
        storage_origins.expects_owned(place)
            || storage_origins.expects_owned_under(place)
            || raw_aliases.aliases_for(place).iter().any(|alias| {
                storage_origins.expects_owned(alias) || storage_origins.expects_owned_under(alias)
            })
    }

    pub(super) fn push_live_owner_diagnostics(&mut self, owners: &OwnerTable, span: Span) {
        for entry in owners.live_entries() {
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

fn region_token_construct_kind(kind: &AggregateKind) -> bool {
    matches!(
        kind,
        AggregateKind::Struct { name, .. }
            if compiler_memory_type_from_constructor_name(name)
                == Some(CompilerMemoryType::OwnerToken)
    )
}
