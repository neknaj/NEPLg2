extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_event_proof::summary_event_proof_with_aliases;
use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryOp;
use super::collection_slot_summary_target::summary_place_for_params_with_aliases;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, ResourceLocal};
use super::place_utils::push_unique_place;

pub(super) fn collect_summary_event_op(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    target: &Place,
    event: CollectionSlotLifecycleEvent,
) {
    let mut target_candidates = Vec::new();
    push_unique_place(&mut target_candidates, target);
    let canonical_target = state.raw_aliases.canonicalize_owner_cell_address(target);
    push_unique_place(&mut target_candidates, &canonical_target);
    for alias in state.raw_aliases.raw_address_aliases_for_value(target) {
        push_unique_place(&mut target_candidates, &alias);
    }
    for alias in state
        .raw_aliases
        .raw_address_aliases_for_value(&canonical_target)
    {
        push_unique_place(&mut target_candidates, &alias);
    }
    for target_candidate in target_candidates {
        let proof_target = state
            .raw_aliases
            .canonicalize_owner_cell_address(&target_candidate);
        let Some(proof) = summary_event_proof_with_aliases(
            engine.types,
            &state.cells,
            &state.raw_aliases,
            &state.pending_reallocs,
            &proof_target,
            event,
        ) else {
            continue;
        };
        let Some(target) =
            summary_place_for_params_with_aliases(params, &state.raw_aliases, &target_candidate)
        else {
            continue;
        };
        out.push(CollectionSlotLifecycleSummaryOp::Event {
            target,
            event,
            proof,
        });
        return;
    }
}
