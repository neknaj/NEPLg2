use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::super::initialized_summary::RawCellInitializationFunctionSummary;
use super::super::model::ResourceFunction;
use super::candidate_key::{
    raw_init_complete_leaf_entry_key, ResourceSummaryDependencyClosureHash,
    ResourceSummaryGenericTypeArgumentKeyInput,
};
use super::dependency_hash::RawInitDependencyClosureHashReject;
use super::stable_mirror::{
    reproject_raw_init_complete_leaf_entry, reproject_raw_init_complete_leaf_entry_result,
    stable_raw_init_complete_leaf_entry, ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
    ResourceSummaryStableRawInitCompleteLeafEntryReject, ResourceSummaryTypeReprojection,
};
use super::{
    ResourceSummaryRawInitCompleteLeafEntryCandidate, ResourceSummaryValueCache,
    ResourceSummaryValueCacheContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum RawInitCompleteLeafEntryCandidateReject {
    MissingSourcePolicy,
    UnstableKey,
    UnstableEntry(ResourceSummaryStableRawInitCompleteLeafEntryReject),
    ReprojectionContext,
    ReprojectionValue(ResourceSummaryRawInitCompleteLeafEntryReprojectionReject),
}

impl ResourceSummaryValueCache {
    pub(in crate::resource) fn raw_init_complete_leaf_entry_candidate(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        dependency_closure_hash: ResourceSummaryDependencyClosureHash,
        summary: &RawCellInitializationFunctionSummary,
    ) -> Result<
        ResourceSummaryRawInitCompleteLeafEntryCandidate,
        RawInitCompleteLeafEntryCandidateReject,
    > {
        let Some(source_capability_policy_hash) =
            context.source_capability_policy_hash_for_function(function)
        else {
            return Err(RawInitCompleteLeafEntryCandidateReject::MissingSourcePolicy);
        };
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let Some(key) = raw_init_complete_leaf_entry_key(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            dependency_closure_hash,
            function,
            type_params,
            generic_type_args,
        ) else {
            return Err(RawInitCompleteLeafEntryCandidateReject::UnstableKey);
        };
        let entry = stable_raw_init_complete_leaf_entry(types, function, summary)
            .map_err(RawInitCompleteLeafEntryCandidateReject::UnstableEntry)?;
        let Some(reprojection) = ResourceSummaryTypeReprojection::new(types, function, type_params)
        else {
            return Err(RawInitCompleteLeafEntryCandidateReject::ReprojectionContext);
        };
        if let Err(reason) =
            reproject_raw_init_complete_leaf_entry_result(&reprojection, &function.name, &entry)
        {
            return Err(RawInitCompleteLeafEntryCandidateReject::ReprojectionValue(
                reason,
            ));
        }
        Ok(ResourceSummaryRawInitCompleteLeafEntryCandidate { key, entry })
    }

    pub(in crate::resource) fn record_raw_init_param_facts_bypass_count(
        &mut self,
        fact_count: usize,
    ) {
        self.stats.resource_summary_value_bypasses += fact_count;
        self.stats
            .resource_summary_value_raw_init_param_facts_bypasses += fact_count;
    }

    pub(in crate::resource) fn record_raw_init_param_facts_dependency_bypass(
        &mut self,
        fact_count: usize,
    ) {
        self.record_raw_init_param_facts_bypass_count(fact_count);
        self.stats
            .resource_summary_value_raw_init_param_facts_dependency_bypasses += fact_count;
    }

    pub(in crate::resource) fn record_raw_init_param_facts_candidate_bypass(
        &mut self,
        reason: RawInitCompleteLeafEntryCandidateReject,
        fact_count: usize,
    ) {
        self.record_raw_init_param_facts_bypass_count(fact_count);
        match reason {
            RawInitCompleteLeafEntryCandidateReject::MissingSourcePolicy => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_missing_source_policy_bypasses +=
                    fact_count;
            }
            RawInitCompleteLeafEntryCandidateReject::UnstableKey => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_unstable_key_bypasses +=
                    fact_count;
            }
            RawInitCompleteLeafEntryCandidateReject::UnstableEntry(reason) => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_unstable_entry_bypasses +=
                    fact_count;
                self.record_raw_init_unstable_entry_bypass(reason, fact_count);
            }
            RawInitCompleteLeafEntryCandidateReject::ReprojectionContext => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_reprojection_bypasses +=
                    fact_count;
                self.stats
                    .resource_summary_value_raw_init_param_facts_reprojection_context_bypasses +=
                    fact_count;
            }
            RawInitCompleteLeafEntryCandidateReject::ReprojectionValue(reason) => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_reprojection_bypasses +=
                    fact_count;
                self.stats
                    .resource_summary_value_raw_init_param_facts_reprojection_value_bypasses +=
                    fact_count;
                self.record_raw_init_reprojection_value_bypass(reason, fact_count);
            }
        }
    }

    fn record_raw_init_reprojection_value_bypass(
        &mut self,
        reason: ResourceSummaryRawInitCompleteLeafEntryReprojectionReject,
        fact_count: usize,
    ) {
        match reason {
            ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellProjection => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_projection_bypasses +=
                    fact_count;
            }
            ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellStableType => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_type_bypasses +=
                    fact_count;
                self.stats
                    .resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_stable_type_bypasses +=
                    fact_count;
            }
            ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamCellResultType => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_type_bypasses +=
                    fact_count;
                self.stats
                    .resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_result_type_bypasses +=
                    fact_count;
            }
            ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamReleaseRequirementProjection => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_reprojection_value_param_release_projection_bypasses +=
                    fact_count;
            }
            ResourceSummaryRawInitCompleteLeafEntryReprojectionReject::ParamReleaseRequirementType => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_reprojection_value_param_release_type_bypasses +=
                    fact_count;
            }
        }
    }

    fn record_raw_init_unstable_entry_bypass(
        &mut self,
        reason: ResourceSummaryStableRawInitCompleteLeafEntryReject,
        fact_count: usize,
    ) {
        match reason {
            ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellProjection => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_unstable_entry_param_cell_projection_bypasses +=
                    fact_count;
            }
            ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamCellType => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_unstable_entry_param_cell_type_bypasses +=
                    fact_count;
            }
            ResourceSummaryStableRawInitCompleteLeafEntryReject::ParamReleaseRequirementType => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_unstable_entry_param_release_type_bypasses +=
                    fact_count;
            }
        }
    }

    pub(in crate::resource) fn record_raw_init_dependency_closure_bypass(
        &mut self,
        reason: RawInitDependencyClosureHashReject,
        fact_count: usize,
    ) {
        self.record_raw_init_param_facts_bypass_count(fact_count);
        self.stats
            .resource_summary_value_raw_init_param_facts_unstable_key_bypasses += fact_count;
        match reason {
            RawInitDependencyClosureHashReject::DependencyGraph => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_dependency_graph_bypasses +=
                    fact_count;
            }
            RawInitDependencyClosureHashReject::DependencyFunctionIdentity => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_dependency_identity_bypasses +=
                    fact_count;
            }
            RawInitDependencyClosureHashReject::DependencyFunctionBody => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_dependency_body_hash_bypasses +=
                    fact_count;
            }
            RawInitDependencyClosureHashReject::DependencySourcePolicy => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_dependency_source_policy_bypasses +=
                    fact_count;
            }
            RawInitDependencyClosureHashReject::DependencyTypeBoundary => {
                self.stats
                    .resource_summary_value_raw_init_param_facts_dependency_type_boundary_bypasses +=
                    fact_count;
            }
        }
    }

    pub(in crate::resource) fn replay_raw_init_complete_leaf_entry(
        &mut self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        dependency_closure_hash: ResourceSummaryDependencyClosureHash,
    ) -> Option<RawCellInitializationFunctionSummary> {
        let source_capability_policy_hash =
            context.source_capability_policy_hash_for_function(function)?;
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let key = raw_init_complete_leaf_entry_key(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            dependency_closure_hash,
            function,
            type_params,
            generic_type_args,
        )?;
        let entry = self.raw_init_complete_leaf_entries.get(&key)?.clone();
        let fact_count = entry.len();
        let Some(reprojection) = ResourceSummaryTypeReprojection::new(types, function, type_params)
        else {
            self.record_raw_init_param_facts_replay_bypass(fact_count);
            return None;
        };
        let Some(summary) =
            reproject_raw_init_complete_leaf_entry(&reprojection, &function.name, &entry)
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

    pub(in crate::resource) fn record_raw_init_complete_leaf_entry_candidates(
        &mut self,
        candidates: Vec<ResourceSummaryRawInitCompleteLeafEntryCandidate>,
    ) {
        let candidates_with_hits = candidates
            .into_iter()
            .map(|candidate| {
                let existed_before_recording = self
                    .raw_init_complete_leaf_entries
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
            self.raw_init_complete_leaf_entries
                .insert(candidate.key, candidate.entry);
            self.stats.resource_summary_value_stores += fact_count;
            self.stats
                .resource_summary_value_raw_init_param_facts_stores += fact_count;
        }
    }
}
