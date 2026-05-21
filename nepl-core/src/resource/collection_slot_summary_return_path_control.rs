extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_match_state::collection_slot_summary_match_arm_entry_state;
use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryOp;
use super::collection_slot_summary_return_path_model::ReturnPathBuildState;
use super::collection_slot_summary_return_path_state::{
    checked_states_after_op, return_path_states_after_ops,
};
use super::initialized::ResourceCheckEngine;
use super::model::{
    Place, PlaceRoot, ResourceConditionFact, ResourceLocal, ResourceMatchArm, ResourceOp,
};
use crate::span::Span;
use crate::types::TypeKind;

#[allow(clippy::too_many_arguments)]
pub(super) fn branch_return_path_states(
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    path: ReturnPathBuildState,
    output: &Place,
    condition: &Place,
    condition_fact: &Option<ResourceConditionFact>,
    then_ops: &[ResourceOp],
    then_value: &Place,
    else_ops: &[ResourceOp],
    else_value: &Place,
    span: Span,
) -> Vec<ReturnPathBuildState> {
    let mut paths = Vec::new();
    if !return_value_is_never(engine, then_value) {
        let selected_op = ResourceOp::Branch {
            output: output.clone(),
            condition: condition.clone(),
            condition_fact: condition_fact.clone(),
            then_ops: then_ops.to_vec(),
            then_value: then_value.clone(),
            else_ops: Vec::new(),
            else_value: never_place(engine),
            span,
        };
        paths.extend(control_arm_return_path_states(
            engine,
            params,
            path.clone(),
            then_ops,
            &selected_op,
        ));
    }
    if !return_value_is_never(engine, else_value) {
        let selected_op = ResourceOp::Branch {
            output: output.clone(),
            condition: condition.clone(),
            condition_fact: condition_fact.clone(),
            then_ops: Vec::new(),
            then_value: never_place(engine),
            else_ops: else_ops.to_vec(),
            else_value: else_value.clone(),
            span,
        };
        paths.extend(control_arm_return_path_states(
            engine,
            params,
            path,
            else_ops,
            &selected_op,
        ));
    }
    paths
}

pub(super) fn match_return_path_states(
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    path: ReturnPathBuildState,
    output: &Place,
    scrutinee: &Place,
    scrutinee_is_borrow_target: bool,
    arms: &[ResourceMatchArm],
    span: Span,
) -> Vec<ReturnPathBuildState> {
    let mut paths = Vec::new();
    for arm in arms {
        if return_value_is_never(engine, &arm.value) {
            continue;
        }
        let Some(arm_state) =
            collection_slot_summary_match_arm_entry_state(engine, &path.state, scrutinee, arm)
        else {
            continue;
        };
        let selected_op = ResourceOp::Match {
            output: output.clone(),
            scrutinee: scrutinee.clone(),
            scrutinee_is_borrow_target,
            arms: vec![arm.clone()],
            span,
        };
        let arm_start = ReturnPathBuildState {
            state: arm_state,
            ops: path.ops.clone(),
        };
        paths.extend(control_arm_return_path_states(
            engine,
            params,
            arm_start,
            &arm.ops,
            &selected_op,
        ));
    }
    paths
}

fn control_arm_return_path_states(
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    path: ReturnPathBuildState,
    arm_ops: &[ResourceOp],
    selected_op: &ResourceOp,
) -> Vec<ReturnPathBuildState> {
    let checked_states = checked_states_after_op(engine, &path.state, selected_op);
    let op_paths = return_path_states_after_ops(engine, params, path.clone(), arm_ops)
        .into_iter()
        .map(|path| path.ops)
        .collect();
    pair_checked_states_with_ops(path, checked_states, op_paths)
}

fn pair_checked_states_with_ops(
    path: ReturnPathBuildState,
    checked_states: Vec<CollectionSlotSummaryBuildState>,
    op_paths: Vec<Vec<CollectionSlotLifecycleSummaryOp>>,
) -> Vec<ReturnPathBuildState> {
    if checked_states.len() == op_paths.len() {
        return checked_states
            .into_iter()
            .zip(op_paths)
            .map(|(state, ops)| ReturnPathBuildState { state, ops })
            .collect();
    }
    debug_assert_eq!(checked_states.len(), op_paths.len());
    let summary_ops = merged_relative_ops(&path.ops, op_paths);
    checked_states
        .into_iter()
        .map(|state| {
            let ops = summary_ops.clone();
            ReturnPathBuildState { state, ops }
        })
        .collect()
}

fn merged_relative_ops(
    prefix: &[CollectionSlotLifecycleSummaryOp],
    op_paths: Vec<Vec<CollectionSlotLifecycleSummaryOp>>,
) -> Vec<CollectionSlotLifecycleSummaryOp> {
    let mut relative_paths = Vec::new();
    for ops in op_paths {
        if ops.starts_with(prefix) {
            relative_paths.push(ops[prefix.len()..].to_vec());
        } else {
            relative_paths.push(ops);
        }
    }
    let mut merged = prefix.to_vec();
    match relative_paths.as_slice() {
        [] => {}
        [single] => merged.extend(single.clone()),
        _ => merged.push(CollectionSlotLifecycleSummaryOp::Merge {
            paths: relative_paths,
        }),
    }
    merged
}

pub(super) fn return_value_is_never(engine: &ResourceCheckEngine<'_>, place: &Place) -> bool {
    matches!(
        engine.types.get(engine.types.resolve_id(place.ty)),
        TypeKind::Never
    )
}

fn never_place(engine: &ResourceCheckEngine<'_>) -> Place {
    Place {
        root: PlaceRoot::Unknown,
        projections: Vec::new(),
        ty: engine.types.never(),
    }
}
