use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::function_alias::FunctionAliasTable;
use super::i32_call_facts::record_direct_call_i32_facts;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_scalar_flow::apply_direct_call_i32_scalar_summary;
use super::initialized_str_layout::seed_str_storage_layout;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{EffectOp, Place, ResourceCallTarget};
use super::place_utils::call_uses_checked_mem_ptr_wrapper;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckOperation;

impl ResourceCheckEngine<'_> {
    pub(super) fn check_direct_call(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &mut RawCellAddressAliases,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
        effect: &EffectOp,
        span: crate::span::Span,
    ) {
        if matches!(effect, EffectOp::InternalAlloc { .. })
            || (matches!(effect, EffectOp::UnsafeMemory { .. })
                && !call_uses_checked_mem_ptr_wrapper(self.types, args))
        {
            pending_reallocs.clear_result(output);
            variant_initializations.clear_result(output);
            return;
        }
        if !self.consume_args(cells, args, ResourceCheckOperation::CallArgument, span) {
            return;
        }
        for arg in args {
            cells.discard_raw_cell_loaded_value_origin(arg);
        }
        let external_inputs_available =
            self.ensure_external_io_initialized_inputs(cells, raw_aliases, effect, args, span);
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
        apply_direct_call_i32_scalar_summary(
            raw_aliases,
            output,
            target,
            args,
            self.i32_scalar_summaries,
            self.types,
        );
        let release_requirements_ok = self.apply_call_raw_cell_initialization_summary(
            cells,
            raw_aliases,
            variant_initializations,
            output,
            target,
            args,
            span,
        );
        if !release_requirements_ok {
            raw_aliases.clear(output);
            pending_reallocs.clear_result(output);
            variant_initializations.clear_result(output);
        } else {
            record_direct_call_i32_facts(raw_aliases, target, output, args);
        }
        self.apply_call_collection_slot_lifecycle_summary(
            cells,
            collection_slots,
            raw_aliases,
            output,
            target,
            args,
            span,
        );
        seed_str_storage_layout(self.types, cells, raw_aliases, output);
        pending_reallocs.clear_result(output);
    }

    pub(super) fn check_indirect_call(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        output: &Place,
        callee: &Place,
        args: &[Place],
        span: crate::span::Span,
    ) {
        let callee_available =
            self.ensure_available(cells, callee, ResourceCheckOperation::IndirectCallee, span);
        let args_available =
            self.consume_args(cells, args, ResourceCheckOperation::CallArgument, span);
        if !(callee_available && args_available) {
            return;
        }
        for arg in args {
            cells.discard_raw_cell_loaded_value_origin(arg);
        }
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
        let release_requirements_ok = self.apply_indirect_call_raw_cell_initialization_summary(
            cells,
            raw_aliases,
            variant_initializations,
            output,
            function_aliases,
            callee,
            args,
            span,
        );
        if !release_requirements_ok {
            raw_aliases.clear(output);
            pending_reallocs.clear_result(output);
            variant_initializations.clear_result(output);
        }
        self.apply_indirect_call_collection_slot_lifecycle_summary(
            cells,
            collection_slots,
            raw_aliases,
            function_aliases,
            output,
            callee,
            args,
            span,
        );
        seed_str_storage_layout(self.types, cells, raw_aliases, output);
        pending_reallocs.clear_result(output);
    }
}
