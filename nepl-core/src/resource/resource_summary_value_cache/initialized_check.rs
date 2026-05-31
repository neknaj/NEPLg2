use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::super::model::ResourceFunction;
use super::super::report::ResourceFunctionCheck;
use super::candidate_key::{
    initialized_function_check_entry_key, ResourceSummaryDependencyClosureHash,
    ResourceSummaryGenericTypeArgumentKeyInput,
};
use super::dependency_hash::ResourceSummaryDependencyClosureHashReject;
use super::stable_mirror::{
    reproject_initialized_function_check_entry_pass,
    reproject_initialized_function_check_entry_result, stable_initialized_function_check_entry,
    ResourceSummaryInitializedFunctionCheckEntryReprojectionReject,
    ResourceSummaryStableInitializedFunctionCheckEntryReject, ResourceSummaryTypeReprojection,
};
use super::{
    ResourceSummaryInitializedFunctionCheckEntryCandidate, ResourceSummaryValueCache,
    ResourceSummaryValueCacheContext,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::resource) enum InitializedFunctionCheckEntryCandidateReject {
    MissingSourcePolicy,
    UnstableKey,
    UnstableEntry(ResourceSummaryStableInitializedFunctionCheckEntryReject),
    ReprojectionContext,
    ReprojectionValue(ResourceSummaryInitializedFunctionCheckEntryReprojectionReject),
}

impl ResourceSummaryValueCache {
    pub(in crate::resource) fn initialized_function_check_entry_candidate(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        dependency_closure_hash: ResourceSummaryDependencyClosureHash,
        check: &ResourceFunctionCheck,
        op_count: usize,
    ) -> Result<
        ResourceSummaryInitializedFunctionCheckEntryCandidate,
        InitializedFunctionCheckEntryCandidateReject,
    > {
        let Some(source_capability_policy_hash) =
            context.source_capability_policy_hash_for_function(function)
        else {
            return Err(InitializedFunctionCheckEntryCandidateReject::MissingSourcePolicy);
        };
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let Some(key) = initialized_function_check_entry_key(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            dependency_closure_hash,
            function,
            type_params,
            generic_type_args,
        ) else {
            return Err(InitializedFunctionCheckEntryCandidateReject::UnstableKey);
        };
        let entry = stable_initialized_function_check_entry(types, function, check)
            .map_err(InitializedFunctionCheckEntryCandidateReject::UnstableEntry)?;
        let Some(reprojection) =
            ResourceSummaryTypeReprojection::new_for_initialized_function_check(
                types,
                function,
                type_params,
            )
        else {
            return Err(InitializedFunctionCheckEntryCandidateReject::ReprojectionContext);
        };
        if let Err(reason) =
            reproject_initialized_function_check_entry_result(&reprojection, &function.name, &entry)
        {
            return Err(InitializedFunctionCheckEntryCandidateReject::ReprojectionValue(reason));
        }
        Ok(ResourceSummaryInitializedFunctionCheckEntryCandidate {
            key,
            entry,
            op_count,
        })
    }

    pub(in crate::resource) fn record_initialized_function_check_dependency_closure_bypass(
        &mut self,
        _reason: ResourceSummaryDependencyClosureHashReject,
        op_count: usize,
    ) {
        self.record_initialized_function_check_bypass_count(op_count);
        self.stats
            .resource_summary_value_initialized_function_check_dependency_bypasses += 1;
    }

    pub(in crate::resource) fn record_initialized_function_check_diagnostic_bypass(
        &mut self,
        op_count: usize,
    ) {
        self.record_initialized_function_check_bypass_count(op_count);
        self.stats
            .resource_summary_value_initialized_function_check_diagnostic_bypasses += 1;
    }

    pub(in crate::resource) fn record_initialized_function_check_candidate_bypass(
        &mut self,
        reason: InitializedFunctionCheckEntryCandidateReject,
        op_count: usize,
    ) {
        self.record_initialized_function_check_bypass_count(op_count);
        match reason {
            InitializedFunctionCheckEntryCandidateReject::MissingSourcePolicy => {
                self.stats
                    .resource_summary_value_initialized_function_check_missing_source_policy_bypasses +=
                    1;
            }
            InitializedFunctionCheckEntryCandidateReject::UnstableKey => {
                self.stats
                    .resource_summary_value_initialized_function_check_unstable_key_bypasses += 1;
            }
            InitializedFunctionCheckEntryCandidateReject::UnstableEntry(reason) => {
                self.stats
                    .resource_summary_value_initialized_function_check_unstable_entry_bypasses += 1;
                self.record_initialized_function_check_unstable_entry_bypass(reason);
            }
            InitializedFunctionCheckEntryCandidateReject::ReprojectionContext => {
                self.stats
                    .resource_summary_value_initialized_function_check_reprojection_bypasses += 1;
                self.stats
                    .resource_summary_value_initialized_function_check_reprojection_context_bypasses +=
                    1;
            }
            InitializedFunctionCheckEntryCandidateReject::ReprojectionValue(reason) => {
                self.stats
                    .resource_summary_value_initialized_function_check_reprojection_bypasses += 1;
                self.stats
                    .resource_summary_value_initialized_function_check_reprojection_value_bypasses +=
                    1;
                self.record_initialized_function_check_reprojection_value_bypass(reason);
            }
        }
    }

    fn record_initialized_function_check_bypass_count(&mut self, op_count: usize) {
        self.stats.resource_summary_value_bypasses += op_count;
        self.stats
            .resource_summary_value_initialized_function_check_bypasses += 1;
    }

    fn record_initialized_function_check_unstable_entry_bypass(
        &mut self,
        reason: ResourceSummaryStableInitializedFunctionCheckEntryReject,
    ) {
        match reason {
            ResourceSummaryStableInitializedFunctionCheckEntryReject::AutoDropPoints => {
                self.stats
                    .resource_summary_value_initialized_function_check_unstable_entry_auto_drop_bypasses +=
                    1;
            }
            ResourceSummaryStableInitializedFunctionCheckEntryReject::Place => {
                self.stats
                    .resource_summary_value_initialized_function_check_unstable_entry_place_bypasses +=
                    1;
            }
            ResourceSummaryStableInitializedFunctionCheckEntryReject::Type => {
                self.stats
                    .resource_summary_value_initialized_function_check_unstable_entry_type_bypasses +=
                    1;
            }
        }
    }

    fn record_initialized_function_check_reprojection_value_bypass(
        &mut self,
        reason: ResourceSummaryInitializedFunctionCheckEntryReprojectionReject,
    ) {
        match reason {
            ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::Place => {
                self.stats
                    .resource_summary_value_initialized_function_check_reprojection_value_place_bypasses +=
                    1;
            }
            ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::PlaceType => {
                self.stats
                    .resource_summary_value_initialized_function_check_reprojection_value_type_bypasses +=
                    1;
                self.stats
                    .resource_summary_value_initialized_function_check_reprojection_value_place_type_bypasses +=
                    1;
            }
            ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::CellStateType => {
                self.stats
                    .resource_summary_value_initialized_function_check_reprojection_value_type_bypasses +=
                    1;
                self.stats
                    .resource_summary_value_initialized_function_check_reprojection_value_cell_state_type_bypasses +=
                    1;
            }
            ResourceSummaryInitializedFunctionCheckEntryReprojectionReject::CollectionSlotStateType => {
                self.stats
                    .resource_summary_value_initialized_function_check_reprojection_value_type_bypasses +=
                    1;
                self.stats
                    .resource_summary_value_initialized_function_check_reprojection_value_collection_slot_state_type_bypasses +=
                    1;
            }
        }
    }

    /// final initialized check の cached pass を、final state を materialize せずに戻す。
    ///
    /// `initialized_function_check_entry_key` は body hash、dependency closure、type
    /// boundary、source capability policy を含むため、key が一致した diagnostic-free /
    /// auto-drop-free entry は現在 compile でも pass として扱える。後続 stage が必要と
    /// するのは function name、auto drop point、deferred counter だけなので、ここでは
    /// final cell / collection slot state の再投影を避ける。
    pub(in crate::resource) fn replay_initialized_function_check_entry_pass(
        &mut self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        dependency_closure_hash: ResourceSummaryDependencyClosureHash,
        op_count: usize,
    ) -> Option<ResourceFunctionCheck> {
        let source_capability_policy_hash =
            context.source_capability_policy_hash_for_function(function)?;
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let key = initialized_function_check_entry_key(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            dependency_closure_hash,
            function,
            type_params,
            generic_type_args,
        )?;
        let entry = self.initialized_function_check_entries.get(&key)?;

        self.stats.resource_summary_value_replay_hits += op_count;
        self.stats.resource_summary_value_lazy_pass_hits += 1;
        self.stats.resource_summary_value_lazy_pass_ops += op_count;
        self.stats
            .resource_summary_value_initialized_function_check_hits += 1;
        let check = reproject_initialized_function_check_entry_pass(&function.name, entry);
        self.record_initialized_function_check_replay_hit_function();
        Some(check)
    }

    pub(in crate::resource) fn record_initialized_function_check_entry_candidates(
        &mut self,
        candidates: Vec<ResourceSummaryInitializedFunctionCheckEntryCandidate>,
    ) {
        let candidates_with_hits = candidates
            .into_iter()
            .map(|candidate| {
                let existed_before_recording = self
                    .initialized_function_check_entries
                    .get(&candidate.key)
                    .is_some_and(|entry| entry == &candidate.entry);
                (candidate, existed_before_recording)
            })
            .collect::<Vec<_>>();

        for (candidate, existed_before_recording) in candidates_with_hits {
            if existed_before_recording {
                self.stats.resource_summary_value_hits += candidate.op_count;
                self.stats
                    .resource_summary_value_initialized_function_check_hits += 1;
                continue;
            }

            self.stats.resource_summary_value_misses += candidate.op_count;
            self.initialized_function_check_entries
                .insert(candidate.key, candidate.entry);
            self.stats.resource_summary_value_stores += candidate.op_count;
            self.stats
                .resource_summary_value_initialized_function_check_stores += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::ast::Effect;
    use crate::span::{FileId, Span};
    use crate::types::TypeCtx;

    use super::super::super::drop_model::ResourceDropPoint;
    use super::super::super::drop_point_path::ResourceDropPointPath;
    use super::super::super::model::{
        Place, ResourceBlock, ResourceBlockId, ResourceFunction, ResourceLocal, ResourceModule,
        ResourceTerminator,
    };
    use super::super::super::owner_summary_type_params::owner_summary_type_params;
    use super::super::super::report::{ResourceCheckDeferred, ResourceFunctionCheck};
    use super::super::super::summary_dependency::build_function_summary_dependencies;
    use super::super::dependency_hash::initialized_function_check_dependency_closure_hash;
    use super::super::stable_mirror::ResourceSummaryStableInitializedFunctionCheckEntryReject;
    use super::*;

    fn test_span() -> Span {
        Span::new(FileId(0), 1, 2)
    }

    fn test_context(policy_hash: u64) -> ResourceSummaryValueCacheContext {
        let mut context = ResourceSummaryValueCacheContext::new(7);
        context.insert_source_policy_hash(FileId(0), policy_hash);
        context
    }

    fn local_identity_function(types: &TypeCtx) -> ResourceFunction {
        let span = test_span();
        let param_place = Place::local(String::from("value"), types.i32());
        ResourceFunction {
            name: String::from("identity"),
            origin_name: String::from("identity"),
            type_params: Vec::new(),
            params: vec![ResourceLocal {
                name: String::from("value"),
                ty: types.i32(),
                mutable: false,
                place: param_place.clone(),
            }],
            result: types.i32(),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: Vec::new(),
                terminator: ResourceTerminator::Return {
                    value: Some(param_place),
                    span,
                },
                span,
            }],
            span,
        }
    }

    fn single_function_module(function: ResourceFunction) -> ResourceModule {
        ResourceModule {
            functions: vec![function],
            entry: None,
            string_literals: Vec::new(),
        }
    }

    /// auto drop plan は後続の drop elaboration が span 付きで消費するため、final check
    /// cache の MVP では保存しない。診断を持たない関数でも、この場合は no-store に倒す。
    #[test]
    fn initialized_function_check_candidate_rejects_auto_drop_points() {
        let types = TypeCtx::new();
        let module = single_function_module(local_identity_function(&types));
        let function = &module.functions[0];
        let context = test_context(11);
        let dependencies = build_function_summary_dependencies(&module);
        let dependency_closure_hash = initialized_function_check_dependency_closure_hash(
            &context,
            &types,
            &module,
            &dependencies,
            0,
        )
        .expect("test function dependency hash should be stable");
        let type_params = owner_summary_type_params(&types, function);
        let check = ResourceFunctionCheck {
            name: function.name.clone(),
            final_cells: Vec::new(),
            final_collection_slots: Vec::new(),
            auto_drop_points: vec![ResourceDropPoint {
                path: ResourceDropPointPath {
                    block: ResourceBlockId(0),
                    steps: Vec::new(),
                },
                span: test_span(),
                auto_drops: Vec::new(),
            }],
            deferred: ResourceCheckDeferred::default(),
        };
        let cache = ResourceSummaryValueCache::new();

        let result = cache.initialized_function_check_entry_candidate(
            &context,
            &types,
            function,
            &type_params,
            dependency_closure_hash,
            &check,
            1,
        );

        assert!(matches!(
            result,
            Err(InitializedFunctionCheckEntryCandidateReject::UnstableEntry(
                ResourceSummaryStableInitializedFunctionCheckEntryReject::AutoDropPoints
            ))
        ));
    }
}
