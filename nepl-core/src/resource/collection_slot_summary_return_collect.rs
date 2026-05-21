extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_return_model::{
    CollectionSlotLifecycleReturnSlot, CollectionSlotLifecycleReturnTransfer,
};
use super::collection_slot_summary_return_unique::push_return_slot;
use super::collection_slot_summary_return_value::collect_return_transfers_from_value_to_suffix;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, ResourceLocal};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn collect_return_transfers_from_ops(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    state_at_start: &CollectionSlotSummaryBuildState,
    ops: &[super::model::ResourceOp],
    value: &Place,
) {
    collect_return_transfers_from_value_to_suffix(
        out,
        engine,
        params,
        state_at_start,
        ops,
        value,
        &[],
        value.ty,
    );
}

pub(super) fn collect_return_storage_markers(
    out: &mut Vec<CollectionSlotLifecycleReturnSlot>,
    markers: &[Place],
    value: &Place,
    state: CollectionSlotState,
) {
    for marker in markers {
        let Some(suffix) = place_suffix_after_prefix(marker, value) else {
            continue;
        };
        push_return_slot(
            out,
            CollectionSlotLifecycleReturnSlot {
                suffix,
                ty: marker.ty,
                state,
            },
        );
    }
}
