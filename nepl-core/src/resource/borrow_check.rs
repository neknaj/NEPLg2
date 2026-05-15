extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeCtx;

use super::borrow_call::{
    propagate_call_return_token, propagate_indirect_call_return_token,
    release_call_temporary_argument_tokens,
};
use super::borrow_scope::check_end_scope;
use super::borrow_state::BorrowTable;
use super::borrow_usage::{
    propagate_construct_borrow_tokens, propagate_match_bind_borrow_token,
    scan_borrow_binding_future, terminator_uses_place, BorrowBindingFuture,
};
use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::model::{
    BorrowKind, BorrowState, BorrowStateEntry, Place, PlaceRoot, ResourceBlock, ResourceFunction,
    ResourceModule, ResourceOp, ResourceTerminator,
};
use super::report::{
    ResourceBorrowCheckDeferred, ResourceBorrowCheckReport, ResourceBorrowDiagnostic,
    ResourceBorrowFunctionCheck, ResourceBorrowOperation,
};
use super::summary::{compute_borrow_token_return_summaries, BorrowTokenReturnSummaryIndex};

pub fn check_resource_borrow_lifetimes(
    module: &ResourceModule,
    types: &TypeCtx,
) -> ResourceBorrowCheckReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceBorrowCheckDeferred::default();
    let summaries = compute_borrow_token_return_summaries(module, types);
    let summary_index = BorrowTokenReturnSummaryIndex::new(&summaries);

    for function in &module.functions {
        let mut engine = ResourceBorrowCheckEngine {
            function: function.name.as_str(),
            types,
            summaries: &summary_index,
            parameter_names: Vec::new(),
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
    pub(super) types: &'a TypeCtx,
    pub(super) summaries: &'a BorrowTokenReturnSummaryIndex<'a>,
    pub(super) parameter_names: Vec<String>,
    pub(super) diagnostics: Vec<ResourceBorrowDiagnostic>,
    pub(super) deferred: ResourceBorrowCheckDeferred,
}

struct BorrowContinuation<'a> {
    op_segments: Vec<&'a [ResourceOp]>,
    terminator: Option<&'a ResourceTerminator>,
}

impl<'a> BorrowContinuation<'a> {
    fn new(terminator: Option<&'a ResourceTerminator>) -> Self {
        Self {
            op_segments: Vec::new(),
            terminator,
        }
    }

    fn with_segment(&self, segment: &'a [ResourceOp]) -> Self {
        let mut op_segments = Vec::with_capacity(self.op_segments.len() + 1);
        if !segment.is_empty() {
            op_segments.push(segment);
        }
        op_segments.extend(self.op_segments.iter().copied());
        Self {
            op_segments,
            terminator: self.terminator,
        }
    }

    fn keeps_borrow_binding(&self, binding: &super::borrow_state::BorrowBinding) -> bool {
        for ops in &self.op_segments {
            match scan_borrow_binding_future(ops, binding) {
                BorrowBindingFuture::Keep => return true,
                BorrowBindingFuture::Ended => return false,
                BorrowBindingFuture::Unused => {}
            }
        }
        self.terminator
            .is_some_and(|terminator| terminator_uses_place(terminator, &binding.token))
    }
}

impl ResourceBorrowCheckEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) -> Vec<BorrowStateEntry> {
        self.parameter_names = function
            .params
            .iter()
            .map(|param| param.name.clone())
            .collect();
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
        self.check_ops_with_terminator(
            borrows,
            function_aliases,
            &block.ops,
            Some(&block.terminator),
        );
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                let continuation = BorrowContinuation::new(Some(&block.terminator));
                self.release_dead_borrow_tokens(borrows, &[], &continuation);
                if let Some(value) = value {
                    self.check_return_escape(borrows, value, *span);
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    pub(super) fn check_ops_with_terminator(
        &mut self,
        borrows: &mut BorrowTable,
        function_aliases: &mut FunctionAliasTable,
        ops: &[ResourceOp],
        terminator: Option<&ResourceTerminator>,
    ) {
        let continuation = BorrowContinuation::new(terminator);
        self.check_ops_with_continuation(borrows, function_aliases, ops, &continuation);
    }

    fn check_ops_with_continuation<'op>(
        &mut self,
        borrows: &mut BorrowTable,
        function_aliases: &mut FunctionAliasTable,
        ops: &'op [ResourceOp],
        continuation: &BorrowContinuation<'op>,
    ) {
        for (index, op) in ops.iter().enumerate() {
            if !matches!(op, ResourceOp::EndScope { .. }) {
                self.release_dead_borrow_tokens(borrows, &ops[index..], continuation);
            }
            let nested_continuation = continuation.with_segment(&ops[index + 1..]);
            self.check_op(borrows, function_aliases, op, &nested_continuation);
        }
    }

    fn release_dead_borrow_tokens(
        &self,
        borrows: &mut BorrowTable,
        future_ops: &[ResourceOp],
        continuation: &BorrowContinuation<'_>,
    ) {
        borrows.release_tokens_not_used_by(|binding| {
            match scan_borrow_binding_future(future_ops, binding) {
                BorrowBindingFuture::Keep => true,
                BorrowBindingFuture::Ended => false,
                BorrowBindingFuture::Unused => continuation.keeps_borrow_binding(binding),
            }
        });
    }

    fn check_op(
        &mut self,
        borrows: &mut BorrowTable,
        function_aliases: &mut FunctionAliasTable,
        op: &ResourceOp,
        continuation: &BorrowContinuation<'_>,
    ) {
        match op {
            ResourceOp::DeclareLocal {
                place, initializer, ..
            } => {
                if let Some(initializer) = initializer {
                    borrows.copy_or_move_token_tree(initializer, place, false);
                    function_aliases.copy_alias(initializer, place);
                }
            }
            ResourceOp::Read {
                source,
                output,
                span,
            } => {
                let is_copy = self.types.is_copy(source.ty);
                let propagated = borrows.copy_or_move_token_tree(source, output, is_copy);
                if is_copy {
                    if !propagated {
                        self.check_source_read(borrows, source, *span);
                    }
                } else {
                    self.check_source_exclusive(
                        borrows,
                        source,
                        ResourceBorrowOperation::Move,
                        *span,
                    );
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
                borrows.release_token_tree(target);
                borrows.copy_or_move_token_tree(value, target, false);
                function_aliases.copy_alias(value, target);
            }
            ResourceOp::Borrow {
                source,
                output,
                kind,
                span,
                ..
            } => self.start_borrow(borrows, source, output, *kind, *span),
            ResourceOp::Move {
                source,
                output,
                span,
            } => {
                borrows.copy_or_move_token_tree(source, output, false);
                self.check_source_exclusive(borrows, source, ResourceBorrowOperation::Move, *span);
                function_aliases.copy_alias(source, output);
            }
            ResourceOp::Drop { place, span } => {
                if !borrows.release_token_tree(place) {
                    self.check_source_exclusive(
                        borrows,
                        place,
                        ResourceBorrowOperation::Drop,
                        *span,
                    );
                }
            }
            ResourceOp::EndScope {
                locals,
                result,
                span,
            } => self.end_scope(borrows, locals, result.as_ref(), *span),
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                let mut then_borrows = borrows.clone();
                let mut else_borrows = borrows.clone();
                let mut then_function_aliases = function_aliases.clone();
                let mut else_function_aliases = function_aliases.clone();
                self.check_ops_with_continuation(
                    &mut then_borrows,
                    &mut then_function_aliases,
                    then_ops,
                    continuation,
                );
                self.check_ops_with_continuation(
                    &mut else_borrows,
                    &mut else_function_aliases,
                    else_ops,
                    continuation,
                );
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
                self.check_ops_with_continuation(
                    &mut condition_borrows,
                    &mut condition_function_aliases,
                    condition_ops,
                    continuation,
                );
                let mut body_borrows = condition_borrows.clone();
                let mut body_function_aliases = condition_function_aliases.clone();
                self.check_ops_with_continuation(
                    &mut body_borrows,
                    &mut body_function_aliases,
                    body_ops,
                    continuation,
                );
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
                    propagate_match_bind_borrow_token(&mut arm_borrows, op, arm);
                    self.check_ops_with_continuation(
                        &mut arm_borrows,
                        &mut arm_function_aliases,
                        &arm.ops,
                        continuation,
                    );
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
            } => {
                propagate_call_return_token(borrows, self.summaries, output, target, args);
                release_call_temporary_argument_tokens(borrows, output, args);
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                ..
            } => {
                propagate_indirect_call_return_token(
                    borrows,
                    function_aliases,
                    self.summaries,
                    output,
                    callee,
                    args,
                );
                release_call_temporary_argument_tokens(borrows, output, args);
            }
            ResourceOp::Expr { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::RawMemory { .. }
            | ResourceOp::RawAddressAlias { .. }
            | ResourceOp::RawAddressView { .. }
            | ResourceOp::StorageOrigin { .. } => {}
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                ..
            } => {
                propagate_construct_borrow_tokens(borrows, output, kind, inputs);
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
        for binding in borrows.bindings_overlapping_token(place) {
            if self.borrow_source_can_escape_return(&binding.source) {
                continue;
            }
            let active = borrows.state(&binding.source);
            if matches!(
                active,
                BorrowState::Shared { .. } | BorrowState::Unique { .. }
            ) {
                self.push_conflict(
                    ResourceBorrowOperation::ReturnValue,
                    &binding.token,
                    active,
                    span,
                );
            }
        }
    }

    fn borrow_source_can_escape_return(&self, source: &Place) -> bool {
        let PlaceRoot::Local(name) = &source.root else {
            return false;
        };
        self.parameter_names.iter().any(|param| param == name)
            && source.projections.first().is_some_and(|projection| {
                matches!(projection, super::model::PlaceProjection::Deref)
            })
    }

    fn check_source_exclusive(
        &mut self,
        borrows: &BorrowTable,
        place: &Place,
        operation: ResourceBorrowOperation,
        span: Span,
    ) {
        if let Some(active @ (BorrowState::Shared { .. } | BorrowState::Unique { .. })) =
            borrows.active_state_for_deref_access(place)
        {
            self.push_conflict(operation, place, active, span);
            return;
        }
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

    fn end_scope(
        &mut self,
        borrows: &mut BorrowTable,
        locals: &[Place],
        result: Option<&Place>,
        span: Span,
    ) {
        check_end_scope(
            self.function,
            &mut self.diagnostics,
            borrows,
            locals,
            result,
            span,
        );
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
