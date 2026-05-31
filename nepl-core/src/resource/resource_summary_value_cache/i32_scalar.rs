extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::super::i32_scalar_return_facts::I32ScalarReturnFacts;
use super::super::model::ResourceFunction;
use super::candidate_key::{
    i32_scalar_return_facts_entry_key, ResourceSummaryDependencyClosureHash,
    ResourceSummaryGenericTypeArgumentKeyInput,
};
use super::stable_mirror::{
    reproject_i32_scalar_return_facts_entry, reproject_i32_scalar_return_facts_entry_result,
    stable_i32_scalar_return_facts_entry,
    ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject,
    ResourceSummaryStableI32ScalarReturnFactsEntryReject, ResourceSummaryTypeReprojection,
};
use super::{
    ResourceSummaryI32ScalarReturnFactsEntryCandidate, ResourceSummaryValueCache,
    ResourceSummaryValueCacheContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum I32ScalarReturnFactsEntryCandidateReject {
    MissingSourcePolicy,
    UnstableKey,
    UnstableEntry(ResourceSummaryStableI32ScalarReturnFactsEntryReject),
    ReprojectionContext,
    ReprojectionValue(ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject),
}

impl ResourceSummaryValueCache {
    pub(in crate::resource) fn i32_scalar_return_facts_entry_candidate(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        dependency_closure_hash: ResourceSummaryDependencyClosureHash,
        facts: &I32ScalarReturnFacts,
    ) -> Result<
        ResourceSummaryI32ScalarReturnFactsEntryCandidate,
        I32ScalarReturnFactsEntryCandidateReject,
    > {
        let Some(source_capability_policy_hash) =
            context.source_capability_policy_hash_for_function(function)
        else {
            return Err(I32ScalarReturnFactsEntryCandidateReject::MissingSourcePolicy);
        };
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let Some(key) = i32_scalar_return_facts_entry_key(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            dependency_closure_hash,
            function,
            type_params,
            generic_type_args,
        ) else {
            return Err(I32ScalarReturnFactsEntryCandidateReject::UnstableKey);
        };
        let entry = stable_i32_scalar_return_facts_entry(types, function, facts)
            .map_err(I32ScalarReturnFactsEntryCandidateReject::UnstableEntry)?;
        let Some(reprojection) = ResourceSummaryTypeReprojection::new(types, function, type_params)
        else {
            return Err(I32ScalarReturnFactsEntryCandidateReject::ReprojectionContext);
        };
        if let Err(reason) = reproject_i32_scalar_return_facts_entry_result(&reprojection, &entry) {
            return Err(I32ScalarReturnFactsEntryCandidateReject::ReprojectionValue(
                reason,
            ));
        }
        Ok(ResourceSummaryI32ScalarReturnFactsEntryCandidate { key, entry })
    }

    pub(in crate::resource) fn record_i32_scalar_return_facts_bypass_count(
        &mut self,
        fact_count: usize,
    ) {
        self.stats.resource_summary_value_bypasses += fact_count;
        self.stats
            .resource_summary_value_i32_scalar_return_facts_bypasses += fact_count;
    }

    pub(in crate::resource) fn record_i32_scalar_return_facts_dependency_bypass(
        &mut self,
        fact_count: usize,
    ) {
        self.record_i32_scalar_return_facts_bypass_count(fact_count);
        self.stats
            .resource_summary_value_i32_scalar_return_facts_dependency_bypasses += fact_count;
    }

    pub(in crate::resource) fn record_i32_scalar_return_facts_candidate_bypass(
        &mut self,
        reason: I32ScalarReturnFactsEntryCandidateReject,
        fact_count: usize,
    ) {
        self.record_i32_scalar_return_facts_bypass_count(fact_count);
        match reason {
            I32ScalarReturnFactsEntryCandidateReject::MissingSourcePolicy => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_missing_source_policy_bypasses +=
                    fact_count;
            }
            I32ScalarReturnFactsEntryCandidateReject::UnstableKey => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_unstable_key_bypasses +=
                    fact_count;
            }
            I32ScalarReturnFactsEntryCandidateReject::UnstableEntry(_) => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_unstable_entry_bypasses +=
                    fact_count;
            }
            I32ScalarReturnFactsEntryCandidateReject::ReprojectionContext => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_reprojection_bypasses +=
                    fact_count;
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_reprojection_context_bypasses +=
                    fact_count;
            }
            I32ScalarReturnFactsEntryCandidateReject::ReprojectionValue(_) => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_reprojection_bypasses +=
                    fact_count;
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses +=
                    fact_count;
            }
        }
    }

    pub(in crate::resource) fn replay_i32_scalar_return_facts_entry(
        &mut self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        dependency_closure_hash: ResourceSummaryDependencyClosureHash,
    ) -> Option<I32ScalarReturnFacts> {
        let source_capability_policy_hash =
            context.source_capability_policy_hash_for_function(function)?;
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let key = i32_scalar_return_facts_entry_key(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            dependency_closure_hash,
            function,
            type_params,
            generic_type_args,
        )?;
        let entry = self.i32_scalar_return_facts_entries.get(&key)?.clone();
        let fact_count = entry.len();
        let Some(reprojection) = ResourceSummaryTypeReprojection::new(types, function, type_params)
        else {
            self.record_i32_scalar_return_facts_replay_bypass(fact_count);
            return None;
        };
        let Some(facts) = reproject_i32_scalar_return_facts_entry(&reprojection, &entry) else {
            self.record_i32_scalar_return_facts_replay_bypass(fact_count);
            return None;
        };

        self.stats.resource_summary_value_replay_hits += fact_count;
        self.stats.resource_summary_value_replayed_ops += fact_count;
        Some(facts)
    }

    pub(in crate::resource) fn record_i32_scalar_return_facts_recomputed_ops(
        &mut self,
        fact_count: usize,
    ) {
        self.stats.resource_summary_value_recomputed_ops += fact_count;
        self.stats
            .resource_summary_value_i32_scalar_return_facts_recomputed_ops += fact_count;
    }

    fn record_i32_scalar_return_facts_replay_bypass(&mut self, fact_count: usize) {
        self.stats.resource_summary_value_replay_bypasses += fact_count;
    }

    pub(in crate::resource) fn record_i32_scalar_return_facts_entry_candidates(
        &mut self,
        candidates: Vec<ResourceSummaryI32ScalarReturnFactsEntryCandidate>,
    ) {
        let candidates_with_hits = candidates
            .into_iter()
            .map(|candidate| {
                let existed_before_recording = self
                    .i32_scalar_return_facts_entries
                    .get(&candidate.key)
                    .is_some_and(|entry| entry == &candidate.entry);
                (candidate, existed_before_recording)
            })
            .collect::<Vec<_>>();

        for (candidate, existed_before_recording) in candidates_with_hits {
            let fact_count = candidate.entry.len();
            if existed_before_recording {
                self.stats.resource_summary_value_hits += fact_count;
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_hits += fact_count;
                continue;
            }

            self.stats.resource_summary_value_misses += fact_count;
            self.i32_scalar_return_facts_entries
                .insert(candidate.key, candidate.entry);
            self.stats.resource_summary_value_stores += fact_count;
            self.stats
                .resource_summary_value_i32_scalar_return_facts_stores += fact_count;
        }
    }
}
