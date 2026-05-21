use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotState::{MaybeReleased, Released};
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_return_collect::{
    collect_return_storage_markers, collect_return_transfers_from_ops, push_return_slot,
};
use super::collection_slot_summary_return_model::{
    CollectionSlotLifecycleReturnSlot, CollectionSlotLifecycleReturnTransfer,
};
use super::initialized::ResourceCheckEngine;
use super::model::{ResourceLocal, ResourceTerminator};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn collect_return_facts_from_terminator(
    out_transfers: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    out: &mut Vec<CollectionSlotLifecycleReturnSlot>,
    collection_slots: &CollectionSlotStateTable,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    block_entry_state: &CollectionSlotSummaryBuildState,
    ops: &[super::model::ResourceOp],
    terminator: &ResourceTerminator,
) {
    let ResourceTerminator::Return {
        value: Some(value), ..
    } = terminator
    else {
        return;
    };
    collect_return_transfers_from_ops(out_transfers, engine, params, block_entry_state, ops, value);
    for entry in collection_slots.entries_covered_by_storage(value) {
        let Some(suffix) = place_suffix_after_prefix(&entry.slot, value) else {
            continue;
        };
        push_return_slot(
            out,
            CollectionSlotLifecycleReturnSlot {
                suffix,
                ty: entry.slot.ty,
                state: entry.state,
            },
        );
    }
    for (markers, state) in [
        (collection_slots.released_storage(), Released),
        (collection_slots.maybe_released_storage(), MaybeReleased),
    ] {
        collect_return_storage_markers(out, markers, value, state);
    }
}
