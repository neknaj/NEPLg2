extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary_build_value_cache_eligibility::function_allows_complete_leaf_entry_replay;
use super::model::{ResourceFunction, ResourceModule};
use super::owner_summary_type_params::owner_summary_type_params;
use super::resource_summary_value_cache::{
    raw_init_dependency_closure_hash, ResourceSummaryRawInitCompleteLeafEntryCandidate,
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
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
        let Ok(dependency_closure_hash) =
            raw_init_dependency_closure_hash(context, types, module, dependencies, function_index)
        else {
            continue;
        };
        let type_params = owner_summary_type_params(types, function);
        let Some(summary) = cache.replay_raw_init_complete_leaf_entry(
            context,
            types,
            function,
            &type_params,
            dependency_closure_hash,
        ) else {
            continue;
        };
        if raw_init_complete_leaf_entry_fact_count(&summary) > 0 {
            summaries.push(summary);
        }
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
    relevant_functions: &[bool],
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
        collect_raw_init_complete_leaf_entry_candidate_from_summary(
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
    for (function_index, function) in module.functions.iter().enumerate() {
        if !relevant_functions
            .get(function_index)
            .copied()
            .unwrap_or(false)
        {
            continue;
        }
        if summaries
            .iter()
            .any(|summary| summary.function == function.name)
        {
            continue;
        }
        let summary = empty_raw_init_complete_leaf_summary(types, function);
        collect_raw_init_complete_leaf_entry_candidate_from_summary(
            &mut candidates,
            cache,
            context,
            types,
            module,
            function,
            function_index,
            dependencies,
            preseeded_functions
                .get(function_index)
                .copied()
                .unwrap_or(false),
            &summary,
        );
    }
    cache.record_raw_init_complete_leaf_entry_candidates(candidates);
}

fn collect_raw_init_complete_leaf_entry_candidate_from_summary(
    candidates: &mut Vec<ResourceSummaryRawInitCompleteLeafEntryCandidate>,
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
    let eligible_fact_count = raw_init_complete_leaf_entry_fact_count(summary);
    // complete leaf entry は現時点で `RawCellInitializationFunctionSummary` の全 surface を
    // mirror する。fact が空の relevant function も empty entry として保存することで、
    // 微小編集時に no-fact function が worklist へ戻る固定費を避ける。新しい surface が
    // 追加されたときは、この fact count と stable mirror の両方を更新し、古い incomplete
    // counter を再び増やすのではなく fail-closed な complete entry として扱えるようにする。
    if !function_allows_complete_leaf_entry_replay(function) {
        cache.record_raw_init_param_facts_dependency_bypass(eligible_fact_count);
        return;
    }
    if !was_preseeded {
        cache.record_raw_init_param_facts_recomputed_ops(eligible_fact_count);
    }
    let type_params = owner_summary_type_params(types, function);
    let dependency_closure_hash = match raw_init_dependency_closure_hash(
        context,
        types,
        module,
        all_dependencies,
        function_index,
    ) {
        Ok(hash) => hash,
        Err(reason) => {
            cache.record_raw_init_dependency_closure_bypass(reason, eligible_fact_count);
            return;
        }
    };
    match cache.raw_init_complete_leaf_entry_candidate(
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

fn empty_raw_init_complete_leaf_summary(
    types: &TypeCtx,
    function: &ResourceFunction,
) -> RawCellInitializationFunctionSummary {
    RawCellInitializationFunctionSummary {
        function: function.name.clone(),
        type_params: owner_summary_type_params(types, function),
        return_cells: Vec::new(),
        return_byte_ranges: Vec::new(),
        param_cells: Vec::new(),
        param_byte_ranges: Vec::new(),
        param_release_requirements: Vec::new(),
        variant_param_cells: Vec::new(),
        variant_param_byte_ranges: Vec::new(),
        variant_required_param_cells: Vec::new(),
        variant_conditions: Vec::new(),
    }
}

fn raw_init_complete_leaf_entry_fact_count(
    summary: &RawCellInitializationFunctionSummary,
) -> usize {
    summary.return_cells.len()
        + summary.return_byte_ranges.len()
        + summary.param_cells.len()
        + summary.param_byte_ranges.len()
        + summary.param_release_requirements.len()
        + summary.variant_param_cells.len()
        + summary.variant_param_byte_ranges.len()
        + summary.variant_required_param_cells.len()
        + summary.variant_conditions.len()
}

#[cfg(test)]
#[path = "initialized_summary_build_value_cache_tests.rs"]
mod tests;
