extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_storage_carrier::type_can_carry_collection_slot_storage;
use super::collection_slot_summary_apply_return_path::CollectionSlotReturnPathState;
use super::collection_slot_summary_model::CollectionSlotLifecycleFunctionSummary;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{Place, ResourceCallTarget};

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_call_collection_slot_lifecycle_summary(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &mut RawCellAddressAliases,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        output: &Place,
        target: &ResourceCallTarget,
        args: &[Place],
        span: crate::span::Span,
    ) -> Option<Vec<CollectionSlotReturnPathState>> {
        let ResourceCallTarget::User { name, type_args } = target else {
            collection_slots.clear_storage_prefix(output);
            self.clear_consumed_collection_slot_args(collection_slots, raw_aliases, args);
            return None;
        };
        let Some(summary) = self.collection_slot_summaries.get(name) else {
            collection_slots.clear_storage_prefix(output);
            self.clear_consumed_collection_slot_args(collection_slots, raw_aliases, args);
            return None;
        };
        self.apply_collection_slot_lifecycle_function_summary(
            cells,
            collection_slots,
            raw_aliases,
            variant_initializations,
            output,
            args,
            type_args,
            summary,
            span,
        )
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
            self.clear_consumed_collection_slot_args(collection_slots, raw_aliases, args);
            return;
        }
        let mut slot_paths = Vec::new();
        let mut cell_paths = Vec::new();
        for function in functions {
            let mut path_slots = collection_slots.clone();
            let mut path_cells = cells.clone();
            if let Some(summary) = self.collection_slot_summaries.get(function.symbol()) {
                let mut path_aliases = raw_aliases.clone();
                let mut path_variants = PendingVariantRawCellInitializations::default();
                self.apply_collection_slot_lifecycle_function_summary(
                    &mut path_cells,
                    &mut path_slots,
                    &mut path_aliases,
                    &mut path_variants,
                    output,
                    args,
                    &[],
                    summary,
                    span,
                );
            } else {
                path_slots.clear_storage_prefix(output);
                self.clear_consumed_collection_slot_args(&mut path_slots, raw_aliases, args);
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
        raw_aliases: &mut RawCellAddressAliases,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        output: &Place,
        args: &[Place],
        type_args: &[crate::types::TypeId],
        summary: &CollectionSlotLifecycleFunctionSummary,
        span: crate::span::Span,
    ) -> Option<Vec<CollectionSlotReturnPathState>> {
        let initial_cells = cells.clone();
        let initial_collection_slots = collection_slots.clone();
        let initial_raw_aliases = raw_aliases.clone();
        let initial_variant_initializations = variant_initializations.clone();
        if summary.return_paths.is_empty() {
            self.apply_collection_slot_lifecycle_summary_ops(
                cells,
                collection_slots,
                raw_aliases,
                args,
                &summary.ops,
                span,
            );
            collection_slots.clear_storage_prefix(output);
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
                args,
                output,
                &summary.return_slots,
            );
            self.apply_collection_slot_return_ranges(
                collection_slots,
                raw_aliases,
                args,
                output,
                &summary.type_params,
                type_args,
                &summary.return_ranges,
            );
            self.clear_consumed_collection_slot_args(collection_slots, raw_aliases, args);
            None
        } else {
            Some(self.apply_collection_slot_return_paths(
                cells,
                collection_slots,
                raw_aliases,
                &initial_cells,
                &initial_collection_slots,
                &initial_raw_aliases,
                &initial_variant_initializations,
                variant_initializations,
                output,
                args,
                &summary.type_params,
                type_args,
                &summary.return_paths,
                span,
            ))
        }
    }

    pub(super) fn clear_consumed_collection_slot_args(
        &self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        args: &[Place],
    ) {
        // 値渡し call argument は通常の resource cell と同じく callee 側へ所有権が移る。
        // collection slot state も caller に残すと、summary を持たない eliminator や
        // callback 境界の後で、すでに消費された一時値が storage owner として残ってしまう。
        for arg in args {
            if !self.types.is_copy(arg.ty)
                && type_can_carry_collection_slot_storage(self.types, arg.ty)
            {
                collection_slots.clear_storage_prefix_with_aliases(arg, raw_aliases);
            }
        }
    }
}
