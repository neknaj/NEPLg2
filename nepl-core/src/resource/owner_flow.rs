use alloc::string::String;

use crate::span::Span;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{AggregateKind, OwnerState, Place, ResourceCallTarget};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_state::OwnerTable;
use super::place_utils::{
    construct_aggregate_field_place, place_with_suffix, places_overlap, replace_place_prefix,
    should_track,
};
use super::report::{ResourceOwnerDiagnostic, ResourceOwnerOperation};
use super::storage_origin::StorageOriginTable;
use super::summary::{OwnerProjectionReturnSummary, OwnerReturnSummary};

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
                transferred = true;
                break;
            }
        }
        if summary.returns_fresh_owner && !transferred {
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
        for arg in summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
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
                transferred = true;
                break;
            }
        }
        if summary.returns_fresh_owner && !transferred {
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
                OwnerState::MaybeFreed => {
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
        match owners.state(&resolved_source) {
            Some(OwnerState::Live { storage }) => {
                owners.set_state(&resolved_source, OwnerState::Moved);
                if should_track(target) {
                    owners.set_state(target, OwnerState::Live { storage });
                    storage_origins.move_origin(&resolved_source, target);
                    raw_aliases.clear(source);
                    raw_aliases.clear(&resolved_source);
                    raw_aliases.mark(target);
                } else {
                    storage_origins.clear(&resolved_source);
                }
            }
            Some(OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed) => {
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
                OwnerState::Live { storage } => {
                    owners.set_state(&entry.place, OwnerState::Moved);
                    if should_track(&target_place) {
                        owners.set_state(&target_place, OwnerState::Live { storage });
                        storage_origins.move_origin(&entry.place, &target_place);
                        raw_aliases.clear(&entry.place);
                        raw_aliases.mark(&target_place);
                    } else {
                        storage_origins.clear(&entry.place);
                    }
                }
                OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed => {
                    self.push_unavailable(operation, &entry.place, entry.state, span);
                }
                OwnerState::NoFreeObligation => {}
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
        match owners.state(&resolved_place) {
            Some(OwnerState::Live { .. }) => {
                owners.set_state(&resolved_place, OwnerState::Moved);
                storage_origins.clear(&resolved_place);
                raw_aliases.clear(place);
                raw_aliases.clear(&resolved_place);
            }
            Some(OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed) => {
                let state = owners
                    .state(&resolved_place)
                    .unwrap_or(OwnerState::NoFreeObligation);
                self.push_unavailable(operation, &resolved_place, state, span);
            }
            Some(OwnerState::NoFreeObligation) | None => {}
        }
        for entry in descendants {
            match entry.state {
                OwnerState::Live { .. } => {
                    owners.set_state(&entry.place, OwnerState::Moved);
                    storage_origins.clear(&entry.place);
                    raw_aliases.clear(&entry.place);
                }
                OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed => {
                    self.push_unavailable(operation, &entry.place, entry.state, span);
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
                owners.set_state(&resolved_place, OwnerState::Freed);
                storage_origins.clear(&resolved_place);
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
                OwnerState::MaybeFreed => {
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

pub(super) fn resolve_owner_alias_place(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
) -> Place {
    match owners.state(place) {
        Some(OwnerState::Live { .. })
        | Some(OwnerState::Moved)
        | Some(OwnerState::Freed)
        | Some(OwnerState::MaybeFreed) => return place.clone(),
        Some(OwnerState::NoFreeObligation) | None => {}
    }
    for alias in raw_aliases.aliases_for(place) {
        match owners.state(&alias) {
            Some(OwnerState::Live { .. })
            | Some(OwnerState::Moved)
            | Some(OwnerState::Freed)
            | Some(OwnerState::MaybeFreed) => return alias,
            Some(OwnerState::NoFreeObligation) | None => {}
        }
    }
    place.clone()
}
