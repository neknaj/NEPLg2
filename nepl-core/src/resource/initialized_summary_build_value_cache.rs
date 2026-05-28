extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary_build_value_cache_eligibility::function_allows_complete_leaf_entry_replay;
use super::model::{ResourceFunction, ResourceModule};
use super::owner_summary_type_params::owner_summary_type_params;
use super::resource_summary_value_cache::{
    ResourceSummaryRawInitParamFactsLeafEntryCandidate, ResourceSummaryValueCache,
    ResourceSummaryValueCacheContext,
};

pub(super) fn preseed_raw_cell_initialization_summaries_from_value_cache(
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    relevant_functions: &[bool],
    dependencies: &[Vec<usize>],
    worklist_relevant_functions: &mut [bool],
    preseeded_functions: &mut [bool],
    summaries: &mut Vec<RawCellInitializationFunctionSummary>,
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
        let type_params = owner_summary_type_params(types, function);
        let Some(summary) =
            cache.replay_raw_init_param_facts_leaf_entry(context, types, function, &type_params)
        else {
            continue;
        };
        summaries.push(summary);
        if let Some(is_relevant) = worklist_relevant_functions.get_mut(function_index) {
            *is_relevant = false;
        }
        if let Some(is_preseeded) = preseeded_functions.get_mut(function_index) {
            *is_preseeded = true;
        }
    }
}

pub(super) fn record_raw_cell_initialization_summary_value_cache_candidates(
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: &[Vec<usize>],
    preseeded_functions: &[bool],
    summaries: &[RawCellInitializationFunctionSummary],
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
        collect_raw_init_param_facts_leaf_entry_candidate_from_summary(
            &mut candidates,
            cache,
            context,
            types,
            function,
            dependencies
                .get(*function_index)
                .map(Vec::as_slice)
                .unwrap_or(&[]),
            preseeded_functions
                .get(*function_index)
                .copied()
                .unwrap_or(false),
            summary,
        );
    }
    cache.record_raw_init_param_facts_leaf_entry_candidates(candidates);
}

fn collect_raw_init_param_facts_leaf_entry_candidate_from_summary(
    candidates: &mut Vec<ResourceSummaryRawInitParamFactsLeafEntryCandidate>,
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    function: &ResourceFunction,
    dependencies: &[usize],
    was_preseeded: bool,
    summary: &RawCellInitializationFunctionSummary,
) {
    let eligible_fact_count = raw_init_param_facts_leaf_entry_fact_count(summary);
    if eligible_fact_count == 0 {
        return;
    }
    if !summary_is_complete_raw_init_param_facts_leaf_entry(summary)
        || !function_allows_complete_leaf_entry_replay(function, dependencies)
    {
        for _ in 0..eligible_fact_count {
            cache.record_raw_init_param_facts_bypass();
        }
        return;
    }
    if !was_preseeded {
        cache.record_raw_init_param_facts_recomputed_ops(eligible_fact_count);
    }
    let type_params = owner_summary_type_params(types, function);
    match cache.raw_init_param_facts_leaf_entry_candidate(
        context,
        types,
        function,
        &type_params,
        summary,
    ) {
        Some(candidate) => candidates.push(candidate),
        None => {
            for _ in 0..eligible_fact_count {
                cache.record_raw_init_param_facts_bypass();
            }
        }
    }
}

fn raw_init_param_facts_leaf_entry_fact_count(
    summary: &RawCellInitializationFunctionSummary,
) -> usize {
    summary.param_cells.len() + summary.param_release_requirements.len()
}

fn summary_is_complete_raw_init_param_facts_leaf_entry(
    summary: &RawCellInitializationFunctionSummary,
) -> bool {
    raw_init_param_facts_leaf_entry_fact_count(summary) > 0
        && summary.return_cells.is_empty()
        && summary.return_byte_ranges.is_empty()
        && summary.param_byte_ranges.is_empty()
        && summary.variant_param_cells.is_empty()
        && summary.variant_param_byte_ranges.is_empty()
        && summary.variant_required_param_cells.is_empty()
        && summary.variant_conditions.is_empty()
}

#[cfg(test)]
#[path = "initialized_summary_build_value_cache_tests.rs"]
mod tests;
