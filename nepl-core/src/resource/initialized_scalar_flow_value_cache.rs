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
    ResourceSummaryReplayPlan, ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
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
    mut replay_plan: Option<&mut ResourceSummaryReplayPlan>,
) {
    if !cache.has_i32_scalar_return_replay_entries(context) {
        return;
    }
    for (function_index, function) in module.functions.iter().enumerate() {
        if !relevant_functions
            .get(function_index)
            .copied()
            .unwrap_or(false)
        {
            continue;
        }
        let type_params = owner_summary_type_params(types, function);
        if let Some(plan) = replay_plan.as_deref_mut() {
            if let Some(facts) = cache.replay_i32_scalar_return_facts_entry_from_plan(
                plan,
                types,
                function_index,
                function,
                &type_params,
            ) {
                preseed_i32_scalar_summary(
                    function,
                    function_index,
                    facts,
                    worklist_relevant_functions,
                    preseeded_functions,
                    summaries,
                );
                continue;
            }
        }
        cache.record_i32_scalar_replay_probe_function();
        let Ok(dependency_closure_hash) = i32_scalar_dependency_closure_hash(
            context,
            types,
            module,
            dependencies,
            function_index,
        ) else {
            cache.record_i32_scalar_replay_miss_function();
            continue;
        };
        let Some(facts) = cache.replay_i32_scalar_return_facts_entry_and_record_plan(
            replay_plan.as_deref_mut(),
            function_index,
            context,
            types,
            function,
            &type_params,
            dependency_closure_hash,
        ) else {
            cache.record_i32_scalar_replay_miss_function();
            continue;
        };
        cache.record_i32_scalar_replay_hit_function();
        preseed_i32_scalar_summary(
            function,
            function_index,
            facts,
            worklist_relevant_functions,
            preseeded_functions,
            summaries,
        );
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
    mut replay_plan: Option<&mut ResourceSummaryReplayPlan>,
) {
    if !cache.stable_entry_collection_enabled() {
        return;
    }
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
            replay_plan.as_deref_mut(),
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
    replay_plan: Option<&mut ResourceSummaryReplayPlan>,
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
        Ok(candidate) => {
            if let Some(plan) = replay_plan {
                cache.record_i32_scalar_summary_candidate_key(plan, function_index, &candidate);
            }
            candidates.push(candidate);
        }
        Err(reason) => {
            #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
            log_i32_scalar_return_facts_candidate_bypass(function, &reason, facts);
            cache.record_i32_scalar_return_facts_candidate_bypass_for_facts(reason, facts);
        }
    }
}

fn preseed_i32_scalar_summary(
    function: &ResourceFunction,
    function_index: usize,
    facts: I32ScalarReturnFacts,
    worklist_relevant_functions: &mut [bool],
    preseeded_functions: &mut [bool],
    summaries: &mut Vec<I32ScalarReturnSummary>,
) {
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

/// i32 scalar return facts を stable cache entry にできなかった関数と fact surface を出力する。
///
/// このログは residual recomputation の原因を狭めるための host-only 診断であり、通常の
/// compiler statistics には含めない。通常の JSON 統計に fact 本体を入れると Web playground の
/// measurement path を重くするため、明示的な環境変数を指定した native 実行だけで出す。
#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
fn log_i32_scalar_return_facts_candidate_bypass(
    function: &ResourceFunction,
    reason: &impl core::fmt::Debug,
    facts: &I32ScalarReturnFacts,
) {
    if std::env::var_os("NEPL_RESOURCE_I32_STABLE_REPROJECTION_DEBUG").is_none() {
        return;
    }
    if let Some(filter) = std::env::var("NEPL_RESOURCE_OP_TIMING_FUNCTION")
        .ok()
        .filter(|filter| !function.name.contains(filter.as_str()))
    {
        let _ = filter;
        return;
    }
    let counts = facts.fact_counts();
    std::eprintln!(
        "[resource-i32-candidate-bypass] function={} reason={:?} total={} aliases={} offsets={} relations={} constants={} return_conditions={} parameter_conditions={}",
        function.name,
        reason,
        counts.total(),
        counts.aliases,
        counts.offsets,
        counts.relations,
        counts.constants,
        counts.return_conditions,
        counts.parameter_conditions
    );
    std::eprintln!(
        "[resource-i32-candidate-bypass-facts] function={} facts={:?}",
        function.name,
        facts
    );
}

#[cfg(test)]
#[path = "initialized_scalar_flow_value_cache_tests.rs"]
mod tests;
