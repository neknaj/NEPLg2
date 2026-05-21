extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_build_ops::collect_summary_ops_from_ops;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummaryIndex, CollectionSlotLifecycleSummaryOp,
};
use super::condition_fact::record_condition_fact_value_constraints;
use super::initialized::ResourceCheckEngine;
use super::initialized_summary_engine::summary_check_engine;
use super::model::{ResourceConditionFact, ResourceLocal, ResourceOp};

pub(super) fn collect_nested_summary_ops(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
) -> Vec<CollectionSlotLifecycleSummaryOp> {
    collect_nested_summary_path(engine, state, params, collection_slot_summaries, ops).0
}

pub(super) fn collect_nested_summary_ops_with_condition(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    condition_fact: Option<&ResourceConditionFact>,
    truthy_path: bool,
    ops: &[ResourceOp],
) -> Vec<CollectionSlotLifecycleSummaryOp> {
    let mut path_state = state.clone();
    apply_summary_condition_fact(&mut path_state, condition_fact, truthy_path);
    collect_nested_summary_ops_from_state(
        engine,
        path_state,
        params,
        collection_slot_summaries,
        ops,
    )
}

pub(super) fn collect_nested_summary_path(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
) -> (
    Vec<CollectionSlotLifecycleSummaryOp>,
    CollectionSlotSummaryBuildState,
) {
    let path_state = state.clone();
    collect_nested_summary_path_from_state(
        engine,
        path_state,
        params,
        collection_slot_summaries,
        ops,
    )
}

pub(super) fn collect_nested_summary_ops_from_state(
    engine: &ResourceCheckEngine<'_>,
    state: CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
) -> Vec<CollectionSlotLifecycleSummaryOp> {
    collect_nested_summary_path_from_state(engine, state, params, collection_slot_summaries, ops).0
}

fn collect_nested_summary_path_from_state(
    engine: &ResourceCheckEngine<'_>,
    mut path_state: CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
) -> (
    Vec<CollectionSlotLifecycleSummaryOp>,
    CollectionSlotSummaryBuildState,
) {
    let mut path_engine = summary_check_engine(engine);
    let mut out = Vec::new();
    collect_summary_ops_from_ops(
        &mut out,
        &mut path_engine,
        &mut path_state,
        params,
        collection_slot_summaries,
        ops,
    );
    (out, path_state)
}

pub(super) fn apply_summary_condition_fact(
    state: &mut CollectionSlotSummaryBuildState,
    condition_fact: Option<&ResourceConditionFact>,
    truthy_path: bool,
) {
    let Some(condition_fact) = condition_fact else {
        return;
    };
    record_condition_fact_value_constraints(&mut state.raw_aliases, condition_fact, truthy_path);
}

pub(super) fn push_merge_summary(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    paths: Vec<Vec<CollectionSlotLifecycleSummaryOp>>,
) {
    if paths.is_empty() || paths.iter().all(Vec::is_empty) {
        return;
    }
    out.push(CollectionSlotLifecycleSummaryOp::Merge { paths });
}
