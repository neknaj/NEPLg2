extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_drop_proof::CollectionSlotDropObligation;
use super::collection_slot_lifecycle::CollectionSlotLifecycleOp;
use super::collection_slot_summary_build_nested::apply_summary_condition_fact;
use super::collection_slot_summary_build_range_step::{
    body_preserves_place, loop_body_increment_step,
};
use super::collection_slot_summary_build_range_witness::{
    loop_body_candidate_slots, loop_body_drops_symbolic_slot, state_after_ops,
};
use super::collection_slot_summary_build_state::{
    CollectionSlotDropTraversalRangeCertificateCandidate, CollectionSlotSummaryBuildState,
};
use super::collection_slot_summary_model::CollectionSlotInitializedRangeDropTraversalCertificate;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceConditionFact, ResourceI32RelationOp, ResourceOp};

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
    let Some(step_index) = loop_body_increment_step(body_ops, &index, &state.raw_aliases) else {
        return Vec::new();
    };
    let body_prefix = &body_ops[..step_index];
    let body_tail = &body_ops[step_index + 1..];
    if !body_preserves_place(body_ops, &initialized_count)
        || !body_preserves_place(body_tail, &index)
    {
        return Vec::new();
    }

    let mut condition_state = state_after_ops(engine, state.clone(), condition_ops);
    apply_summary_condition_fact(&mut condition_state, condition_fact, true);

    let mut out = Vec::new();
    for (storage, expected_ty, element_stride) in
        loop_body_candidate_slots(body_prefix, &index, &condition_state.raw_aliases)
    {
        let storage = condition_state
            .raw_aliases
            .canonicalize_owner_cell_address(&storage);
        if !body_preserves_place(body_ops, &storage) {
            continue;
        }
        if !loop_body_drops_symbolic_slot(
            engine,
            &condition_state,
            body_prefix,
            &storage,
            &index,
            &initialized_count,
            expected_ty,
            element_stride,
        ) {
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

fn initialized_range_loop_bound(
    raw_aliases: &RawCellAddressAliases,
    fact: Option<&ResourceConditionFact>,
) -> Option<(Place, Place)> {
    let fact = fact?;
    let mut out = LoopBoundSearch::None;
    collect_initialized_range_loop_bound(raw_aliases, fact, &mut out);
    match out {
        LoopBoundSearch::One(bound) => Some(bound),
        LoopBoundSearch::None | LoopBoundSearch::Ambiguous => None,
    }
}

fn collect_initialized_range_loop_bound(
    raw_aliases: &RawCellAddressAliases,
    fact: &ResourceConditionFact,
    out: &mut LoopBoundSearch,
) {
    match fact {
        ResourceConditionFact::I32Relation { left, op, right } => match op {
            ResourceI32RelationOp::Lt => push_loop_bound(
                out,
                raw_aliases.canonicalize_scalar(left),
                raw_aliases.canonicalize_scalar(right),
            ),
            ResourceI32RelationOp::Gt => push_loop_bound(
                out,
                raw_aliases.canonicalize_scalar(right),
                raw_aliases.canonicalize_scalar(left),
            ),
            ResourceI32RelationOp::Eq
            | ResourceI32RelationOp::Ne
            | ResourceI32RelationOp::Le
            | ResourceI32RelationOp::Ge => {}
        },
        ResourceConditionFact::All(facts) => {
            for fact in facts {
                collect_initialized_range_loop_bound(raw_aliases, fact, out);
            }
        }
        ResourceConditionFact::Any(_)
        | ResourceConditionFact::EqZero { .. }
        | ResourceConditionFact::NeZero { .. }
        | ResourceConditionFact::Positive { .. }
        | ResourceConditionFact::NonPositive { .. }
        | ResourceConditionFact::Negative { .. }
        | ResourceConditionFact::NonNegative { .. } => {}
    }
}

enum LoopBoundSearch {
    None,
    One((Place, Place)),
    Ambiguous,
}

fn push_loop_bound(out: &mut LoopBoundSearch, index: Place, initialized_count: Place) {
    let candidate = (index, initialized_count);
    match out {
        LoopBoundSearch::None => *out = LoopBoundSearch::One(candidate),
        LoopBoundSearch::One(existing) if existing == &candidate => {}
        LoopBoundSearch::One(_) | LoopBoundSearch::Ambiguous => {
            *out = LoopBoundSearch::Ambiguous;
        }
    }
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
