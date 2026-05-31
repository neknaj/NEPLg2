extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::super::initialized_alias_flow::RawCellAddressReturnSummary;
use super::super::model::ResourceFunction;
use super::candidate_key::{
    raw_alias_return_entry_key, ResourceSummaryDependencyClosureHash,
    ResourceSummaryGenericTypeArgumentKeyInput,
};
use super::stable_mirror::{
    reproject_raw_alias_return_entry, reproject_raw_alias_return_entry_result,
    stable_raw_alias_return_entry, ResourceSummaryRawAliasReturnEntryReprojectionReject,
    ResourceSummaryStableRawAliasReturnEntryReject, ResourceSummaryTypeReprojection,
};
use super::{
    ResourceSummaryRawAliasReturnEntryCandidate, ResourceSummaryValueCache,
    ResourceSummaryValueCacheContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum RawAliasReturnEntryCandidateReject {
    MissingSourcePolicy,
    UnstableKey,
    UnstableEntry(ResourceSummaryStableRawAliasReturnEntryReject),
    ReprojectionContext,
    ReprojectionValue(ResourceSummaryRawAliasReturnEntryReprojectionReject),
}

impl ResourceSummaryValueCache {
    pub(in crate::resource) fn raw_alias_return_entry_candidate(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        dependency_closure_hash: ResourceSummaryDependencyClosureHash,
        summary: &RawCellAddressReturnSummary,
    ) -> Result<ResourceSummaryRawAliasReturnEntryCandidate, RawAliasReturnEntryCandidateReject>
    {
        let Some(source_capability_policy_hash) =
            context.source_capability_policy_hash_for_function(function)
        else {
            return Err(RawAliasReturnEntryCandidateReject::MissingSourcePolicy);
        };
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let Some(key) = raw_alias_return_entry_key(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            dependency_closure_hash,
            function,
            type_params,
            generic_type_args,
        ) else {
            return Err(RawAliasReturnEntryCandidateReject::UnstableKey);
        };
        let entry = stable_raw_alias_return_entry(types, function, summary)
            .map_err(RawAliasReturnEntryCandidateReject::UnstableEntry)?;
        let Some(reprojection) = ResourceSummaryTypeReprojection::new(types, function, type_params)
        else {
            return Err(RawAliasReturnEntryCandidateReject::ReprojectionContext);
        };
        if let Err(reason) = reproject_raw_alias_return_entry_result(&reprojection, &entry) {
            return Err(RawAliasReturnEntryCandidateReject::ReprojectionValue(
                reason,
            ));
        }
        Ok(ResourceSummaryRawAliasReturnEntryCandidate { key, entry })
    }

    pub(in crate::resource) fn replay_raw_alias_return_entry(
        &mut self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        dependency_closure_hash: ResourceSummaryDependencyClosureHash,
    ) -> Option<RawCellAddressReturnSummary> {
        let source_capability_policy_hash =
            context.source_capability_policy_hash_for_function(function)?;
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let key = raw_alias_return_entry_key(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            dependency_closure_hash,
            function,
            type_params,
            generic_type_args,
        )?;
        let entry = self.raw_alias_return_entries.get(&key)?.clone();
        let alias_count = entry.len();
        let Some(reprojection) = ResourceSummaryTypeReprojection::new(types, function, type_params)
        else {
            self.record_raw_alias_return_entry_replay_bypass(alias_count);
            return None;
        };
        let Some(summary) = reproject_raw_alias_return_entry(&reprojection, &entry) else {
            self.record_raw_alias_return_entry_replay_bypass(alias_count);
            return None;
        };

        self.stats.resource_summary_value_replay_hits += alias_count;
        self.stats.resource_summary_value_replayed_ops += alias_count;
        Some(summary)
    }

    pub(in crate::resource) fn record_raw_alias_return_entry_recomputed_ops(
        &mut self,
        alias_count: usize,
    ) {
        self.stats.resource_summary_value_recomputed_ops += alias_count;
        self.stats
            .resource_summary_value_raw_alias_return_entry_recomputed_ops += alias_count;
    }

    pub(in crate::resource) fn record_raw_alias_return_entry_dependency_bypass(
        &mut self,
        alias_count: usize,
    ) {
        self.record_raw_alias_return_entry_bypass_count(alias_count);
        self.stats
            .resource_summary_value_raw_alias_return_entry_dependency_bypasses += alias_count;
    }

    pub(in crate::resource) fn record_raw_alias_return_entry_candidate_bypass(
        &mut self,
        reason: RawAliasReturnEntryCandidateReject,
        alias_count: usize,
    ) {
        self.record_raw_alias_return_entry_bypass_count(alias_count);
        match reason {
            RawAliasReturnEntryCandidateReject::MissingSourcePolicy => {
                self.stats
                    .resource_summary_value_raw_alias_return_entry_missing_source_policy_bypasses +=
                    alias_count;
            }
            RawAliasReturnEntryCandidateReject::UnstableKey => {
                self.stats
                    .resource_summary_value_raw_alias_return_entry_unstable_key_bypasses +=
                    alias_count;
            }
            RawAliasReturnEntryCandidateReject::UnstableEntry(_) => {
                self.stats
                    .resource_summary_value_raw_alias_return_entry_unstable_entry_bypasses +=
                    alias_count;
            }
            RawAliasReturnEntryCandidateReject::ReprojectionContext => {
                self.stats
                    .resource_summary_value_raw_alias_return_entry_reprojection_bypasses +=
                    alias_count;
                self.stats
                    .resource_summary_value_raw_alias_return_entry_reprojection_context_bypasses +=
                    alias_count;
            }
            RawAliasReturnEntryCandidateReject::ReprojectionValue(reason) => {
                self.stats
                    .resource_summary_value_raw_alias_return_entry_reprojection_bypasses +=
                    alias_count;
                self.stats
                    .resource_summary_value_raw_alias_return_entry_reprojection_value_bypasses +=
                    alias_count;
                self.record_raw_alias_reprojection_value_bypass(reason, alias_count);
            }
        }
    }

    fn record_raw_alias_reprojection_value_bypass(
        &mut self,
        reason: ResourceSummaryRawAliasReturnEntryReprojectionReject,
        alias_count: usize,
    ) {
        match reason {
            ResourceSummaryRawAliasReturnEntryReprojectionReject::ParameterIndex => {
                self.stats
                    .resource_summary_value_raw_alias_return_entry_reprojection_value_parameter_index_bypasses +=
                    alias_count;
            }
            ResourceSummaryRawAliasReturnEntryReprojectionReject::ParameterProjection => {
                self.stats
                    .resource_summary_value_raw_alias_return_entry_reprojection_value_parameter_projection_bypasses +=
                    alias_count;
            }
            ResourceSummaryRawAliasReturnEntryReprojectionReject::ParameterType => {
                self.stats
                    .resource_summary_value_raw_alias_return_entry_reprojection_value_parameter_type_bypasses +=
                    alias_count;
            }
            ResourceSummaryRawAliasReturnEntryReprojectionReject::ReturnProjection => {
                self.stats
                    .resource_summary_value_raw_alias_return_entry_reprojection_value_return_projection_bypasses +=
                    alias_count;
            }
            ResourceSummaryRawAliasReturnEntryReprojectionReject::ReturnType => {
                self.stats
                    .resource_summary_value_raw_alias_return_entry_reprojection_value_return_type_bypasses +=
                    alias_count;
            }
        }
    }

    fn record_raw_alias_return_entry_bypass_count(&mut self, alias_count: usize) {
        self.stats.resource_summary_value_bypasses += alias_count;
        self.stats
            .resource_summary_value_raw_alias_return_entry_bypasses += alias_count;
    }

    fn record_raw_alias_return_entry_replay_bypass(&mut self, alias_count: usize) {
        self.stats.resource_summary_value_replay_bypasses += alias_count;
    }

    pub(in crate::resource) fn record_raw_alias_return_entry_candidates(
        &mut self,
        candidates: Vec<ResourceSummaryRawAliasReturnEntryCandidate>,
    ) {
        let candidates_with_hits = candidates
            .into_iter()
            .map(|candidate| {
                let existed_before_recording = self
                    .raw_alias_return_entries
                    .get(&candidate.key)
                    .is_some_and(|entry| entry == &candidate.entry);
                (candidate, existed_before_recording)
            })
            .collect::<Vec<_>>();

        for (candidate, existed_before_recording) in candidates_with_hits {
            let alias_count = candidate.entry.len();
            if existed_before_recording {
                self.stats.resource_summary_value_hits += alias_count;
                self.stats
                    .resource_summary_value_raw_alias_return_entry_hits += alias_count;
                continue;
            }

            self.stats.resource_summary_value_misses += alias_count;
            self.raw_alias_return_entries
                .insert(candidate.key, candidate.entry);
            self.stats.resource_summary_value_stores += alias_count;
            self.stats
                .resource_summary_value_raw_alias_return_entry_stores += alias_count;
        }
    }
}
