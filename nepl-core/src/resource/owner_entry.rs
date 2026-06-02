use alloc::vec;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::model::{OwnerStateEntry, ResourceModule};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_check_utils::merge_owner_deferred;
use super::owner_obligation_value_cache::{
    owner_obligation_check_cache_input, record_owner_obligation_check_value_cache_candidate,
    replay_owner_obligation_check_from_value_cache, OwnerObligationCheckCacheInput,
};
use super::report::{
    ResourceOwnerCheckDeferred, ResourceOwnerCheckReport, ResourceOwnerFunctionCheck,
};
use super::resource_summary_value_cache::{
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};
use super::summary::compute_owner_return_summaries_with_recomputations;
use super::summary::OwnerReturnSummaryIndex;
use super::summary_dependency::ResourceSummaryDependencyGraph;
use super::timing::ResourceStageTimer;

pub fn check_resource_owner_obligations(
    module: &ResourceModule,
    types: &TypeCtx,
) -> ResourceOwnerCheckReport {
    check_resource_owner_obligations_inner(module, types, None, None)
}

pub fn check_resource_owner_obligations_with_summary_cache(
    module: &ResourceModule,
    types: &TypeCtx,
    summary_value_cache: &mut ResourceSummaryValueCache,
    summary_value_cache_context: &ResourceSummaryValueCacheContext,
) -> ResourceOwnerCheckReport {
    check_resource_owner_obligations_inner(
        module,
        types,
        Some(summary_value_cache),
        Some(summary_value_cache_context),
    )
}

fn check_resource_owner_obligations_inner(
    module: &ResourceModule,
    types: &TypeCtx,
    mut summary_value_cache: Option<&mut ResourceSummaryValueCache>,
    summary_value_cache_context: Option<&ResourceSummaryValueCacheContext>,
) -> ResourceOwnerCheckReport {
    let stage_start = ResourceStageTimer::start();
    let mut function_results = vec![None; module.functions.len()];
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceOwnerCheckDeferred::default();
    let dependency_graph = summary_value_cache
        .as_ref()
        .map(|_| ResourceSummaryDependencyGraph::build(module));
    let mut owner_check_pass_plan = match (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
        dependency_graph.as_ref(),
    ) {
        (Some(cache), Some(context), Some(graph)) => {
            Some(cache.begin_owner_obligation_check_pass_plan(context, types, module, graph))
        }
        _ => None,
    };
    let mut pending_checks = Vec::new();
    if summary_value_cache.is_some() && summary_value_cache_context.is_some() {
        for (function_index, function) in module.functions.iter().enumerate() {
            let function_op_count = resource_function_op_count(function);
            if let (Some(cache), Some(plan)) = (
                summary_value_cache.as_deref_mut(),
                owner_check_pass_plan.as_mut(),
            ) {
                if let Some(replayed_check) = cache.replay_unchanged_owner_obligation_check_pass(
                    plan,
                    function_index,
                    function,
                    function_op_count,
                ) {
                    merge_owner_deferred(&mut deferred, replayed_check.deferred);
                    function_results[function_index] = Some(replayed_check);
                    continue;
                }
            }
            let function_check_cache_input = owner_obligation_check_cache_input(
                summary_value_cache.as_deref_mut(),
                summary_value_cache_context,
                types,
                module,
                dependency_graph
                    .as_ref()
                    .map(ResourceSummaryDependencyGraph::dependencies),
                function_index,
                function,
                function_op_count,
            );
            if let Some(cache) = summary_value_cache.as_deref_mut() {
                cache.record_owner_obligation_check_replay_probe_function();
            }
            if let Some(replayed_check) = replay_owner_obligation_check_from_value_cache(
                summary_value_cache.as_deref_mut(),
                summary_value_cache_context,
                types,
                function,
                function_check_cache_input.as_ref(),
                function_op_count,
            ) {
                merge_owner_deferred(&mut deferred, replayed_check.deferred);
                if let Some(plan) = owner_check_pass_plan.as_mut() {
                    plan.record_pass(function_index, replayed_check.deferred);
                }
                function_results[function_index] = Some(replayed_check);
                continue;
            }
            if let Some(cache) = summary_value_cache.as_deref_mut() {
                cache.record_owner_obligation_check_replay_miss_function();
            }
            pending_checks.push(OwnerObligationPendingCheck {
                function_index,
                function_op_count,
                cache_input: function_check_cache_input,
            });
        }
        if pending_checks.is_empty() {
            if let (Some(cache), Some(plan)) = (
                summary_value_cache.as_deref_mut(),
                owner_check_pass_plan.take(),
            ) {
                cache.finish_owner_obligation_check_pass_plan(plan);
            }
            if let Some(cache) = summary_value_cache.as_deref_mut() {
                cache.record_owner_return_summary_pass_cache_skip(module.functions.len());
            }
            stage_start.log("resource_owner_summaries_skipped_by_pass_cache");
            return ResourceOwnerCheckReport {
                functions: owner_obligation_function_results(function_results),
                diagnostics,
                deferred,
            };
        }
    } else {
        pending_checks = module
            .functions
            .iter()
            .enumerate()
            .map(|(function_index, function)| OwnerObligationPendingCheck {
                function_index,
                function_op_count: resource_function_op_count(function),
                cache_input: None,
            })
            .collect();
    }

    let (summaries, owner_summary_recomputations) =
        compute_owner_return_summaries_with_recomputations(module, types);
    if let Some(cache) = summary_value_cache.as_deref_mut() {
        cache.record_owner_return_summary_stage(owner_summary_recomputations, summaries.len());
    }
    let summary_index = OwnerReturnSummaryIndex::new(&summaries);
    stage_start.log("resource_owner_summaries");
    let stage_start = ResourceStageTimer::start();

    for pending in pending_checks {
        let function = &module.functions[pending.function_index];
        if let Some(cache) = summary_value_cache.as_deref_mut() {
            cache.record_owner_obligation_function_check(pending.function_op_count);
        }
        let mut engine = ResourceOwnerCheckEngine {
            function: function.name.as_str(),
            types,
            summaries: &summary_index,
            diagnostics: Vec::new(),
            deferred: ResourceOwnerCheckDeferred::default(),
            owner_extent_requirements: Vec::new(),
            memory_span_requirements: Vec::new(),
            params: &function.params,
        };
        let final_owners: Vec<OwnerStateEntry> = engine.check_function(function);
        let function_deferred = engine.deferred;
        let function_has_diagnostics = !engine.diagnostics.is_empty();
        merge_owner_deferred(&mut deferred, function_deferred);
        diagnostics.extend(engine.diagnostics);
        let function_check = ResourceOwnerFunctionCheck {
            name: function.name.clone(),
            final_owners,
            deferred: function_deferred,
        };
        record_owner_obligation_check_value_cache_candidate(
            summary_value_cache.as_deref_mut(),
            summary_value_cache_context,
            types,
            function,
            pending.cache_input.as_ref(),
            &function_check,
            function_has_diagnostics,
            pending.function_op_count,
        );
        if !function_has_diagnostics {
            if let Some(plan) = owner_check_pass_plan.as_mut() {
                plan.record_pass(pending.function_index, function_deferred);
            }
        }
        function_results[pending.function_index] = Some(function_check);
    }
    if let (Some(cache), Some(plan)) = (
        summary_value_cache.as_deref_mut(),
        owner_check_pass_plan.take(),
    ) {
        cache.finish_owner_obligation_check_pass_plan(plan);
    }
    stage_start.log("resource_owner_function_checks");

    ResourceOwnerCheckReport {
        functions: owner_obligation_function_results(function_results),
        diagnostics,
        deferred,
    }
}

struct OwnerObligationPendingCheck {
    function_index: usize,
    function_op_count: usize,
    cache_input: Option<OwnerObligationCheckCacheInput>,
}

/// replay できた関数と再検査した関数を、`ResourceModule` の関数順へ戻す。
///
/// owner obligation report は後続 gate の診断面だけでなく debug / stats 出力でも関数順に
/// 読まれるため、pass-cache hit と miss check を分けて実行しても順序は変えない。
fn owner_obligation_function_results(
    function_results: Vec<Option<ResourceOwnerFunctionCheck>>,
) -> Vec<ResourceOwnerFunctionCheck> {
    function_results
        .into_iter()
        .map(|function| {
            function.expect("owner obligation replay/check should fill every function result")
        })
        .collect()
}

fn resource_function_op_count(function: &super::model::ResourceFunction) -> usize {
    function.blocks.iter().map(|block| block.ops.len()).sum()
}

#[cfg(test)]
#[path = "owner_obligation_value_cache_tests.rs"]
mod owner_obligation_value_cache_tests;
