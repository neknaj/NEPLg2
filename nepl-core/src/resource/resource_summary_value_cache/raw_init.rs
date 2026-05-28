use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::super::initialized_summary::RawCellInitializationFunctionSummary;
use super::super::model::ResourceFunction;
use super::candidate_key::{
    raw_init_param_facts_leaf_entry_candidate_key_and_entry, raw_init_param_facts_leaf_entry_key,
    ResourceSummaryGenericTypeArgumentKeyInput,
};
use super::stable_mirror::{
    reproject_raw_init_param_facts_leaf_entry, ResourceSummaryTypeReprojection,
};
use super::{
    ResourceSummaryRawInitParamFactsLeafEntryCandidate, ResourceSummaryValueCache,
    ResourceSummaryValueCacheContext,
};

impl ResourceSummaryValueCache {
    pub(in crate::resource) fn raw_init_param_facts_leaf_entry_candidate(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        summary: &RawCellInitializationFunctionSummary,
    ) -> Option<ResourceSummaryRawInitParamFactsLeafEntryCandidate> {
        let Some(source_capability_policy_hash) =
            context.source_capability_policy_hash_for_function(function)
        else {
            return None;
        };
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let (key, entry) = raw_init_param_facts_leaf_entry_candidate_key_and_entry(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            function,
            type_params,
            generic_type_args,
            summary,
        )?;
        let reprojection = ResourceSummaryTypeReprojection::new(types, function, type_params)?;
        reproject_raw_init_param_facts_leaf_entry(&reprojection, &function.name, &entry)?;
        Some(ResourceSummaryRawInitParamFactsLeafEntryCandidate { key, entry })
    }

    pub(in crate::resource) fn record_raw_init_param_facts_bypass(&mut self) {
        self.stats.resource_summary_value_bypasses += 1;
        self.stats
            .resource_summary_value_raw_init_param_facts_bypasses += 1;
    }

    pub(in crate::resource) fn replay_raw_init_param_facts_leaf_entry(
        &mut self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
    ) -> Option<RawCellInitializationFunctionSummary> {
        let source_capability_policy_hash =
            context.source_capability_policy_hash_for_function(function)?;
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let key = raw_init_param_facts_leaf_entry_key(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            function,
            type_params,
            generic_type_args,
        )?;
        let entry = self.raw_init_param_facts_leaf_entries.get(&key)?.clone();
        let fact_count = entry.len();
        let Some(reprojection) = ResourceSummaryTypeReprojection::new(types, function, type_params)
        else {
            self.record_raw_init_param_facts_replay_bypass(fact_count);
            return None;
        };
        let Some(summary) =
            reproject_raw_init_param_facts_leaf_entry(&reprojection, &function.name, &entry)
        else {
            self.record_raw_init_param_facts_replay_bypass(fact_count);
            return None;
        };

        self.stats.resource_summary_value_replay_hits += fact_count;
        self.stats.resource_summary_value_replayed_ops += fact_count;
        Some(summary)
    }

    pub(in crate::resource) fn record_raw_init_param_facts_recomputed_ops(
        &mut self,
        fact_count: usize,
    ) {
        self.stats.resource_summary_value_recomputed_ops += fact_count;
    }

    fn record_raw_init_param_facts_replay_bypass(&mut self, fact_count: usize) {
        self.stats.resource_summary_value_replay_bypasses += fact_count;
    }

    pub(in crate::resource) fn record_raw_init_param_facts_leaf_entry_candidates(
        &mut self,
        candidates: Vec<ResourceSummaryRawInitParamFactsLeafEntryCandidate>,
    ) {
        let candidates_with_hits = candidates
            .into_iter()
            .map(|candidate| {
                let existed_before_recording = self
                    .raw_init_param_facts_leaf_entries
                    .get(&candidate.key)
                    .is_some_and(|entry| entry == &candidate.entry);
                (candidate, existed_before_recording)
            })
            .collect::<Vec<_>>();

        for (candidate, existed_before_recording) in candidates_with_hits {
            let fact_count = candidate.entry.len();
            if existed_before_recording {
                self.stats.resource_summary_value_hits += fact_count;
                self.stats.resource_summary_value_raw_init_param_facts_hits += fact_count;
                continue;
            }

            self.stats.resource_summary_value_misses += fact_count;
            self.raw_init_param_facts_leaf_entries
                .insert(candidate.key, candidate.entry);
            self.stats.resource_summary_value_stores += fact_count;
            self.stats
                .resource_summary_value_raw_init_param_facts_stores += fact_count;
        }
    }
}
