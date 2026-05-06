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
use super::initialized_drop_scope::auto_drop_scope_locals;
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary_build::compute_raw_cell_initialization_function_summaries;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{
    CellState, CellStateEntry, EffectOp, Place, ResourceBlock, ResourceCallTarget,
    ResourceExprKind, ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::place_utils::{reference_target_place, should_track, type_preserves_raw_address_alias};
use super::raw_realloc::PendingRawReallocs;
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
    let raw_init_summaries =
        compute_raw_cell_initialization_function_summaries(module, types, &raw_alias_summaries);

    for function in &module.functions {
        let mut engine = ResourceCheckEngine {
            function: function.name.as_str(),
            types,
            raw_alias_summaries: &raw_alias_summaries,
            raw_init_summaries: &raw_init_summaries,
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
    pub(super) function: &'a str,
    pub(super) types: &'a TypeCtx,
    pub(super) raw_alias_summaries: &'a [RawCellAddressReturnSummary],
    pub(super) raw_init_summaries: &'a [RawCellInitializationFunctionSummary],
    pub(super) diagnostics: Vec<ResourceCheckDiagnostic>,
    pub(super) deferred: ResourceCheckDeferred,
}

impl ResourceCheckEngine<'_> {
    fn check_function(&mut self, function: &ResourceFunction) -> Vec<CellStateEntry> {
        let mut cells = CellTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut function_aliases = FunctionAliasTable::default();
        let mut pending_reallocs = PendingRawReallocs::default();
        let mut variant_initializations = PendingVariantRawCellInitializations::default();
        for param in &function.params {
            cells.mark_initialized(&param.place);
            cells.mark_external_raw_storage_root(&param.place);
            raw_aliases.mark(&param.place);
            if let Some(target_ty) = self.reference_target_type(param.place.ty) {
                let target = reference_target_place(&param.place, target_ty);
                cells.mark_initialized(&target);
                cells.mark_external_raw_storage_root(&target);
                raw_aliases.mark(&target);
            }
        }
        for block in &function.blocks {
            self.check_block(
                &mut cells,
                &mut raw_aliases,
                &mut function_aliases,
                &mut pending_reallocs,
                &mut variant_initializations,
                block,
            );
        }
        cells.into_entries()
    }

    fn check_block(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        block: &ResourceBlock,
    ) {
        self.check_ops(
            cells,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
            &block.ops,
        );
        match &block.terminator {
            ResourceTerminator::Return { value, span } => {
                if let Some(value) = value {
                    self.consume_by_value(cells, value, ResourceCheckOperation::ReturnValue, *span);
                }
            }
            ResourceTerminator::Unreachable { .. } | ResourceTerminator::RawBody { .. } => {}
        }
    }

    pub(super) fn check_ops(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        ops: &[ResourceOp],
    ) {
        for op in ops {
            self.check_op(
                cells,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
                op,
            );
        }
    }

    fn check_op(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
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
                        self.copy_raw_alias_and_rekey_cells_preferring_target(
                            cells,
                            raw_aliases,
                            initializer,
                            place,
                        );
                        function_aliases.copy_alias(initializer, place);
                        pending_reallocs.copy_result(initializer, place);
                        variant_initializations.copy_result(initializer, place);
                    } else {
                        cells.set_state(place, CellState::Uninit);
                        raw_aliases.clear(place);
                        pending_reallocs.clear_result(place);
                        variant_initializations.clear_result(place);
                    }
                } else {
                    cells.set_state(place, CellState::Uninit);
                    raw_aliases.clear(place);
                    pending_reallocs.clear_result(place);
                    variant_initializations.clear_result(place);
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
                    pending_reallocs.copy_result(source, output);
                    variant_initializations.copy_result(source, output);
                }
            }
            ResourceOp::Assign {
                target,
                value,
                span,
            } => {
                if self.consume_by_value(cells, value, ResourceCheckOperation::AssignValue, *span) {
                    cells.mark_initialized(target);
                    self.copy_raw_alias_and_rekey_cells_preferring_target(
                        cells,
                        raw_aliases,
                        value,
                        target,
                    );
                    function_aliases.copy_alias(value, target);
                    pending_reallocs.copy_result(value, target);
                    variant_initializations.copy_result(value, target);
                } else {
                    raw_aliases.clear(target);
                    pending_reallocs.clear_result(target);
                    variant_initializations.clear_result(target);
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
                    raw_aliases.mark(output);
                    let target = reference_target_place(output, source.ty);
                    cells.mark_initialized(&target);
                    self.copy_raw_alias_and_rekey_cells(cells, raw_aliases, source, &target);
                    pending_reallocs.clear_result(output);
                    variant_initializations.clear_result(output);
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
                    self.copy_raw_alias_and_rekey_cells_preferring_target(
                        cells,
                        raw_aliases,
                        source,
                        output,
                    );
                    function_aliases.copy_alias(source, output);
                    pending_reallocs.copy_result(source, output);
                    variant_initializations.copy_result(source, output);
                }
            }
            ResourceOp::Drop { place, span } => {
                if self.ensure_available(cells, place, ResourceCheckOperation::Drop, *span) {
                    cells.set_state(place, CellState::Dropped);
                    raw_aliases.clear(place);
                    pending_reallocs.clear_result(place);
                    variant_initializations.clear_result(place);
                }
            }
            ResourceOp::EndScope { locals, span, .. } => auto_drop_scope_locals(
                self.types,
                cells,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
                locals,
                *span,
            ),
            ResourceOp::CallEffect { .. } => {}
            ResourceOp::FunctionValue { output, name, .. } => {
                cells.mark_initialized(output);
                raw_aliases.clear(output);
                function_aliases.set_alias(output, name.clone());
                pending_reallocs.clear_result(output);
                variant_initializations.clear_result(output);
            }
            ResourceOp::Call {
                output,
                target,
                args,
                effect,
                span,
                ..
            } => {
                if matches!(effect, EffectOp::InternalAlloc { .. })
                    || (matches!(effect, EffectOp::UnsafeMemory { .. })
                        && !call_uses_checked_mem_ptr_wrapper(self.types, args))
                {
                    pending_reallocs.clear_result(output);
                    variant_initializations.clear_result(output);
                    return;
                }
                let args_available =
                    self.consume_args(cells, args, ResourceCheckOperation::CallArgument, *span);
                if args_available {
                    let external_inputs_available = self.ensure_external_io_initialized_inputs(
                        cells,
                        raw_aliases,
                        effect,
                        args,
                        *span,
                    );
                    if !external_inputs_available {
                        raw_aliases.clear(output);
                        pending_reallocs.clear_result(output);
                        variant_initializations.clear_result(output);
                        return;
                    }
                    cells.mark_initialized(output);
                    self.apply_external_io_initialized_effect(cells, raw_aliases, effect, args);
                    if !self.apply_call_return_raw_alias(raw_aliases, output, target, args) {
                        raw_aliases.clear(output);
                    }
                    let release_requirements_ok = self.apply_call_raw_cell_initialization_summary(
                        cells,
                        raw_aliases,
                        variant_initializations,
                        output,
                        target,
                        args,
                        *span,
                    );
                    if !release_requirements_ok {
                        raw_aliases.clear(output);
                        pending_reallocs.clear_result(output);
                        variant_initializations.clear_result(output);
                    }
                    pending_reallocs.clear_result(output);
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
                    let release_requirements_ok = self
                        .apply_indirect_call_raw_cell_initialization_summary(
                            cells,
                            raw_aliases,
                            variant_initializations,
                            output,
                            function_aliases,
                            callee,
                            args,
                            *span,
                        );
                    if !release_requirements_ok {
                        raw_aliases.clear(output);
                        pending_reallocs.clear_result(output);
                        variant_initializations.clear_result(output);
                    }
                    pending_reallocs.clear_result(output);
                }
            }
            ResourceOp::RawMemory {
                operation,
                output,
                args,
                span,
            } => self.check_raw_memory(
                cells,
                raw_aliases,
                pending_reallocs,
                operation,
                output,
                args,
                *span,
            ),
            ResourceOp::RawAddressAlias { source, target, .. } => {
                self.copy_raw_address_alias_and_rekey_cells(cells, raw_aliases, source, target);
                pending_reallocs.copy_result(source, target);
                variant_initializations.copy_result(source, target);
            }
            ResourceOp::RawAddressView { source, target, .. } => {
                self.copy_raw_address_alias_and_rekey_cells(cells, raw_aliases, source, target);
                pending_reallocs.clear_result(target);
                variant_initializations.clear_result(target);
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
                    pending_reallocs.clear_result(output);
                    variant_initializations.clear_result(output);
                }
            }
            ResourceOp::Branch {
                output,
                condition,
                condition_fact,
                then_ops,
                then_value,
                else_ops,
                else_value,
                span,
            } => {
                self.check_branch(
                    cells,
                    raw_aliases,
                    function_aliases,
                    pending_reallocs,
                    variant_initializations,
                    output,
                    condition,
                    condition_fact.as_ref(),
                    then_ops,
                    then_value,
                    else_ops,
                    else_value,
                    *span,
                );
            }
            ResourceOp::Loop {
                condition_ops,
                condition,
                body_ops,
                span,
            } => {
                self.check_loop(
                    cells,
                    raw_aliases,
                    function_aliases,
                    pending_reallocs,
                    variant_initializations,
                    condition_ops,
                    condition,
                    body_ops,
                    *span,
                );
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                span,
            } => {
                self.check_match(
                    cells,
                    raw_aliases,
                    function_aliases,
                    pending_reallocs,
                    variant_initializations,
                    output,
                    scrutinee,
                    arms,
                    *span,
                );
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

    fn check_expr(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        kind: ResourceExprKind,
        output: &Place,
    ) {
        match kind {
            ResourceExprKind::LiteralI32(value) => {
                cells.mark_initialized(output);
                raw_aliases.set_i32_value(output, value);
            }
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
        if !matches!(kind, ResourceExprKind::LiteralI32(_))
            && !expr_kind_preserves_raw_alias(kind)
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
        match cells.availability_state_with_types(self.types, place) {
            CellState::Initialized(_) => true,
            state => {
                self.push_unavailable(operation, place, state, span);
                false
            }
        }
    }

    fn reference_target_type(&self, ty: TypeId) -> Option<TypeId> {
        let resolved = self.types.resolve_named_type_id(self.types.resolve_id(ty));
        match self.types.get_ref(resolved) {
            TypeKind::Reference(target, _) => Some(*target),
            _ => None,
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

fn call_uses_checked_mem_ptr_wrapper(types: &TypeCtx, args: &[Place]) -> bool {
    args.first()
        .map(|arg| is_mem_ptr_type(types, arg.ty))
        .unwrap_or(false)
}

fn is_mem_ptr_type(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { name, .. } => name == "MemPtr",
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(types.get_ref(base), TypeKind::Struct { name, .. } if name == "MemPtr")
        }
        _ => false,
    }
}
