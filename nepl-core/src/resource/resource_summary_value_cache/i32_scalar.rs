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
    reproject_i32_scalar_return_facts_entry_result, stable_i32_scalar_return_facts_entry,
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
                self.record_i32_scalar_return_facts_unstable_entry_bypass(reason, fact_count);
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
                self.record_i32_scalar_return_facts_reprojection_value_bypass(reason, fact_count);
            }
        }
    }

    /// i32 scalar return facts の candidate 化に失敗した量を記録する。
    ///
    /// `ReprojectionValue` は、現在の関数型や projection に対して保存済み fact を
    /// 安定な形へ戻せなかったことを意味する。合計値だけでは次の改善対象を選べないため、
    /// この経路では失敗した fact の種類別件数も同時に記録する。
    pub(in crate::resource) fn record_i32_scalar_return_facts_candidate_bypass_for_facts(
        &mut self,
        reason: I32ScalarReturnFactsEntryCandidateReject,
        facts: &I32ScalarReturnFacts,
    ) {
        if matches!(
            reason,
            I32ScalarReturnFactsEntryCandidateReject::ReprojectionValue(_)
        ) {
            self.record_i32_scalar_return_facts_reprojection_value_kind_bypasses(facts);
        }
        self.record_i32_scalar_return_facts_candidate_bypass(reason, facts.len());
    }

    /// `ReprojectionValue` で失われた i32 scalar fact を種類別に積算する。
    ///
    /// ここで数える値は cache の hit/miss 判定には使わない。RPN の小規模 edit で
    /// 残っている再計算が alias 由来なのか condition 由来なのかを、Web playground と
    /// Node 計測から同じ JSON schema で読めるようにするための観測値である。
    pub(in crate::resource) fn record_i32_scalar_return_facts_reprojection_value_kind_bypasses(
        &mut self,
        facts: &I32ScalarReturnFacts,
    ) {
        let counts = facts.fact_counts();
        self.stats
            .resource_summary_value_i32_scalar_return_facts_reprojection_value_alias_bypasses +=
            counts.aliases;
        self.stats
            .resource_summary_value_i32_scalar_return_facts_reprojection_value_offset_bypasses +=
            counts.offsets;
        self.stats
            .resource_summary_value_i32_scalar_return_facts_reprojection_value_relation_bypasses +=
            counts.relations;
        self.stats
            .resource_summary_value_i32_scalar_return_facts_reprojection_value_constant_bypasses +=
            counts.constants;
        self.stats
            .resource_summary_value_i32_scalar_return_facts_reprojection_value_return_condition_bypasses +=
            counts.return_conditions;
        self.stats
            .resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_condition_bypasses +=
            counts.parameter_conditions;
    }

    fn record_i32_scalar_return_facts_unstable_entry_bypass(
        &mut self,
        reason: I32ScalarReturnFactsEntryCandidateReject,
        fact_count: usize,
    ) {
        let I32ScalarReturnFactsEntryCandidateReject::UnstableEntry(reason) = reason else {
            return;
        };
        match reason {
            ResourceSummaryStableI32ScalarReturnFactsEntryReject::ReturnProjection => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_unstable_entry_return_projection_bypasses +=
                    fact_count;
            }
            ResourceSummaryStableI32ScalarReturnFactsEntryReject::ParameterProjection => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_unstable_entry_parameter_projection_bypasses +=
                    fact_count;
            }
            ResourceSummaryStableI32ScalarReturnFactsEntryReject::ScalarType => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_unstable_entry_scalar_type_bypasses +=
                    fact_count;
            }
        }
    }

    fn record_i32_scalar_return_facts_reprojection_value_bypass(
        &mut self,
        reason: I32ScalarReturnFactsEntryCandidateReject,
        fact_count: usize,
    ) {
        let I32ScalarReturnFactsEntryCandidateReject::ReprojectionValue(reason) = reason else {
            return;
        };
        match reason {
            ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ReturnProjection => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_reprojection_value_return_projection_bypasses +=
                    fact_count;
            }
            ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ParameterProjection => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_projection_bypasses +=
                    fact_count;
            }
            ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ScalarType => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_reprojection_value_scalar_type_bypasses +=
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
        let Some(source_capability_policy_hash) =
            context.source_capability_policy_hash_for_function(function)
        else {
            self.stats
                .resource_summary_value_i32_scalar_return_facts_replay_missing_source_policy_functions +=
                1;
            return None;
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
            self.stats
                .resource_summary_value_i32_scalar_return_facts_replay_unstable_key_functions += 1;
            return None;
        };
        let Some(entry) = self.i32_scalar_return_facts_entries.get(&key).cloned() else {
            self.stats
                .resource_summary_value_i32_scalar_return_facts_replay_entry_miss_functions += 1;
            return None;
        };
        let fact_count = entry.len();
        let Some(reprojection) = ResourceSummaryTypeReprojection::new(types, function, type_params)
        else {
            self.record_i32_scalar_return_facts_replay_bypass(fact_count);
            self.stats
                .resource_summary_value_i32_scalar_return_facts_replay_reprojection_context_functions +=
                1;
            return None;
        };
        let facts = match reproject_i32_scalar_return_facts_entry_result(&reprojection, &entry) {
            Ok(facts) => facts,
            Err(reason) => {
                self.record_i32_scalar_return_facts_replay_bypass(fact_count);
                self.record_i32_scalar_return_facts_replay_reprojection_value_function(reason);
                return None;
            }
        };

        self.stats.resource_summary_value_replay_hits += fact_count;
        self.stats.resource_summary_value_replayed_ops += fact_count;
        Some(facts)
    }

    fn record_i32_scalar_return_facts_replay_reprojection_value_function(
        &mut self,
        reason: ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject,
    ) {
        self.stats
            .resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_functions +=
            1;
        match reason {
            ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ReturnProjection => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_return_projection_functions +=
                    1;
            }
            ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ParameterProjection => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_parameter_projection_functions +=
                    1;
            }
            ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ScalarType => {
                self.stats
                    .resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_scalar_type_functions +=
                    1;
            }
        }
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
            self.stats
                .resource_summary_value_i32_scalar_return_facts_misses += fact_count;
            self.i32_scalar_return_facts_entries
                .insert(candidate.key, candidate.entry);
            self.stats.resource_summary_value_stores += fact_count;
            self.stats
                .resource_summary_value_i32_scalar_return_facts_stores += fact_count;
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::types::TypeCtx;

    use super::super::super::i32_scalar_return_facts::{
        I32ScalarParameterCondition, I32ScalarReturnAlias, I32ScalarReturnCondition,
        I32ScalarReturnConstant, I32ScalarReturnFacts, I32ScalarReturnOffset,
        I32ScalarReturnRelation,
    };
    use super::super::super::model::{I32ValueCondition, ResourceI32RelationOp};
    use super::*;

    /// `ReprojectionValue` で complete entry を保存できない場合、aggregate の
    /// fact 数だけでは次の stable mirror 改善対象を選べない。種類別 counter は、
    /// alias / offset / condition など、どの surface が失われたかを JSON stats へ
    /// 出すために同じ fact 集合から積算する。
    #[test]
    fn i32_scalar_reprojection_value_bypass_counts_fact_kinds() {
        let mut cache = ResourceSummaryValueCache::new();
        let types = TypeCtx::new();
        let facts = I32ScalarReturnFacts {
            aliases: vec![I32ScalarReturnAlias {
                return_projection: Vec::new(),
                parameter_index: 0,
                parameter_projection: Vec::new(),
                scalar_ty: types.i32(),
            }],
            offsets: vec![I32ScalarReturnOffset {
                return_projection: Vec::new(),
                parameter_index: 0,
                parameter_projection: Vec::new(),
                scalar_ty: types.i32(),
                offset: 1,
            }],
            relations: vec![I32ScalarReturnRelation {
                left_return_projection: Vec::new(),
                op: ResourceI32RelationOp::Eq,
                right_return_projection: Vec::new(),
                scalar_ty: types.i32(),
            }],
            constants: vec![I32ScalarReturnConstant {
                return_projection: Vec::new(),
                scalar_ty: types.i32(),
                value: 7,
            }],
            return_conditions: vec![I32ScalarReturnCondition {
                return_projection: Vec::new(),
                scalar_ty: types.i32(),
                condition: I32ValueCondition::Positive,
            }],
            parameter_conditions: vec![I32ScalarParameterCondition {
                parameter_index: 0,
                parameter_projection: Vec::new(),
                scalar_ty: types.i32(),
                condition: I32ValueCondition::NonNegative,
            }],
        };

        cache.record_i32_scalar_return_facts_candidate_bypass_for_facts(
            I32ScalarReturnFactsEntryCandidateReject::ReprojectionValue(
                ResourceSummaryI32ScalarReturnFactsEntryReprojectionReject::ScalarType,
            ),
            &facts,
        );

        let stats = cache.stats();
        assert_eq!(
            stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses,
            6
        );
        assert_eq!(
            stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_alias_bypasses,
            1
        );
        assert_eq!(
            stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_offset_bypasses,
            1
        );
        assert_eq!(
            stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_relation_bypasses,
            1
        );
        assert_eq!(
            stats.resource_summary_value_i32_scalar_return_facts_reprojection_value_constant_bypasses,
            1
        );
        assert_eq!(
            stats
                .resource_summary_value_i32_scalar_return_facts_reprojection_value_return_condition_bypasses,
            1
        );
        assert_eq!(
            stats
                .resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_condition_bypasses,
            1
        );
    }
}
