extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::CollectionSlotLifecycleFunctionSummary;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceCallTarget};

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_call_collection_slot_lifecycle_summary(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
        span: crate::span::Span,
    ) {
        let ResourceCallTarget::User { name, .. } = target else {
            collection_slots.clear_storage_prefix(output);
            return;
        };
        let Some(summary) = self.collection_slot_summaries.get(name) else {
            collection_slots.clear_storage_prefix(output);
            return;
        };
        self.apply_collection_slot_lifecycle_function_summary(
            cells,
            collection_slots,
            raw_aliases,
            output,
            args,
            summary,
            span,
        );
    }

    pub(super) fn apply_indirect_call_collection_slot_lifecycle_summary(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        function_aliases: &FunctionAliasTable,
        output: &Place,
        callee: &Place,
        args: &[Place],
        span: crate::span::Span,
    ) {
        let functions = function_aliases.functions(callee);
        if functions.is_empty() {
            collection_slots.clear_storage_prefix(output);
            return;
        }
        let mut slot_paths = Vec::new();
        let mut cell_paths = Vec::new();
        for function in functions {
            let mut path_slots = collection_slots.clone();
            let mut path_cells = cells.clone();
            if let Some(summary) = self.collection_slot_summaries.get(function) {
                self.apply_collection_slot_lifecycle_function_summary(
                    &mut path_cells,
                    &mut path_slots,
                    raw_aliases,
                    output,
                    args,
                    summary,
                    span,
                );
            } else {
                path_slots.clear_storage_prefix(output);
            }
            slot_paths.push(path_slots);
            cell_paths.push(path_cells);
        }
        *collection_slots = CollectionSlotStateTable::merge_paths(&slot_paths);
        *cells = CellTable::merge_paths(&cell_paths);
    }

    fn apply_collection_slot_lifecycle_function_summary(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        summary: &CollectionSlotLifecycleFunctionSummary,
        span: crate::span::Span,
    ) {
        let initial_cells = cells.clone();
        let initial_collection_slots = collection_slots.clone();
        let initial_raw_aliases = raw_aliases.clone();
        self.apply_collection_slot_lifecycle_summary_ops(
            cells,
            collection_slots,
            raw_aliases,
            args,
            &summary.ops,
            span,
        );
        collection_slots.clear_storage_prefix(output);
        if summary.return_paths.is_empty() {
            self.apply_collection_slot_return_transfers(
                collection_slots,
                raw_aliases,
                args,
                output,
                &summary.return_transfers,
                span,
            );
            self.apply_collection_slot_return_slots(
                collection_slots,
                output,
                &summary.return_slots,
            );
        } else {
            self.apply_collection_slot_return_paths(
                collection_slots,
                &initial_cells,
                &initial_collection_slots,
                &initial_raw_aliases,
                output,
                args,
                &summary.return_paths,
                span,
            );
        }
    }
}
