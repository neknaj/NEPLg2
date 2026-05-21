extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use super::collection_slot_summary_build_ops::collect_summary_ops_from_op;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_return_path_control::{
    branch_return_path_states, match_return_path_states,
};
use super::collection_slot_summary_return_path_model::ReturnPathBuildState;
use super::drop_point_path::ResourceDropPointPath;
use super::initialized::ResourceCheckEngine;
use super::initialized_path_state::ResourceCheckState;
use super::initialized_summary_engine::summary_check_engine;
use super::model::{ResourceBlockId, ResourceLocal, ResourceOp};

pub(super) fn return_path_states_after_ops(
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    start: ReturnPathBuildState,
    ops: &[ResourceOp],
) -> Vec<ReturnPathBuildState> {
    let mut paths = vec![start];
    for op in ops {
        let mut next = Vec::new();
        for path in paths {
            next.extend(return_path_states_after_op(engine, params, path, op));
        }
        paths = next;
    }
    paths
}

fn return_path_states_after_op(
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    path: ReturnPathBuildState,
    op: &ResourceOp,
) -> Vec<ReturnPathBuildState> {
    match op {
        ResourceOp::Branch {
            output,
            condition,
            condition_fact,
            then_ops,
            then_value,
            else_ops,
            else_value,
            span,
        } => branch_return_path_states(
            engine,
            params,
            path,
            output,
            condition,
            condition_fact,
            then_ops,
            then_value,
            else_ops,
            else_value,
            *span,
        ),
        ResourceOp::Match {
            output,
            scrutinee,
            scrutinee_is_borrow_target,
            arms,
            span,
        } => match_return_path_states(
            engine,
            params,
            path,
            output,
            scrutinee,
            *scrutinee_is_borrow_target,
            arms,
            *span,
        ),
        ResourceOp::Loop { .. } => {
            let mut advanced = checked_states_after_op(engine, &path.state, op);
            let mut summary_ops = Vec::new();
            collect_summary_ops_from_op(
                &mut summary_ops,
                engine,
                &path.state,
                params,
                engine.collection_slot_summaries,
                op,
            );
            if advanced.is_empty() {
                advanced.push(path.state.clone());
            }
            advanced
                .into_iter()
                .map(|state| {
                    let mut ops = path.ops.clone();
                    ops.extend(summary_ops.clone());
                    ReturnPathBuildState { state, ops }
                })
                .collect()
        }
        ResourceOp::Expr { .. }
        | ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::FunctionValue { .. }
        | ResourceOp::Call { .. }
        | ResourceOp::IndirectCall { .. }
        | ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::Construct { .. } => {
            let mut summary_ops = Vec::new();
            collect_summary_ops_from_op(
                &mut summary_ops,
                engine,
                &path.state,
                params,
                engine.collection_slot_summaries,
                op,
            );
            let mut states = checked_states_after_op(engine, &path.state, op);
            if states.is_empty() {
                states.push(path.state.clone());
            }
            states
                .into_iter()
                .map(|state| {
                    let mut ops = path.ops.clone();
                    ops.extend(summary_ops.clone());
                    ReturnPathBuildState { state, ops }
                })
                .collect()
        }
    }
}

pub(super) fn checked_states_after_op(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    op: &ResourceOp,
) -> Vec<CollectionSlotSummaryBuildState> {
    let mut path_engine = summary_check_engine(engine);
    let mut checked = state.clone();
    path_engine.check_op(
        &mut checked.cells,
        &mut checked.collection_slots,
        &mut checked.raw_aliases,
        &mut checked.function_aliases,
        &mut checked.pending_reallocs,
        &mut checked.variant_initializations,
        op,
        ResourceDropPointPath {
            block: ResourceBlockId(usize::MAX),
            steps: Vec::new(),
        },
    );
    let alternatives = core::mem::take(&mut path_engine.path_alternatives).into_states();
    if alternatives.is_empty() {
        vec![checked]
    } else {
        alternatives
            .into_iter()
            .map(summary_state_from_check_state)
            .collect()
    }
}

fn summary_state_from_check_state(state: ResourceCheckState) -> CollectionSlotSummaryBuildState {
    CollectionSlotSummaryBuildState {
        cells: state.cells,
        collection_slots: state.collection_slots,
        raw_aliases: state.raw_aliases,
        function_aliases: state.function_aliases,
        pending_reallocs: state.pending_reallocs,
        variant_initializations: state.variant_initializations,
        drop_traversal_range_certificates: Vec::new(),
    }
}
