extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::i32_scalar_return_facts::I32ScalarReturnFacts;
use super::initialized_scalar_flow::I32ScalarReturnSummary;
use super::model::{ResourceFunction, ResourceModule};
use super::owner_summary_type_params::owner_summary_type_params;
use super::resource_summary_value_cache::{
    i32_scalar_dependency_closure_hash, ResourceSummaryI32ScalarReturnFactsEntryCandidate,
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};

pub(super) fn preseed_i32_scalar_return_summaries_from_value_cache(
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    relevant_functions: &[bool],
    dependencies: &[Vec<usize>],
    worklist_relevant_functions: &mut [bool],
    preseeded_functions: &mut [bool],
    summaries: &mut Vec<I32ScalarReturnSummary>,
) {
    for (function_index, function) in module.functions.iter().enumerate() {
        if !relevant_functions
            .get(function_index)
            .copied()
            .unwrap_or(false)
        {
            continue;
        }
        let Ok(dependency_closure_hash) = i32_scalar_dependency_closure_hash(
            context,
            types,
            module,
            dependencies,
            function_index,
        ) else {
            continue;
        };
        let type_params = owner_summary_type_params(types, function);
        let Some(facts) = cache.replay_i32_scalar_return_facts_entry(
            context,
            types,
            function,
            &type_params,
            dependency_closure_hash,
        ) else {
            continue;
        };
        if !facts.is_empty() {
            summaries.push(I32ScalarReturnSummary {
                function: function.name.clone(),
                parameters: function
                    .params
                    .iter()
                    .map(|param| param.place.clone())
                    .collect(),
                facts,
            });
        }
        if let Some(is_relevant) = worklist_relevant_functions.get_mut(function_index) {
            *is_relevant = false;
        }
        if let Some(is_preseeded) = preseeded_functions.get_mut(function_index) {
            *is_preseeded = true;
        }
    }
}

pub(super) fn record_i32_scalar_return_summary_value_cache_candidates(
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: &[Vec<usize>],
    relevant_functions: &[bool],
    preseeded_functions: &[bool],
    summaries: &[I32ScalarReturnSummary],
) {
    let mut candidates = Vec::new();
    let mut summary_by_function = BTreeMap::new();
    for summary in summaries {
        summary_by_function.insert(summary.function.as_str(), summary);
    }
    let empty_facts = I32ScalarReturnFacts::default();
    for (function_index, function) in module.functions.iter().enumerate() {
        if !relevant_functions
            .get(function_index)
            .copied()
            .unwrap_or(false)
        {
            continue;
        }
        if preseeded_functions
            .get(function_index)
            .copied()
            .unwrap_or(false)
        {
            continue;
        }
        let facts = summary_by_function
            .get(function.name.as_str())
            .map(|summary| &summary.facts)
            .unwrap_or(&empty_facts);
        collect_i32_scalar_return_facts_entry_candidate_from_summary(
            &mut candidates,
            cache,
            context,
            types,
            module,
            function,
            function_index,
            dependencies,
            facts,
        );
    }
    cache.record_i32_scalar_return_facts_entry_candidates(candidates);
}

fn collect_i32_scalar_return_facts_entry_candidate_from_summary(
    candidates: &mut Vec<ResourceSummaryI32ScalarReturnFactsEntryCandidate>,
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    function: &ResourceFunction,
    function_index: usize,
    all_dependencies: &[Vec<usize>],
    facts: &I32ScalarReturnFacts,
) {
    let fact_count = facts.len();
    cache.record_i32_scalar_return_facts_recomputed_ops(fact_count);
    let type_params = owner_summary_type_params(types, function);
    let dependency_closure_hash = match i32_scalar_dependency_closure_hash(
        context,
        types,
        module,
        all_dependencies,
        function_index,
    ) {
        Ok(hash) => hash,
        Err(_) => {
            cache.record_i32_scalar_return_facts_dependency_bypass(fact_count);
            return;
        }
    };
    match cache.i32_scalar_return_facts_entry_candidate(
        context,
        types,
        function,
        &type_params,
        dependency_closure_hash,
        facts,
    ) {
        Ok(candidate) => candidates.push(candidate),
        Err(reason) => {
            cache.record_i32_scalar_return_facts_candidate_bypass(reason, fact_count);
        }
    }
}

#[cfg(test)]
#[path = "initialized_scalar_flow_value_cache_tests.rs"]
mod tests;
