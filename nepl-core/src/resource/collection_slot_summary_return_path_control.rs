extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use super::collection_slot_summary_build_nested::apply_summary_condition_fact;
use super::collection_slot_summary_build_state::{
    CollectionSlotDropTraversalRangeCertificateCandidate, CollectionSlotSummaryBuildState,
    CollectionSlotTransformRangeCertificateCandidate,
};
use super::collection_slot_summary_match_state::collection_slot_summary_match_arm_entry_state;
use super::collection_slot_summary_return_path_condition::collect_return_path_preconditions;
use super::collection_slot_summary_return_path_model::ReturnPathBuildState;
use super::collection_slot_summary_return_path_state::return_path_states_after_ops;
use super::initialized::ResourceCheckEngine;
use super::initialized_path_state::ResourceCheckState;
use super::initialized_summary_engine::summary_check_engine;
use super::model::{Place, ResourceConditionFact, ResourceLocal, ResourceMatchArm, ResourceOp};
use super::report::ResourceCheckOperation;
use crate::span::Span;
use crate::types::TypeKind;

type ReturnPathCertificates = (
    Vec<CollectionSlotDropTraversalRangeCertificateCandidate>,
    Vec<CollectionSlotTransformRangeCertificateCandidate>,
);

#[allow(clippy::too_many_arguments)]
pub(super) fn branch_return_path_states(
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    path: ReturnPathBuildState,
    output: &Place,
    _condition: &Place,
    condition_fact: &Option<ResourceConditionFact>,
    then_ops: &[ResourceOp],
    then_value: &Place,
    else_ops: &[ResourceOp],
    else_value: &Place,
    span: Span,
) -> Vec<ReturnPathBuildState> {
    let mut paths = Vec::new();
    if !return_value_is_never(engine, then_value) {
        paths.extend(control_arm_return_path_states(
            engine,
            params,
            path.clone(),
            condition_fact.as_ref().map(|fact| (fact, true)),
            then_ops,
            then_value,
            output,
            ResourceCheckOperation::BranchValue,
            span,
        ));
    }
    if !return_value_is_never(engine, else_value) {
        paths.extend(control_arm_return_path_states(
            engine,
            params,
            path,
            condition_fact.as_ref().map(|fact| (fact, false)),
            else_ops,
            else_value,
            output,
            ResourceCheckOperation::BranchValue,
            span,
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
    _scrutinee_is_borrow_target: bool,
    arms: &[ResourceMatchArm],
    _span: Span,
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
        let arm_start = ReturnPathBuildState {
            state: arm_state,
            preconditions: path.preconditions.clone(),
            ops: path.ops.clone(),
        };
        paths.extend(control_arm_return_path_states(
            engine,
            params,
            arm_start,
            None,
            &arm.ops,
            &arm.value,
            output,
            ResourceCheckOperation::MatchValue,
            arm.span,
        ));
    }
    paths
}

fn control_arm_return_path_states(
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    mut path: ReturnPathBuildState,
    condition_fact: Option<(&ResourceConditionFact, bool)>,
    arm_ops: &[ResourceOp],
    arm_value: &Place,
    output: &Place,
    operation: ResourceCheckOperation,
    span: Span,
) -> Vec<ReturnPathBuildState> {
    if let Some((condition_fact, truthy_path)) = condition_fact {
        collect_return_path_preconditions(
            &mut path.preconditions,
            engine,
            params,
            &path.state,
            condition_fact,
            truthy_path,
        );
        apply_summary_condition_fact(&mut path.state, Some(condition_fact), truthy_path);
    }
    let mut out = Vec::new();
    for arm_path in return_path_states_after_ops(engine, params, path, arm_ops) {
        let ReturnPathBuildState {
            state,
            preconditions,
            ops,
        } = arm_path;
        let certificates = (
            state.drop_traversal_range_certificates.clone(),
            state.transform_range_certificates.clone(),
        );
        let mut transfer_engine = summary_check_engine(engine);
        let mut paths_available = true;
        let states = transfer_engine.transfer_control_value_path_states(
            vec![summary_state_to_resource_state(state)],
            arm_value,
            output,
            operation,
            span,
            &mut paths_available,
        );
        for state in states {
            out.push(ReturnPathBuildState {
                state: summary_state_from_resource_state(state, &certificates),
                preconditions: preconditions.clone(),
                ops: ops.clone(),
            });
        }
    }
    out
}

pub(super) fn return_value_branch_arm_start(
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    mut path: ReturnPathBuildState,
    condition_fact: Option<(&ResourceConditionFact, bool)>,
) -> ReturnPathBuildState {
    if let Some((condition_fact, truthy_path)) = condition_fact {
        collect_return_path_preconditions(
            &mut path.preconditions,
            engine,
            params,
            &path.state,
            condition_fact,
            truthy_path,
        );
        apply_summary_condition_fact(&mut path.state, Some(condition_fact), truthy_path);
    }
    path
}

fn summary_state_to_resource_state(state: CollectionSlotSummaryBuildState) -> ResourceCheckState {
    ResourceCheckState::new(
        state.cells,
        state.collection_slots,
        state.raw_aliases,
        state.function_aliases,
        state.pending_reallocs,
        state.variant_initializations,
    )
}

fn summary_state_from_resource_state(
    state: ResourceCheckState,
    certificates: &ReturnPathCertificates,
) -> CollectionSlotSummaryBuildState {
    CollectionSlotSummaryBuildState {
        cells: state.cells,
        collection_slots: state.collection_slots,
        raw_aliases: state.raw_aliases,
        function_aliases: state.function_aliases,
        pending_reallocs: state.pending_reallocs,
        variant_initializations: state.variant_initializations,
        drop_traversal_range_certificates: certificates.0.clone(),
        transform_range_certificates: certificates.1.clone(),
    }
}

pub(super) fn return_value_is_never(engine: &ResourceCheckEngine<'_>, place: &Place) -> bool {
    matches!(
        engine.types.get(engine.types.resolve_id(place.ty)),
        TypeKind::Never
    )
}
