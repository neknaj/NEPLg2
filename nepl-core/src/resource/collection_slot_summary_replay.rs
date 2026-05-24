extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryOp, CollectionSlotLifecycleSummaryRelocateProof,
};
use super::collection_slot_summary_replay_drop_traversal::apply_drop_traversal_summary_op;
use super::collection_slot_summary_replay_transform_range::apply_transform_range_summary_op;
use super::collection_slot_summary_target::instantiate_summary_target_with_aliases;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_lifecycle_summary_ops(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        args: &[Place],
        ops: &[CollectionSlotLifecycleSummaryOp],
        span: crate::span::Span,
    ) {
        for op in ops {
            match op {
                CollectionSlotLifecycleSummaryOp::Event {
                    target,
                    event,
                    proof,
                } => {
                    if let Some(target) =
                        instantiate_summary_target_with_aliases(self, args, raw_aliases, target)
                    {
                        self.apply_collection_slot_lifecycle_summary_event_with_aliases(
                            cells,
                            collection_slots,
                            raw_aliases,
                            &target,
                            *event,
                            *proof,
                            span,
                        );
                    }
                }
                CollectionSlotLifecycleSummaryOp::Relocate {
                    old_storage,
                    new_storage,
                    proof,
                } => {
                    match proof {
                        CollectionSlotLifecycleSummaryRelocateProof::RawStorageRelocation => {}
                    }
                    let Some(old_storage) = instantiate_summary_target_with_aliases(
                        self,
                        args,
                        raw_aliases,
                        old_storage,
                    ) else {
                        continue;
                    };
                    let Some(new_storage) = instantiate_summary_target_with_aliases(
                        self,
                        args,
                        raw_aliases,
                        new_storage,
                    ) else {
                        continue;
                    };
                    self.apply_certified_collection_storage_relocate_with_aliases(
                        collection_slots,
                        raw_aliases,
                        &old_storage,
                        &new_storage,
                        span,
                    );
                }
                CollectionSlotLifecycleSummaryOp::DropTraversal {
                    storage,
                    initialized_count,
                    expected_ty,
                    coverage,
                } => {
                    apply_drop_traversal_summary_op(
                        self,
                        cells,
                        collection_slots,
                        raw_aliases,
                        args,
                        storage,
                        initialized_count,
                        *expected_ty,
                        coverage,
                        span,
                    );
                }
                CollectionSlotLifecycleSummaryOp::TransformRange {
                    source_storage,
                    source_initialized_count,
                    output_storage,
                    output_initialized_count,
                    expected_ty,
                    certificate,
                } => {
                    apply_transform_range_summary_op(
                        self,
                        cells,
                        collection_slots,
                        raw_aliases,
                        args,
                        source_storage,
                        source_initialized_count,
                        output_storage,
                        output_initialized_count,
                        *expected_ty,
                        *certificate,
                        span,
                    );
                }
                CollectionSlotLifecycleSummaryOp::TransformRangeSourceDrain {
                    source_storage,
                    source_initialized_count,
                    expected_ty,
                    certificate,
                } => {
                    let Some(source_storage) = instantiate_summary_target_with_aliases(
                        self,
                        args,
                        raw_aliases,
                        source_storage,
                    ) else {
                        continue;
                    };
                    let Some(source_initialized_count) = instantiate_summary_target_with_aliases(
                        self,
                        args,
                        raw_aliases,
                        source_initialized_count,
                    ) else {
                        continue;
                    };
                    self.apply_certified_collection_slot_transform_source_drain_with_aliases(
                        cells,
                        collection_slots,
                        raw_aliases,
                        &source_storage,
                        &source_initialized_count,
                        *expected_ty,
                        *certificate,
                        span,
                    );
                }
                CollectionSlotLifecycleSummaryOp::Merge { paths } => {
                    if paths.is_empty() {
                        continue;
                    }
                    let mut merged_slot_paths = Vec::new();
                    let mut merged_cell_paths = Vec::new();
                    for path_ops in paths {
                        let mut path_slots = collection_slots.clone();
                        let mut path_cells = cells.clone();
                        self.apply_collection_slot_lifecycle_summary_ops(
                            &mut path_cells,
                            &mut path_slots,
                            raw_aliases,
                            args,
                            path_ops,
                            span,
                        );
                        merged_slot_paths.push(path_slots);
                        merged_cell_paths.push(path_cells);
                    }
                    *collection_slots = CollectionSlotStateTable::merge_paths(&merged_slot_paths);
                    *cells = CellTable::merge_paths(&merged_cell_paths);
                }
                CollectionSlotLifecycleSummaryOp::Loop {
                    condition_ops,
                    body_ops,
                } => {
                    let mut condition_path = collection_slots.clone();
                    let mut condition_cells = cells.clone();
                    self.apply_collection_slot_lifecycle_summary_ops(
                        &mut condition_cells,
                        &mut condition_path,
                        raw_aliases,
                        args,
                        condition_ops,
                        span,
                    );
                    let exit_path = condition_path.clone();
                    let exit_cells = condition_cells.clone();
                    let mut body_path = condition_path;
                    let mut body_cells = condition_cells;
                    self.apply_collection_slot_lifecycle_summary_ops(
                        &mut body_cells,
                        &mut body_path,
                        raw_aliases,
                        args,
                        body_ops,
                        span,
                    );
                    *collection_slots =
                        CollectionSlotStateTable::merge_paths(&[exit_path, body_path]);
                    *cells = CellTable::merge_paths(&[exit_cells, body_cells]);
                }
            }
        }
    }

    pub(super) fn apply_collection_slot_lifecycle_summary_ops_state_only(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        args: &[Place],
        ops: &[CollectionSlotLifecycleSummaryOp],
        span: crate::span::Span,
    ) {
        let diagnostics_len = self.diagnostics.len();
        self.apply_collection_slot_lifecycle_summary_ops(
            cells,
            collection_slots,
            raw_aliases,
            args,
            ops,
            span,
        );
        self.diagnostics.truncate(diagnostics_len);
    }
}
