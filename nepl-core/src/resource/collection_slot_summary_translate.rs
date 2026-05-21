extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_build_ops::push_merge_summary;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummaryIndex, CollectionSlotLifecycleSummaryOp,
};
use super::collection_slot_summary_target::{instantiate_summary_target, summary_place_for_params};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceCallTarget, ResourceLocal};

pub(super) fn collect_direct_call_summary_ops(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    target: &ResourceCallTarget,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
) {
    let ResourceCallTarget::User { name, .. } = target else {
        return;
    };
    let Some(summary) = collection_slot_summaries.get(name) else {
        return;
    };
    translate_summary_ops_through_args(out, engine, args, params, raw_aliases, &summary.ops);
}

pub(super) fn collect_indirect_call_summary_ops(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    callee: &Place,
    args: &[Place],
    params: &[ResourceLocal],
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
) {
    let mut paths = Vec::new();
    for function in state.function_aliases.functions(callee) {
        let mut path = Vec::new();
        if let Some(summary) = collection_slot_summaries.get(function) {
            translate_summary_ops_through_args(
                &mut path,
                engine,
                args,
                params,
                &state.raw_aliases,
                &summary.ops,
            );
        }
        paths.push(path);
    }
    push_merge_summary(out, paths);
}

pub(super) fn translate_summary_ops_through_args(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    ops: &[CollectionSlotLifecycleSummaryOp],
) {
    for op in ops {
        match op {
            CollectionSlotLifecycleSummaryOp::Event {
                target,
                event,
                proof,
            } => {
                let Some(actual) = instantiate_summary_target(engine, args, target) else {
                    continue;
                };
                let actual = raw_aliases.canonicalize_owner_cell_address(&actual);
                if let Some(target) = summary_place_for_params(params, &actual) {
                    out.push(CollectionSlotLifecycleSummaryOp::Event {
                        target,
                        event: *event,
                        proof: *proof,
                    });
                }
            }
            CollectionSlotLifecycleSummaryOp::Relocate {
                old_storage,
                new_storage,
                proof,
            } => {
                let Some(actual_old) = instantiate_summary_target(engine, args, old_storage) else {
                    continue;
                };
                let Some(actual_new) = instantiate_summary_target(engine, args, new_storage) else {
                    continue;
                };
                let actual_old = raw_aliases.canonicalize_owner_cell_address(&actual_old);
                let actual_new = raw_aliases.canonicalize_owner_cell_address(&actual_new);
                if let (Some(old_storage), Some(new_storage)) = (
                    summary_place_for_params(params, &actual_old),
                    summary_place_for_params(params, &actual_new),
                ) {
                    out.push(CollectionSlotLifecycleSummaryOp::Relocate {
                        old_storage,
                        new_storage,
                        proof: *proof,
                    });
                }
            }
            CollectionSlotLifecycleSummaryOp::DropTraversal {
                storage,
                expected_ty,
                proof,
            } => {
                let Some(actual) = instantiate_summary_target(engine, args, storage) else {
                    continue;
                };
                let actual = raw_aliases.canonicalize_owner_cell_address(&actual);
                if let Some(storage) = summary_place_for_params(params, &actual) {
                    out.push(CollectionSlotLifecycleSummaryOp::DropTraversal {
                        storage,
                        expected_ty: *expected_ty,
                        proof: *proof,
                    });
                }
            }
            CollectionSlotLifecycleSummaryOp::Merge { paths } => {
                let mut translated_paths = Vec::new();
                for path in paths {
                    let mut translated = Vec::new();
                    translate_summary_ops_through_args(
                        &mut translated,
                        engine,
                        args,
                        params,
                        raw_aliases,
                        path,
                    );
                    translated_paths.push(translated);
                }
                push_merge_summary(out, translated_paths);
            }
            CollectionSlotLifecycleSummaryOp::Loop {
                condition_ops,
                body_ops,
            } => {
                let mut translated_condition = Vec::new();
                translate_summary_ops_through_args(
                    &mut translated_condition,
                    engine,
                    args,
                    params,
                    raw_aliases,
                    condition_ops,
                );
                let mut translated_body = Vec::new();
                translate_summary_ops_through_args(
                    &mut translated_body,
                    engine,
                    args,
                    params,
                    raw_aliases,
                    body_ops,
                );
                if !translated_condition.is_empty() || !translated_body.is_empty() {
                    out.push(CollectionSlotLifecycleSummaryOp::Loop {
                        condition_ops: translated_condition,
                        body_ops: translated_body,
                    });
                }
            }
        }
    }
}
