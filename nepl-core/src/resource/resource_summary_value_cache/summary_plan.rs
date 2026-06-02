extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::super::model::ResourceModule;
use super::super::summary_dependency::ResourceSummaryDependencyGraph;
use super::context::ResourceSummaryValueCacheContext;
use super::function_fingerprint::ResourceFunctionLocalFingerprint;
use super::key::ResourceSummaryValueCacheKey;
use super::ResourceSummaryValueCache;
use super::{
    ResourceSummaryI32ScalarReturnFactsEntryCandidate, ResourceSummaryRawAliasReturnEntryCandidate,
    ResourceSummaryRawInitCompleteLeafEntryCandidate,
};

/// summary fixed-point の前段で使う changed-function replay plan。
///
/// この plan は、前回 compile で stable entry として保存できた summary を、現在 compile
/// でも dependency closure hash を再構築せずに再投影できる関数だけへ限定する。
/// caller が callee summary index を読むため、skip しても summary 自体は必ず現在の
/// `TypeCtx` へ materialize する。保存するのは stable key と fingerprint だけであり、
/// `TypeId`、`Span`、`SourceMap`、summary 本体は snapshot に保持しない。
pub(in crate::resource) struct ResourceSummaryReplayPlan {
    current_snapshot: Option<ResourceSummaryReplaySnapshot>,
    affected_functions: Vec<bool>,
    previous_keys: Vec<Option<ResourceSummaryValueCacheKey>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct ResourceSummaryReplaySnapshot {
    namespace_hash: u64,
    functions: Vec<ResourceSummaryReplaySnapshotEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ResourceSummaryReplaySnapshotEntry {
    fingerprint: ResourceFunctionLocalFingerprint,
    key: Option<ResourceSummaryValueCacheKey>,
}

impl ResourceSummaryValueCache {
    pub(in crate::resource) fn begin_raw_alias_summary_replay_plan(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        module: &ResourceModule,
        dependency_graph: &ResourceSummaryDependencyGraph,
        relevant_functions: &[bool],
    ) -> ResourceSummaryReplayPlan {
        ResourceSummaryReplayPlan::new(
            self.raw_alias_return_summary_snapshot.as_ref(),
            context,
            types,
            module,
            dependency_graph.dependents(),
            relevant_functions,
        )
    }

    pub(in crate::resource) fn finish_raw_alias_summary_replay_plan(
        &mut self,
        plan: ResourceSummaryReplayPlan,
    ) {
        self.raw_alias_return_summary_snapshot = plan.into_snapshot();
    }

    pub(in crate::resource) fn begin_i32_scalar_summary_replay_plan(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        module: &ResourceModule,
        dependency_graph: &ResourceSummaryDependencyGraph,
        relevant_functions: &[bool],
    ) -> ResourceSummaryReplayPlan {
        ResourceSummaryReplayPlan::new(
            self.i32_scalar_return_summary_snapshot.as_ref(),
            context,
            types,
            module,
            dependency_graph.dependents(),
            relevant_functions,
        )
    }

    pub(in crate::resource) fn finish_i32_scalar_summary_replay_plan(
        &mut self,
        plan: ResourceSummaryReplayPlan,
    ) {
        self.i32_scalar_return_summary_snapshot = plan.into_snapshot();
    }

    pub(in crate::resource) fn begin_raw_init_summary_replay_plan(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        module: &ResourceModule,
        dependency_graph: &ResourceSummaryDependencyGraph,
        relevant_functions: &[bool],
    ) -> ResourceSummaryReplayPlan {
        ResourceSummaryReplayPlan::new(
            self.raw_init_complete_leaf_summary_snapshot.as_ref(),
            context,
            types,
            module,
            dependency_graph.raw_init_dependents(),
            relevant_functions,
        )
    }

    pub(in crate::resource) fn finish_raw_init_summary_replay_plan(
        &mut self,
        plan: ResourceSummaryReplayPlan,
    ) {
        self.raw_init_complete_leaf_summary_snapshot = plan.into_snapshot();
    }

    pub(in crate::resource) fn record_raw_alias_summary_plan_skip(&mut self, alias_count: usize) {
        self.stats
            .resource_summary_value_raw_alias_return_entry_plan_skip_functions += 1;
        self.stats
            .resource_summary_value_raw_alias_return_entry_plan_skip_ops += alias_count;
    }

    pub(in crate::resource) fn record_raw_alias_summary_candidate_key(
        &self,
        plan: &mut ResourceSummaryReplayPlan,
        function_index: usize,
        candidate: &ResourceSummaryRawAliasReturnEntryCandidate,
    ) {
        plan.record_key(function_index, candidate.key.clone());
    }

    pub(in crate::resource) fn record_i32_scalar_summary_plan_skip(&mut self, fact_count: usize) {
        self.stats
            .resource_summary_value_i32_scalar_return_facts_plan_skip_functions += 1;
        self.stats
            .resource_summary_value_i32_scalar_return_facts_plan_skip_ops += fact_count;
    }

    pub(in crate::resource) fn record_i32_scalar_summary_candidate_key(
        &self,
        plan: &mut ResourceSummaryReplayPlan,
        function_index: usize,
        candidate: &ResourceSummaryI32ScalarReturnFactsEntryCandidate,
    ) {
        plan.record_key(function_index, candidate.key.clone());
    }

    pub(in crate::resource) fn record_raw_init_summary_plan_skip(&mut self, fact_count: usize) {
        self.stats
            .resource_summary_value_raw_init_param_facts_plan_skip_functions += 1;
        self.stats
            .resource_summary_value_raw_init_param_facts_plan_skip_ops += fact_count;
    }

    pub(in crate::resource) fn record_raw_init_summary_candidate_key(
        &self,
        plan: &mut ResourceSummaryReplayPlan,
        function_index: usize,
        candidate: &ResourceSummaryRawInitCompleteLeafEntryCandidate,
    ) {
        plan.record_key(function_index, candidate.key.clone());
    }
}

impl ResourceSummaryReplayPlan {
    fn new(
        previous_snapshot: Option<&ResourceSummaryReplaySnapshot>,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        module: &ResourceModule,
        dependents: &[Vec<usize>],
        relevant_functions: &[bool],
    ) -> Self {
        let Some(current_snapshot) =
            ResourceSummaryReplaySnapshot::from_module(context, types, module)
        else {
            return Self::conservative(module.functions.len());
        };
        let Some(previous_snapshot) = previous_snapshot else {
            return Self::all_affected(current_snapshot);
        };
        if previous_snapshot.namespace_hash != current_snapshot.namespace_hash
            || previous_snapshot.functions.len() != current_snapshot.functions.len()
            || previous_snapshot.functions.len() != relevant_functions.len()
            || !snapshots_keep_function_order(previous_snapshot, &current_snapshot)
        {
            return Self::all_affected(current_snapshot);
        }

        let locally_changed = previous_snapshot
            .functions
            .iter()
            .zip(current_snapshot.functions.iter())
            .map(|(previous, current)| previous.fingerprint != current.fingerprint)
            .collect::<Vec<_>>();
        let affected_functions = affected_closure_from_local_changes(&locally_changed, dependents);
        let previous_keys = previous_snapshot
            .functions
            .iter()
            .zip(affected_functions.iter())
            .zip(relevant_functions.iter())
            .map(|((previous, is_affected), is_relevant)| {
                if *is_relevant && !*is_affected {
                    previous.key.clone()
                } else {
                    None
                }
            })
            .collect();

        Self {
            current_snapshot: Some(current_snapshot),
            affected_functions,
            previous_keys,
        }
    }

    fn conservative(function_count: usize) -> Self {
        Self {
            current_snapshot: None,
            affected_functions: vec![true; function_count],
            previous_keys: vec![None; function_count],
        }
    }

    fn all_affected(current_snapshot: ResourceSummaryReplaySnapshot) -> Self {
        let function_count = current_snapshot.functions.len();
        Self {
            current_snapshot: Some(current_snapshot),
            affected_functions: vec![true; function_count],
            previous_keys: vec![None; function_count],
        }
    }

    pub(super) fn previous_key(
        &self,
        function_index: usize,
    ) -> Option<ResourceSummaryValueCacheKey> {
        if self
            .affected_functions
            .get(function_index)
            .copied()
            .unwrap_or(true)
        {
            return None;
        }
        self.previous_keys.get(function_index).cloned().flatten()
    }

    pub(super) fn record_key(&mut self, function_index: usize, key: ResourceSummaryValueCacheKey) {
        let Some(snapshot) = self.current_snapshot.as_mut() else {
            return;
        };
        let Some(entry) = snapshot.functions.get_mut(function_index) else {
            return;
        };
        entry.key = Some(key);
    }

    fn into_snapshot(self) -> Option<ResourceSummaryReplaySnapshot> {
        self.current_snapshot
    }
}

impl ResourceSummaryReplaySnapshot {
    fn from_module(
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        module: &ResourceModule,
    ) -> Option<Self> {
        let mut functions = Vec::new();
        for function in &module.functions {
            functions.push(ResourceSummaryReplaySnapshotEntry {
                fingerprint: ResourceFunctionLocalFingerprint::from_function(
                    context, types, function,
                )?,
                key: None,
            });
        }
        Some(Self {
            namespace_hash: context.namespace_stable_hash(),
            functions,
        })
    }
}

fn snapshots_keep_function_order(
    previous: &ResourceSummaryReplaySnapshot,
    current: &ResourceSummaryReplaySnapshot,
) -> bool {
    previous
        .functions
        .iter()
        .zip(current.functions.iter())
        .all(|(previous, current)| previous.fingerprint.same_identity(&current.fingerprint))
}

fn affected_closure_from_local_changes(
    locally_changed: &[bool],
    dependents: &[Vec<usize>],
) -> Vec<bool> {
    let mut affected = locally_changed.to_vec();
    let mut pending = locally_changed
        .iter()
        .enumerate()
        .filter_map(|(index, changed)| changed.then_some(index))
        .collect::<VecDeque<_>>();
    while let Some(function_index) = pending.pop_front() {
        for dependent in dependents
            .get(function_index)
            .map(Vec::as_slice)
            .unwrap_or(&[])
        {
            if let Some(is_affected) = affected.get_mut(*dependent) {
                if !*is_affected {
                    *is_affected = true;
                    pending.push_back(*dependent);
                }
            }
        }
    }
    affected
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::ast::Effect;
    use crate::span::{FileId, Span};
    use crate::types::TypeCtx;

    use super::super::super::model::{
        EffectOp, Place, PlaceRoot, ResourceBlock, ResourceBlockId, ResourceCallTarget,
        ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
    };
    use super::super::super::summary_dependency::ResourceSummaryDependencyGraph;
    use super::super::key::{ResourceSummaryFunctionIdentity, ResourceSummaryValueCacheKey};
    use super::super::ResourceSummaryValueCacheContext;
    use super::*;

    fn test_context(policy_hash: u64) -> ResourceSummaryValueCacheContext {
        let mut context = ResourceSummaryValueCacheContext::new(7);
        context.insert_source_policy_hash(FileId(0), policy_hash);
        context
    }

    #[test]
    fn summary_replay_plan_reuses_unchanged_relevant_keys() {
        let types = TypeCtx::new();
        let module = module_with_functions(vec![
            function_with_ops(&types, "caller", vec![call(&types, "callee")], 1),
            function_with_ops(&types, "callee", Vec::new(), 2),
        ]);
        let graph = ResourceSummaryDependencyGraph::build(&module);
        let context = test_context(11);
        let relevant = vec![true, true];
        let mut first = ResourceSummaryReplayPlan::new(
            None,
            &context,
            &types,
            &module,
            graph.dependents(),
            &relevant,
        );
        let caller_key = summary_key("caller");
        let callee_key = summary_key("callee");
        first.record_key(0, caller_key.clone());
        first.record_key(1, callee_key.clone());
        let snapshot = first.into_snapshot();

        let second = ResourceSummaryReplayPlan::new(
            snapshot.as_ref(),
            &context,
            &types,
            &module,
            graph.dependents(),
            &relevant,
        );

        assert_eq!(second.affected_functions, vec![false, false]);
        assert_eq!(second.previous_key(0), Some(caller_key));
        assert_eq!(second.previous_key(1), Some(callee_key));
    }

    #[test]
    fn summary_replay_plan_marks_changed_callee_and_reverse_dependents() {
        let types = TypeCtx::new();
        let original = module_with_functions(vec![
            function_with_ops(&types, "caller", vec![call(&types, "callee")], 1),
            function_with_ops(&types, "callee", Vec::new(), 2),
        ]);
        let edited = module_with_functions(vec![
            function_with_ops(&types, "caller", vec![call(&types, "callee")], 1),
            function_with_ops(&types, "callee", Vec::new(), 3),
        ]);
        let original_graph = ResourceSummaryDependencyGraph::build(&original);
        let edited_graph = ResourceSummaryDependencyGraph::build(&edited);
        let context = test_context(11);
        let relevant = vec![true, true];
        let mut first = ResourceSummaryReplayPlan::new(
            None,
            &context,
            &types,
            &original,
            original_graph.dependents(),
            &relevant,
        );
        first.record_key(0, summary_key("caller"));
        first.record_key(1, summary_key("callee"));
        let snapshot = first.into_snapshot();

        let second = ResourceSummaryReplayPlan::new(
            snapshot.as_ref(),
            &context,
            &types,
            &edited,
            edited_graph.dependents(),
            &relevant,
        );

        assert_eq!(second.affected_functions, vec![true, true]);
        assert!(second.previous_key(0).is_none());
        assert!(second.previous_key(1).is_none());
    }

    #[test]
    fn summary_replay_plan_keeps_irrelevant_functions_unkeyed() {
        let types = TypeCtx::new();
        let module = module_with_functions(vec![
            function_with_ops(&types, "used", Vec::new(), 1),
            function_with_ops(&types, "unused", Vec::new(), 2),
        ]);
        let graph = ResourceSummaryDependencyGraph::build(&module);
        let context = test_context(11);
        let mut first = ResourceSummaryReplayPlan::new(
            None,
            &context,
            &types,
            &module,
            graph.dependents(),
            &[true, true],
        );
        first.record_key(0, summary_key("used"));
        first.record_key(1, summary_key("unused"));
        let snapshot = first.into_snapshot();

        let second = ResourceSummaryReplayPlan::new(
            snapshot.as_ref(),
            &context,
            &types,
            &module,
            graph.dependents(),
            &[true, false],
        );

        assert_eq!(second.affected_functions, vec![false, false]);
        assert!(second.previous_key(0).is_some());
        assert!(second.previous_key(1).is_none());
    }

    #[test]
    fn summary_replay_plan_rejects_previous_namespace() {
        let types = TypeCtx::new();
        let module =
            module_with_functions(vec![function_with_ops(&types, "stable", Vec::new(), 1)]);
        let graph = ResourceSummaryDependencyGraph::build(&module);
        let first_context = test_context(11);
        let mut second_context = ResourceSummaryValueCacheContext::new(8);
        second_context.insert_source_policy_hash(FileId(0), 11);
        let relevant = vec![true];
        let mut first = ResourceSummaryReplayPlan::new(
            None,
            &first_context,
            &types,
            &module,
            graph.dependents(),
            &relevant,
        );
        first.record_key(0, summary_key("stable"));
        let snapshot = first.into_snapshot();

        let second = ResourceSummaryReplayPlan::new(
            snapshot.as_ref(),
            &second_context,
            &types,
            &module,
            graph.dependents(),
            &relevant,
        );

        assert_eq!(second.affected_functions, vec![true]);
        assert!(second.previous_key(0).is_none());
    }

    #[test]
    fn summary_replay_plan_tracks_source_capability_policy_changes() {
        let types = TypeCtx::new();
        let module =
            module_with_functions(vec![function_with_ops(&types, "stable", Vec::new(), 1)]);
        let graph = ResourceSummaryDependencyGraph::build(&module);
        let first_context = test_context(11);
        let second_context = test_context(12);
        let relevant = vec![true];
        let mut first = ResourceSummaryReplayPlan::new(
            None,
            &first_context,
            &types,
            &module,
            graph.dependents(),
            &relevant,
        );
        first.record_key(0, summary_key("stable"));
        let snapshot = first.into_snapshot();

        let second = ResourceSummaryReplayPlan::new(
            snapshot.as_ref(),
            &second_context,
            &types,
            &module,
            graph.dependents(),
            &relevant,
        );

        assert_eq!(second.affected_functions, vec![true]);
        assert!(second.previous_key(0).is_none());
    }

    fn summary_key(name: &str) -> ResourceSummaryValueCacheKey {
        ResourceSummaryValueCacheKey::new_raw_alias_return_entry(
            7,
            ResourceSummaryFunctionIdentity::new(name, name)
                .expect("test function identity should be valid"),
            11,
            13,
            17,
            19,
            23,
        )
    }

    fn module_with_functions(functions: Vec<ResourceFunction>) -> ResourceModule {
        ResourceModule {
            functions,
            entry: None,
            string_literals: Vec::new(),
        }
    }

    fn function_with_ops(
        types: &TypeCtx,
        name: &str,
        mut ops: Vec<ResourceOp>,
        literal: i32,
    ) -> ResourceFunction {
        let value = Place::temporary(crate::resource::model::ResourceId(0), types.i32());
        ops.push(ResourceOp::Expr {
            kind: crate::resource::model::ResourceExprKind::LiteralI32(literal),
            output: value.clone(),
            ty: types.i32(),
            span: span(),
        });
        ResourceFunction {
            name: name.to_string(),
            origin_name: name.to_string(),
            type_params: Vec::new(),
            params: Vec::new(),
            result: types.i32(),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops,
                terminator: ResourceTerminator::Return {
                    value: Some(value),
                    span: span(),
                },
                span: span(),
            }],
            span: span(),
        }
    }

    fn call(types: &TypeCtx, name: &str) -> ResourceOp {
        ResourceOp::Call {
            output: Place {
                root: PlaceRoot::Local(alloc::format!("{name}_out")),
                projections: Vec::new(),
                ty: types.i32(),
            },
            target: ResourceCallTarget::User {
                name: name.to_string(),
                type_args: Vec::new(),
            },
            args: Vec::new(),
            effect: EffectOp::Pure,
            span: span(),
        }
    }

    fn span() -> Span {
        Span::new(FileId(0), 1, 2)
    }
}
