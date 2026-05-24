use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotState::{MaybeReleased, Released};
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::CollectionSlotLifecycleReturnPath;
use super::collection_slot_summary_projection::summary_suffix_for_params;
use super::collection_slot_summary_return_collect::{
    collect_return_storage_markers, collect_return_transfers_from_ops,
};
use super::collection_slot_summary_return_model::{
    CollectionSlotLifecycleReturnRange, CollectionSlotLifecycleReturnSlot,
    CollectionSlotLifecycleReturnTransfer,
};
use super::collection_slot_summary_return_path::collect_return_paths_from_ops;
use super::collection_slot_summary_return_range::collect_return_ranges_for_value;
use super::collection_slot_summary_return_unique::push_return_slot;
use super::initialized::ResourceCheckEngine;
use super::model::{ResourceLocal, ResourceTerminator};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn collect_return_facts_from_terminator(
    out_transfers: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    out: &mut Vec<CollectionSlotLifecycleReturnSlot>,
    out_ranges: &mut Vec<CollectionSlotLifecycleReturnRange>,
    out_paths: &mut Vec<CollectionSlotLifecycleReturnPath>,
    state_at_return: &CollectionSlotSummaryBuildState,
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
    collect_return_paths_from_ops(out_paths, engine, params, block_entry_state, ops, value);
    collect_return_ranges_for_value(out_ranges, params, state_at_return, value, &[]);
    for entry in state_at_return
        .collection_slots
        .entries_covered_by_storage_with_aliases(value, &state_at_return.raw_aliases)
    {
        let Some(suffix) = place_suffix_after_prefix(&entry.slot, value) else {
            continue;
        };
        let Some(suffix) = summary_suffix_for_params(params, &suffix) else {
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
        (
            state_at_return.collection_slots.released_storage(),
            Released,
        ),
        (
            state_at_return.collection_slots.maybe_released_storage(),
            MaybeReleased,
        ),
    ] {
        collect_return_storage_markers(out, params, markers, value, state);
    }
}
