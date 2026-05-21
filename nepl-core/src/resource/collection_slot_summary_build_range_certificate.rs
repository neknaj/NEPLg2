extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_drop_proof::CollectionSlotDropObligation;
use super::collection_slot_lifecycle::CollectionSlotLifecycleOp;
use super::collection_slot_summary_build_nested::apply_summary_condition_fact;
use super::collection_slot_summary_build_range_bound::initialized_range_loop_bound;
use super::collection_slot_summary_build_range_preserve::body_preserves_place;
use super::collection_slot_summary_build_range_step::loop_body_increment_step;
use super::collection_slot_summary_build_range_witness::{
    loop_body_candidate_slots, loop_body_drops_symbolic_slot,
};
use super::collection_slot_summary_build_state::{
    CollectionSlotDropTraversalRangeCertificateCandidate, CollectionSlotSummaryBuildState,
};
use super::collection_slot_summary_model::CollectionSlotInitializedRangeDropTraversalCertificate;
use super::collection_slot_summary_return_state::collection_slot_summary_state_after_ops;
use super::initialized::ResourceCheckEngine;
use super::model::{ResourceConditionFact, ResourceOp};

pub(super) fn loop_drop_traversal_range_certificates(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    condition_ops: &[ResourceOp],
    condition_fact: Option<&ResourceConditionFact>,
    body_ops: &[ResourceOp],
) -> Vec<CollectionSlotDropTraversalRangeCertificateCandidate> {
    let Some((index, initialized_count)) =
        initialized_range_loop_bound(&state.raw_aliases, condition_fact)
    else {
        return Vec::new();
    };
    if state.raw_aliases.i32_value(&index) != Some(0) {
        return Vec::new();
    }
    let Some(step_index) = loop_body_increment_step(body_ops, &index) else {
        return Vec::new();
    };
    let body_prefix = &body_ops[..step_index];
    let body_tail = &body_ops[step_index + 1..];

    let mut condition_state = collection_slot_summary_state_after_ops(engine, state, condition_ops);
    apply_summary_condition_fact(&mut condition_state, condition_fact, true);
    let state_after_step =
        collection_slot_summary_state_after_ops(engine, &condition_state, &body_ops[..=step_index]);
    let preserves_count = body_preserves_place(
        engine,
        &condition_state.raw_aliases,
        body_ops,
        &initialized_count,
    );
    let preserves_tail_index =
        body_preserves_place(engine, &state_after_step.raw_aliases, body_tail, &index);
    if !preserves_count || !preserves_tail_index {
        return Vec::new();
    }

    let mut out = Vec::new();
    for (storage, expected_ty, element_stride) in
        loop_body_candidate_slots(engine, &condition_state, body_prefix, &index)
    {
        let storage = condition_state
            .raw_aliases
            .canonicalize_owner_cell_address(&storage);
        if !body_preserves_place(engine, &condition_state.raw_aliases, body_ops, &storage) {
            continue;
        }
        let drops_symbolic_slot = loop_body_drops_symbolic_slot(
            engine,
            &condition_state,
            body_prefix,
            &storage,
            &index,
            &initialized_count,
            expected_ty,
            element_stride,
        );
        if !drops_symbolic_slot {
            continue;
        }
        let candidate = CollectionSlotDropTraversalRangeCertificateCandidate {
            storage,
            initialized_count: initialized_count.clone(),
            expected_ty,
            certificate: CollectionSlotInitializedRangeDropTraversalCertificate {
                element_stride,
                drop_obligation: CollectionSlotDropObligation::DropLoadedValue {
                    operation: CollectionSlotLifecycleOp::DropInitialized,
                    value_ty: expected_ty,
                },
            },
        };
        if !out
            .iter()
            .any(|existing| range_candidate_eq(existing, &candidate))
        {
            out.push(candidate);
        }
    }
    out
}

fn range_candidate_eq(
    left: &CollectionSlotDropTraversalRangeCertificateCandidate,
    right: &CollectionSlotDropTraversalRangeCertificateCandidate,
) -> bool {
    left.storage == right.storage
        && left.initialized_count == right.initialized_count
        && left.expected_ty == right.expected_ty
        && left.certificate == right.certificate
}
