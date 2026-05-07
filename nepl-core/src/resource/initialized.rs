extern crate alloc;

use alloc::vec::Vec;

use crate::runtime_helpers::helper_base_name;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::drop_model::ResourceDropPoint;
use super::drop_point_path::{ResourceDropPointPath, ResourceDropPointStep};
use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::{
    apply_direct_call_raw_alias_summary, apply_indirect_call_raw_alias_summary,
    compute_raw_cell_address_return_summaries, construct_raw_cell_address_alias_fields,
    expr_kind_preserves_raw_alias, RawCellAddressReturnSummary,
};
use super::initialized_drop_scope::auto_drop_scope_locals_with_record;
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary_build::compute_raw_cell_initialization_function_summaries;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{
    CellState, CellStateEntry, EffectOp, Place, ResourceBlock, ResourceCallTarget,
    ResourceExprKind, ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::place_utils::{
    call_uses_checked_mem_ptr_wrapper, construct_aggregate_field_place, reference_target_place,
    structural_i32_projection_preserves_raw_address, type_preserves_raw_address_alias,
};
use super::raw_realloc::PendingRawReallocs;
use super::report::{
    ResourceCheckDeferred, ResourceCheckDiagnostic, ResourceCheckOperation, ResourceCheckReport,
    ResourceFunctionCheck,
};
use super::timing::ResourceStageTimer;

pub fn check_resource_initialized_moves(
    module: &ResourceModule,
    types: &TypeCtx,
) -> ResourceCheckReport {
    let stage_start = ResourceStageTimer::start();
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceCheckDeferred::default();
    let raw_alias_summaries = compute_raw_cell_address_return_summaries(module, types);
    stage_start.log("resource_initialized_raw_alias_summaries");
    let stage_start = ResourceStageTimer::start();
    let raw_init_summaries =
        compute_raw_cell_initialization_function_summaries(module, types, &raw_alias_summaries);
    stage_start.log("resource_initialized_raw_init_summaries");
    let stage_start = ResourceStageTimer::start();

    for function in &module.functions {
        let mut engine = ResourceCheckEngine {
            function: function.name.as_str(),
            types,
            raw_alias_summaries: &raw_alias_summaries,
            raw_init_summaries: &raw_init_summaries,
            diagnostics: Vec::new(),
            auto_drop_points: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
        };
        let final_cells = engine.check_function(function);
        merge_deferred(&mut deferred, engine.deferred);
        diagnostics.extend(engine.diagnostics);
        functions.push(ResourceFunctionCheck {
            name: function.name.clone(),
            final_cells,
            auto_drop_points: engine.auto_drop_points,
            deferred: engine.deferred,
        });
    }
    stage_start.log("resource_initialized_function_checks");

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
    pub(super) auto_drop_points: Vec<ResourceDropPoint>,
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
            ResourceDropPointPath {
                block: block.id,
                steps: Vec::new(),
            },
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
        path: ResourceDropPointPath,
    ) {
        for (index, op) in ops.iter().enumerate() {
            self.check_op(
                cells,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
                op,
                path.clone().with_step(ResourceDropPointStep::Op { index }),
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
        path: ResourceDropPointPath,
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
                        cells.copy_initialized_raw_byte_ranges_through_value(initializer, place);
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
                    if structural_i32_projection_preserves_raw_address(self.types, source, output) {
                        self.copy_raw_address_alias_and_rekey_cells(
                            cells,
                            raw_aliases,
                            source,
                            output,
                        );
                    } else {
                        self.copy_raw_alias_and_rekey_cells(cells, raw_aliases, source, output);
                    }
                    cells.copy_initialized_raw_byte_ranges_through_value(source, output);
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
                self.record_assignment_overwrite_drop(
                    cells,
                    raw_aliases,
                    function_aliases,
                    pending_reallocs,
                    variant_initializations,
                    target,
                    path,
                    *span,
                );
                if self.consume_by_value(cells, value, ResourceCheckOperation::AssignValue, *span) {
                    cells.mark_initialized(target);
                    cells.clear_initialized_raw_byte_ranges_through_value(target);
                    self.copy_raw_alias_and_rekey_cells_preferring_target(
                        cells,
                        raw_aliases,
                        value,
                        target,
                    );
                    cells.copy_initialized_raw_byte_ranges_through_value(value, target);
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
                    cells.copy_initialized_raw_byte_ranges_through_value(source, output);
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
            ResourceOp::EndScope { locals, span, .. } => {
                let auto_drops = auto_drop_scope_locals_with_record(
                    self.types,
                    cells,
                    raw_aliases,
                    function_aliases,
                    pending_reallocs,
                    variant_initializations,
                    locals,
                    *span,
                );
                if !auto_drops.is_empty() {
                    self.auto_drop_points.push(ResourceDropPoint {
                        path,
                        span: *span,
                        auto_drops,
                    });
                }
            }
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
                    } else {
                        self.record_i32_scale_result(raw_aliases, target, output, args);
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
                if self.raw_address_view_source_is_known(cells, raw_aliases, source) {
                    self.copy_raw_address_alias_and_rekey_cells(cells, raw_aliases, source, target);
                } else {
                    raw_aliases.record_raw_address_view_origin(source, target);
                }
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
                    for (index, input) in inputs.iter().enumerate() {
                        let field = construct_aggregate_field_place(output, kind, index, input);
                        cells.copy_initialized_raw_byte_ranges_through_value(input, &field);
                    }
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
                    path.clone().with_step(ResourceDropPointStep::BranchThen),
                    path.with_step(ResourceDropPointStep::BranchElse),
                );
            }
            ResourceOp::Loop {
                condition_ops,
                condition,
                condition_fact,
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
                    condition_fact.as_ref(),
                    body_ops,
                    *span,
                    path.clone().with_step(ResourceDropPointStep::LoopCondition),
                    path.with_step(ResourceDropPointStep::LoopBody),
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
                    path,
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

    fn record_i32_scale_result(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        target: &ResourceCallTarget,
        output: &Place,
        args: &[Place],
    ) {
        if resource_call_target_base_name(target) != Some("mul") {
            return;
        }
        let [left, right] = args else {
            return;
        };
        if let Some(scale) = positive_i32_value_as_usize(raw_aliases, left) {
            raw_aliases.add_i32_scale(right, output, scale);
        } else if let Some(scale) = positive_i32_value_as_usize(raw_aliases, right) {
            raw_aliases.add_i32_scale(left, output, scale);
        }
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

    fn reference_target_type(&self, ty: TypeId) -> Option<TypeId> {
        let resolved = self.types.resolve_named_type_id(self.types.resolve_id(ty));
        match self.types.get_ref(resolved) {
            TypeKind::Reference(target, _) => Some(*target),
            _ => None,
        }
    }
}

fn merge_deferred(target: &mut ResourceCheckDeferred, source: ResourceCheckDeferred) {
    target.branch_merges += source.branch_merges;
    target.loop_merges += source.loop_merges;
    target.match_merges += source.match_merges;
}

fn resource_call_target_base_name(target: &ResourceCallTarget) -> Option<&str> {
    match target {
        ResourceCallTarget::Builtin { name } | ResourceCallTarget::User { name, .. } => {
            Some(helper_base_name(name))
        }
        ResourceCallTarget::Trait { method, .. } => Some(helper_base_name(method)),
    }
}

fn positive_i32_value_as_usize(
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
) -> Option<usize> {
    let value = raw_aliases.i32_value(place)?;
    usize::try_from(value).ok().filter(|value| *value > 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::String;

    use super::super::model::ResourceId;

    #[test]
    fn records_i32_scale_result_for_mangled_mul_call() {
        let types = TypeCtx::new();
        let raw_alias_summaries = Vec::new();
        let raw_init_summaries = Vec::new();
        let engine = ResourceCheckEngine {
            function: "test",
            types: &types,
            raw_alias_summaries: &raw_alias_summaries,
            raw_init_summaries: &raw_init_summaries,
            diagnostics: Vec::new(),
            auto_drop_points: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
        };
        let source = Place::local(String::from("i"), types.i32());
        let source_read = Place::temporary(ResourceId(1), types.i32());
        let constant = Place::temporary(ResourceId(2), types.i32());
        let output = Place::temporary(ResourceId(3), types.i32());
        let mut raw_aliases = RawCellAddressAliases::default();

        raw_aliases.copy_alias_if_tracked(&source, &source_read);
        raw_aliases.set_i32_value(&constant, 4);
        engine.record_i32_scale_result(
            &mut raw_aliases,
            &ResourceCallTarget::User {
                name: String::from("mul__i32_i32__i32__pure"),
                type_args: Vec::new(),
            },
            &output,
            &[source_read, constant],
        );

        assert_eq!(raw_aliases.i32_scaled_source(&output), Some((source, 4)));
    }
}
