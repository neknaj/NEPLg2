extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_build_nested::push_merge_summary;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummaryIndex, CollectionSlotLifecycleSummaryOp,
};
use super::collection_slot_summary_target::{
    instantiate_summary_target_with_aliases, summary_place_for_params_with_aliases_and_types,
    translate_summary_target_for_params_with_aliases,
};
use super::collection_slot_summary_translate_drop::translate_drop_traversal_summary_op;
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
                if let Some(target) = translate_summary_target_for_params_with_aliases(
                    engine,
                    args,
                    params,
                    raw_aliases,
                    target,
                ) {
                    out.push(CollectionSlotLifecycleSummaryOp::Event {
                        target,
                        event: *event,
                        proof: *proof,
                    });
                    continue;
                }
                let Some(actual) =
                    instantiate_summary_target_with_aliases(engine, args, raw_aliases, target)
                else {
                    continue;
                };
                if let Some(target) = summary_place_for_params_with_aliases_and_types(
                    params,
                    Some(engine.types),
                    raw_aliases,
                    &actual,
                ) {
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
                let Some(actual_old) =
                    instantiate_summary_target_with_aliases(engine, args, raw_aliases, old_storage)
                else {
                    continue;
                };
                let Some(actual_new) =
                    instantiate_summary_target_with_aliases(engine, args, raw_aliases, new_storage)
                else {
                    continue;
                };
                let actual_old = raw_aliases.canonicalize_owner_cell_address(&actual_old);
                let actual_new = raw_aliases.canonicalize_owner_cell_address(&actual_new);
                if let (Some(old_storage), Some(new_storage)) = (
                    summary_place_for_params_with_aliases_and_types(
                        params,
                        Some(engine.types),
                        raw_aliases,
                        &actual_old,
                    ),
                    summary_place_for_params_with_aliases_and_types(
                        params,
                        Some(engine.types),
                        raw_aliases,
                        &actual_new,
                    ),
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
                initialized_count,
                expected_ty,
                coverage,
            } => {
                translate_drop_traversal_summary_op(
                    out,
                    engine,
                    args,
                    params,
                    raw_aliases,
                    storage,
                    initialized_count,
                    *expected_ty,
                    coverage,
                );
            }
            CollectionSlotLifecycleSummaryOp::TransformRange {
                source_storage,
                source_initialized_count,
                output_storage,
                output_initialized_count,
                expected_ty,
                certificate,
            } => {
                let Some(source_storage) = translate_summary_place_through_args(
                    engine,
                    args,
                    params,
                    raw_aliases,
                    source_storage,
                ) else {
                    continue;
                };
                let Some(source_initialized_count) = translate_summary_place_through_args(
                    engine,
                    args,
                    params,
                    raw_aliases,
                    source_initialized_count,
                ) else {
                    continue;
                };
                let output_storage = translate_summary_place_through_args(
                    engine,
                    args,
                    params,
                    raw_aliases,
                    output_storage,
                );
                let output_initialized_count = translate_summary_place_through_args(
                    engine,
                    args,
                    params,
                    raw_aliases,
                    output_initialized_count,
                );
                match (output_storage, output_initialized_count) {
                    (Some(output_storage), Some(output_initialized_count)) => {
                        out.push(CollectionSlotLifecycleSummaryOp::TransformRange {
                            source_storage,
                            source_initialized_count,
                            output_storage,
                            output_initialized_count,
                            expected_ty: *expected_ty,
                            certificate: *certificate,
                        });
                    }
                    (None, _) | (Some(_), None) => {
                        out.push(
                            CollectionSlotLifecycleSummaryOp::TransformRangeSourceDrain {
                                source_storage,
                                source_initialized_count,
                                expected_ty: *expected_ty,
                                certificate: *certificate,
                            },
                        );
                    }
                }
            }
            CollectionSlotLifecycleSummaryOp::TransformRangeSourceDrain {
                source_storage,
                source_initialized_count,
                expected_ty,
                certificate,
            } => {
                let Some(source_storage) = translate_summary_place_through_args(
                    engine,
                    args,
                    params,
                    raw_aliases,
                    source_storage,
                ) else {
                    continue;
                };
                let Some(source_initialized_count) = translate_summary_place_through_args(
                    engine,
                    args,
                    params,
                    raw_aliases,
                    source_initialized_count,
                ) else {
                    continue;
                };
                out.push(
                    CollectionSlotLifecycleSummaryOp::TransformRangeSourceDrain {
                        source_storage,
                        source_initialized_count,
                        expected_ty: *expected_ty,
                        certificate: *certificate,
                    },
                );
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

fn translate_summary_place_through_args(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    place: &super::collection_slot_summary_model::CollectionSlotLifecycleSummaryPlace,
) -> Option<super::collection_slot_summary_model::CollectionSlotLifecycleSummaryPlace> {
    if let Some(translated) =
        translate_summary_target_for_params_with_aliases(engine, args, params, raw_aliases, place)
    {
        return Some(translated);
    }
    let actual = instantiate_summary_target_with_aliases(engine, args, raw_aliases, place)?;
    summary_place_for_params_with_aliases_and_types(
        params,
        Some(engine.types),
        raw_aliases,
        &actual,
    )
}
