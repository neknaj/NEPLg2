extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_state_merge::merge_collection_slot_states;
use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryPlace;
use super::collection_slot_summary_return_model::{
    CollectionSlotLifecycleReturnSlot, CollectionSlotLifecycleReturnTransfer,
};
use super::model::{Place, ResourceLocal};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn collect_return_transfers_from_params(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    params: &[ResourceLocal],
    value: &Place,
) {
    for (parameter_index, param) in params.iter().enumerate() {
        let Some(source_suffix) = place_suffix_after_prefix(value, &param.place) else {
            continue;
        };
        push_return_transfer(
            out,
            CollectionSlotLifecycleReturnTransfer {
                source: CollectionSlotLifecycleSummaryPlace {
                    parameter_index,
                    suffix: source_suffix,
                    ty: value.ty,
                },
                target_suffix: Vec::new(),
                target_ty: value.ty,
            },
        );
    }
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

pub(super) fn push_return_slot(
    out: &mut Vec<CollectionSlotLifecycleReturnSlot>,
    slot: CollectionSlotLifecycleReturnSlot,
) {
    if let Some(existing) = out
        .iter_mut()
        .find(|existing| existing.suffix == slot.suffix && existing.ty == slot.ty)
    {
        existing.state = merge_collection_slot_states(existing.state, slot.state);
    } else {
        out.push(slot);
    }
}

fn push_return_transfer(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    transfer: CollectionSlotLifecycleReturnTransfer,
) {
    if !out.iter().any(|existing| existing == &transfer) {
        out.push(transfer);
    }
}
