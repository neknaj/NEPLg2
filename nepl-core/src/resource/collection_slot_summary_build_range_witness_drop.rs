extern crate alloc;

use alloc::boxed::Box;

use crate::layout::storage_size_bytes;
use crate::types::TypeId;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_return_state::collection_slot_summary_state_after_ops;
use super::drop_requirement::resource_type_needs_drop_code;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, PlaceProjection, ResourceOffset, ResourceOp};
use super::place_utils::raw_memory_cell_place;

pub(super) struct LoopBodyDropWitnessCandidate {
    pub(super) storage: Place,
    pub(super) expected_ty: TypeId,
    pub(super) element_stride: usize,
    pub(super) load_index: usize,
    pub(super) drop_index: usize,
}

pub(super) struct LoopBodyCandidateSlot {
    pub(super) storage: Place,
    pub(super) expected_ty: TypeId,
    pub(super) element_stride: usize,
    pub(super) load_index: usize,
}

pub(super) fn drop_witness_candidate(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    body_prefix: &[ResourceOp],
    index: &Place,
    initialized_count: &Place,
    candidate: LoopBodyCandidateSlot,
) -> Option<LoopBodyDropWitnessCandidate> {
    if candidate.element_stride != storage_size_bytes(engine.types, candidate.expected_ty)
        || candidate.element_stride == 0
    {
        return None;
    }
    if !resource_type_needs_drop_code(engine.types, candidate.expected_ty) {
        return Some(drop_witness_at(&candidate, candidate.load_index));
    }
    let mut previous_prefix_drops = prefix_drops_symbolic_slot(
        engine,
        state,
        body_prefix,
        index,
        initialized_count,
        &drop_witness_at(&candidate, candidate.load_index),
    );
    for drop_index in candidate.load_index + 1..body_prefix.len() {
        if !matches!(body_prefix.get(drop_index), Some(ResourceOp::Drop { .. })) {
            continue;
        }
        let witness = drop_witness_at(&candidate, drop_index);
        let current_prefix_drops = prefix_drops_symbolic_slot(
            engine,
            state,
            body_prefix,
            index,
            initialized_count,
            &witness,
        );
        if !previous_prefix_drops && current_prefix_drops {
            return Some(witness);
        }
        previous_prefix_drops = current_prefix_drops;
    }
    None
}

fn drop_witness_at(
    candidate: &LoopBodyCandidateSlot,
    drop_index: usize,
) -> LoopBodyDropWitnessCandidate {
    LoopBodyDropWitnessCandidate {
        storage: candidate.storage.clone(),
        expected_ty: candidate.expected_ty,
        element_stride: candidate.element_stride,
        load_index: candidate.load_index,
        drop_index,
    }
}

pub(super) fn prefix_drops_symbolic_slot(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    body_prefix: &[ResourceOp],
    index: &Place,
    initialized_count: &Place,
    candidate: &LoopBodyDropWitnessCandidate,
) -> bool {
    let slot_address = candidate.storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
            place: Box::new(index.clone()),
            scale: candidate.element_stride,
        }),
        engine.types.i32(),
    );
    let slot = slot_address
        .clone()
        .with_projection(PlaceProjection::Deref, candidate.expected_ty);
    let mut probe = state.clone();
    probe
        .cells
        .mark_initialized(&raw_memory_cell_place(&slot_address, candidate.expected_ty));
    probe.collection_slots.set_slot_state(
        &slot,
        CollectionSlotState::Initialized(candidate.expected_ty),
    );
    let mut probe = collection_slot_summary_state_after_ops(
        engine,
        &probe,
        &body_prefix[..=candidate.drop_index],
    );
    engine
        .collection_slot_drop_traversal_result(
            &mut probe.cells,
            &mut probe.collection_slots,
            &probe.raw_aliases,
            &candidate.storage,
            initialized_count,
            candidate.expected_ty,
        )
        .is_ok()
}
