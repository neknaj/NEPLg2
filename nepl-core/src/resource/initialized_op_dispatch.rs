extern crate alloc;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::drop_model::ResourceDropPoint;
use super::drop_point_path::{ResourceDropPointPath, ResourceDropPointStep};
use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::construct_raw_cell_address_alias_fields;
use super::initialized_drop_scope::auto_drop_scope_locals_with_record;
use super::initialized_str_layout::seed_str_storage_layout;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{CellState, ResourceOp};
use super::place_utils::{
    construct_aggregate_field_place, reference_target_place,
    structural_i32_projection_preserves_raw_address,
};
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckOperation;

/// ResourceOp ごとの initialized-state dispatch を受け持つ。
///
/// `ResourceCheckEngine` 本体は関数単位の state 構築、path alternative、summary 連携を
/// 管理する。個別 op の状態遷移は分岐が多いため、この module に分けて責務の境界を保つ。
impl ResourceCheckEngine<'_> {
    pub(super) fn check_op(
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
        if super::initialized_collection_slot_dispatch::check_initialized_collection_slot_op(
            self,
            cells,
            collection_slots,
            raw_aliases,
            pending_reallocs,
            op,
        ) {
            return;
        }
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
                        cells.transfer_raw_cell_loaded_value_origin(initializer, place);
                        self.transfer_slot_state_if_moved_with_aliases(
                            cells,
                            collection_slots,
                            initializer,
                            place,
                            raw_aliases,
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
                    cells.transfer_raw_cell_loaded_value_origin(source, output);
                    self.transfer_slot_state_if_moved_with_aliases(
                        cells,
                        collection_slots,
                        source,
                        output,
                        raw_aliases,
                        *span,
                    );
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
                    cells.transfer_raw_cell_loaded_value_origin(value, target);
                    self.transfer_slot_state_if_moved_with_aliases(
                        cells,
                        collection_slots,
                        value,
                        target,
                        raw_aliases,
                        *span,
                    );
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
                    cells.transfer_raw_cell_loaded_value_origin(source, output);
                    self.transfer_slot_state_with_aliases(
                        collection_slots,
                        source,
                        output,
                        raw_aliases,
                        *span,
                    );
                    function_aliases.copy_alias(source, output);
                    pending_reallocs.copy_result(source, output);
                    variant_initializations.copy_result(source, output);
                    seed_str_storage_layout(self.types, cells, raw_aliases, output);
                }
            }
            ResourceOp::Drop { place, span } => {
                if self.ensure_available(cells, place, ResourceCheckOperation::Drop, *span) {
                    cells.record_raw_cell_loaded_value_drop(place, self.types);
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
            ResourceOp::FunctionValue {
                output, identity, ..
            } => {
                cells.mark_initialized(output);
                raw_aliases.clear(output);
                function_aliases.set_alias(output, identity.clone());
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
                function_aliases,
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
                collection_slots,
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
                let target_was_initialized = matches!(
                    cells.availability_state_with_types(self.types, target),
                    CellState::Initialized(_)
                );
                if self.raw_address_view_source_is_known(cells, raw_aliases, source) {
                    self.copy_raw_address_alias_and_rekey_cells(cells, raw_aliases, source, target);
                } else {
                    raw_aliases.record_raw_address_view_origin(source, target);
                }
                if target_was_initialized {
                    // RawAddressView は、直前の call や intrinsic が生成した値に
                    // raw address の別名関係を付与するための証明操作であり、
                    // 値そのものの初期化状態を上書きしてはならない。
                    cells.set_state(target, CellState::Initialized(target.ty));
                }
                pending_reallocs.clear_result(target);
                variant_initializations.clear_result(target);
            }
            ResourceOp::StorageOrigin { .. } => {}
            ResourceOp::CollectionSlotLifecycle { .. }
            | ResourceOp::CollectionStorageRelocate { .. }
            | ResourceOp::CollectionSlotDropTraversal { .. }
            | ResourceOp::CollectionSlotTransformRange { .. } => {}
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
                        cells.transfer_raw_cell_loaded_value_origin(input, &field);
                        self.transfer_slot_state_if_moved_with_aliases(
                            cells,
                            collection_slots,
                            input,
                            &field,
                            raw_aliases,
                            *span,
                        );
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
}
