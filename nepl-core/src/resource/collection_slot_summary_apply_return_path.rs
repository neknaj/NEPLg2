extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_state_identity::place_covers_slot;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::CollectionSlotLifecycleReturnPath;
use super::collection_slot_summary_return_model::CollectionSlotLifecycleReturnTransfer;
use super::collection_slot_summary_target::instantiate_summary_target;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_return_transfers(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        args: &[Place],
        output: &Place,
        transfers: &[CollectionSlotLifecycleReturnTransfer],
        span: crate::span::Span,
    ) {
        for transfer in transfers {
            let Some(source) = instantiate_summary_target(self, args, &transfer.source) else {
                continue;
            };
            let source = raw_aliases.canonicalize_owner_cell_address(&source);
            let target = super::place_utils::place_with_suffix(
                output,
                &transfer.target_suffix,
                transfer.target_ty,
            );
            let target = raw_aliases.canonicalize_owner_cell_address(&target);
            self.transfer_slot_state(collection_slots, &source, &target, span);
        }
    }

    pub(super) fn apply_collection_slot_return_paths(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        initial_cells: &CellTable,
        initial_collection_slots: &CollectionSlotStateTable,
        initial_raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        paths: &[CollectionSlotLifecycleReturnPath],
        span: crate::span::Span,
    ) {
        let mut path_slots = Vec::new();
        for path in paths {
            let mut cells = initial_cells.clone();
            let mut slots = initial_collection_slots.clone();
            self.apply_collection_slot_lifecycle_summary_ops_state_only(
                &mut cells,
                &mut slots,
                initial_raw_aliases,
                args,
                &path.ops,
                span,
            );
            slots.clear_storage_prefix(output);
            self.apply_collection_slot_return_transfers(
                &mut slots,
                initial_raw_aliases,
                args,
                output,
                &path.return_transfers,
                span,
            );
            self.apply_collection_slot_return_slots(&mut slots, output, &path.return_slots);
            path_slots.push(slots);
        }
        let merged = CollectionSlotStateTable::merge_paths(&path_slots);
        self.apply_merged_return_output_slots(collection_slots, output, &merged);
    }

    fn apply_merged_return_output_slots(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        output: &Place,
        merged: &CollectionSlotStateTable,
    ) {
        collection_slots.clear_storage_prefix(output);
        for entry in merged.entries_covered_by_storage(output) {
            collection_slots.set_return_slot_state(&entry.slot, entry.state);
        }
        for marker in merged.released_storage() {
            if place_covers_slot(marker, output) {
                collection_slots.set_return_slot_state(marker, CollectionSlotState::Released);
            }
        }
        for marker in merged.maybe_released_storage() {
            if place_covers_slot(marker, output) {
                collection_slots.set_return_slot_state(marker, CollectionSlotState::MaybeReleased);
            }
        }
    }
}
