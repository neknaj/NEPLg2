extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;

use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    AggregateKind, OwnerState, OwnerStateEntry, Place, RawMemoryOp, ResourceBlock,
    ResourceCallTarget, ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::owner_state::OwnerTable;
use super::place_utils::{
    construct_aggregate_field_place, place_with_suffix, places_overlap, replace_place_prefix,
    should_track,
};
use super::report::{
    ResourceOwnerCheckDeferred, ResourceOwnerCheckReport, ResourceOwnerDiagnostic,
    ResourceOwnerFunctionCheck, ResourceOwnerOperation,
};
use super::summary::{
    compute_owner_return_summaries, OwnerProjectionReturnSummary, OwnerReturnSummary,
};

pub fn check_resource_owner_obligations(module: &ResourceModule) -> ResourceOwnerCheckReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceOwnerCheckDeferred::default();
    let summaries = compute_owner_return_summaries(module);

    for function in &module.functions {
        let mut engine = ResourceOwnerCheckEngine {
            function: function.name.as_str(),
            summaries: &summaries,
            diagnostics: Vec::new(),
            deferred: ResourceOwnerCheckDeferred::default(),
        };
        let final_owners = engine.check_function(function);
        merge_owner_deferred(&mut deferred, engine.deferred);
        diagnostics.extend(engine.diagnostics);
        functions.push(ResourceOwnerFunctionCheck {
            name: function.name.clone(),
            final_owners,
            deferred: engine.deferred,
        });
    }

    ResourceOwnerCheckReport {
        functions,
        diagnostics,
        deferred,
    }
}

pub(super) struct ResourceOwnerCheckEngine<'a> {
    pub(super) function: &'a str,
    pub(super) summaries: &'a [OwnerReturnSummary],
    pub(super) diagnostics: Vec<ResourceOwnerDiagnostic>,
    pub(super) deferred: ResourceOwnerCheckDeferred,
}

impl ResourceOwnerCheckEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) -> Vec<OwnerStateEntry> {
        let mut owners = OwnerTable::default();
        let mut function_aliases = FunctionAliasTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        for block in &function.blocks {
            self.check_block(&mut owners, &mut function_aliases, &mut raw_aliases, block);
        }
        self.push_live_owner_diagnostics(&owners, function.span);
        owners.into_entries()
    }

    fn check_block(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        block: &ResourceBlock,
    ) {
        self.check_ops(owners, function_aliases, raw_aliases, &block.ops);
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if let Some(value) = value {
                    self.move_owner_out(
                        owners,
                        raw_aliases,
                        value,
                        ResourceOwnerOperation::ReturnValue,
                        *span,
                    );
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    pub(super) fn check_ops(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(owners, function_aliases, raw_aliases, op);
        }
    }

    fn check_op(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        op: &ResourceOp,
    ) {
        match op {
            ResourceOp::DeclareLocal {
                place,
                initializer,
                span,
                ..
            } => {
                if let Some(initializer) = initializer {
                    self.transfer_owner(
                        owners,
                        raw_aliases,
                        initializer,
                        place,
                        ResourceOwnerOperation::DeclareInitializer,
                        *span,
                    );
                    function_aliases.copy_alias(initializer, place);
                }
            }
            ResourceOp::Read {
                source,
                output,
                span: _,
            } => {
                raw_aliases.copy_alias_or_seed(source, output);
                function_aliases.copy_alias(source, output);
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                self.report_overwritten_owners(owners, target, value, *span);
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    value,
                    target,
                    ResourceOwnerOperation::AssignValue,
                    *span,
                );
                function_aliases.copy_alias(value, target);
            }
            ResourceOp::Move {
                source,
                output,
                span,
            } => {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    source,
                    output,
                    ResourceOwnerOperation::Move,
                    *span,
                );
                function_aliases.copy_alias(source, output);
            }
            ResourceOp::RawMemory {
                operation,
                output,
                args,
                span,
            } => match operation {
                RawMemoryOp::Alloc => {
                    owners.allocate(output);
                    raw_aliases.mark(output);
                }
                RawMemoryOp::Dealloc => {
                    if let Some(ptr) = args.first() {
                        self.release_owner(
                            owners,
                            raw_aliases,
                            ptr,
                            ResourceOwnerOperation::Dealloc,
                            *span,
                        );
                    }
                }
                RawMemoryOp::Realloc => {
                    if let Some(ptr) = args.first() {
                        if self.release_owner(
                            owners,
                            raw_aliases,
                            ptr,
                            ResourceOwnerOperation::ReallocInput,
                            *span,
                        ) {
                            owners.allocate(output);
                            raw_aliases.mark(output);
                        }
                    }
                }
                RawMemoryOp::Load
                | RawMemoryOp::Store
                | RawMemoryOp::BulkCopy
                | RawMemoryOp::BulkMove
                | RawMemoryOp::MemorySize
                | RawMemoryOp::MemoryGrow
                | RawMemoryOp::Fill
                | RawMemoryOp::Other { .. } => {}
            },
            ResourceOp::Branch {
                output,
                then_ops,
                then_value,
                else_ops,
                else_value,
                span,
                ..
            } => {
                let mut then_owners = owners.clone();
                let mut else_owners = owners.clone();
                let mut then_function_aliases = function_aliases.clone();
                let mut else_function_aliases = function_aliases.clone();
                let mut then_raw_aliases = raw_aliases.clone();
                let mut else_raw_aliases = raw_aliases.clone();
                self.check_ops(
                    &mut then_owners,
                    &mut then_function_aliases,
                    &mut then_raw_aliases,
                    then_ops,
                );
                self.check_ops(
                    &mut else_owners,
                    &mut else_function_aliases,
                    &mut else_raw_aliases,
                    else_ops,
                );
                self.transfer_owner(
                    &mut then_owners,
                    &mut then_raw_aliases,
                    then_value,
                    output,
                    ResourceOwnerOperation::BranchValue,
                    *span,
                );
                self.transfer_owner(
                    &mut else_owners,
                    &mut else_raw_aliases,
                    else_value,
                    output,
                    ResourceOwnerOperation::BranchValue,
                    *span,
                );
                *owners = OwnerTable::merge_paths(&[then_owners, else_owners]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    then_function_aliases,
                    else_function_aliases,
                ]);
                *raw_aliases =
                    RawCellAddressAliases::merge_paths(&[then_raw_aliases, else_raw_aliases]);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                let mut condition_owners = owners.clone();
                let mut condition_function_aliases = function_aliases.clone();
                let mut condition_raw_aliases = raw_aliases.clone();
                self.check_ops(
                    &mut condition_owners,
                    &mut condition_function_aliases,
                    &mut condition_raw_aliases,
                    condition_ops,
                );
                let mut body_owners = condition_owners.clone();
                let mut body_function_aliases = condition_function_aliases.clone();
                let mut body_raw_aliases = condition_raw_aliases.clone();
                self.check_ops(
                    &mut body_owners,
                    &mut body_function_aliases,
                    &mut body_raw_aliases,
                    body_ops,
                );
                *owners = OwnerTable::merge_paths(&[condition_owners, body_owners]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    condition_function_aliases,
                    body_function_aliases,
                ]);
                *raw_aliases =
                    RawCellAddressAliases::merge_paths(&[condition_raw_aliases, body_raw_aliases]);
            }
            ResourceOp::Match {
                output, arms, span, ..
            } => {
                let mut arm_paths = Vec::new();
                let mut function_alias_paths = Vec::new();
                let mut raw_alias_paths = Vec::new();
                for arm in arms {
                    let mut arm_owners = owners.clone();
                    let mut arm_function_aliases = function_aliases.clone();
                    let mut arm_raw_aliases = raw_aliases.clone();
                    self.check_ops(
                        &mut arm_owners,
                        &mut arm_function_aliases,
                        &mut arm_raw_aliases,
                        &arm.ops,
                    );
                    self.transfer_owner(
                        &mut arm_owners,
                        &mut arm_raw_aliases,
                        &arm.value,
                        output,
                        ResourceOwnerOperation::MatchValue,
                        *span,
                    );
                    arm_paths.push(arm_owners);
                    function_alias_paths.push(arm_function_aliases);
                    raw_alias_paths.push(arm_raw_aliases);
                }
                if !arm_paths.is_empty() {
                    *owners = OwnerTable::merge_paths(&arm_paths);
                    *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
                    *raw_aliases = RawCellAddressAliases::merge_paths(&raw_alias_paths);
                }
            }
            ResourceOp::FunctionValue { output, name, .. } => {
                function_aliases.set_alias(output, name.clone());
            }
            ResourceOp::Call {
                output,
                target,
                args,
                span,
                ..
            } => self.apply_call_return_owner(owners, raw_aliases, output, target, args, *span),
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                span,
                ..
            } => self.apply_indirect_call_return_owner(
                owners,
                function_aliases,
                raw_aliases,
                output,
                callee,
                args,
                *span,
            ),
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                span,
            } => {
                self.construct_owner_fields(owners, raw_aliases, output, kind, inputs, *span);
                construct_function_alias_fields(function_aliases, output, kind, inputs);
            }
            ResourceOp::RawAddressAlias { source, target, .. } => {
                raw_aliases.copy_alias_or_seed(source, target);
            }
            ResourceOp::Expr { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::CallEffect { .. } => {}
        }
    }

    fn construct_owner_fields(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
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
                input,
                &field,
                ResourceOwnerOperation::ConstructInput,
                span,
            );
        }
    }

    fn apply_call_return_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
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
        self.apply_owner_return_summary(owners, raw_aliases, output, args, summary, span);
    }

    fn apply_indirect_call_return_owner(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        output: &Place,
        callee: &Place,
        args: &[Place],
        span: Span,
    ) {
        let functions = function_aliases.functions(callee);
        if functions.is_empty() {
            self.apply_unknown_indirect_call_return_owner(owners, raw_aliases, output, args, span);
            return;
        }
        for function in functions {
            if let Some(summary) = self
                .summaries
                .iter()
                .find(|summary| summary.function == function.as_str())
            {
                self.apply_owner_return_summary(owners, raw_aliases, output, args, summary, span);
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
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        for arg in args.iter().filter(|arg| arg.ty == output.ty) {
            if self.has_transferable_owner(owners, raw_aliases, arg) {
                self.transfer_owner(
                    owners,
                    raw_aliases,
                    arg,
                    output,
                    ResourceOwnerOperation::ReturnValue,
                    span,
                );
                return;
            }
        }
    }

    fn apply_owner_return_summary(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
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
        }
        for projection in &summary.projection_returns {
            let output_projection = place_with_suffix(output, &projection.suffix, projection.ty);
            self.apply_owner_projection_return_summary(
                owners,
                raw_aliases,
                &output_projection,
                args,
                projection,
                span,
            );
        }
    }

    fn apply_owner_projection_return_summary(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
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

    fn report_overwritten_owners(
        &mut self,
        owners: &mut OwnerTable,
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
        }
    }

    fn transfer_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
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
                    raw_aliases.clear(source);
                    raw_aliases.clear(&resolved_source);
                    raw_aliases.mark(target);
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
                        raw_aliases.clear(&entry.place);
                        raw_aliases.mark(&target_place);
                    }
                }
                OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed => {
                    self.push_unavailable(operation, &entry.place, entry.state, span);
                }
                OwnerState::NoFreeObligation => {}
            }
        }
    }

    fn move_owner_out(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
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
                    raw_aliases.clear(&entry.place);
                }
                OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed => {
                    self.push_unavailable(operation, &entry.place, entry.state, span);
                }
                OwnerState::NoFreeObligation => {}
            }
        }
    }

    fn release_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
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
                raw_aliases.clear(place);
                raw_aliases.clear(&resolved_place);
                true
            }
            Some(state) => {
                self.push_unavailable(operation, &resolved_place, state, span);
                false
            }
            None => {
                self.push_unavailable(operation, place, OwnerState::NoFreeObligation, span);
                false
            }
        }
    }

    fn push_live_owner_diagnostics(&mut self, owners: &OwnerTable, span: Span) {
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

fn merge_owner_deferred(
    target: &mut ResourceOwnerCheckDeferred,
    source: ResourceOwnerCheckDeferred,
) {
    target.branch_merges += source.branch_merges;
    target.loop_merges += source.loop_merges;
    target.match_merges += source.match_merges;
}
