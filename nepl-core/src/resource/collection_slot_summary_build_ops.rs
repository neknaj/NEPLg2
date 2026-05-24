extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use crate::types::TypeKind;

use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_build_drop_traversal::collect_summary_drop_traversal_op;
use super::collection_slot_summary_build_event::collect_summary_event_op;
use super::collection_slot_summary_build_nested::{
    apply_summary_condition_fact, collect_nested_summary_ops,
    collect_nested_summary_ops_from_state, collect_nested_summary_ops_with_condition,
    collect_nested_summary_path, collect_nested_summary_path_from_state, push_merge_summary,
};
use super::collection_slot_summary_build_range_certificate::loop_drop_traversal_range_certificates;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_build_transform_range::{
    collect_summary_transform_range_op, loop_transform_range_certificates,
    transform_range_certificate_candidate_for_op,
};
use super::collection_slot_summary_match_state::collection_slot_summary_match_arm_entry_state;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummaryIndex, CollectionSlotLifecycleSummaryOp,
    CollectionSlotLifecycleSummaryRelocateProof,
};
use super::collection_slot_summary_target::summary_place_for_params_with_aliases;
use super::collection_slot_summary_translate::{
    collect_direct_call_summary_ops, collect_indirect_call_summary_ops,
};
use super::drop_point_path::ResourceDropPointPath;
use super::initialized::ResourceCheckEngine;
use super::initialized_path_state::{ResourceCheckState, ResourcePathAlternatives};
use super::model::{ResourceBlockId, ResourceLocal, ResourceOp};
use super::report::ResourceCheckOperation;

pub(super) fn collect_summary_ops_from_ops(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &mut ResourceCheckEngine<'_>,
    state: &mut CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
) {
    for op in ops {
        collect_summary_ops_from_op(out, engine, state, params, collection_slot_summaries, op);
        let mut pending_range_certificates = if let ResourceOp::Loop {
            condition_ops,
            condition_fact,
            body_ops,
            ..
        } = op
        {
            loop_drop_traversal_range_certificates(
                engine,
                state,
                condition_ops,
                condition_fact.as_ref(),
                body_ops,
            )
        } else {
            Vec::new()
        };
        let mut pending_transform_range_certificates = if let ResourceOp::Loop {
            condition_ops,
            condition_fact,
            body_ops,
            ..
        } = op
        {
            loop_transform_range_certificates(
                engine,
                state,
                condition_ops,
                condition_fact.as_ref(),
                body_ops,
            )
        } else {
            Vec::new()
        };
        apply_summary_state_after_op(engine, state, params, collection_slot_summaries, op);
        state.retain_drop_traversal_range_certificates_after_op(engine.types, op);
        state.retain_transform_range_certificates_after_op(engine.types, op);
        state
            .drop_traversal_range_certificates
            .append(&mut pending_range_certificates);
        state
            .transform_range_certificates
            .append(&mut pending_transform_range_certificates);
        engine.auto_drop_points.clear();
    }
}

fn apply_summary_state_after_op(
    engine: &mut ResourceCheckEngine<'_>,
    state: &mut CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    op: &ResourceOp,
) {
    let pre_state = state.clone();
    engine.check_ops(
        &mut state.cells,
        &mut state.collection_slots,
        &mut state.raw_aliases,
        &mut state.function_aliases,
        &mut state.pending_reallocs,
        &mut state.variant_initializations,
        core::slice::from_ref(op),
        ResourceDropPointPath {
            block: ResourceBlockId(usize::MAX),
            steps: Vec::new(),
        },
    );
    apply_summary_transform_range_state(engine, state, op);
    apply_nested_summary_transform_range_control_state(
        engine,
        state,
        &pre_state,
        params,
        collection_slot_summaries,
        op,
    );
}

fn apply_summary_transform_range_state(
    engine: &mut ResourceCheckEngine<'_>,
    state: &mut CollectionSlotSummaryBuildState,
    op: &ResourceOp,
) {
    let ResourceOp::CollectionSlotTransformRange {
        source_storage,
        source_initialized_count,
        output_storage,
        output_initialized_count,
        expected_ty,
        span,
    } = op
    else {
        return;
    };
    let Some(candidate) = transform_range_certificate_candidate_for_op(
        state,
        source_storage,
        source_initialized_count,
        output_storage,
        output_initialized_count,
        *expected_ty,
    ) else {
        return;
    };
    engine.apply_certified_collection_slot_transform_range_with_aliases(
        &mut state.cells,
        &mut state.collection_slots,
        &state.raw_aliases,
        &candidate.source_storage,
        &candidate.source_initialized_count,
        &candidate.output_storage,
        &candidate.output_initialized_count,
        candidate.expected_ty,
        candidate.certificate,
        *span,
    );
}

fn apply_nested_summary_transform_range_control_state(
    engine: &mut ResourceCheckEngine<'_>,
    state: &mut CollectionSlotSummaryBuildState,
    pre_state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    op: &ResourceOp,
) {
    if !op_contains_transform_range_marker(op) {
        return;
    }
    let Some(control_state) = nested_summary_transform_range_control_state(
        engine,
        pre_state,
        params,
        collection_slot_summaries,
        op,
    ) else {
        return;
    };
    state.collection_slots = control_state.collection_slots;
    if let Some(paths) = control_state.path_alternatives {
        engine.path_alternatives = ResourcePathAlternatives::from_states(paths);
    }
}

struct NestedSummaryTransformRangeControlState {
    collection_slots: CollectionSlotStateTable,
    path_alternatives: Option<Vec<ResourceCheckState>>,
}

fn nested_summary_transform_range_control_state(
    engine: &mut ResourceCheckEngine<'_>,
    pre_state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    op: &ResourceOp,
) -> Option<NestedSummaryTransformRangeControlState> {
    match op {
        ResourceOp::Branch {
            output,
            condition_fact,
            then_ops,
            then_value,
            else_ops,
            else_value,
            span,
            ..
        } => {
            let mut then_state = pre_state.clone();
            apply_summary_condition_fact(&mut then_state, condition_fact.as_ref(), true);
            let then_state = collect_nested_summary_path_from_state(
                engine,
                then_state,
                params,
                collection_slot_summaries,
                then_ops,
            )
            .1;
            let mut else_state = pre_state.clone();
            apply_summary_condition_fact(&mut else_state, condition_fact.as_ref(), false);
            let else_state = collect_nested_summary_path_from_state(
                engine,
                else_state,
                params,
                collection_slot_summaries,
                else_ops,
            )
            .1;
            let mut paths = Vec::new();
            let mut paths_available = true;
            if !summary_place_is_never(engine, then_value) {
                paths.extend(engine.transfer_control_value_path_states(
                    vec![summary_state_to_resource_state(then_state)],
                    then_value,
                    output,
                    ResourceCheckOperation::BranchValue,
                    *span,
                    &mut paths_available,
                ));
            }
            if !summary_place_is_never(engine, else_value) {
                paths.extend(engine.transfer_control_value_path_states(
                    vec![summary_state_to_resource_state(else_state)],
                    else_value,
                    output,
                    ResourceCheckOperation::BranchValue,
                    *span,
                    &mut paths_available,
                ));
            }
            control_state_from_resource_paths(paths)
        }
        ResourceOp::Loop {
            condition_ops,
            condition_fact,
            body_ops,
            ..
        } => {
            let mut condition_state = collect_nested_summary_path_from_state(
                engine,
                pre_state.clone(),
                params,
                collection_slot_summaries,
                condition_ops,
            )
            .1;
            let mut exit_state = condition_state.clone();
            apply_summary_condition_fact(&mut exit_state, condition_fact.as_ref(), false);
            apply_summary_condition_fact(&mut condition_state, condition_fact.as_ref(), true);
            let body_state = collect_nested_summary_path_from_state(
                engine,
                condition_state,
                params,
                collection_slot_summaries,
                body_ops,
            )
            .1;
            let paths = vec![exit_state.collection_slots, body_state.collection_slots];
            Some(NestedSummaryTransformRangeControlState {
                collection_slots: CollectionSlotStateTable::merge_paths(&paths),
                path_alternatives: None,
            })
        }
        ResourceOp::Match {
            output,
            scrutinee,
            arms,
            ..
        } => {
            let mut paths = Vec::new();
            let mut paths_available = true;
            for arm in arms {
                let Some(arm_state) = collection_slot_summary_match_arm_entry_state(
                    engine, pre_state, scrutinee, arm,
                ) else {
                    continue;
                };
                let arm_state = collect_nested_summary_path_from_state(
                    engine,
                    arm_state,
                    params,
                    collection_slot_summaries,
                    &arm.ops,
                )
                .1;
                if summary_place_is_never(engine, &arm.value) {
                    continue;
                }
                paths.extend(engine.transfer_control_value_path_states(
                    vec![summary_state_to_resource_state(arm_state)],
                    &arm.value,
                    output,
                    ResourceCheckOperation::MatchValue,
                    arm.span,
                    &mut paths_available,
                ));
            }
            control_state_from_resource_paths(paths)
        }
        _ => None,
    }
}

fn control_state_from_resource_paths(
    paths: Vec<ResourceCheckState>,
) -> Option<NestedSummaryTransformRangeControlState> {
    if paths.is_empty() {
        return None;
    }
    let collection_slot_paths = paths
        .iter()
        .map(|path| path.collection_slots.clone())
        .collect::<Vec<_>>();
    Some(NestedSummaryTransformRangeControlState {
        collection_slots: CollectionSlotStateTable::merge_paths(&collection_slot_paths),
        path_alternatives: Some(paths),
    })
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

fn summary_place_is_never(engine: &ResourceCheckEngine<'_>, place: &super::model::Place) -> bool {
    matches!(
        engine.types.get_ref(engine.types.resolve_id(place.ty)),
        TypeKind::Never
    )
}

fn op_contains_transform_range_marker(op: &ResourceOp) -> bool {
    match op {
        ResourceOp::CollectionSlotTransformRange { .. } => true,
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            ops_contain_transform_range_marker(then_ops)
                || ops_contain_transform_range_marker(else_ops)
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            ops_contain_transform_range_marker(condition_ops)
                || ops_contain_transform_range_marker(body_ops)
        }
        ResourceOp::Match { arms, .. } => arms
            .iter()
            .any(|arm| ops_contain_transform_range_marker(&arm.ops)),
        ResourceOp::DeclareLocal { .. }
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
        | ResourceOp::Construct { .. }
        | ResourceOp::Expr { .. } => false,
    }
}

fn ops_contain_transform_range_marker(ops: &[ResourceOp]) -> bool {
    ops.iter().any(op_contains_transform_range_marker)
}

pub(super) fn collect_summary_ops_from_op(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
    op: &ResourceOp,
) {
    match op {
        ResourceOp::CollectionSlotLifecycle { target, event, .. } => {
            collect_summary_event_op(out, engine, state, params, target, *event);
        }
        ResourceOp::CollectionStorageRelocate {
            old_storage,
            new_storage,
            ..
        } => {
            let old_storage_place = state
                .raw_aliases
                .canonicalize_owner_cell_address(old_storage);
            let new_storage_place = state
                .raw_aliases
                .canonicalize_owner_cell_address(new_storage);
            if !state
                .pending_reallocs
                .certified_storage_relocation_available(&old_storage_place, &new_storage_place)
            {
                return;
            }
            if let (Some(old_storage), Some(new_storage)) = (
                summary_place_for_params_with_aliases(
                    params,
                    &state.raw_aliases,
                    &old_storage_place,
                ),
                summary_place_for_params_with_aliases(
                    params,
                    &state.raw_aliases,
                    &new_storage_place,
                ),
            ) {
                out.push(CollectionSlotLifecycleSummaryOp::Relocate {
                    old_storage,
                    new_storage,
                    proof: CollectionSlotLifecycleSummaryRelocateProof::RawStorageRelocation,
                });
            }
        }
        ResourceOp::CollectionSlotDropTraversal {
            storage,
            initialized_count,
            expected_ty,
            ..
        } => {
            collect_summary_drop_traversal_op(
                out,
                engine,
                state,
                params,
                storage,
                initialized_count,
                *expected_ty,
            );
        }
        ResourceOp::CollectionSlotTransformRange {
            source_storage,
            source_initialized_count,
            output_storage,
            output_initialized_count,
            expected_ty,
            ..
        } => {
            collect_summary_transform_range_op(
                out,
                state,
                params,
                source_storage,
                source_initialized_count,
                output_storage,
                output_initialized_count,
                *expected_ty,
            );
        }
        ResourceOp::Call { target, args, .. } => {
            collect_direct_call_summary_ops(
                out,
                engine,
                target,
                args,
                params,
                &state.raw_aliases,
                collection_slot_summaries,
            );
        }
        ResourceOp::IndirectCall { callee, args, .. } => {
            collect_indirect_call_summary_ops(
                out,
                engine,
                state,
                callee,
                args,
                params,
                collection_slot_summaries,
            );
        }
        ResourceOp::Branch {
            condition_fact,
            then_ops,
            else_ops,
            ..
        } => {
            let then_path = collect_nested_summary_ops_with_condition(
                engine,
                state,
                params,
                collection_slot_summaries,
                condition_fact.as_ref(),
                true,
                then_ops,
            );
            let else_path = collect_nested_summary_ops_with_condition(
                engine,
                state,
                params,
                collection_slot_summaries,
                condition_fact.as_ref(),
                false,
                else_ops,
            );
            push_merge_summary(out, vec![then_path, else_path]);
        }
        ResourceOp::Loop {
            condition_ops,
            condition_fact,
            body_ops,
            ..
        } => {
            let (condition_ops, mut condition_state) = collect_nested_summary_path(
                engine,
                state,
                params,
                collection_slot_summaries,
                condition_ops,
            );
            apply_summary_condition_fact(&mut condition_state, condition_fact.as_ref(), true);
            let body_ops = collect_nested_summary_ops_from_state(
                engine,
                condition_state,
                params,
                collection_slot_summaries,
                body_ops,
            );
            if !condition_ops.is_empty() || !body_ops.is_empty() {
                out.push(CollectionSlotLifecycleSummaryOp::Loop {
                    condition_ops,
                    body_ops,
                });
            }
        }
        ResourceOp::Match {
            scrutinee, arms, ..
        } => {
            let mut paths = Vec::new();
            for arm in arms {
                let Some(arm_state) =
                    collection_slot_summary_match_arm_entry_state(engine, state, scrutinee, arm)
                else {
                    continue;
                };
                paths.push(collect_nested_summary_ops(
                    engine,
                    &arm_state,
                    params,
                    collection_slot_summaries,
                    &arm.ops,
                ));
            }
            push_merge_summary(out, paths);
        }
        ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::FunctionValue { .. }
        | ResourceOp::RawMemory { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::Construct { .. }
        | ResourceOp::Expr { .. } => {}
    }
}
