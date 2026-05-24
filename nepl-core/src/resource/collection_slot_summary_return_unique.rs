extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_state_merge::merge_collection_slot_states;
use super::collection_slot_summary_return_model::{
    CollectionSlotLifecycleReturnRange, CollectionSlotLifecycleReturnSlot,
    CollectionSlotLifecycleReturnTransfer,
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
    if out
        .iter()
        .any(|existing| return_transfer_covers(existing, &transfer))
    {
        return;
    }
    out.retain(|existing| !return_transfer_covers(&transfer, existing));
    if !out.iter().any(|existing| existing == &transfer) {
        out.push(transfer);
    }
}

pub(super) fn push_return_range(
    out: &mut Vec<CollectionSlotLifecycleReturnRange>,
    range: CollectionSlotLifecycleReturnRange,
) {
    if !out.iter().any(|existing| existing == &range) {
        out.push(range);
    }
}

fn return_transfer_covers(
    prefix: &CollectionSlotLifecycleReturnTransfer,
    nested: &CollectionSlotLifecycleReturnTransfer,
) -> bool {
    if prefix.source.parameter_index != nested.source.parameter_index {
        return false;
    }
    let Some(source_suffix) = suffix_after_prefix(&nested.source.suffix, &prefix.source.suffix)
    else {
        return false;
    };
    let Some(target_suffix) = suffix_after_prefix(&nested.target_suffix, &prefix.target_suffix)
    else {
        return false;
    };
    source_suffix == target_suffix
}

fn suffix_after_prefix<'a, T: PartialEq>(value: &'a [T], prefix: &[T]) -> Option<&'a [T]> {
    if value.len() < prefix.len() || value[..prefix.len()] != *prefix {
        return None;
    }
    Some(&value[prefix.len()..])
}
