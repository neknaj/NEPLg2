extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::collection_slot_state_table::{CollectionSlotStateEntry, CollectionSlotStateTable};
use super::collection_slot_summary_build::compute_collection_slot_lifecycle_function_summaries;
use super::collection_slot_summary_model::CollectionSlotLifecycleFunctionSummaryIndex;
use super::drop_model::ResourceDropPoint;
use super::drop_point_path::{ResourceDropPointPath, ResourceDropPointStep};
use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::{
    apply_direct_call_raw_alias_summary, apply_indirect_call_raw_alias_summary,
    compute_raw_cell_address_return_summaries, construct_raw_cell_address_alias_fields,
    expr_kind_preserves_raw_alias, RawCellAddressReturnSummaryIndex,
};
use super::initialized_drop_scope::auto_drop_scope_locals_with_record;
use super::initialized_scalar_flow::{
    compute_i32_scalar_return_summaries, I32ScalarReturnSummaryIndex,
};
use super::initialized_str_layout::seed_str_storage_layout;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::initialized_summary_build::compute_raw_cell_initialization_function_summaries;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{
    CellState, CellStateEntry, Place, ResourceBlock, ResourceCallTarget, ResourceExprKind,
    ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::place_utils::{
    construct_aggregate_field_place, reference_target_place,
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
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(&raw_alias_summaries);
    stage_start.log("resource_initialized_raw_alias_summaries");
    let stage_start = ResourceStageTimer::start();
    let i32_scalar_summaries =
        compute_i32_scalar_return_summaries(module, types, &raw_alias_summary_index);
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(&i32_scalar_summaries);
    stage_start.log("resource_initialized_i32_scalar_summaries");
    let stage_start = ResourceStageTimer::start();
    let raw_init_summaries = compute_raw_cell_initialization_function_summaries(
        module,
        types,
        &raw_alias_summaries,
        &i32_scalar_summaries,
    );
    let raw_init_summary_index =
        RawCellInitializationFunctionSummaryIndex::new(&raw_init_summaries);
    stage_start.log("resource_initialized_raw_init_summaries");
    let stage_start = ResourceStageTimer::start();
    let collection_slot_summaries = compute_collection_slot_lifecycle_function_summaries(
        module,
        types,
        &raw_alias_summaries,
        &i32_scalar_summaries,
        &raw_init_summaries,
    );
    let collection_slot_summary_index =
        CollectionSlotLifecycleFunctionSummaryIndex::new(&collection_slot_summaries);
    stage_start.log("resource_initialized_collection_slot_summaries");
    let stage_start = ResourceStageTimer::start();

    for function in &module.functions {
        let mut engine = ResourceCheckEngine {
            function: function.name.as_str(),
            types,
            raw_alias_summaries: &raw_alias_summary_index,
            i32_scalar_summaries: &i32_scalar_summary_index,
            raw_init_summaries: &raw_init_summary_index,
            collection_slot_summaries: &collection_slot_summary_index,
            diagnostics: Vec::new(),
            auto_drop_points: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
        };
        let (final_cells, final_collection_slots) = engine.check_function(function);
        merge_deferred(&mut deferred, engine.deferred);
        diagnostics.extend(engine.diagnostics);
        functions.push(ResourceFunctionCheck {
            name: function.name.clone(),
            final_cells,
            final_collection_slots,
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
    pub(super) raw_alias_summaries: &'a RawCellAddressReturnSummaryIndex<'a>,
    pub(super) i32_scalar_summaries: &'a I32ScalarReturnSummaryIndex<'a>,
    pub(super) raw_init_summaries: &'a RawCellInitializationFunctionSummaryIndex<'a>,
    pub(super) collection_slot_summaries: &'a CollectionSlotLifecycleFunctionSummaryIndex<'a>,
    pub(super) diagnostics: Vec<ResourceCheckDiagnostic>,
    pub(super) auto_drop_points: Vec<ResourceDropPoint>,
    pub(super) deferred: ResourceCheckDeferred,
}

impl ResourceCheckEngine<'_> {
    fn check_function(
        &mut self,
        function: &ResourceFunction,
    ) -> (Vec<CellStateEntry>, Vec<CollectionSlotStateEntry>) {
        let mut cells = CellTable::default();
        let mut collection_slots = CollectionSlotStateTable::new();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut function_aliases = FunctionAliasTable::default();
        let mut pending_reallocs = PendingRawReallocs::default();
        let mut variant_initializations = PendingVariantRawCellInitializations::default();
        for param in &function.params {
            cells.mark_initialized(&param.place);
            self.seed_external_raw_storage_parameter(&mut cells, &mut raw_aliases, &param.place);
            seed_str_storage_layout(self.types, &mut cells, &mut raw_aliases, &param.place);
            if let Some(target_ty) = self.reference_target_type(param.place.ty) {
                let target = reference_target_place(&param.place, target_ty);
                cells.mark_initialized(&target);
                self.seed_external_raw_storage_parameter(&mut cells, &mut raw_aliases, &target);
                seed_str_storage_layout(self.types, &mut cells, &mut raw_aliases, &target);
            }
        }
        for block in &function.blocks {
            self.check_block(
                &mut cells,
                &mut collection_slots,
                &mut raw_aliases,
                &mut function_aliases,
                &mut pending_reallocs,
                &mut variant_initializations,
                block,
            );
        }
        (cells.into_entries(), collection_slots.entries().to_vec())
    }

    fn check_block(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        block: &ResourceBlock,
    ) {
        self.check_ops(
            cells,
            collection_slots,
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
        collection_slots: &mut CollectionSlotStateTable,
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
                collection_slots,
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
        collection_slots: &mut CollectionSlotStateTable,
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
                        cells.copy_initialized_raw_byte_ranges_through_value_aliases(
                            initializer,
                            place,
                            raw_aliases,
                        );
                        self.transfer_slot_state_if_moved(
                            collection_slots,
                            initializer,
                            place,
                            *span,
                        );
                        function_aliases.copy_alias(initializer, place);
                        pending_reallocs.copy_result(initializer, place);
                        variant_initializations.copy_result(initializer, place);
                        seed_str_storage_layout(self.types, cells, raw_aliases, place);
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
                    cells.copy_initialized_raw_byte_ranges_through_value_aliases(
                        source,
                        output,
                        raw_aliases,
                    );
                    self.transfer_slot_state_if_moved(collection_slots, source, output, *span);
                    function_aliases.copy_alias(source, output);
                    pending_reallocs.copy_result(source, output);
                    variant_initializations.copy_result(source, output);
                    seed_str_storage_layout(self.types, cells, raw_aliases, output);
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
                    cells.copy_initialized_raw_byte_ranges_through_value_aliases(
                        value,
                        target,
                        raw_aliases,
                    );
                    self.transfer_slot_state_if_moved(collection_slots, value, target, *span);
                    function_aliases.copy_alias(value, target);
                    pending_reallocs.copy_result(value, target);
                    variant_initializations.copy_result(value, target);
                    seed_str_storage_layout(self.types, cells, raw_aliases, target);
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
                    seed_str_storage_layout(self.types, cells, raw_aliases, &target);
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
                    cells.copy_initialized_raw_byte_ranges_through_value_aliases(
                        source,
                        output,
                        raw_aliases,
                    );
                    self.transfer_slot_state(collection_slots, source, output, *span);
                    function_aliases.copy_alias(source, output);
                    pending_reallocs.copy_result(source, output);
                    variant_initializations.copy_result(source, output);
                    seed_str_storage_layout(self.types, cells, raw_aliases, output);
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
            } => self.check_direct_call(
                cells,
                collection_slots,
                raw_aliases,
                pending_reallocs,
                variant_initializations,
                output,
                target,
                args,
                effect,
                *span,
            ),
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                span,
                ..
            } => self.check_indirect_call(
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
                output,
                callee,
                args,
                *span,
            ),
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
            ResourceOp::StorageOrigin { .. } => {}
            ResourceOp::CollectionSlotLifecycle {
                target,
                event,
                span,
            } => {
                self.apply_collection_slot_lifecycle_with_aliases(
                    cells,
                    collection_slots,
                    raw_aliases,
                    target,
                    *event,
                    *span,
                );
            }
            ResourceOp::CollectionStorageRelocate {
                old_storage,
                new_storage,
                span,
            } => {
                self.apply_collection_storage_relocate_with_aliases(
                    collection_slots,
                    raw_aliases,
                    old_storage,
                    new_storage,
                    *span,
                );
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
                        cells.copy_initialized_raw_byte_ranges_through_value_aliases(
                            input,
                            &field,
                            raw_aliases,
                        );
                        self.transfer_slot_state_if_moved(collection_slots, input, &field, *span);
                    }
                    construct_function_alias_fields(function_aliases, output, kind, inputs);
                    seed_str_storage_layout(self.types, cells, raw_aliases, output);
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
                    collection_slots,
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
                    collection_slots,
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
                scrutinee_is_borrow_target,
                arms,
                span,
                ..
            } => {
                self.check_match(
                    cells,
                    collection_slots,
                    raw_aliases,
                    function_aliases,
                    pending_reallocs,
                    variant_initializations,
                    output,
                    scrutinee,
                    *scrutinee_is_borrow_target,
                    arms,
                    *span,
                    path,
                );
            }
        }
    }

    pub(super) fn apply_call_return_raw_alias(
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

    pub(super) fn apply_indirect_call_return_raw_alias(
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
            ResourceExprKind::LayoutSizeOf(ty) => {
                cells.mark_initialized(output);
                raw_aliases.set_i32_type_size(output, ty);
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
        if !matches!(
            kind,
            ResourceExprKind::LiteralI32(_) | ResourceExprKind::LayoutSizeOf(_)
        ) && !expr_kind_preserves_raw_alias(kind)
            && !(matches!(kind, ResourceExprKind::Deref)
                && type_preserves_raw_address_alias(self.types, output.ty))
        {
            raw_aliases.clear(output);
        }
        seed_str_storage_layout(self.types, cells, raw_aliases, output);
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
