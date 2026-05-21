extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_state_merge::merge_collection_slot_states;
use super::collection_slot_summary_return_model::{
    CollectionSlotLifecycleReturnSlot, CollectionSlotLifecycleReturnTransfer,
};

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

pub(super) fn push_return_transfer(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    transfer: CollectionSlotLifecycleReturnTransfer,
) {
    if !out.iter().any(|existing| existing == &transfer) {
        out.push(transfer);
    }
}
