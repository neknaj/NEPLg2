extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::initialized_alias_flow::RawCellAddressReturnSummary;
use super::model::{ResourceFunction, ResourceModule};
use super::owner_summary_type_params::owner_summary_type_params;
use super::resource_summary_value_cache::{
    raw_alias_dependency_closure_hash, ResourceSummaryRawAliasReturnEntryCandidate,
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};

pub(super) fn preseed_raw_alias_return_summaries_from_value_cache(
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: &[Vec<usize>],
    initially_skipped_functions: &mut [bool],
    preseeded_functions: &mut [bool],
    summaries: &mut Vec<RawCellAddressReturnSummary>,
) {
    for (function_index, function) in module.functions.iter().enumerate() {
        let Ok(dependency_closure_hash) =
            raw_alias_dependency_closure_hash(context, types, module, dependencies, function_index)
        else {
            continue;
        };
        let type_params = owner_summary_type_params(types, function);
        let Some(summary) = cache.replay_raw_alias_return_entry(
            context,
            types,
            function,
            &type_params,
            dependency_closure_hash,
        ) else {
            continue;
        };
        if !summary.aliases.is_empty() {
            summaries.push(summary);
        }
        if let Some(is_skipped) = initially_skipped_functions.get_mut(function_index) {
            *is_skipped = true;
        }
        if let Some(is_preseeded) = preseeded_functions.get_mut(function_index) {
            *is_preseeded = true;
        }
    }
}

pub(super) fn record_raw_alias_return_summary_value_cache_candidates(
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    dependencies: &[Vec<usize>],
    preseeded_functions: &[bool],
    summaries: &[RawCellAddressReturnSummary],
) {
    let mut candidates = Vec::new();
    let mut summary_by_function = BTreeMap::new();
    for summary in summaries {
        summary_by_function.insert(summary.function.as_str(), summary);
    }
    for (function_index, function) in module.functions.iter().enumerate() {
        let empty_summary;
        let summary = match summary_by_function.get(function.name.as_str()) {
            Some(summary) => *summary,
            None => {
                empty_summary = empty_raw_alias_return_summary(function);
                &empty_summary
            }
        };
        collect_raw_alias_return_entry_candidate_from_summary(
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
            summary,
        );
    }
    cache.record_raw_alias_return_entry_candidates(candidates);
}

fn collect_raw_alias_return_entry_candidate_from_summary(
    candidates: &mut Vec<ResourceSummaryRawAliasReturnEntryCandidate>,
    cache: &mut ResourceSummaryValueCache,
    context: &ResourceSummaryValueCacheContext,
    types: &TypeCtx,
    module: &ResourceModule,
    function: &ResourceFunction,
    function_index: usize,
    all_dependencies: &[Vec<usize>],
    was_preseeded: bool,
    summary: &RawCellAddressReturnSummary,
) {
    let alias_count = summary.aliases.len();
    if !was_preseeded {
        cache.record_raw_alias_return_entry_recomputed_ops(alias_count);
    }
    let type_params = owner_summary_type_params(types, function);
    let dependency_closure_hash = match raw_alias_dependency_closure_hash(
        context,
        types,
        module,
        all_dependencies,
        function_index,
    ) {
        Ok(hash) => hash,
        Err(_) => {
            cache.record_raw_alias_return_entry_dependency_bypass(alias_count);
            return;
        }
    };
    match cache.raw_alias_return_entry_candidate(
        context,
        types,
        function,
        &type_params,
        dependency_closure_hash,
        summary,
    ) {
        Ok(candidate) => candidates.push(candidate),
        Err(reason) => {
            cache.record_raw_alias_return_entry_candidate_bypass(reason, alias_count);
        }
    }
}

fn empty_raw_alias_return_summary(function: &ResourceFunction) -> RawCellAddressReturnSummary {
    RawCellAddressReturnSummary {
        function: function.name.clone(),
        parameters: function
            .params
            .iter()
            .map(|param| param.place.clone())
            .collect(),
        aliases: Vec::new(),
    }
}
