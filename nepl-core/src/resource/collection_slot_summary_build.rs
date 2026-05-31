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
use super::summary_dependency::build_function_summary_dependencies;
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
    compute_collection_slot_lifecycle_function_summaries_with_recomputations(
        module,
        types,
        raw_alias_summaries,
        i32_scalar_summaries,
        raw_init_summaries,
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
    mut summary_value_cache: Option<&mut ResourceSummaryValueCache>,
    summary_value_cache_context: Option<&ResourceSummaryValueCacheContext>,
) -> (Vec<CollectionSlotLifecycleFunctionSummary>, usize) {
    let mut summaries = Vec::new();
    let relevant_functions = collection_slot_summary_relevant_functions(module, types);
    let dependencies = build_function_summary_dependencies(module);
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
            &dependencies,
            &mut worklist_relevant_functions,
            &mut preseeded_functions,
            &mut summaries,
        );
    }
    let mut worklist = SummaryWorklist::new_filtered(module, worklist_relevant_functions);
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(raw_alias_summaries);
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(i32_scalar_summaries);
    let raw_init_summary_index = RawCellInitializationFunctionSummaryIndex::new(raw_init_summaries);
    while let Some(function_index) = worklist.pop() {
        let collection_summary_index = CollectionSlotLifecycleFunctionSummaryIndex::new(&summaries);
        let function = &module.functions[function_index];
        let function_start = ResourceFunctionTimer::start();
        let summary = function_collection_slot_lifecycle_summary(
            function,
            types,
            &raw_alias_summary_index,
            &i32_scalar_summary_index,
            &raw_init_summary_index,
            &collection_summary_index,
        );
        function_start.log("collection_slot_summary", function);
        if update_collection_slot_lifecycle_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    if let (Some(cache), Some(context)) = (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        record_resource_summary_value_cache_candidates(
            cache,
            context,
            types,
            module,
            &dependencies,
            &preseeded_functions,
            &summaries,
        );
    }
    let recomputations = worklist.recomputations();
    (summaries, recomputations)
}

fn update_collection_slot_lifecycle_summary(
    summaries: &mut Vec<CollectionSlotLifecycleFunctionSummary>,
    summary: CollectionSlotLifecycleFunctionSummary,
) -> bool {
    let has_facts = !summary.ops.is_empty()
        || !summary.return_transfers.is_empty()
        || !summary.return_slots.is_empty()
        || !summary.return_ranges.is_empty()
        || !summary.return_paths.is_empty();
    let position = summaries
        .iter()
        .position(|existing| existing.function == summary.function);
    match (has_facts, position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
        }
        (true, None) => {
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
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
    return_paths.retain(collection_return_path_has_lifecycle_facts);
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

fn collection_return_path_has_lifecycle_facts(path: &CollectionSlotLifecycleReturnPath) -> bool {
    !path.ops.is_empty()
        || !path.return_transfers.is_empty()
        || !path.return_slots.is_empty()
        || !path.return_ranges.is_empty()
}

#[cfg(test)]
#[path = "collection_slot_summary_build_tests.rs"]
mod tests;
