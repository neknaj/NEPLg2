use crate::span::Span;
use alloc::string::String;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{AggregateKind, OwnerState, Place};
use super::owner_alias::{aliased_owner_descendant_entries, resolve_owner_alias_place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_transfer::{free_owner_state, move_owner_state_out, transfer_owner_state};
use super::place_utils::{
    construct_aggregate_field_place, place_with_suffix, places_overlap, replace_place_prefix,
    should_track,
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
        for (index, input) in inputs.iter().enumerate() {
            let field = construct_aggregate_field_place(output, kind, index, input);
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
    }

    pub(super) fn report_overwritten_owners(
        &mut self,
        owners: &mut OwnerTable,
        storage_origins: &mut StorageOriginTable,
        target: &Place,
        value: &Place,
        span: Span,
    ) {
        for entry in owners.live_entries_under(target) {
            if places_overlap(&entry.place, value) {
                continue;
            }
            if replacement_preserves_live_storage(owners, &entry.state, value) {
                continue;
            }
            match entry.state {
                OwnerState::Live { storage } => {
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
        if source == target || !should_track(source) {
            return;
        }
        let resolved_source = if raw_views.contains_under(source) {
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
                OwnerState::Live { storage } => {
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
    overwritten: &OwnerState,
    value: &Place,
) -> bool {
    matches!(
        (overwritten, owners.state(value)),
        (
            OwnerState::Live {
                storage: overwritten_storage,
            },
            Some(OwnerState::Live {
                storage: replacement_storage,
            }),
        ) if *overwritten_storage == replacement_storage
    )
}
