use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::super::model::ResourceFunction;
use super::super::report::ResourceOwnerFunctionCheck;
use super::candidate_key::{
    owner_obligation_check_entry_key, ResourceSummaryDependencyClosureHash,
    ResourceSummaryGenericTypeArgumentKeyInput,
};
use super::dependency_hash::ResourceSummaryDependencyClosureHashReject;
use super::stable_mirror::{
    reproject_owner_obligation_check_entry_pass, stable_owner_obligation_check_entry,
};
use super::{
    ResourceSummaryOwnerObligationCheckEntryCandidate, ResourceSummaryValueCache,
    ResourceSummaryValueCacheContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum OwnerObligationCheckEntryCandidateReject {
    MissingSourcePolicy,
    UnstableKey,
}

impl ResourceSummaryValueCache {
    pub(in crate::resource) fn owner_obligation_check_entry_candidate(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        dependency_closure_hash: ResourceSummaryDependencyClosureHash,
        check: &ResourceOwnerFunctionCheck,
        op_count: usize,
    ) -> Result<
        ResourceSummaryOwnerObligationCheckEntryCandidate,
        OwnerObligationCheckEntryCandidateReject,
    > {
        let Some(source_capability_policy_hash) =
            context.source_capability_policy_hash_for_function(function)
        else {
            return Err(OwnerObligationCheckEntryCandidateReject::MissingSourcePolicy);
        };
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let Some(key) = owner_obligation_check_entry_key(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            dependency_closure_hash,
            function,
            type_params,
            generic_type_args,
        ) else {
            return Err(OwnerObligationCheckEntryCandidateReject::UnstableKey);
        };
        let entry = stable_owner_obligation_check_entry(check);
        Ok(ResourceSummaryOwnerObligationCheckEntryCandidate {
            key,
            entry,
            op_count,
        })
    }

    pub(in crate::resource) fn record_owner_obligation_check_dependency_closure_bypass(
        &mut self,
        _reason: ResourceSummaryDependencyClosureHashReject,
        op_count: usize,
    ) {
        self.record_owner_obligation_check_bypass_count(op_count);
        self.stats
            .resource_summary_value_owner_obligation_check_dependency_bypasses += 1;
    }

    pub(in crate::resource) fn record_owner_obligation_check_diagnostic_bypass(
        &mut self,
        op_count: usize,
    ) {
        self.record_owner_obligation_check_bypass_count(op_count);
        self.stats
            .resource_summary_value_owner_obligation_check_diagnostic_bypasses += 1;
    }

    pub(in crate::resource) fn record_owner_obligation_check_candidate_bypass(
        &mut self,
        reason: OwnerObligationCheckEntryCandidateReject,
        op_count: usize,
    ) {
        self.record_owner_obligation_check_bypass_count(op_count);
        match reason {
            OwnerObligationCheckEntryCandidateReject::MissingSourcePolicy => {
                self.stats
                    .resource_summary_value_owner_obligation_check_missing_source_policy_bypasses +=
                    1;
            }
            OwnerObligationCheckEntryCandidateReject::UnstableKey => {
                self.stats
                    .resource_summary_value_owner_obligation_check_unstable_key_bypasses += 1;
            }
        }
    }

    fn record_owner_obligation_check_bypass_count(&mut self, op_count: usize) {
        self.stats.resource_summary_value_bypasses += op_count;
        self.stats
            .resource_summary_value_owner_obligation_check_bypasses += 1;
    }

    /// owner obligation の cached pass を、owner state を materialize せずに戻す。
    ///
    /// key は body hash、dependency closure、type boundary、source capability policy を
    /// 含む。entry が diagnostic-free として保存されている場合、現在の compile gate は
    /// diagnostics だけを消費するため、`final_owners` を再構築せず pass として扱える。
    pub(in crate::resource) fn replay_owner_obligation_check_entry_pass(
        &mut self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        dependency_closure_hash: ResourceSummaryDependencyClosureHash,
        op_count: usize,
    ) -> Option<ResourceOwnerFunctionCheck> {
        let source_capability_policy_hash =
            context.source_capability_policy_hash_for_function(function)?;
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let key = owner_obligation_check_entry_key(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            dependency_closure_hash,
            function,
            type_params,
            generic_type_args,
        )?;
        let entry = self.owner_obligation_check_entries.get(&key)?;

        self.stats.resource_summary_value_replay_hits += op_count;
        self.stats.resource_summary_value_lazy_pass_hits += 1;
        self.stats.resource_summary_value_lazy_pass_ops += op_count;
        self.stats
            .resource_summary_value_owner_obligation_check_hits += 1;
        let check = reproject_owner_obligation_check_entry_pass(&function.name, entry);
        self.record_owner_obligation_check_replay_hit_function();
        Some(check)
    }

    pub(in crate::resource) fn record_owner_obligation_check_entry_candidates(
        &mut self,
        candidates: Vec<ResourceSummaryOwnerObligationCheckEntryCandidate>,
    ) {
        let candidates_with_hits = candidates
            .into_iter()
            .map(|candidate| {
                let existed_before_recording = self
                    .owner_obligation_check_entries
                    .get(&candidate.key)
                    .is_some_and(|entry| entry == &candidate.entry);
                (candidate, existed_before_recording)
            })
            .collect::<Vec<_>>();

        for (candidate, existed_before_recording) in candidates_with_hits {
            if existed_before_recording {
                self.stats.resource_summary_value_hits += candidate.op_count;
                self.stats
                    .resource_summary_value_owner_obligation_check_hits += 1;
                continue;
            }

            self.stats.resource_summary_value_misses += candidate.op_count;
            self.owner_obligation_check_entries
                .insert(candidate.key, candidate.entry);
            self.stats.resource_summary_value_stores += candidate.op_count;
            self.stats
                .resource_summary_value_owner_obligation_check_stores += 1;
        }
    }
}
