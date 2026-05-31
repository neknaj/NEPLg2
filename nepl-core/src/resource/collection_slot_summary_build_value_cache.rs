extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummary, CollectionSlotLifecycleSummaryDropTraversalCoverage,
    CollectionSlotLifecycleSummaryOp,
};
use super::model::{ResourceFunction, ResourceModule};
use super::owner_summary_type_params::owner_summary_type_params;
use super::resource_summary_value_cache::{
    ResourceSummaryDropTraversalForallLeafEntryCandidate, ResourceSummaryValueCache,
    ResourceSummaryValueCacheContext,
};

pub(super) fn preseed_collection_slot_lifecycle_summaries_from_value_cache(
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    relevant_functions: &[bool],
    dependencies: &[Vec<usize>],
    worklist_relevant_functions: &mut [bool],
    preseeded_functions: &mut [bool],
    summaries: &mut Vec<CollectionSlotLifecycleFunctionSummary>,
) {
    for (function_index, function) in module.functions.iter().enumerate() {
        if !relevant_functions
            .get(function_index)
            .copied()
            .unwrap_or(false)
        {
            continue;
        }
        let dependencies = dependencies
            .get(function_index)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        if !function_allows_complete_leaf_entry_replay(function, dependencies) {
            continue;
        }
        cache.record_drop_traversal_replay_probe_function();
        let type_params = owner_summary_type_params(types, function);
        let Some(ops) =
            cache.replay_drop_traversal_forall_leaf_entry(context, types, function, &type_params)
        else {
            cache.record_drop_traversal_replay_miss_function();
            continue;
        };
        cache.record_drop_traversal_replay_hit_function();
        summaries.push(CollectionSlotLifecycleFunctionSummary {
            function: function.name.clone(),
            type_params,
            ops,
            return_transfers: Vec::new(),
            return_slots: Vec::new(),
            return_ranges: Vec::new(),
            return_paths: Vec::new(),
        });
        if let Some(is_relevant) = worklist_relevant_functions.get_mut(function_index) {
            *is_relevant = false;
        }
        if let Some(is_preseeded) = preseeded_functions.get_mut(function_index) {
            *is_preseeded = true;
        }
    }
}

pub(super) fn record_resource_summary_value_cache_candidates(
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: &[Vec<usize>],
    preseeded_functions: &[bool],
    summaries: &[CollectionSlotLifecycleFunctionSummary],
) {
    let mut candidates = Vec::new();
    let mut functions = BTreeMap::new();
    for (index, function) in module.functions.iter().enumerate() {
        functions.insert(function.name.as_str(), (index, function));
    }
    for summary in summaries {
        let Some((function_index, function)) = functions.get(summary.function.as_str()) else {
            continue;
        };
        if preseeded_functions
            .get(*function_index)
            .copied()
            .unwrap_or(false)
        {
            continue;
        }
        collect_resource_summary_value_cache_leaf_entry_candidate_from_summary(
            &mut candidates,
            cache,
            context,
            types,
            function,
            dependencies
                .get(*function_index)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            summary,
        );
    }
    cache.record_drop_traversal_forall_leaf_entry_candidates(candidates);
}

fn collect_resource_summary_value_cache_leaf_entry_candidate_from_summary(
    candidates: &mut Vec<ResourceSummaryDropTraversalForallLeafEntryCandidate>,
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    function: &ResourceFunction,
    dependencies: &[usize],
    summary: &CollectionSlotLifecycleFunctionSummary,
) {
    let eligible_op_count = top_level_forall_drop_traversal_op_count(&summary.ops);
    if eligible_op_count == 0 {
        return;
    }
    if !summary_is_complete_forall_drop_traversal_leaf_entry(summary)
        || !function_allows_complete_leaf_entry_replay(function, dependencies)
    {
        for _ in 0..eligible_op_count {
            cache.record_drop_traversal_forall_bypass();
        }
        return;
    }
    cache.record_drop_traversal_forall_recomputed_ops(eligible_op_count);
    match cache.drop_traversal_forall_leaf_entry_candidate(
        context,
        types,
        function,
        &summary.type_params,
        &summary.ops,
    ) {
        Some(candidate) => candidates.push(candidate),
        None => {
            for _ in 0..eligible_op_count {
                cache.record_drop_traversal_forall_bypass();
            }
        }
    }
}

fn function_allows_complete_leaf_entry_replay(
    function: &ResourceFunction,
    dependencies: &[usize],
) -> bool {
    dependencies.is_empty() && !function_has_indirect_call(function)
}

fn summary_is_complete_forall_drop_traversal_leaf_entry(
    summary: &CollectionSlotLifecycleFunctionSummary,
) -> bool {
    !summary.ops.is_empty()
        && summary.return_transfers.is_empty()
        && summary.return_slots.is_empty()
        && summary.return_ranges.is_empty()
        && summary.return_paths.is_empty()
        && summary.ops.iter().all(op_is_forall_drop_traversal_leaf)
}

fn top_level_forall_drop_traversal_op_count(ops: &[CollectionSlotLifecycleSummaryOp]) -> usize {
    ops.iter()
        .filter(|op| op_is_forall_drop_traversal_leaf(*op))
        .count()
}

fn op_is_forall_drop_traversal_leaf(op: &CollectionSlotLifecycleSummaryOp) -> bool {
    matches!(
        op,
        CollectionSlotLifecycleSummaryOp::DropTraversal {
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                _
            ),
            ..
        }
    )
}

fn function_has_indirect_call(function: &ResourceFunction) -> bool {
    function
        .blocks
        .iter()
        .any(|block| ops_have_indirect_call(&block.ops))
}

fn ops_have_indirect_call(ops: &[super::model::ResourceOp]) -> bool {
    for op in ops {
        match op {
            super::model::ResourceOp::IndirectCall { .. } => return true,
            super::model::ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                if ops_have_indirect_call(then_ops) || ops_have_indirect_call(else_ops) {
                    return true;
                }
            }
            super::model::ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                if ops_have_indirect_call(condition_ops) || ops_have_indirect_call(body_ops) {
                    return true;
                }
            }
            super::model::ResourceOp::Match { arms, .. } => {
                if arms.iter().any(|arm| ops_have_indirect_call(&arm.ops)) {
                    return true;
                }
            }
            super::model::ResourceOp::Expr { .. }
            | super::model::ResourceOp::DeclareLocal { .. }
            | super::model::ResourceOp::Read { .. }
            | super::model::ResourceOp::Assign { .. }
            | super::model::ResourceOp::Borrow { .. }
            | super::model::ResourceOp::Move { .. }
            | super::model::ResourceOp::Drop { .. }
            | super::model::ResourceOp::EndScope { .. }
            | super::model::ResourceOp::CallEffect { .. }
            | super::model::ResourceOp::FunctionValue { .. }
            | super::model::ResourceOp::Call { .. }
            | super::model::ResourceOp::RawMemory { .. }
            | super::model::ResourceOp::RawAddressAlias { .. }
            | super::model::ResourceOp::RawAddressView { .. }
            | super::model::ResourceOp::StorageOrigin { .. }
            | super::model::ResourceOp::CollectionSlotLifecycle { .. }
            | super::model::ResourceOp::CollectionStorageRelocate { .. }
            | super::model::ResourceOp::CollectionSlotDropTraversal { .. }
            | super::model::ResourceOp::CollectionSlotTransformRange { .. }
            | super::model::ResourceOp::Construct { .. } => {}
        }
    }
    false
}
