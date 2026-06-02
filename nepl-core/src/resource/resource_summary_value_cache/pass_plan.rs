extern crate alloc;

use alloc::collections::VecDeque;
use alloc::vec;
use alloc::vec::Vec;

use super::super::model::{ResourceFunction, ResourceModule};
use super::super::report::{
    ResourceCheckDeferred, ResourceFunctionCheck, ResourceOwnerCheckDeferred,
    ResourceOwnerFunctionCheck,
};
use super::super::summary_dependency::ResourceSummaryDependencyGraph;
use super::context::ResourceSummaryValueCacheContext;
use super::function_fingerprint::ResourceFunctionLocalFingerprint;
use super::ResourceSummaryValueCache;
use crate::types::TypeCtx;

/// final initialized check の changed-function replay plan。
///
/// この plan は 1 回の Resource static check に閉じた値である。前回 compile で
/// diagnostic-free / auto-drop-free pass として保存できた関数のうち、現在 compile でも
/// 関数本文、型境界、source capability policy、依存先 closure が変わっていないものだけを
/// checker 起動なしで pass として戻す。`TypeId`、`Span`、`SourceMap`、final cell state は
/// snapshot に保存しない。
pub(in crate::resource) struct InitializedFunctionCheckPassPlan {
    current_snapshot: Option<InitializedFunctionCheckPassSnapshot>,
    affected_functions: Vec<bool>,
    previous_passes: Vec<Option<ResourceCheckDeferred>>,
}

/// owner obligation check の changed-function replay plan。
///
/// owner obligation は owner return summary を参照するため、関数単体の body hash だけで
/// 再利用可否を決めない。この plan は initialized check と同じく、local fingerprint の
/// 変化を reverse dependency closure へ広げ、影響を受けない diagnostic-free pass だけを
/// owner state を materialize せずに戻す。
pub(in crate::resource) struct OwnerObligationCheckPassPlan {
    current_snapshot: Option<OwnerObligationCheckPassSnapshot>,
    affected_functions: Vec<bool>,
    previous_passes: Vec<Option<ResourceOwnerCheckDeferred>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct InitializedFunctionCheckPassSnapshot {
    namespace_hash: u64,
    functions: Vec<InitializedFunctionCheckPassSnapshotEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct InitializedFunctionCheckPassSnapshotEntry {
    fingerprint: ResourceFunctionLocalFingerprint,
    pass: Option<ResourceCheckDeferred>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(super) struct OwnerObligationCheckPassSnapshot {
    namespace_hash: u64,
    functions: Vec<OwnerObligationCheckPassSnapshotEntry>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct OwnerObligationCheckPassSnapshotEntry {
    fingerprint: ResourceFunctionLocalFingerprint,
    pass: Option<ResourceOwnerCheckDeferred>,
}

impl ResourceSummaryValueCache {
    /// 前回の initialized pass snapshot があるかを返す。
    ///
    /// stable entry collection が無効な cold compile では、新しい snapshot を保存しない。
    /// その場合に previous snapshot も無ければ pass plan は全関数 affected になるだけなので、
    /// 呼び出し側はこの query で fingerprint 構築を省ける。
    pub(in crate::resource) fn has_initialized_function_check_pass_snapshot(&self) -> bool {
        self.initialized_function_check_pass_snapshot.is_some()
    }

    pub(in crate::resource) fn begin_initialized_function_check_pass_plan(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        module: &ResourceModule,
        dependency_graph: &ResourceSummaryDependencyGraph,
    ) -> InitializedFunctionCheckPassPlan {
        InitializedFunctionCheckPassPlan::new(
            self.initialized_function_check_pass_snapshot.as_ref(),
            context,
            types,
            module,
            dependency_graph,
        )
    }

    pub(in crate::resource) fn replay_unchanged_initialized_function_check_pass(
        &mut self,
        plan: &mut InitializedFunctionCheckPassPlan,
        function_index: usize,
        function: &ResourceFunction,
        op_count: usize,
    ) -> Option<ResourceFunctionCheck> {
        let deferred = plan.previous_pass(function_index)?;
        self.record_initialized_function_check_plan_skip(op_count);
        let check = ResourceFunctionCheck {
            name: function.name.clone(),
            final_cells: Vec::new(),
            final_collection_slots: Vec::new(),
            auto_drop_points: Vec::new(),
            deferred,
        };
        plan.record_pass(function_index, deferred);
        Some(check)
    }

    pub(in crate::resource) fn finish_initialized_function_check_pass_plan(
        &mut self,
        plan: InitializedFunctionCheckPassPlan,
    ) {
        self.initialized_function_check_pass_snapshot = plan.into_snapshot();
    }

    fn record_initialized_function_check_plan_skip(&mut self, op_count: usize) {
        self.stats.resource_summary_value_replay_hits += op_count;
        self.stats.resource_summary_value_lazy_pass_hits += 1;
        self.stats.resource_summary_value_lazy_pass_ops += op_count;
        self.stats
            .resource_summary_value_initialized_function_check_hits += 1;
        self.stats
            .resource_summary_value_initialized_function_check_plan_skip_functions += 1;
        self.stats
            .resource_summary_value_initialized_function_check_plan_skip_ops += op_count;
    }

    pub(in crate::resource) fn begin_owner_obligation_check_pass_plan(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        module: &ResourceModule,
        dependency_graph: &ResourceSummaryDependencyGraph,
    ) -> OwnerObligationCheckPassPlan {
        OwnerObligationCheckPassPlan::new(
            self.owner_obligation_check_pass_snapshot.as_ref(),
            context,
            types,
            module,
            dependency_graph,
        )
    }

    /// 前回の owner obligation pass snapshot があるかを返す。
    ///
    /// owner obligation は owner return summary の変化を reverse dependency closure へ
    /// 広げるため、snapshot が無い cold compile では plan skip が発生しない。stable entry
    /// collection も無効なら、plan 自体を作らない方が同じ検査結果を少ない work で得られる。
    pub(in crate::resource) fn has_owner_obligation_check_pass_snapshot(&self) -> bool {
        self.owner_obligation_check_pass_snapshot.is_some()
    }

    pub(in crate::resource) fn replay_unchanged_owner_obligation_check_pass(
        &mut self,
        plan: &mut OwnerObligationCheckPassPlan,
        function_index: usize,
        function: &ResourceFunction,
        op_count: usize,
    ) -> Option<ResourceOwnerFunctionCheck> {
        let deferred = plan.previous_pass(function_index)?;
        self.record_owner_obligation_check_plan_skip(op_count);
        let check = ResourceOwnerFunctionCheck {
            name: function.name.clone(),
            final_owners: Vec::new(),
            deferred,
        };
        plan.record_pass(function_index, deferred);
        Some(check)
    }

    pub(in crate::resource) fn finish_owner_obligation_check_pass_plan(
        &mut self,
        plan: OwnerObligationCheckPassPlan,
    ) {
        self.owner_obligation_check_pass_snapshot = plan.into_snapshot();
    }

    fn record_owner_obligation_check_plan_skip(&mut self, op_count: usize) {
        self.stats.resource_summary_value_replay_hits += op_count;
        self.stats.resource_summary_value_lazy_pass_hits += 1;
        self.stats.resource_summary_value_lazy_pass_ops += op_count;
        self.stats
            .resource_summary_value_owner_obligation_check_hits += 1;
        self.stats
            .resource_summary_value_owner_obligation_check_plan_skip_functions += 1;
        self.stats
            .resource_summary_value_owner_obligation_check_plan_skip_ops += op_count;
    }
}

impl InitializedFunctionCheckPassPlan {
    fn new(
        previous_snapshot: Option<&InitializedFunctionCheckPassSnapshot>,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        module: &ResourceModule,
        dependency_graph: &ResourceSummaryDependencyGraph,
    ) -> Self {
        let Some(current_snapshot) =
            InitializedFunctionCheckPassSnapshot::from_module(context, types, module)
        else {
            return Self::conservative(module.functions.len());
        };
        let Some(previous_snapshot) = previous_snapshot else {
            return Self::all_affected(current_snapshot);
        };
        if previous_snapshot.namespace_hash != current_snapshot.namespace_hash
            || previous_snapshot.functions.len() != current_snapshot.functions.len()
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
        let affected_functions =
            affected_closure_from_local_changes(&locally_changed, dependency_graph);
        let previous_passes = previous_snapshot
            .functions
            .iter()
            .zip(affected_functions.iter())
            .map(
                |(previous, is_affected)| {
                    if *is_affected {
                        None
                    } else {
                        previous.pass
                    }
                },
            )
            .collect();

        Self {
            current_snapshot: Some(current_snapshot),
            affected_functions,
            previous_passes,
        }
    }

    fn conservative(function_count: usize) -> Self {
        Self {
            current_snapshot: None,
            affected_functions: vec![true; function_count],
            previous_passes: vec![None; function_count],
        }
    }

    fn all_affected(current_snapshot: InitializedFunctionCheckPassSnapshot) -> Self {
        let function_count = current_snapshot.functions.len();
        Self {
            current_snapshot: Some(current_snapshot),
            affected_functions: vec![true; function_count],
            previous_passes: vec![None; function_count],
        }
    }

    fn previous_pass(&self, function_index: usize) -> Option<ResourceCheckDeferred> {
        if self
            .affected_functions
            .get(function_index)
            .copied()
            .unwrap_or(true)
        {
            return None;
        }
        self.previous_passes.get(function_index).copied().flatten()
    }

    pub(in crate::resource) fn record_pass(
        &mut self,
        function_index: usize,
        deferred: ResourceCheckDeferred,
    ) {
        let Some(snapshot) = self.current_snapshot.as_mut() else {
            return;
        };
        let Some(entry) = snapshot.functions.get_mut(function_index) else {
            return;
        };
        entry.pass = Some(deferred);
    }

    fn into_snapshot(self) -> Option<InitializedFunctionCheckPassSnapshot> {
        self.current_snapshot
    }
}

impl InitializedFunctionCheckPassSnapshot {
    fn from_module(
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        module: &ResourceModule,
    ) -> Option<Self> {
        let mut functions = Vec::new();
        for function in &module.functions {
            functions.push(InitializedFunctionCheckPassSnapshotEntry {
                fingerprint: ResourceFunctionLocalFingerprint::from_function(
                    context, types, function,
                )?,
                pass: None,
            });
        }
        Some(Self {
            namespace_hash: context.namespace_stable_hash(),
            functions,
        })
    }
}

impl OwnerObligationCheckPassPlan {
    fn new(
        previous_snapshot: Option<&OwnerObligationCheckPassSnapshot>,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        module: &ResourceModule,
        dependency_graph: &ResourceSummaryDependencyGraph,
    ) -> Self {
        let Some(current_snapshot) =
            OwnerObligationCheckPassSnapshot::from_module(context, types, module)
        else {
            return Self::conservative(module.functions.len());
        };
        let Some(previous_snapshot) = previous_snapshot else {
            return Self::all_affected(current_snapshot);
        };
        if previous_snapshot.namespace_hash != current_snapshot.namespace_hash
            || previous_snapshot.functions.len() != current_snapshot.functions.len()
            || !owner_snapshots_keep_function_order(previous_snapshot, &current_snapshot)
        {
            return Self::all_affected(current_snapshot);
        }

        let locally_changed = previous_snapshot
            .functions
            .iter()
            .zip(current_snapshot.functions.iter())
            .map(|(previous, current)| previous.fingerprint != current.fingerprint)
            .collect::<Vec<_>>();
        let affected_functions =
            affected_closure_from_local_changes(&locally_changed, dependency_graph);
        let previous_passes = previous_snapshot
            .functions
            .iter()
            .zip(affected_functions.iter())
            .map(
                |(previous, is_affected)| {
                    if *is_affected {
                        None
                    } else {
                        previous.pass
                    }
                },
            )
            .collect();

        Self {
            current_snapshot: Some(current_snapshot),
            affected_functions,
            previous_passes,
        }
    }

    fn conservative(function_count: usize) -> Self {
        Self {
            current_snapshot: None,
            affected_functions: vec![true; function_count],
            previous_passes: vec![None; function_count],
        }
    }

    fn all_affected(current_snapshot: OwnerObligationCheckPassSnapshot) -> Self {
        let function_count = current_snapshot.functions.len();
        Self {
            current_snapshot: Some(current_snapshot),
            affected_functions: vec![true; function_count],
            previous_passes: vec![None; function_count],
        }
    }

    fn previous_pass(&self, function_index: usize) -> Option<ResourceOwnerCheckDeferred> {
        if self
            .affected_functions
            .get(function_index)
            .copied()
            .unwrap_or(true)
        {
            return None;
        }
        self.previous_passes.get(function_index).copied().flatten()
    }

    pub(in crate::resource) fn record_pass(
        &mut self,
        function_index: usize,
        deferred: ResourceOwnerCheckDeferred,
    ) {
        let Some(snapshot) = self.current_snapshot.as_mut() else {
            return;
        };
        let Some(entry) = snapshot.functions.get_mut(function_index) else {
            return;
        };
        entry.pass = Some(deferred);
    }

    fn into_snapshot(self) -> Option<OwnerObligationCheckPassSnapshot> {
        self.current_snapshot
    }
}

impl OwnerObligationCheckPassSnapshot {
    fn from_module(
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        module: &ResourceModule,
    ) -> Option<Self> {
        let mut functions = Vec::new();
        for function in &module.functions {
            functions.push(OwnerObligationCheckPassSnapshotEntry {
                fingerprint: ResourceFunctionLocalFingerprint::from_function(
                    context, types, function,
                )?,
                pass: None,
            });
        }
        Some(Self {
            namespace_hash: context.namespace_stable_hash(),
            functions,
        })
    }
}

fn snapshots_keep_function_order(
    previous: &InitializedFunctionCheckPassSnapshot,
    current: &InitializedFunctionCheckPassSnapshot,
) -> bool {
    previous
        .functions
        .iter()
        .zip(current.functions.iter())
        .all(|(previous, current)| previous.fingerprint.same_identity(&current.fingerprint))
}

fn owner_snapshots_keep_function_order(
    previous: &OwnerObligationCheckPassSnapshot,
    current: &OwnerObligationCheckPassSnapshot,
) -> bool {
    previous
        .functions
        .iter()
        .zip(current.functions.iter())
        .all(|(previous, current)| previous.fingerprint.same_identity(&current.fingerprint))
}

fn affected_closure_from_local_changes(
    locally_changed: &[bool],
    dependency_graph: &ResourceSummaryDependencyGraph,
) -> Vec<bool> {
    let mut affected = locally_changed.to_vec();
    let mut pending = locally_changed
        .iter()
        .enumerate()
        .filter_map(|(index, changed)| changed.then_some(index))
        .collect::<VecDeque<_>>();
    while let Some(function_index) = pending.pop_front() {
        for dependent in dependency_graph
            .dependents()
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
    use super::*;

    fn test_context(policy_hash: u64) -> ResourceSummaryValueCacheContext {
        let mut context = ResourceSummaryValueCacheContext::new(7);
        context.insert_source_policy_hash(FileId(0), policy_hash);
        context
    }

    #[test]
    fn initialized_pass_plan_reuses_unchanged_passes_without_marking_dependents() {
        let types = TypeCtx::new();
        let module = module_with_functions(vec![
            function_with_ops(&types, "caller", vec![call(&types, "callee")], 1),
            function_with_ops(&types, "callee", Vec::new(), 2),
        ]);
        let graph = ResourceSummaryDependencyGraph::build(&module);
        let context = test_context(11);
        let mut first =
            InitializedFunctionCheckPassPlan::new(None, &context, &types, &module, &graph);
        first.record_pass(0, ResourceCheckDeferred::default());
        first.record_pass(1, ResourceCheckDeferred::default());
        let snapshot = first.into_snapshot();

        let second = InitializedFunctionCheckPassPlan::new(
            snapshot.as_ref(),
            &context,
            &types,
            &module,
            &graph,
        );

        assert_eq!(second.affected_functions, vec![false, false]);
        assert!(second.previous_pass(0).is_some());
        assert!(second.previous_pass(1).is_some());
    }

    #[test]
    fn initialized_pass_plan_marks_changed_callee_and_reverse_dependents() {
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
        let mut first = InitializedFunctionCheckPassPlan::new(
            None,
            &context,
            &types,
            &original,
            &original_graph,
        );
        first.record_pass(0, ResourceCheckDeferred::default());
        first.record_pass(1, ResourceCheckDeferred::default());
        let snapshot = first.into_snapshot();

        let second = InitializedFunctionCheckPassPlan::new(
            snapshot.as_ref(),
            &context,
            &types,
            &edited,
            &edited_graph,
        );

        assert_eq!(second.affected_functions, vec![true, true]);
        assert!(second.previous_pass(0).is_none());
        assert!(second.previous_pass(1).is_none());
    }

    #[test]
    fn initialized_pass_plan_rejects_previous_namespace() {
        let types = TypeCtx::new();
        let module =
            module_with_functions(vec![function_with_ops(&types, "stable", Vec::new(), 1)]);
        let graph = ResourceSummaryDependencyGraph::build(&module);
        let first_context = test_context(11);
        let mut second_context = ResourceSummaryValueCacheContext::new(8);
        second_context.insert_source_policy_hash(FileId(0), 11);
        let mut first =
            InitializedFunctionCheckPassPlan::new(None, &first_context, &types, &module, &graph);
        first.record_pass(0, ResourceCheckDeferred::default());
        let snapshot = first.into_snapshot();

        let second = InitializedFunctionCheckPassPlan::new(
            snapshot.as_ref(),
            &second_context,
            &types,
            &module,
            &graph,
        );

        assert_eq!(second.affected_functions, vec![true]);
        assert!(second.previous_pass(0).is_none());
    }

    #[test]
    fn initialized_pass_plan_rejects_function_order_changes() {
        let types = TypeCtx::new();
        let original = module_with_functions(vec![
            function_with_ops(&types, "first", Vec::new(), 1),
            function_with_ops(&types, "second", Vec::new(), 2),
        ]);
        let reordered = module_with_functions(vec![
            function_with_ops(&types, "second", Vec::new(), 2),
            function_with_ops(&types, "first", Vec::new(), 1),
        ]);
        let original_graph = ResourceSummaryDependencyGraph::build(&original);
        let reordered_graph = ResourceSummaryDependencyGraph::build(&reordered);
        let context = test_context(11);
        let mut first = InitializedFunctionCheckPassPlan::new(
            None,
            &context,
            &types,
            &original,
            &original_graph,
        );
        first.record_pass(0, ResourceCheckDeferred::default());
        first.record_pass(1, ResourceCheckDeferred::default());
        let snapshot = first.into_snapshot();

        let second = InitializedFunctionCheckPassPlan::new(
            snapshot.as_ref(),
            &context,
            &types,
            &reordered,
            &reordered_graph,
        );

        assert_eq!(second.affected_functions, vec![true, true]);
        assert!(second.previous_pass(0).is_none());
        assert!(second.previous_pass(1).is_none());
    }

    #[test]
    fn initialized_pass_plan_tracks_source_capability_policy_changes() {
        let types = TypeCtx::new();
        let module =
            module_with_functions(vec![function_with_ops(&types, "stable", Vec::new(), 1)]);
        let graph = ResourceSummaryDependencyGraph::build(&module);
        let first_context = test_context(11);
        let second_context = test_context(12);
        let mut first =
            InitializedFunctionCheckPassPlan::new(None, &first_context, &types, &module, &graph);
        first.record_pass(0, ResourceCheckDeferred::default());
        let snapshot = first.into_snapshot();

        let second = InitializedFunctionCheckPassPlan::new(
            snapshot.as_ref(),
            &second_context,
            &types,
            &module,
            &graph,
        );

        assert_eq!(second.affected_functions, vec![true]);
        assert!(second.previous_pass(0).is_none());
    }

    #[test]
    fn owner_pass_plan_marks_changed_callee_and_reverse_dependents() {
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
        let mut first =
            OwnerObligationCheckPassPlan::new(None, &context, &types, &original, &original_graph);
        first.record_pass(0, ResourceOwnerCheckDeferred::default());
        first.record_pass(1, ResourceOwnerCheckDeferred::default());
        let snapshot = first.into_snapshot();

        let second = OwnerObligationCheckPassPlan::new(
            snapshot.as_ref(),
            &context,
            &types,
            &edited,
            &edited_graph,
        );

        assert_eq!(second.affected_functions, vec![true, true]);
        assert!(second.previous_pass(0).is_none());
        assert!(second.previous_pass(1).is_none());
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
