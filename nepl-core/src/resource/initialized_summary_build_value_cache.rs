extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary_build_value_cache_eligibility::function_allows_complete_leaf_entry_replay;
use super::model::{ResourceFunction, ResourceModule};
use super::owner_summary_type_params::owner_summary_type_params;
use super::resource_summary_value_cache::{
    raw_init_dependency_closure_hash, RawInitParamFactsLeafEntryCandidateReject,
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
        if !function_allows_complete_leaf_entry_replay(function) {
            continue;
        }
        let Some(dependency_closure_hash) =
            raw_init_dependency_closure_hash(context, types, module, dependencies, function_index)
        else {
            continue;
        };
        let type_params = owner_summary_type_params(types, function);
        let Some(summary) = cache.replay_raw_init_param_facts_leaf_entry(
            context,
            types,
            function,
            &type_params,
            dependency_closure_hash,
        ) else {
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
            module,
            function,
            *function_index,
            dependencies,
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
    module: &ResourceModule,
    function: &ResourceFunction,
    function_index: usize,
    all_dependencies: &[Vec<usize>],
    was_preseeded: bool,
    summary: &RawCellInitializationFunctionSummary,
) {
    let eligible_fact_count = raw_init_param_facts_leaf_entry_fact_count(summary);
    if eligible_fact_count == 0 {
        return;
    }
    if !summary_is_complete_raw_init_param_facts_leaf_entry(summary) {
        cache.record_raw_init_param_facts_incomplete_leaf_bypass(eligible_fact_count);
        return;
    }
    if !function_allows_complete_leaf_entry_replay(function) {
        cache.record_raw_init_param_facts_dependency_bypass(eligible_fact_count);
        return;
    }
    if !was_preseeded {
        cache.record_raw_init_param_facts_recomputed_ops(eligible_fact_count);
    }
    let type_params = owner_summary_type_params(types, function);
    let Some(dependency_closure_hash) =
        raw_init_dependency_closure_hash(context, types, module, all_dependencies, function_index)
    else {
        cache.record_raw_init_param_facts_candidate_bypass(
            RawInitParamFactsLeafEntryCandidateReject::UnstableKey,
            eligible_fact_count,
        );
        return;
    };
    match cache.raw_init_param_facts_leaf_entry_candidate(
        context,
        types,
        function,
        &type_params,
        dependency_closure_hash,
        summary,
    ) {
        Ok(candidate) => candidates.push(candidate),
        Err(reason) => {
            cache.record_raw_init_param_facts_candidate_bypass(reason, eligible_fact_count);
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
