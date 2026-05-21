extern crate alloc;

use alloc::boxed::Box;

use crate::layout::storage_size_bytes;
use crate::types::TypeId;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_return_state::collection_slot_summary_state_after_ops;
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
    pub(super) loaded: Place,
}

pub(super) fn drop_witness_candidate(
    engine: &ResourceCheckEngine<'_>,
    body_prefix: &[ResourceOp],
    candidate: LoopBodyCandidateSlot,
) -> Option<LoopBodyDropWitnessCandidate> {
    if candidate.element_stride != storage_size_bytes(engine.types, candidate.expected_ty)
        || candidate.element_stride == 0
    {
        return None;
    }
    let drop_index = candidate_drop_index(&candidate, body_prefix)?;
    Some(LoopBodyDropWitnessCandidate {
        storage: candidate.storage,
        expected_ty: candidate.expected_ty,
        element_stride: candidate.element_stride,
        load_index: candidate.load_index,
        drop_index,
    })
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

fn candidate_drop_index(candidate: &LoopBodyCandidateSlot, ops: &[ResourceOp]) -> Option<usize> {
    ops.iter()
        .enumerate()
        .skip(candidate.load_index + 1)
        .find_map(|(op_index, op)| match op {
            ResourceOp::Drop { place, .. } if *place == candidate.loaded => Some(op_index),
            _ => None,
        })
}
