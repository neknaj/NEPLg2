extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_event_proof::summary_event_proof_with_aliases;
use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryOp;
use super::collection_slot_summary_target::summary_place_for_params;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, ResourceLocal};

pub(super) fn collect_summary_event_op(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    target: &Place,
    event: CollectionSlotLifecycleEvent,
) {
    let target = state.raw_aliases.canonicalize_owner_cell_address(target);
    let Some(proof) = summary_event_proof_with_aliases(
        engine.types,
        &state.cells,
        &state.raw_aliases,
        &state.pending_reallocs,
        &target,
        event,
    ) else {
        return;
    };
    let Some(target) = summary_place_for_params(params, &target) else {
        return;
    };
    out.push(CollectionSlotLifecycleSummaryOp::Event {
        target,
        event,
        proof,
    });
}
