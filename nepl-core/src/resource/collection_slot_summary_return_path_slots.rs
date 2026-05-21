extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_return_collect::collect_return_storage_markers;
use super::collection_slot_summary_return_model::CollectionSlotLifecycleReturnSlot;
use super::collection_slot_summary_return_unique::push_return_slot;
use super::model::{Place, PlaceProjection};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn translate_return_slots(
    slots: &[CollectionSlotLifecycleReturnSlot],
    target_suffix: &[PlaceProjection],
) -> Vec<CollectionSlotLifecycleReturnSlot> {
    let mut out = Vec::new();
    for slot in slots {
        let mut suffix = target_suffix.to_vec();
        suffix.extend_from_slice(&slot.suffix);
        push_return_slot(
            &mut out,
            CollectionSlotLifecycleReturnSlot {
                suffix,
                ty: slot.ty,
                state: slot.state,
            },
        );
    }
    out
}

pub(super) fn collect_return_slots_for_value(
    out: &mut Vec<CollectionSlotLifecycleReturnSlot>,
    state: &CollectionSlotSummaryBuildState,
    value: &Place,
    target_suffix: &[PlaceProjection],
) {
    for entry in state.collection_slots.entries_covered_by_storage(value) {
        let Some(suffix) = place_suffix_after_prefix(&entry.slot, value) else {
            continue;
        };
        let mut composed_suffix = target_suffix.to_vec();
        composed_suffix.extend(suffix);
        push_return_slot(
            out,
            CollectionSlotLifecycleReturnSlot {
                suffix: composed_suffix,
                ty: entry.slot.ty,
                state: entry.state,
            },
        );
    }
    for (markers, marker_state) in [
        (
            state.collection_slots.released_storage(),
            CollectionSlotState::Released,
        ),
        (
            state.collection_slots.maybe_released_storage(),
            CollectionSlotState::MaybeReleased,
        ),
    ] {
        let mut marker_slots = Vec::new();
        collect_return_storage_markers(&mut marker_slots, markers, value, marker_state);
        for mut slot in marker_slots {
            let mut composed_suffix = target_suffix.to_vec();
            composed_suffix.append(&mut slot.suffix);
            push_return_slot(
                out,
                CollectionSlotLifecycleReturnSlot {
                    suffix: composed_suffix,
                    ty: slot.ty,
                    state: slot.state,
                },
            );
        }
    }
}
