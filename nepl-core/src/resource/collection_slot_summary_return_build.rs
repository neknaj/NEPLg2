use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_lifecycle::CollectionSlotState::{MaybeReleased, Released};
use super::collection_slot_state_merge::merge_collection_slot_states;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleReturnSlot, CollectionSlotLifecycleReturnTransfer,
    CollectionSlotLifecycleSummaryPlace,
};
use super::model::{Place, ResourceLocal, ResourceTerminator};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn collect_return_facts_from_terminator(
    out_transfers: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    out: &mut Vec<CollectionSlotLifecycleReturnSlot>,
    collection_slots: &CollectionSlotStateTable,
    params: &[ResourceLocal],
    terminator: &ResourceTerminator,
) {
    let ResourceTerminator::Return {
        value: Some(value), ..
    } = terminator
    else {
        return;
    };
    collect_return_transfers_from_params(out_transfers, params, value);
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

fn collect_return_transfers_from_params(
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

fn collect_return_storage_markers(
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

fn push_return_slot(
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
