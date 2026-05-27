extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummary, CollectionSlotLifecycleReturnPath,
};
use super::collection_slot_summary_projection::compose_translated_summary_suffix_for_params;
use super::collection_slot_summary_return_model::CollectionSlotLifecycleReturnTransfer;
use super::collection_slot_summary_return_path_condition::translate_return_path_preconditions_for_call;
use super::collection_slot_summary_return_path_model::{push_return_path, ReturnPathBuildState};
use super::collection_slot_summary_return_path_slots::translate_return_slots;
use super::collection_slot_summary_return_path_variant::return_path_matches_callsite_variants;
use super::collection_slot_summary_return_range::translate_return_ranges;
use super::collection_slot_summary_return_unique::push_return_transfer;
use super::collection_slot_summary_target::{
    instantiate_summary_target_with_aliases, summary_place_for_params_with_aliases,
};
use super::collection_slot_summary_translate::translate_summary_ops_through_args;
use super::i32_scalar_return_facts::translate_i32_scalar_return_facts_for_call;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, PlaceProjection, ResourceCallTarget, ResourceLocal};

pub(super) fn collect_return_paths_from_call_summary(
    out: &mut Vec<CollectionSlotLifecycleReturnPath>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    callsite: ReturnPathBuildState,
    args: &[Place],
    target: &ResourceCallTarget,
    target_suffix: &[PlaceProjection],
) {
    let ResourceCallTarget::User { name, type_args } = target else {
        return;
    };
    if let Some(summary) = engine.collection_slot_summaries.get(name) {
        collect_return_paths_from_summary(
            out,
            engine,
            params,
            callsite,
            args,
            summary,
            type_args,
            target_suffix,
        );
    }
}

pub(super) fn collect_return_paths_from_indirect_call_summary(
    out: &mut Vec<CollectionSlotLifecycleReturnPath>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    callsite: ReturnPathBuildState,
    callee: &Place,
    args: &[Place],
    target_suffix: &[PlaceProjection],
) {
    for function in callsite.state.function_aliases.functions(callee) {
        if let Some(summary) = engine.collection_slot_summaries.get(function) {
            collect_return_paths_from_summary(
                out,
                engine,
                params,
                callsite.clone(),
                args,
                summary,
                &[],
                target_suffix,
            );
        }
    }
}

fn collect_return_paths_from_summary(
    out: &mut Vec<CollectionSlotLifecycleReturnPath>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    callsite: ReturnPathBuildState,
    args: &[Place],
    summary: &CollectionSlotLifecycleFunctionSummary,
    type_args: &[crate::types::TypeId],
    target_suffix: &[PlaceProjection],
) {
    if summary.return_paths.is_empty() {
        collect_legacy_return_path_from_summary(
            out,
            engine,
            params,
            callsite,
            args,
            summary,
            type_args,
            target_suffix,
        );
        return;
    }
    for callee_path in &summary.return_paths {
        if !return_path_matches_callsite_variants(
            engine,
            args,
            &callsite.state.raw_aliases,
            &callsite.state.variant_initializations,
            callee_path,
        ) {
            continue;
        }
        let mut translated_ops = Vec::new();
        translate_summary_ops_through_args(
            &mut translated_ops,
            engine,
            args,
            params,
            &callsite.state.raw_aliases,
            &callee_path.ops,
        );
        let mut return_transfers = Vec::new();
        translate_return_transfers(
            &mut return_transfers,
            engine,
            params,
            &callsite,
            args,
            &callee_path.return_transfers,
            target_suffix,
        );
        let return_slots = translate_return_slots(
            engine,
            args,
            params,
            &callee_path.return_slots,
            target_suffix,
        );
        let return_ranges = translate_return_ranges(
            engine,
            args,
            params,
            &summary.type_params,
            type_args,
            &callee_path.return_ranges,
            target_suffix,
        );
        if translated_ops.is_empty()
            && return_transfers.is_empty()
            && return_slots.is_empty()
            && return_ranges.is_empty()
        {
            continue;
        }
        let preconditions = translate_return_path_preconditions_for_call(
            engine,
            args,
            params,
            &callsite.state.raw_aliases,
            &callee_path.preconditions,
        );
        // collection lifecycle を持たない callee path はこの summary では運ばない。
        // lifecycle fact がある path だけに scalar facts を付随させると、caller 側の
        // range 証明に必要な条件は残しつつ、scalar-only path の再翻訳を避けられる。
        let i32_scalar_facts = translate_i32_scalar_return_facts_for_call(
            params,
            engine.types,
            &callsite.state.raw_aliases,
            args,
            target_suffix,
            &callee_path.i32_scalar_facts,
        );
        let mut ops = callsite.ops.clone();
        ops.extend(translated_ops);
        push_return_path(
            out,
            CollectionSlotLifecycleReturnPath {
                return_variant: if target_suffix.is_empty() {
                    callee_path.return_variant.clone()
                } else {
                    None
                },
                preconditions,
                ops,
                return_transfers,
                return_slots,
                return_ranges,
                i32_scalar_facts,
            },
        );
    }
}

fn collect_legacy_return_path_from_summary(
    out: &mut Vec<CollectionSlotLifecycleReturnPath>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    callsite: ReturnPathBuildState,
    args: &[Place],
    summary: &CollectionSlotLifecycleFunctionSummary,
    type_args: &[crate::types::TypeId],
    target_suffix: &[PlaceProjection],
) {
    let mut translated_ops = Vec::new();
    translate_summary_ops_through_args(
        &mut translated_ops,
        engine,
        args,
        params,
        &callsite.state.raw_aliases,
        &summary.ops,
    );
    let mut return_transfers = Vec::new();
    translate_return_transfers(
        &mut return_transfers,
        engine,
        params,
        &callsite,
        args,
        &summary.return_transfers,
        target_suffix,
    );
    let return_slots =
        translate_return_slots(engine, args, params, &summary.return_slots, target_suffix);
    let return_ranges = translate_return_ranges(
        engine,
        args,
        params,
        &summary.type_params,
        type_args,
        &summary.return_ranges,
        target_suffix,
    );
    if translated_ops.is_empty()
        && return_transfers.is_empty()
        && return_slots.is_empty()
        && return_ranges.is_empty()
    {
        return;
    }
    let mut ops = callsite.ops;
    ops.extend(translated_ops);
    push_return_path(
        out,
        CollectionSlotLifecycleReturnPath {
            return_variant: None,
            preconditions: Vec::new(),
            ops,
            return_transfers,
            return_slots,
            return_ranges,
            i32_scalar_facts: Default::default(),
        },
    );
}

fn translate_return_transfers(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    callsite: &ReturnPathBuildState,
    args: &[Place],
    transfers: &[CollectionSlotLifecycleReturnTransfer],
    target_suffix: &[PlaceProjection],
) {
    for transfer in transfers {
        let Some(source) = instantiate_summary_target_with_aliases(
            engine,
            args,
            &callsite.state.raw_aliases,
            &transfer.source,
        ) else {
            continue;
        };
        let source = callsite
            .state
            .raw_aliases
            .canonicalize_owner_cell_address(&source);
        let Some(source) =
            summary_place_for_params_with_aliases(params, &callsite.state.raw_aliases, &source)
        else {
            continue;
        };
        let Some(composed_target_suffix) = compose_translated_summary_suffix_for_params(
            engine,
            args,
            params,
            target_suffix,
            &transfer.target_suffix,
        ) else {
            continue;
        };
        push_return_transfer(
            out,
            CollectionSlotLifecycleReturnTransfer {
                source,
                target_suffix: composed_target_suffix,
                target_ty: transfer.target_ty,
            },
        );
    }
}
