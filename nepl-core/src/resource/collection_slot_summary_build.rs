extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::collection_slot_summary_build_ops::collect_summary_ops_from_ops;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_build_value_cache::{
    preseed_collection_slot_lifecycle_summaries_from_value_cache,
    record_resource_summary_value_cache_candidates,
};
use super::collection_slot_owner_carrier::type_carries_collection_slot_owner;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummary, CollectionSlotLifecycleFunctionSummaryIndex,
    CollectionSlotLifecycleReturnPath,
};
use super::collection_slot_summary_relevance::collection_slot_summary_relevant_functions;
use super::collection_slot_summary_return_build::collect_return_facts_from_terminator;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias_flow::{
    RawCellAddressReturnSummary, RawCellAddressReturnSummaryIndex,
};
use super::initialized_scalar_flow::{I32ScalarReturnSummary, I32ScalarReturnSummaryIndex};
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::model::{ResourceFunction, ResourceModule};
use super::owner_summary_type_params::owner_summary_type_params;
use super::report::ResourceCheckDeferred;
use super::resource_summary_value_cache::{
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};
use super::summary_dependency::ResourceSummaryDependencyGraph;
use super::summary_index::SummaryNameIndex;
use super::summary_worklist::SummaryWorklist;
use super::timing::ResourceFunctionTimer;

#[cfg(test)]
pub(super) fn compute_collection_slot_lifecycle_function_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    i32_scalar_summaries: &[I32ScalarReturnSummary],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
    summary_value_cache: Option<&mut ResourceSummaryValueCache>,
    summary_value_cache_context: Option<&ResourceSummaryValueCacheContext>,
) -> Vec<CollectionSlotLifecycleFunctionSummary> {
    let dependency_graph = ResourceSummaryDependencyGraph::build(module);
    compute_collection_slot_lifecycle_function_summaries_with_recomputations(
        module,
        types,
        raw_alias_summaries,
        i32_scalar_summaries,
        raw_init_summaries,
        &dependency_graph,
        summary_value_cache,
        summary_value_cache_context,
    )
    .0
}

pub(super) fn compute_collection_slot_lifecycle_function_summaries_with_recomputations(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    i32_scalar_summaries: &[I32ScalarReturnSummary],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
    dependency_graph: &ResourceSummaryDependencyGraph,
    mut summary_value_cache: Option<&mut ResourceSummaryValueCache>,
    summary_value_cache_context: Option<&ResourceSummaryValueCacheContext>,
) -> (Vec<CollectionSlotLifecycleFunctionSummary>, usize) {
    let mut summaries = Vec::new();
    let relevant_functions = collection_slot_summary_relevant_functions(module, types);
    let mut worklist_relevant_functions = relevant_functions.clone();
    let mut preseeded_functions = vec![false; module.functions.len()];
    if let (Some(cache), Some(context)) = (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        preseed_collection_slot_lifecycle_summaries_from_value_cache(
            cache,
            context,
            types,
            module,
            &relevant_functions,
            dependency_graph.dependencies(),
            &mut worklist_relevant_functions,
            &mut preseeded_functions,
            &mut summaries,
        );
    }
    let mut worklist = SummaryWorklist::new_filtered_with_dependency_graph(
        module,
        worklist_relevant_functions,
        dependency_graph,
    );
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(raw_alias_summaries);
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(i32_scalar_summaries);
    let raw_init_summary_index = RawCellInitializationFunctionSummaryIndex::new(raw_init_summaries);
    let mut summary_name_index = SummaryNameIndex::from_entries(&summaries);
    while let Some(function_index) = worklist.pop() {
        let function = &module.functions[function_index];
        let function_start = ResourceFunctionTimer::start();
        let summary = {
            let collection_summary_index = summary_name_index.as_summary_index(&summaries);
            function_collection_slot_lifecycle_summary(
                function,
                types,
                &raw_alias_summary_index,
                &i32_scalar_summary_index,
                &raw_init_summary_index,
                &collection_summary_index,
            )
        };
        function_start.log("collection_slot_summary", function);
        if update_collection_slot_lifecycle_summary(
            &mut summaries,
            &mut summary_name_index,
            summary,
        ) {
            worklist.notify_changed(function_index);
        }
    }
    if let (Some(cache), Some(context)) = (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        let candidate_skipped_functions = worklist.unrecomputed_initial_skips(&preseeded_functions);
        record_resource_summary_value_cache_candidates(
            cache,
            context,
            types,
            module,
            dependency_graph.dependencies(),
            &candidate_skipped_functions,
            &summaries,
        );
    }
    let recomputations = worklist.recomputations();
    (summaries, recomputations)
}

fn update_collection_slot_lifecycle_summary(
    summaries: &mut Vec<CollectionSlotLifecycleFunctionSummary>,
    summary_name_index: &mut SummaryNameIndex,
    summary: CollectionSlotLifecycleFunctionSummary,
) -> bool {
    let has_facts = !summary.ops.is_empty()
        || !summary.return_transfers.is_empty()
        || !summary.return_slots.is_empty()
        || !summary.return_ranges.is_empty()
        || !summary.return_paths.is_empty();
    let function = summary.function.clone();
    let position = summary_name_index.position(&function);
    match (has_facts, position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
        }
        (true, None) => {
            summary_name_index.insert_at_end(&function, summaries.len());
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
            summary_name_index.remove_and_shift(&function, index);
            true
        }
        (false, None) => false,
    }
}

fn function_collection_slot_lifecycle_summary(
    function: &ResourceFunction,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    i32_scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
) -> CollectionSlotLifecycleFunctionSummary {
    let signature_carries_owner = function_signature_carries_collection_slot_owner(types, function);
    if !signature_carries_owner {
        return CollectionSlotLifecycleFunctionSummary {
            function: function.name.clone(),
            type_params: owner_summary_type_params(types, function),
            ops: Vec::new(),
            return_transfers: Vec::new(),
            return_slots: Vec::new(),
            return_ranges: Vec::new(),
            return_paths: Vec::new(),
        };
    }
    let mut engine = ResourceCheckEngine {
        function: function.name.as_str(),
        types,
        raw_alias_summaries,
        i32_scalar_summaries,
        raw_init_summaries,
        collection_slot_summaries,
        transform_range_certificates: None,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
    };
    let mut state = CollectionSlotSummaryBuildState::new(types, function);
    let mut ops = Vec::new();
    let mut return_transfers = Vec::new();
    let mut return_slots = Vec::new();
    let mut return_ranges = Vec::new();
    let mut return_paths = Vec::<CollectionSlotLifecycleReturnPath>::new();
    for block in &function.blocks {
        let block_entry_state = state.clone();
        collect_summary_ops_from_ops(
            &mut ops,
            &mut engine,
            &mut state,
            &function.params,
            collection_slot_summaries,
            &block.ops,
        );
        collect_return_facts_from_terminator(
            &mut return_transfers,
            &mut return_slots,
            &mut return_ranges,
            &mut return_paths,
            &state,
            &engine,
            &function.params,
            &block_entry_state,
            &block.ops,
            &block.terminator,
        );
    }
    return_paths.retain(collection_return_path_carries_replay_facts);
    CollectionSlotLifecycleFunctionSummary {
        function: function.name.clone(),
        type_params: owner_summary_type_params(types, function),
        ops,
        return_transfers,
        return_slots,
        return_ranges,
        return_paths,
    }
}

fn function_signature_carries_collection_slot_owner(
    types: &TypeCtx,
    function: &ResourceFunction,
) -> bool {
    function
        .params
        .iter()
        .any(|param| type_carries_collection_slot_owner(types, param.place.ty))
        || type_carries_collection_slot_owner(types, function.result)
}

fn collection_return_path_carries_replay_facts(path: &CollectionSlotLifecycleReturnPath) -> bool {
    !path.ops.is_empty()
        || !path.return_transfers.is_empty()
        || !path.return_slots.is_empty()
        || !path.return_ranges.is_empty()
        || path.return_variant.is_some()
}

#[cfg(test)]
#[path = "collection_slot_summary_build_tests.rs"]
mod tests;
