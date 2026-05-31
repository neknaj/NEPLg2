extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::model::{OwnerStateEntry, ResourceModule};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_check_utils::merge_owner_deferred;
use super::owner_obligation_value_cache::{
    owner_obligation_check_cache_input, record_owner_obligation_check_value_cache_candidate,
    replay_owner_obligation_check_from_value_cache,
};
use super::report::{
    ResourceOwnerCheckDeferred, ResourceOwnerCheckReport, ResourceOwnerFunctionCheck,
};
use super::resource_summary_value_cache::{
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};
use super::summary::compute_owner_return_summaries;
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
    let mut functions = Vec::new();
    let mut diagnostics = Vec::new();
    let mut deferred = ResourceOwnerCheckDeferred::default();
    let dependency_graph = summary_value_cache
        .as_ref()
        .map(|_| ResourceSummaryDependencyGraph::build(module));
    let summaries = compute_owner_return_summaries(module, types);
    let summary_index = OwnerReturnSummaryIndex::new(&summaries);
    stage_start.log("resource_owner_summaries");
    let stage_start = ResourceStageTimer::start();

    for (function_index, function) in module.functions.iter().enumerate() {
        let function_op_count = resource_function_op_count(function);
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
            functions.push(replayed_check);
            continue;
        }
        if let Some(cache) = summary_value_cache.as_deref_mut() {
            cache.record_owner_obligation_check_replay_miss_function();
        }
        if let Some(cache) = summary_value_cache.as_deref_mut() {
            cache.record_owner_obligation_function_check(function_op_count);
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
            function_check_cache_input.as_ref(),
            &function_check,
            function_has_diagnostics,
            function_op_count,
        );
        functions.push(function_check);
    }
    stage_start.log("resource_owner_function_checks");

    ResourceOwnerCheckReport {
        functions,
        diagnostics,
        deferred,
    }
}

fn resource_function_op_count(function: &super::model::ResourceFunction) -> usize {
    function.blocks.iter().map(|block| block.ops.len()).sum()
}

#[cfg(test)]
#[path = "owner_obligation_value_cache_tests.rs"]
mod owner_obligation_value_cache_tests;
