extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::{
    apply_direct_call_raw_alias_summary, apply_indirect_call_raw_alias_summary,
    compute_raw_cell_address_return_summaries, construct_raw_cell_address_alias_fields,
    expr_kind_preserves_raw_alias, RawCellAddressReturnSummary,
};
use super::model::{
    CellState, CellStateEntry, EffectOp, Place, ResourceBlock, ResourceCallTarget,
    ResourceExprKind, ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::place_utils::{match_bind_payload_place, should_track};
use super::report::{
    ResourceCheckDeferred, ResourceCheckDiagnostic, ResourceCheckOperation, ResourceCheckReport,
    ResourceFunctionCheck,
};

pub fn check_resource_initialized_moves(
    module: &ResourceModule,
    types: &TypeCtx,
) -> ResourceCheckReport {
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceCheckDeferred::default();
    let raw_alias_summaries = compute_raw_cell_address_return_summaries(module, types);

    for function in &module.functions {
        let mut engine = ResourceCheckEngine {
            function: function.name.as_str(),
            types,
            raw_alias_summaries: &raw_alias_summaries,
            diagnostics: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
        };
        let final_cells = engine.check_function(function);
        merge_deferred(&mut deferred, engine.deferred);
        diagnostics.extend(engine.diagnostics);
        functions.push(ResourceFunctionCheck {
            name: function.name.clone(),
            final_cells,
            deferred: engine.deferred,
        });
    }

    ResourceCheckReport {
        functions,
        diagnostics,
        deferred,
    }
}

pub(super) struct ResourceCheckEngine<'a> {
    function: &'a str,
    pub(super) types: &'a TypeCtx,
    raw_alias_summaries: &'a [RawCellAddressReturnSummary],
    diagnostics: Vec<ResourceCheckDiagnostic>,
    deferred: ResourceCheckDeferred,
}

impl ResourceCheckEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) -> Vec<CellStateEntry> {
        let mut cells = CellTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut function_aliases = FunctionAliasTable::default();
        for param in &function.params {
            cells.mark_initialized(&param.place);
            cells.mark_external_raw_storage_root(&param.place);
            raw_aliases.mark(&param.place);
        }
        for block in &function.blocks {
            self.check_block(&mut cells, &mut raw_aliases, &mut function_aliases, block);
        }
        cells.into_entries()
    }

    fn check_block(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        block: &ResourceBlock,
    ) {
        self.check_ops(cells, raw_aliases, function_aliases, &block.ops);
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if let Some(value) = value {
                    self.consume_by_value(cells, value, ResourceCheckOperation::ReturnValue, *span);
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    fn check_ops(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(cells, raw_aliases, function_aliases, op);
        }
    }

    fn check_op(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        op: &ResourceOp,
    ) {
        match op {
            ResourceOp::Expr {
                kind,
                output,
                span: _,
                ..
            } => self.check_expr(cells, raw_aliases, *kind, output),
            ResourceOp::DeclareLocal {
                place,
                initializer,
                span,
                ..
            } => {
                if let Some(initializer) = initializer {
                    if self.consume_by_value(
                        cells,
                        initializer,
                        ResourceCheckOperation::DeclareInitializer,
                        *span,
                    ) {
                        cells.mark_initialized(place);
                        self.copy_raw_alias_and_rekey_cells(cells, raw_aliases, initializer, place);
                        function_aliases.copy_alias(initializer, place);
                    } else {
                        cells.set_state(place, CellState::Uninit);
                        raw_aliases.clear(place);
                    }
                } else {
                    cells.set_state(place, CellState::Uninit);
                    raw_aliases.clear(place);
                }
            }
            ResourceOp::Read {
                source,
                output,
                span,
            } => {
                if self.consume_by_value(cells, source, ResourceCheckOperation::Read, *span) {
                    cells.mark_initialized(output);
                    self.copy_raw_alias_and_rekey_cells(cells, raw_aliases, source, output);
                    function_aliases.copy_alias(source, output);
                }
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                if self.consume_by_value(cells, value, ResourceCheckOperation::AssignValue, *span) {
                    cells.mark_initialized(target);
                    self.copy_raw_alias_and_rekey_cells(cells, raw_aliases, value, target);
                    function_aliases.copy_alias(value, target);
                } else {
                    raw_aliases.clear(target);
                }
            }
            ResourceOp::Borrow {
                source,
                output,
                span,
                ..
            } => {
                if self.ensure_available(cells, source, ResourceCheckOperation::Borrow, *span) {
                    cells.mark_initialized(output);
                    self.copy_raw_alias_and_rekey_cells(cells, raw_aliases, source, output);
                }
            }
            ResourceOp::Move {
                source,
                output,
                span,
            } => {
                if self.ensure_available(cells, source, ResourceCheckOperation::Move, *span) {
                    cells.set_state(source, CellState::Moved);
                    cells.mark_initialized(output);
                    self.copy_raw_alias_and_rekey_cells(cells, raw_aliases, source, output);
                    function_aliases.copy_alias(source, output);
                }
            }
            ResourceOp::Drop { place, span } => {
                if self.ensure_available(cells, place, ResourceCheckOperation::Drop, *span) {
                    cells.set_state(place, CellState::Dropped);
                    raw_aliases.clear(place);
                }
            }
            ResourceOp::CallEffect { .. } => {}
            ResourceOp::FunctionValue { output, name, .. } => {
                cells.mark_initialized(output);
                raw_aliases.clear(output);
                function_aliases.set_alias(output, name.clone());
            }
            ResourceOp::Call {
                output,
                target,
                args,
                effect,
                span,
                ..
            } => {
                if matches!(
                    effect,
                    EffectOp::InternalAlloc | EffectOp::UnsafeMemory { .. }
                ) {
                    return;
                }
                let args_available =
                    self.consume_args(cells, args, ResourceCheckOperation::CallArgument, *span);
                if args_available {
                    cells.mark_initialized(output);
                    if !self.apply_call_return_raw_alias(raw_aliases, output, target, args) {
                        raw_aliases.clear(output);
                    }
                }
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                span,
                ..
            } => {
                let callee_available = self.ensure_available(
                    cells,
                    callee,
                    ResourceCheckOperation::IndirectCallee,
                    *span,
                );
                let args_available =
                    self.consume_args(cells, args, ResourceCheckOperation::CallArgument, *span);
                if callee_available && args_available {
                    cells.mark_initialized(output);
                    if !self.apply_indirect_call_return_raw_alias(
                        raw_aliases,
                        function_aliases,
                        output,
                        callee,
                        args,
                    ) {
                        raw_aliases.clear(output);
                    }
                }
            }
            ResourceOp::RawMemory {
                operation,
                output,
                args,
                span,
            } => self.check_raw_memory(cells, raw_aliases, operation, output, args, *span),
            ResourceOp::RawAddressAlias { source, target, .. } => {
                self.copy_raw_address_alias_and_rekey_cells(cells, raw_aliases, source, target);
            }
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                span,
                ..
            } => {
                let inputs_available =
                    self.consume_args(cells, inputs, ResourceCheckOperation::ConstructInput, *span);
                if inputs_available {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    construct_raw_cell_address_alias_fields(raw_aliases, output, kind, inputs);
                    construct_function_alias_fields(function_aliases, output, kind, inputs);
                }
            }
            ResourceOp::Branch {
                output,
                condition,
                then_ops,
                then_value,
                else_ops,
                else_value,
                span,
                condition_fact: _,
            } => {
                let condition_available = self.consume_by_value(
                    cells,
                    condition,
                    ResourceCheckOperation::BranchCondition,
                    *span,
                );
                let mut then_cells = cells.clone();
                let mut else_cells = cells.clone();
                let mut then_aliases = raw_aliases.clone();
                let mut else_aliases = raw_aliases.clone();
                let mut then_function_aliases = function_aliases.clone();
                let mut else_function_aliases = function_aliases.clone();
                self.check_ops(
                    &mut then_cells,
                    &mut then_aliases,
                    &mut then_function_aliases,
                    then_ops,
                );
                self.check_ops(
                    &mut else_cells,
                    &mut else_aliases,
                    &mut else_function_aliases,
                    else_ops,
                );
                let then_available = self.consume_by_value(
                    &mut then_cells,
                    then_value,
                    ResourceCheckOperation::BranchValue,
                    *span,
                );
                let else_available = self.consume_by_value(
                    &mut else_cells,
                    else_value,
                    ResourceCheckOperation::BranchValue,
                    *span,
                );
                if then_available {
                    self.copy_raw_alias_and_rekey_cells(
                        &mut then_cells,
                        &mut then_aliases,
                        then_value,
                        output,
                    );
                    then_function_aliases.copy_alias(then_value, output);
                }
                if else_available {
                    self.copy_raw_alias_and_rekey_cells(
                        &mut else_cells,
                        &mut else_aliases,
                        else_value,
                        output,
                    );
                    else_function_aliases.copy_alias(else_value, output);
                }
                *cells = CellTable::merge_paths(&[then_cells, else_cells]);
                *raw_aliases = RawCellAddressAliases::merge_paths(&[then_aliases, else_aliases]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    then_function_aliases,
                    else_function_aliases,
                ]);
                if condition_available && then_available && else_available {
                    cells.mark_initialized(output);
                } else {
                    raw_aliases.clear(output);
                }
            }
            ResourceOp::Loop {
                condition_ops,
                condition,
                body_ops,
                span,
            } => {
                let mut condition_cells = cells.clone();
                let mut condition_aliases = raw_aliases.clone();
                let mut condition_function_aliases = function_aliases.clone();
                self.check_ops(
                    &mut condition_cells,
                    &mut condition_aliases,
                    &mut condition_function_aliases,
                    condition_ops,
                );
                self.consume_by_value(
                    &mut condition_cells,
                    condition,
                    ResourceCheckOperation::LoopCondition,
                    *span,
                );
                let mut body_cells = condition_cells.clone();
                let mut body_aliases = condition_aliases.clone();
                let mut body_function_aliases = condition_function_aliases.clone();
                self.check_ops(
                    &mut body_cells,
                    &mut body_aliases,
                    &mut body_function_aliases,
                    body_ops,
                );
                *cells = CellTable::merge_paths(&[condition_cells, body_cells]);
                *raw_aliases =
                    RawCellAddressAliases::merge_paths(&[condition_aliases, body_aliases]);
                *function_aliases = FunctionAliasTable::merge_paths(&[
                    condition_function_aliases,
                    body_function_aliases,
                ]);
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                span,
            } => {
                let scrutinee_available = self.consume_by_value(
                    cells,
                    scrutinee,
                    ResourceCheckOperation::MatchScrutinee,
                    *span,
                );
                let mut arms_available = true;
                let mut arm_paths = Vec::new();
                let mut alias_paths = Vec::new();
                let mut function_alias_paths = Vec::new();
                for arm in arms {
                    let mut arm_cells = cells.clone();
                    let mut arm_aliases = raw_aliases.clone();
                    let mut arm_function_aliases = function_aliases.clone();
                    if let Some(bind_local) = &arm.bind_local {
                        arm_cells.mark_initialized(bind_local);
                        if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
                            self.copy_raw_alias_and_rekey_cells(
                                &mut arm_cells,
                                &mut arm_aliases,
                                &source,
                                bind_local,
                            );
                            arm_function_aliases.copy_alias(&source, bind_local);
                        } else {
                            arm_aliases.clear(bind_local);
                        }
                    }
                    self.check_ops(
                        &mut arm_cells,
                        &mut arm_aliases,
                        &mut arm_function_aliases,
                        &arm.ops,
                    );
                    let arm_available = self.consume_by_value(
                        &mut arm_cells,
                        &arm.value,
                        ResourceCheckOperation::MatchValue,
                        arm.span,
                    );
                    arms_available &= arm_available;
                    if arm_available {
                        self.copy_raw_alias_and_rekey_cells(
                            &mut arm_cells,
                            &mut arm_aliases,
                            &arm.value,
                            output,
                        );
                        arm_function_aliases.copy_alias(&arm.value, output);
                    }
                    arm_paths.push(arm_cells);
                    alias_paths.push(arm_aliases);
                    function_alias_paths.push(arm_function_aliases);
                }
                if !arm_paths.is_empty() {
                    *cells = CellTable::merge_paths(&arm_paths);
                    *raw_aliases = RawCellAddressAliases::merge_paths(&alias_paths);
                    *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
                }
                if scrutinee_available && arms_available {
                    cells.mark_initialized(output);
                } else {
                    raw_aliases.clear(output);
                }
            }
        }
    }

    fn apply_call_return_raw_alias(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
    ) -> bool {
        apply_direct_call_raw_alias_summary(
            raw_aliases,
            output,
            target,
            args,
            self.raw_alias_summaries,
            self.types,
        )
    }

    fn apply_indirect_call_return_raw_alias(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &FunctionAliasTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
    ) -> bool {
        apply_indirect_call_raw_alias_summary(
            raw_aliases,
            function_aliases,
            output,
            callee,
            args,
            self.raw_alias_summaries,
            self.types,
        )
    }

    fn copy_raw_alias_and_rekey_cells(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        source: &Place,
        target: &Place,
    ) {
        self.copy_raw_alias_and_rekey_cells_with_mode(cells, raw_aliases, source, target, false);
    }

    fn copy_raw_address_alias_and_rekey_cells(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        source: &Place,
        target: &Place,
    ) {
        self.copy_raw_alias_and_rekey_cells_with_mode(cells, raw_aliases, source, target, true);
    }

    fn copy_raw_alias_and_rekey_cells_with_mode(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        source: &Place,
        target: &Place,
        force_raw_address: bool,
    ) {
        let source_tracks_raw_address = raw_aliases.contains_exact(source);
        let source_canonical = raw_aliases.canonicalize(source);
        let source_aliases = raw_aliases.aliases_for(source);
        let source_is_known_raw_address = force_raw_address
            || source_tracks_raw_address
            || source_canonical != *source
            || source_aliases.len() > 1;
        let source_is_external_raw_storage = source_is_known_raw_address
            && source_aliases
                .iter()
                .any(|alias| cells.external_raw_storage_overlaps(alias));
        raw_aliases.copy_alias_or_seed(source, target);
        let target_canonical = raw_aliases.canonicalize(target);
        if source_is_external_raw_storage {
            for alias in &source_aliases {
                cells.mark_external_raw_storage_root(alias);
            }
            cells.mark_external_raw_storage_root(target);
            cells.mark_external_raw_storage_root(&target_canonical);
        }
        if source_is_known_raw_address {
            cells.rekey_raw_cells(&source_canonical, &target_canonical);
        }
    }

    fn check_expr(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        kind: ResourceExprKind,
        output: &Place,
    ) {
        match kind {
            ResourceExprKind::Literal
            | ResourceExprKind::Block
            | ResourceExprKind::Let
            | ResourceExprKind::Set
            | ResourceExprKind::Intrinsic
            | ResourceExprKind::Deref
            | ResourceExprKind::Drop
            | ResourceExprKind::Loop => cells.mark_initialized(output),
            ResourceExprKind::LocalRead
            | ResourceExprKind::FunctionValue
            | ResourceExprKind::Call
            | ResourceExprKind::IndirectCall
            | ResourceExprKind::Branch
            | ResourceExprKind::Match
            | ResourceExprKind::Construct
            | ResourceExprKind::Borrow => {}
        }
        if !expr_kind_preserves_raw_alias(kind)
            && !(matches!(kind, ResourceExprKind::Deref)
                && type_preserves_raw_address_alias(self.types, output.ty))
        {
            raw_aliases.clear(output);
        }
    }

    pub(super) fn ensure_no_live_non_copy_raw_cells(
        &mut self,
        cells: &CellTable,
        address: &Place,
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        let conflicts = cells.live_non_copy_raw_cells_under(address, self.types);
        for conflict in &conflicts {
            self.push_unavailable(operation, &conflict.place, conflict.state.clone(), span);
        }
        conflicts.is_empty()
    }

    pub(super) fn ensure_args(
        &mut self,
        cells: &mut CellTable,
        args: &[Place],
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        let mut available = true;
        for arg in args {
            available &= self.ensure_available(cells, arg, operation, span);
        }
        available
    }

    fn consume_args(
        &mut self,
        cells: &mut CellTable,
        args: &[Place],
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        let mut available = true;
        for arg in args {
            available &= self.consume_by_value(cells, arg, operation, span);
        }
        available
    }

    pub(super) fn consume_by_value(
        &mut self,
        cells: &mut CellTable,
        place: &Place,
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        if !self.ensure_available(cells, place, operation, span) {
            return false;
        }
        if should_track(place) && !self.types.is_copy(place.ty) {
            cells.set_state(place, CellState::Moved);
        }
        true
    }

    pub(super) fn ensure_available(
        &mut self,
        cells: &CellTable,
        place: &Place,
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        if !should_track(place) {
            return true;
        }
        match cells.availability_state(place) {
            CellState::Initialized(_) => true,
            state => {
                self.push_unavailable(operation, place, state, span);
                false
            }
        }
    }

    fn push_unavailable(
        &mut self,
        operation: ResourceCheckOperation,
        place: &Place,
        state: CellState,
        span: Span,
    ) {
        self.diagnostics
            .push(ResourceCheckDiagnostic::CellUnavailable {
                function: String::from(self.function),
                operation,
                place: place.clone(),
                state,
                span,
            });
    }
}

fn merge_deferred(target: &mut ResourceCheckDeferred, source: ResourceCheckDeferred) {
    target.branch_merges += source.branch_merges;
    target.loop_merges += source.loop_merges;
    target.match_merges += source.match_merges;
}

fn type_preserves_raw_address_alias(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { name, .. } => name == "MemPtr" || name == "RegionToken",
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(
                types.get_ref(base),
                TypeKind::Struct { name, .. } if name == "MemPtr" || name == "RegionToken"
            )
        }
        _ => false,
    }
}
