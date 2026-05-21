extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use super::collection_slot_summary_build_drop_traversal::collect_summary_drop_traversal_op;
use super::collection_slot_summary_build_event::collect_summary_event_op;
use super::collection_slot_summary_build_nested::{
    apply_summary_condition_fact, collect_nested_summary_ops,
    collect_nested_summary_ops_from_state, collect_nested_summary_ops_with_condition,
    collect_nested_summary_path, push_merge_summary,
};
use super::collection_slot_summary_build_range_certificate::loop_drop_traversal_range_certificates;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_match_state::collection_slot_summary_match_arm_entry_state;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummaryIndex, CollectionSlotLifecycleSummaryOp,
    CollectionSlotLifecycleSummaryRelocateProof,
};
use super::collection_slot_summary_target::summary_place_for_params;
use super::collection_slot_summary_translate::{
    collect_direct_call_summary_ops, collect_indirect_call_summary_ops,
};
use super::drop_point_path::ResourceDropPointPath;
use super::initialized::ResourceCheckEngine;
use super::model::{ResourceBlockId, ResourceLocal, ResourceOp};

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
        state.retain_drop_traversal_range_certificates_after_op(engine.types, op);
        state
            .drop_traversal_range_certificates
            .append(&mut pending_range_certificates);
        engine.auto_drop_points.clear();
    }
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
                summary_place_for_params(params, &old_storage_place),
                summary_place_for_params(params, &new_storage_place),
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
