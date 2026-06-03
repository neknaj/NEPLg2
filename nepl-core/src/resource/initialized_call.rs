extern crate alloc;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::function_alias::FunctionAliasTable;
use super::i32_call_facts::record_direct_call_i32_facts;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_call_args::discard_call_arg_loaded_value_origins;
use super::initialized_call_effect::direct_call_invalidates_result;
use super::initialized_path_state::{ResourceCheckState, ResourcePathAlternatives};
use super::initialized_scalar_flow::apply_direct_call_i32_scalar_summary;
use super::initialized_str_layout::seed_str_storage_layout;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{EffectOp, Place, ResourceCallTarget};
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckOperation;

impl ResourceCheckEngine<'_> {
    pub(super) fn check_direct_call(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
        effect: &EffectOp,
        span: crate::span::Span,
    ) {
        if direct_call_invalidates_result(self.types, effect, args) {
            pending_reallocs.clear_result(output);
            variant_initializations.clear_result(output);
            return;
        }
        if !self.consume_args(cells, args, ResourceCheckOperation::CallArgument, span) {
            return;
        }
        discard_call_arg_loaded_value_origins(cells, args);
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
        let return_path_states = self.apply_call_collection_slot_lifecycle_summary(
            cells,
            collection_slots,
            raw_aliases,
            variant_initializations,
            output,
            target,
            args,
            span,
        );
        seed_str_storage_layout(self.types, cells, raw_aliases, output);
        pending_reallocs.clear_result(output);
        // return path summary が存在する場合、空集合にも意味がある。
        // call-site の concrete variant 条件に合う通常復帰 path がないときは、
        // その feasible path 自体を後続検査から外す必要がある。
        if let Some(return_path_states) = return_path_states {
            let alternatives = return_path_states
                .into_iter()
                .map(|mut state| {
                    seed_str_storage_layout(
                        self.types,
                        &mut state.cells,
                        &mut state.raw_aliases,
                        output,
                    );
                    ResourceCheckState::new(
                        state.cells,
                        state.collection_slots,
                        state.raw_aliases,
                        function_aliases.clone(),
                        pending_reallocs.clone(),
                        state.variant_initializations,
                    )
                })
                .collect();
            self.path_alternatives =
                ResourcePathAlternatives::from_states_preserving_result_variants(
                    alternatives,
                    output,
                );
        }
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
        discard_call_arg_loaded_value_origins(cells, args);
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
