extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;

use super::borrow_state::BorrowTable;
use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::model::{
    BorrowKind, BorrowState, BorrowStateEntry, Place, ResourceBlock, ResourceCallTarget,
    ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::report::{
    ResourceBorrowCheckDeferred, ResourceBorrowCheckReport, ResourceBorrowDiagnostic,
    ResourceBorrowFunctionCheck, ResourceBorrowOperation,
};
use super::summary::{compute_borrow_token_return_summaries, BorrowTokenReturnSummary};

pub fn check_resource_borrow_lifetimes(module: &ResourceModule) -> ResourceBorrowCheckReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceBorrowCheckDeferred::default();
    let summaries = compute_borrow_token_return_summaries(module);

    for function in &module.functions {
        let mut engine = ResourceBorrowCheckEngine {
            function: function.name.as_str(),
            summaries: &summaries,
            diagnostics: Vec::new(),
            deferred: ResourceBorrowCheckDeferred::default(),
        };
        let final_borrows = engine.check_function(function);
        merge_borrow_deferred(&mut deferred, engine.deferred);
        diagnostics.extend(engine.diagnostics);
        functions.push(ResourceBorrowFunctionCheck {
            name: function.name.clone(),
            final_borrows,
            deferred: engine.deferred,
        });
    }

    ResourceBorrowCheckReport {
        functions,
        diagnostics,
        deferred,
    }
}

pub(super) struct ResourceBorrowCheckEngine<'a> {
    pub(super) function: &'a str,
    pub(super) summaries: &'a [BorrowTokenReturnSummary],
    pub(super) diagnostics: Vec<ResourceBorrowDiagnostic>,
    pub(super) deferred: ResourceBorrowCheckDeferred,
}

impl ResourceBorrowCheckEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) -> Vec<BorrowStateEntry> {
        let mut borrows = BorrowTable::default();
        let mut function_aliases = FunctionAliasTable::default();
        for block in &function.blocks {
            self.check_block(&mut borrows, &mut function_aliases, block);
        }
        borrows.into_entries()
    }

    fn check_block(
        &mut self,
        borrows: &mut BorrowTable,
        function_aliases: &mut FunctionAliasTable,
        block: &ResourceBlock,
    ) {
        self.check_ops(borrows, function_aliases, &block.ops);
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if let Some(value) = value {
                    self.check_return_escape(borrows, value, *span);
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    pub(super) fn check_ops(
        &mut self,
        borrows: &mut BorrowTable,
        function_aliases: &mut FunctionAliasTable,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(borrows, function_aliases, op);
        }
    }

    fn check_op(
        &mut self,
        borrows: &mut BorrowTable,
        function_aliases: &mut FunctionAliasTable,
        op: &ResourceOp,
    ) {
        match op {
            ResourceOp::DeclareLocal {
                place, initializer, ..
            } => {
                if let Some(initializer) = initializer {
                    borrows.transfer_token(initializer, place);
                    function_aliases.copy_alias(initializer, place);
                }
            }
            ResourceOp::Read {
                source,
                output,
                span,
            } => {
                if !borrows.copy_or_move_token(source, output) {
                    self.check_source_read(borrows, source, *span);
                }
                function_aliases.copy_alias(source, output);
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                self.check_source_exclusive(
                    borrows,
                    target,
                    ResourceBorrowOperation::Assign,
                    *span,
                );
                borrows.transfer_token(value, target);
                function_aliases.copy_alias(value, target);
            }
            ResourceOp::Borrow {
                source,
                output,
                kind,
                span,
            } => self.start_borrow(borrows, source, output, *kind, *span),
            ResourceOp::Move {
                source,
                output,
                span,
            } => {
                if !borrows.transfer_token(source, output) {
                    self.check_source_exclusive(
                        borrows,
                        source,
                        ResourceBorrowOperation::Move,
                        *span,
                    );
                }
                function_aliases.copy_alias(source, output);
            }
            ResourceOp::Drop { place, span } => {
                if !borrows.release_token(place) {
                    self.check_source_exclusive(
                        borrows,
                        place,
                        ResourceBorrowOperation::Drop,
                        *span,
                    );
                }
            }
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                let mut then_borrows = borrows.clone();
                let mut else_borrows = borrows.clone();
                let mut then_function_aliases = function_aliases.clone();
                let mut else_function_aliases = function_aliases.clone();
                self.check_ops(&mut then_borrows, &mut then_function_aliases, then_ops);
                self.check_ops(&mut else_borrows, &mut else_function_aliases, else_ops);
                *borrows = BorrowTable::merge_paths(&[then_borrows, else_borrows]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    then_function_aliases,
                    else_function_aliases,
                ]);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                let mut condition_borrows = borrows.clone();
                let mut condition_function_aliases = function_aliases.clone();
                self.check_ops(
                    &mut condition_borrows,
                    &mut condition_function_aliases,
                    condition_ops,
                );
                let mut body_borrows = condition_borrows.clone();
                let mut body_function_aliases = condition_function_aliases.clone();
                self.check_ops(&mut body_borrows, &mut body_function_aliases, body_ops);
                *borrows = BorrowTable::merge_paths(&[condition_borrows, body_borrows]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    condition_function_aliases,
                    body_function_aliases,
                ]);
            }
            ResourceOp::Match { arms, .. } => {
                let mut arm_paths = Vec::new();
                let mut function_alias_paths = Vec::new();
                for arm in arms {
                    let mut arm_borrows = borrows.clone();
                    let mut arm_function_aliases = function_aliases.clone();
                    self.check_ops(&mut arm_borrows, &mut arm_function_aliases, &arm.ops);
                    arm_paths.push(arm_borrows);
                    function_alias_paths.push(arm_function_aliases);
                }
                if !arm_paths.is_empty() {
                    *borrows = BorrowTable::merge_paths(&arm_paths);
                    *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
                }
            }
            ResourceOp::FunctionValue { output, name, .. } => {
                function_aliases.set_alias(output, name.clone());
            }
            ResourceOp::Call {
                output,
                target,
                args,
                ..
            } => self.propagate_call_return_token(borrows, output, target, args),
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                ..
            } => self.propagate_indirect_call_return_token(
                borrows,
                function_aliases,
                output,
                callee,
                args,
            ),
            ResourceOp::Expr { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::RawMemory { .. }
            | ResourceOp::RawAddressAlias { .. } => {}
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                ..
            } => {
                construct_function_alias_fields(function_aliases, output, kind, inputs);
            }
        }
    }

    fn start_borrow(
        &mut self,
        borrows: &mut BorrowTable,
        source: &Place,
        output: &Place,
        kind: BorrowKind,
        span: Span,
    ) {
        let active = borrows.active_state_overlapping(source);
        match (kind, active) {
            (BorrowKind::Shared, Some(active @ BorrowState::Unique { .. })) => {
                self.push_conflict(ResourceBorrowOperation::SharedBorrow, source, active, span);
            }
            (
                BorrowKind::Unique,
                Some(active @ (BorrowState::Shared { .. } | BorrowState::Unique { .. })),
            ) => {
                self.push_conflict(ResourceBorrowOperation::UniqueBorrow, source, active, span);
            }
            (BorrowKind::Shared, _) => borrows.add_shared(source, output),
            (BorrowKind::Unique, _) => borrows.add_unique(source, output),
        }
    }

    fn check_source_read(&mut self, borrows: &BorrowTable, place: &Place, span: Span) {
        if let Some(active @ BorrowState::Unique { .. }) = borrows.unique_state_overlapping(place) {
            self.push_conflict(ResourceBorrowOperation::Read, place, active, span);
        }
    }

    fn check_return_escape(&mut self, borrows: &BorrowTable, place: &Place, span: Span) {
        if let Some(binding) = borrows.binding(place) {
            let active = borrows.state(&binding.source);
            if matches!(
                active,
                BorrowState::Shared { .. } | BorrowState::Unique { .. }
            ) {
                self.push_conflict(ResourceBorrowOperation::ReturnValue, place, active, span);
            }
        }
    }

    fn propagate_call_return_token(
        &self,
        borrows: &mut BorrowTable,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
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
        for arg in summary
            .parameter_indices
            .iter()
            .filter_map(|index| args.get(*index))
        {
            if borrows.copy_or_move_token(arg, output) {
                return;
            }
        }
    }

    fn propagate_indirect_call_return_token(
        &self,
        borrows: &mut BorrowTable,
        function_aliases: &FunctionAliasTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
    ) {
        let functions = function_aliases.functions(callee);
        if functions.is_empty() {
            self.propagate_unknown_indirect_call_return_token(borrows, output, args);
            return;
        }
        for function in functions {
            if let Some(summary) = self
                .summaries
                .iter()
                .find(|summary| summary.function == function.as_str())
            {
                for arg in summary
                    .parameter_indices
                    .iter()
                    .filter_map(|index| args.get(*index))
                {
                    if borrows.copy_or_move_token(arg, output) {
                        return;
                    }
                }
            }
        }
    }

    fn propagate_unknown_indirect_call_return_token(
        &self,
        borrows: &mut BorrowTable,
        output: &Place,
        args: &[Place],
    ) {
        for arg in args.iter().filter(|arg| arg.ty == output.ty) {
            if borrows.copy_or_move_token(arg, output) {
                return;
            }
        }
    }

    fn check_source_exclusive(
        &mut self,
        borrows: &BorrowTable,
        place: &Place,
        operation: ResourceBorrowOperation,
        span: Span,
    ) {
        match borrows.active_state_overlapping(place) {
            Some(active @ (BorrowState::Shared { .. } | BorrowState::Unique { .. })) => {
                self.push_conflict(operation, place, active, span);
            }
            Some(BorrowState::Unborrowed | BorrowState::Released) | None => {}
        }
    }

    fn push_conflict(
        &mut self,
        operation: ResourceBorrowOperation,
        place: &Place,
        active: BorrowState,
        span: Span,
    ) {
        self.diagnostics
            .push(ResourceBorrowDiagnostic::BorrowConflict {
                function: String::from(self.function),
                operation,
                place: place.clone(),
                active,
                span,
            });
    }
}

fn merge_borrow_deferred(
    target: &mut ResourceBorrowCheckDeferred,
    source: ResourceBorrowCheckDeferred,
) {
    target.branch_merges += source.branch_merges;
    target.loop_merges += source.loop_merges;
    target.match_merges += source.match_merges;
}
