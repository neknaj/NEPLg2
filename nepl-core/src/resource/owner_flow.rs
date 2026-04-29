use crate::span::Span;
use alloc::string::String;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{AggregateKind, OwnerState, Place, PlaceProjection, ResourceCallTarget};
use super::owner_alias::{aliased_owner_descendant_entries, resolve_owner_alias_place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_address::{raw_address_return_ownership, RawAddressReturnOwnership};
use super::owner_state::OwnerTable;
use super::owner_transfer::{free_owner_state, move_owner_state_out, transfer_owner_state};
use super::place_utils::{
    construct_aggregate_field_place, place_with_suffix, places_overlap, replace_place_prefix,
    should_track,
};
use super::report::{ResourceOwnerDiagnostic, ResourceOwnerOperation};
use super::storage_origin::StorageOriginTable;
use super::summary::{OwnerProjectionReturnSummary, OwnerProjectionSource, OwnerReturnSummary};

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn construct_owner_fields(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
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
                storage_origins,
                input,
                &field,
                ResourceOwnerOperation::ConstructInput,
                span,
            );
            if matches!(kind, AggregateKind::Enum { .. }) && owners.state(&field).is_none() {
                owners.set_state(&field, OwnerState::NoFreeObligation);
            }
        }
    }

    pub(super) fn apply_call_return_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
        span: Span,
    ) {
        let ResourceCallTarget::User { name, .. } = target else {
            return;
        };
        match raw_address_return_ownership(name) {
            Some(RawAddressReturnOwnership::NonOwningAddressView) => return,
            None => {}
        }
        let Some(summary) = self
            .summaries
            .iter()
            .find(|summary| summary.function == name.as_str())
        else {
            return;
        };
        self.apply_owner_return_summary(
            owners,
            raw_aliases,
            storage_origins,
            output,
            args,
            summary,
            span,
        );
    }

    pub(super) fn apply_indirect_call_return_owner(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
        span: Span,
    ) {
        let functions = function_aliases.functions(callee);
        if functions.is_empty() {
            self.apply_unknown_indirect_call_return_owner(
                owners,
                raw_aliases,
                storage_origins,
                output,
                args,
                span,
            );
            return;
        }
        for function in functions {
            if let Some(summary) = self
                .summaries
                .iter()
                .find(|summary| summary.function == function.as_str())
            {
                self.apply_owner_return_summary(
                    owners,
                    raw_aliases,
                    storage_origins,
                    output,
                    args,
                    summary,
                    span,
                );
                if self.has_transferable_owner(owners, raw_aliases, output) {
                    return;
                }
            }
        }
    }

    fn apply_unknown_indirect_call_return_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        let mut returned_index = None;
        for (index, arg) in args
            .iter()
            .enumerate()
            .filter(|(_, arg)| arg.ty == output.ty)
        {
            if self.has_transferable_owner(owners, raw_aliases, arg) {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    arg,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                returned_index = Some(index);
                break;
            }
        }
        for (index, arg) in args.iter().enumerate() {
            if returned_index == Some(index) {
                continue;
            }
            self.consume_call_argument_owner(owners, raw_aliases, storage_origins, arg, span);
        }
    }

    fn apply_owner_return_summary(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        output: &Place,
        args: &[Place],
        summary: &OwnerReturnSummary,
        span: Span,
    ) {
        let mut transferred = false;
        for arg in summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
        {
            if self.has_returnable_parameter_owner(owners, raw_aliases, arg) {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    arg,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                transferred = true;
                break;
            }
        }
        for source in &summary.parameter_sources {
            if transferred {
                break;
            }
            let Some(source_place) = owner_projection_source_place(args, source) else {
                continue;
            };
            if self.has_returnable_parameter_owner(owners, raw_aliases, &source_place) {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    &source_place,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                transferred = true;
            }
        }
        if summary.returns_maybe_owner && !transferred {
            owners.set_state(output, OwnerState::MaybeFreed { storage: None });
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        } else if summary.returns_fresh_owner && !transferred {
            owners.allocate(output);
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        }
        for projection in &summary.projection_returns {
            let output_projection = place_with_suffix(output, &projection.suffix, projection.ty);
            self.apply_owner_projection_return_summary(
                owners,
                raw_aliases,
                storage_origins,
                &output_projection,
                args,
                projection,
                span,
            );
        }
        for marker in &summary.projection_markers {
            let marker_place = place_with_suffix(output, &marker.suffix, marker.ty);
            if owners.state(&marker_place).is_none() {
                owners.set_state(&marker_place, OwnerState::NoFreeObligation);
            }
        }
        self.consume_owner_summary_parameters(
            owners,
            raw_aliases,
            storage_origins,
            args,
            summary,
            span,
        );
    }

    fn apply_owner_projection_return_summary(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        output: &Place,
        args: &[Place],
        summary: &OwnerProjectionReturnSummary,
        span: Span,
    ) {
        let mut transferred = false;
        if owners.state(output).is_none() {
            owners.set_state(output, OwnerState::NoFreeObligation);
        }
        if self.has_transferable_owner(owners, raw_aliases, output) {
            return;
        }
        for arg in summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
        {
            if self.has_returnable_parameter_owner(owners, raw_aliases, arg) {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    arg,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                transferred = true;
                break;
            }
        }
        for source in &summary.parameter_sources {
            if transferred {
                break;
            }
            let Some(source_place) = owner_projection_source_place(args, source) else {
                continue;
            };
            if self.has_returnable_parameter_owner(owners, raw_aliases, &source_place) {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    &source_place,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                transferred = true;
            }
        }
        if summary.returns_maybe_owner && !transferred {
            owners.set_state(output, OwnerState::MaybeFreed { storage: None });
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        } else if summary.returns_fresh_owner && !transferred {
            owners.allocate(output);
            raw_aliases.mark(output);
            storage_origins.mark_owned(output);
        }
    }

    fn consume_owner_summary_parameters(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        args: &[Place],
        summary: &OwnerReturnSummary,
        span: Span,
    ) {
        for arg in summary
            .consumed_parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
        {
            self.consume_call_argument_owner(owners, raw_aliases, storage_origins, arg, span);
        }
        for source in &summary.consumed_parameter_sources {
            let Some(source_place) = owner_projection_source_place(args, source) else {
                continue;
            };
            self.consume_call_argument_owner(
                owners,
                raw_aliases,
                storage_origins,
                &source_place,
                span,
            );
        }
    }

    fn consume_call_argument_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        arg: &Place,
        span: Span,
    ) {
        if self.has_transferable_owner(owners, raw_aliases, arg) {
            self.move_owner_out(
                owners,
                raw_aliases,
                storage_origins,
                arg,
                ResourceOwnerOperation::CallArgument,
                span,
            );
        }
    }

    fn has_transferable_owner(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
    ) -> bool {
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        owners.has_transferable_owner(&resolved_place)
    }

    fn has_returnable_parameter_owner(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
    ) -> bool {
        !self.place_is_non_owning_raw_address_view(owners, raw_aliases, place)
            && self.has_transferable_owner(owners, raw_aliases, place)
    }

    fn place_is_non_owning_raw_address_view(
        &self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
    ) -> bool {
        self.types.resolve_id(place.ty) == self.types.i32()
            && !owners.has_transferable_owner(place)
            && !owners.has_tracked_state_under(place)
            && raw_aliases
                .aliases_for(place)
                .iter()
                .any(|alias| alias != place && place_has_raw_address_projection(alias))
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
        storage_origins: &mut StorageOriginTable,
        source: &Place,
        target: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) {
        if source == target || !should_track(source) {
            return;
        }
        let resolved_source = resolve_owner_alias_place(owners, raw_aliases, source);
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
            Some(OwnerState::Moved | OwnerState::Freed) => {
                let state = owners
                    .state(&resolved_source)
                    .unwrap_or(OwnerState::NoFreeObligation);
                self.push_unavailable(operation, &resolved_source, state, span);
            }
            Some(OwnerState::NoFreeObligation) | None => {}
        }
        for entry in descendants {
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
                OwnerState::Moved | OwnerState::Freed => {
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
                OwnerState::Moved | OwnerState::Freed => {
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
            Some(OwnerState::Moved | OwnerState::Freed) => {
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
                OwnerState::Moved | OwnerState::Freed => {
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
                OwnerState::Moved | OwnerState::Freed => {
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
        storage_origins: &mut StorageOriginTable,
        place: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        if !should_track(place) {
            return false;
        }
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        match owners.state(&resolved_place) {
            Some(OwnerState::Live { .. }) => {
                free_owner_state(owners, raw_aliases, storage_origins, &resolved_place);
                raw_aliases.clear(place);
                raw_aliases.clear(&resolved_place);
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

    fn storage_origin_expects_owned(
        &self,
        storage_origins: &StorageOriginTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
    ) -> bool {
        storage_origins.expects_owned(place)
            || raw_aliases
                .aliases_for(place)
                .iter()
                .any(|alias| storage_origins.expects_owned(alias))
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
                OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed => {}
            }
        }
    }

    fn push_unavailable(
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

fn owner_projection_source_place(args: &[Place], source: &OwnerProjectionSource) -> Option<Place> {
    let arg = args.get(source.parameter_index)?;
    Some(place_with_suffix(arg, &source.suffix, source.ty))
}

fn place_has_raw_address_projection(place: &Place) -> bool {
    place
        .projections
        .iter()
        .any(|projection| matches!(projection, PlaceProjection::StorageOffset(_)))
}
