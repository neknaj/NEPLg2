extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryOp;
use super::model::ResourceFunction;

mod body_hash;
mod candidate_key;
mod context;
mod dependency_hash;
mod i32_scalar;
mod initialized_check;
mod key;
mod raw_alias;
mod raw_init;
mod stable_hash;
mod stable_mirror;
mod stable_type_key;
mod type_boundary;

pub use self::context::ResourceSummaryValueCacheContext;
pub(super) use self::dependency_hash::{
    i32_scalar_dependency_closure_hash, initialized_function_check_dependency_closure_hash,
    raw_alias_dependency_closure_hash, raw_init_dependency_closure_hash,
};

pub(in crate::resource) use self::candidate_key::ResourceSummaryDependencyClosureHash;
use self::candidate_key::{
    drop_traversal_forall_leaf_entry_candidate_key_and_entry, drop_traversal_forall_leaf_entry_key,
    ResourceSummaryGenericTypeArgumentKeyInput,
};
use self::key::ResourceSummaryValueCacheKey;
use self::stable_mirror::{
    reproject_drop_traversal_forall_leaf_entry, ResourceSummaryStableDropTraversalForallLeafEntry,
    ResourceSummaryStableI32ScalarReturnFactsEntry,
    ResourceSummaryStableInitializedFunctionCheckEntry, ResourceSummaryStableRawAliasReturnEntry,
    ResourceSummaryStableRawInitCompleteLeafEntry, ResourceSummaryTypeReprojection,
};

/// Resource IR summary value cache の累積統計。
///
/// この統計は compiled-output cache とは別に、Resource IR の証明結果を stable value
/// として保存・再投影できるかを観測するために使う。`resource_summary_value_hits` は
/// 既存 stable value が再投影可能だった候補 hit であり、fixed-point worklist の skip
/// までは意味しない。実際に summary op を replay して compile work を減らす段階では、
/// `resource_summary_value_replay_*` を別 counter として増やす。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceSummaryValueCacheStats {
    pub resource_static_function_count: usize,
    pub resource_static_op_count: usize,
    pub resource_raw_alias_summary_recomputations: usize,
    pub resource_raw_alias_summary_count: usize,
    pub resource_i32_scalar_summary_recomputations: usize,
    pub resource_i32_scalar_summary_count: usize,
    pub resource_raw_init_summary_recomputations: usize,
    pub resource_raw_init_summary_count: usize,
    pub resource_collection_slot_summary_recomputations: usize,
    pub resource_collection_slot_summary_count: usize,
    pub resource_initialized_function_checks: usize,
    pub resource_initialized_function_check_ops: usize,
    pub resource_summary_value_hits: usize,
    pub resource_summary_value_misses: usize,
    pub resource_summary_value_stores: usize,
    pub resource_summary_value_bypasses: usize,
    pub resource_summary_value_replay_hits: usize,
    pub resource_summary_value_replay_bypasses: usize,
    pub resource_summary_value_replayed_ops: usize,
    pub resource_summary_value_lazy_pass_hits: usize,
    pub resource_summary_value_lazy_pass_ops: usize,
    pub resource_summary_value_recomputed_ops: usize,
    pub resource_summary_value_drop_traversal_forall_recomputed_ops: usize,
    pub resource_summary_value_raw_alias_return_entry_recomputed_ops: usize,
    pub resource_summary_value_i32_scalar_return_facts_recomputed_ops: usize,
    pub resource_summary_value_raw_init_param_facts_recomputed_ops: usize,
    pub resource_summary_value_drop_traversal_forall_hits: usize,
    pub resource_summary_value_drop_traversal_forall_stores: usize,
    pub resource_summary_value_drop_traversal_forall_bypasses: usize,
    pub resource_summary_value_drop_traversal_forall_replay_probe_functions: usize,
    pub resource_summary_value_drop_traversal_forall_replay_hit_functions: usize,
    pub resource_summary_value_drop_traversal_forall_replay_miss_functions: usize,
    pub resource_summary_value_raw_alias_return_entry_hits: usize,
    pub resource_summary_value_raw_alias_return_entry_stores: usize,
    pub resource_summary_value_raw_alias_return_entry_bypasses: usize,
    pub resource_summary_value_raw_alias_return_entry_replay_probe_functions: usize,
    pub resource_summary_value_raw_alias_return_entry_replay_hit_functions: usize,
    pub resource_summary_value_raw_alias_return_entry_replay_miss_functions: usize,
    pub resource_summary_value_raw_alias_return_entry_dependency_bypasses: usize,
    pub resource_summary_value_raw_alias_return_entry_missing_source_policy_bypasses: usize,
    pub resource_summary_value_raw_alias_return_entry_unstable_key_bypasses: usize,
    pub resource_summary_value_raw_alias_return_entry_unstable_entry_bypasses: usize,
    pub resource_summary_value_raw_alias_return_entry_reprojection_bypasses: usize,
    pub resource_summary_value_raw_alias_return_entry_reprojection_context_bypasses: usize,
    pub resource_summary_value_raw_alias_return_entry_reprojection_value_bypasses: usize,
    pub resource_summary_value_raw_alias_return_entry_reprojection_value_parameter_index_bypasses:
        usize,
    pub resource_summary_value_raw_alias_return_entry_reprojection_value_parameter_projection_bypasses:
        usize,
    pub resource_summary_value_raw_alias_return_entry_reprojection_value_parameter_type_bypasses:
        usize,
    pub resource_summary_value_raw_alias_return_entry_reprojection_value_return_projection_bypasses:
        usize,
    pub resource_summary_value_raw_alias_return_entry_reprojection_value_return_type_bypasses:
        usize,
    pub resource_summary_value_i32_scalar_return_facts_hits: usize,
    pub resource_summary_value_i32_scalar_return_facts_stores: usize,
    pub resource_summary_value_i32_scalar_return_facts_misses: usize,
    pub resource_summary_value_i32_scalar_return_facts_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_replay_probe_functions: usize,
    pub resource_summary_value_i32_scalar_return_facts_replay_hit_functions: usize,
    pub resource_summary_value_i32_scalar_return_facts_replay_miss_functions: usize,
    pub resource_summary_value_i32_scalar_return_facts_dependency_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_missing_source_policy_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_unstable_key_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_unstable_entry_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_unstable_entry_return_projection_bypasses:
        usize,
    pub resource_summary_value_i32_scalar_return_facts_unstable_entry_parameter_projection_bypasses:
        usize,
    pub resource_summary_value_i32_scalar_return_facts_unstable_entry_scalar_type_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_reprojection_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_reprojection_context_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_reprojection_value_return_projection_bypasses:
        usize,
    pub resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_projection_bypasses:
        usize,
    pub resource_summary_value_i32_scalar_return_facts_reprojection_value_scalar_type_bypasses:
        usize,
    pub resource_summary_value_i32_scalar_return_facts_reprojection_value_alias_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_reprojection_value_offset_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_reprojection_value_relation_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_reprojection_value_constant_bypasses: usize,
    pub resource_summary_value_i32_scalar_return_facts_reprojection_value_return_condition_bypasses:
        usize,
    pub resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_condition_bypasses:
        usize,
    pub resource_summary_value_i32_scalar_return_facts_replay_missing_source_policy_functions:
        usize,
    pub resource_summary_value_i32_scalar_return_facts_replay_unstable_key_functions: usize,
    pub resource_summary_value_i32_scalar_return_facts_replay_entry_miss_functions: usize,
    pub resource_summary_value_i32_scalar_return_facts_replay_reprojection_context_functions: usize,
    pub resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_functions: usize,
    pub resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_return_projection_functions:
        usize,
    pub resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_parameter_projection_functions:
        usize,
    pub resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_scalar_type_functions:
        usize,
    pub resource_summary_value_initialized_function_check_hits: usize,
    pub resource_summary_value_initialized_function_check_stores: usize,
    pub resource_summary_value_initialized_function_check_bypasses: usize,
    pub resource_summary_value_initialized_function_check_replay_probe_functions: usize,
    pub resource_summary_value_initialized_function_check_replay_hit_functions: usize,
    pub resource_summary_value_initialized_function_check_replay_miss_functions: usize,
    pub resource_summary_value_initialized_function_check_dependency_bypasses: usize,
    pub resource_summary_value_initialized_function_check_diagnostic_bypasses: usize,
    pub resource_summary_value_initialized_function_check_missing_source_policy_bypasses: usize,
    pub resource_summary_value_initialized_function_check_unstable_key_bypasses: usize,
    pub resource_summary_value_initialized_function_check_unstable_entry_bypasses: usize,
    pub resource_summary_value_initialized_function_check_unstable_entry_auto_drop_bypasses: usize,
    pub resource_summary_value_initialized_function_check_unstable_entry_place_bypasses: usize,
    pub resource_summary_value_initialized_function_check_unstable_entry_type_bypasses: usize,
    pub resource_summary_value_initialized_function_check_reprojection_bypasses: usize,
    pub resource_summary_value_initialized_function_check_reprojection_context_bypasses: usize,
    pub resource_summary_value_initialized_function_check_reprojection_value_bypasses: usize,
    pub resource_summary_value_initialized_function_check_reprojection_value_place_bypasses: usize,
    pub resource_summary_value_initialized_function_check_reprojection_value_type_bypasses: usize,
    pub resource_summary_value_initialized_function_check_reprojection_value_place_type_bypasses:
        usize,
    pub resource_summary_value_initialized_function_check_reprojection_value_projection_result_type_bypasses:
        usize,
    pub resource_summary_value_initialized_function_check_reprojection_value_cell_state_type_bypasses:
        usize,
    pub resource_summary_value_initialized_function_check_reprojection_value_collection_slot_state_type_bypasses:
        usize,
    pub resource_summary_value_raw_init_param_facts_hits: usize,
    pub resource_summary_value_raw_init_param_facts_stores: usize,
    pub resource_summary_value_raw_init_param_facts_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_replay_probe_functions: usize,
    pub resource_summary_value_raw_init_param_facts_replay_hit_functions: usize,
    pub resource_summary_value_raw_init_param_facts_replay_miss_functions: usize,
    pub resource_summary_value_raw_init_param_facts_incomplete_leaf_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_dependency_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_missing_source_policy_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_unstable_key_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_dependency_graph_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_dependency_identity_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_dependency_body_hash_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_dependency_source_policy_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_dependency_type_boundary_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_unstable_entry_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_unstable_entry_surface_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_unstable_entry_param_cell_projection_bypasses:
        usize,
    pub resource_summary_value_raw_init_param_facts_unstable_entry_param_cell_type_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_unstable_entry_param_release_type_bypasses:
        usize,
    pub resource_summary_value_raw_init_param_facts_reprojection_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_reprojection_context_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_reprojection_value_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_reprojection_value_empty_entry_bypasses: usize,
    pub resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_projection_bypasses:
        usize,
    pub resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_type_bypasses:
        usize,
    pub resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_stable_type_bypasses:
        usize,
    pub resource_summary_value_raw_init_param_facts_reprojection_value_param_cell_result_type_bypasses:
        usize,
    pub resource_summary_value_raw_init_param_facts_reprojection_value_param_release_projection_bypasses:
        usize,
    pub resource_summary_value_raw_init_param_facts_reprojection_value_param_release_type_bypasses:
        usize,
}

/// initialized-state checker の summary stage を session 統計へ畳むための分類。
///
/// 各 stage の value cache 実装状況は異なるが、same-session code edit の残り時間を
/// root-cause ごとに分けるには、まず「どの固定点計算が何回走ったか」を同じ観測面へ
/// 出す必要がある。ここでは実行量だけを記録し、cache 可否や safety 判定は各 stage の
/// 既存実装に委譲する。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ResourceSummaryComputationStage {
    RawAlias,
    I32Scalar,
    RawInit,
    CollectionSlot,
}

/// `CompilerSession` が所有する Resource IR summary value cache の境界。
///
/// `LoaderSessionCache` は未型付けの loader artifact を扱うため、typed public
/// signature、dependency surface、source capability、generic type argument に
/// 依存する Resource IR proof artifact はこの別 cache に置く。cache owner は統計と
/// 永続化境界だけを受け持ち、`TypeId` を含む summary value から安定 value へ変換する
/// 責務は private submodule の `stable_mirror` に分ける。
#[derive(Debug, Default)]
pub struct ResourceSummaryValueCache {
    stats: ResourceSummaryValueCacheStats,
    drop_traversal_forall_leaf_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableDropTraversalForallLeafEntry>,
    raw_alias_return_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableRawAliasReturnEntry>,
    i32_scalar_return_facts_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableI32ScalarReturnFactsEntry>,
    initialized_function_check_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableInitializedFunctionCheckEntry>,
    raw_init_complete_leaf_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableRawInitCompleteLeafEntry>,
}

#[derive(Debug, Clone)]
pub(super) struct ResourceSummaryDropTraversalForallLeafEntryCandidate {
    key: ResourceSummaryValueCacheKey,
    entry: ResourceSummaryStableDropTraversalForallLeafEntry,
}

#[derive(Debug, Clone)]
pub(super) struct ResourceSummaryRawInitCompleteLeafEntryCandidate {
    key: ResourceSummaryValueCacheKey,
    entry: ResourceSummaryStableRawInitCompleteLeafEntry,
}

#[derive(Debug, Clone)]
pub(super) struct ResourceSummaryRawAliasReturnEntryCandidate {
    key: ResourceSummaryValueCacheKey,
    entry: ResourceSummaryStableRawAliasReturnEntry,
}

#[derive(Debug, Clone)]
pub(super) struct ResourceSummaryI32ScalarReturnFactsEntryCandidate {
    key: ResourceSummaryValueCacheKey,
    entry: ResourceSummaryStableI32ScalarReturnFactsEntry,
}

#[derive(Debug, Clone)]
pub(super) struct ResourceSummaryInitializedFunctionCheckEntryCandidate {
    key: ResourceSummaryValueCacheKey,
    entry: ResourceSummaryStableInitializedFunctionCheckEntry,
    op_count: usize,
}

impl ResourceSummaryValueCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.stats = ResourceSummaryValueCacheStats::default();
        self.drop_traversal_forall_leaf_entries.clear();
        self.raw_alias_return_entries.clear();
        self.i32_scalar_return_facts_entries.clear();
        self.initialized_function_check_entries.clear();
        self.raw_init_complete_leaf_entries.clear();
    }

    pub fn stats(&self) -> ResourceSummaryValueCacheStats {
        self.stats
    }

    /// Resource static check の入力規模を session 統計へ記録する。
    ///
    /// warm edit で summary value の再計算が 0 になっても、全関数を probe し続けると
    /// 秒単位の固定費が残る。function 数と op 数を同じ JSON 境界へ出し、次の
    /// changed-function-only proof replay が「入力を小さくした」のか「replay の中身だけを
    /// 軽くした」のかを分けて評価できるようにする。
    pub(super) fn record_resource_static_input_shape(
        &mut self,
        function_count: usize,
        op_count: usize,
    ) {
        self.stats.resource_static_function_count += function_count;
        self.stats.resource_static_op_count += op_count;
    }

    pub(super) fn record_raw_alias_replay_probe_function(&mut self) {
        self.stats
            .resource_summary_value_raw_alias_return_entry_replay_probe_functions += 1;
    }

    pub(super) fn record_raw_alias_replay_hit_function(&mut self) {
        self.stats
            .resource_summary_value_raw_alias_return_entry_replay_hit_functions += 1;
    }

    pub(super) fn record_raw_alias_replay_miss_function(&mut self) {
        self.stats
            .resource_summary_value_raw_alias_return_entry_replay_miss_functions += 1;
    }

    pub(super) fn record_i32_scalar_replay_probe_function(&mut self) {
        self.stats
            .resource_summary_value_i32_scalar_return_facts_replay_probe_functions += 1;
    }

    pub(super) fn record_i32_scalar_replay_hit_function(&mut self) {
        self.stats
            .resource_summary_value_i32_scalar_return_facts_replay_hit_functions += 1;
    }

    pub(super) fn record_i32_scalar_replay_miss_function(&mut self) {
        self.stats
            .resource_summary_value_i32_scalar_return_facts_replay_miss_functions += 1;
    }

    pub(super) fn record_raw_init_replay_probe_function(&mut self) {
        self.stats
            .resource_summary_value_raw_init_param_facts_replay_probe_functions += 1;
    }

    pub(super) fn record_raw_init_replay_hit_function(&mut self) {
        self.stats
            .resource_summary_value_raw_init_param_facts_replay_hit_functions += 1;
    }

    pub(super) fn record_raw_init_replay_miss_function(&mut self) {
        self.stats
            .resource_summary_value_raw_init_param_facts_replay_miss_functions += 1;
    }

    pub(super) fn record_drop_traversal_replay_probe_function(&mut self) {
        self.stats
            .resource_summary_value_drop_traversal_forall_replay_probe_functions += 1;
    }

    pub(super) fn record_drop_traversal_replay_hit_function(&mut self) {
        self.stats
            .resource_summary_value_drop_traversal_forall_replay_hit_functions += 1;
    }

    pub(super) fn record_drop_traversal_replay_miss_function(&mut self) {
        self.stats
            .resource_summary_value_drop_traversal_forall_replay_miss_functions += 1;
    }

    pub(super) fn record_initialized_function_check_replay_probe_function(&mut self) {
        self.stats
            .resource_summary_value_initialized_function_check_replay_probe_functions += 1;
    }

    pub(super) fn record_initialized_function_check_replay_hit_function(&mut self) {
        self.stats
            .resource_summary_value_initialized_function_check_replay_hit_functions += 1;
    }

    pub(super) fn record_initialized_function_check_replay_miss_function(&mut self) {
        self.stats
            .resource_summary_value_initialized_function_check_replay_miss_functions += 1;
    }

    /// initialized-state checker の stage 再計算数を session 統計へ記録する。
    ///
    /// これは summary value cache の hit/miss とは別に、cache replay 後にも残っている
    /// 固定費を Web / Node の same-session 測定 JSON で追うための観測点である。
    /// 統計だけを増やし、各 stage の判定結果や safety proof には影響しない。
    pub(super) fn record_initialized_summary_stage(
        &mut self,
        stage: ResourceSummaryComputationStage,
        recomputations: usize,
        summary_count: usize,
    ) {
        match stage {
            ResourceSummaryComputationStage::RawAlias => {
                self.stats.resource_raw_alias_summary_recomputations += recomputations;
                self.stats.resource_raw_alias_summary_count += summary_count;
            }
            ResourceSummaryComputationStage::I32Scalar => {
                self.stats.resource_i32_scalar_summary_recomputations += recomputations;
                self.stats.resource_i32_scalar_summary_count += summary_count;
            }
            ResourceSummaryComputationStage::RawInit => {
                self.stats.resource_raw_init_summary_recomputations += recomputations;
                self.stats.resource_raw_init_summary_count += summary_count;
            }
            ResourceSummaryComputationStage::CollectionSlot => {
                self.stats.resource_collection_slot_summary_recomputations += recomputations;
                self.stats.resource_collection_slot_summary_count += summary_count;
            }
        }
    }

    /// final initialized function check の実行量を session 統計へ記録する。
    ///
    /// raw-init summary replay が成功しても、各関数の本体 check が全て走る限り
    /// `compile_ms` は秒単位で残る。この counter は、その残り固定費を cache stats と
    /// 同じ JSON 境界で観測するために保持する。
    pub(super) fn record_initialized_function_check(&mut self, op_count: usize) {
        self.stats.resource_initialized_function_checks += 1;
        self.stats.resource_initialized_function_check_ops += op_count;
    }

    /// complete leaf-only `DropTraversal + ForallInitializedRange` entry 候補を作る。
    ///
    /// この候補は compile work skip ではなく、現在の function boundary へ fail-closed に
    /// 逆投影できる stable entry だけを store/hit 統計へ渡すための境界である。entry は
    /// top-level op の順序と重複を保持し、部分的に変換できる leaf だけを保存しない。
    pub(super) fn drop_traversal_forall_leaf_entry_candidate(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        ops: &[CollectionSlotLifecycleSummaryOp],
    ) -> Option<ResourceSummaryDropTraversalForallLeafEntryCandidate> {
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
        let (key, entry) = drop_traversal_forall_leaf_entry_candidate_key_and_entry(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            function,
            type_params,
            generic_type_args,
            ops,
        )?;
        let reprojection = ResourceSummaryTypeReprojection::new(types, function, type_params)?;
        reproject_drop_traversal_forall_leaf_entry(&reprojection, &entry)?;
        Some(ResourceSummaryDropTraversalForallLeafEntryCandidate { key, entry })
    }

    pub(super) fn record_drop_traversal_forall_bypass(&mut self) {
        self.stats.resource_summary_value_bypasses += 1;
        self.stats
            .resource_summary_value_drop_traversal_forall_bypasses += 1;
    }

    /// complete leaf entry を現在の compile session の summary op 列として replay する。
    ///
    /// この API は fixed-point worklist の前でだけ使う。key が存在しない場合は通常の
    /// recompute に戻る miss であり、replay counter は動かさない。既存 entry があるのに
    /// 現在の型境界へ再投影できない場合だけ、cache value が意図的に使えなかった
    /// replay bypass として記録する。
    pub(super) fn replay_drop_traversal_forall_leaf_entry(
        &mut self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
    ) -> Option<Vec<CollectionSlotLifecycleSummaryOp>> {
        let source_capability_policy_hash =
            context.source_capability_policy_hash_for_function(function)?;
        let generic_type_args = if function.type_params.is_empty() && type_params.is_empty() {
            ResourceSummaryGenericTypeArgumentKeyInput::NonGeneric
        } else {
            ResourceSummaryGenericTypeArgumentKeyInput::TemplateBoundaryOnly
        };
        let key = drop_traversal_forall_leaf_entry_key(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            function,
            type_params,
            generic_type_args,
        )?;
        let entry = self.drop_traversal_forall_leaf_entries.get(&key)?.clone();
        let op_count = entry.len();
        let Some(reprojection) = ResourceSummaryTypeReprojection::new(types, function, type_params)
        else {
            self.record_drop_traversal_forall_replay_bypass(op_count);
            return None;
        };
        let Some(ops) = reproject_drop_traversal_forall_leaf_entry(&reprojection, &entry) else {
            self.record_drop_traversal_forall_replay_bypass(op_count);
            return None;
        };

        self.stats.resource_summary_value_replay_hits += op_count;
        self.stats.resource_summary_value_replayed_ops += op_count;
        Some(ops)
    }

    pub(super) fn record_drop_traversal_forall_recomputed_ops(&mut self, op_count: usize) {
        self.stats.resource_summary_value_recomputed_ops += op_count;
        self.stats
            .resource_summary_value_drop_traversal_forall_recomputed_ops += op_count;
    }

    fn record_drop_traversal_forall_replay_bypass(&mut self, op_count: usize) {
        self.stats.resource_summary_value_replay_bypasses += op_count;
    }

    /// keyable な `DropTraversal + ForallInitializedRange` 候補を session cache に記録する。
    ///
    /// hit 判定は、この呼び出しが始まる前から cache に存在した complete entry だけを
    /// 対象にする。同じ summary build pass 内で先に store した entry を即 hit と数えると、
    /// 微小変更時の再利用可能性を過大評価するためである。
    pub(super) fn record_drop_traversal_forall_leaf_entry_candidates(
        &mut self,
        candidates: Vec<ResourceSummaryDropTraversalForallLeafEntryCandidate>,
    ) {
        let candidates_with_hits = candidates
            .into_iter()
            .map(|candidate| {
                let existed_before_recording = self
                    .drop_traversal_forall_leaf_entries
                    .get(&candidate.key)
                    .is_some_and(|entry| entry == &candidate.entry);
                (candidate, existed_before_recording)
            })
            .collect::<Vec<_>>();

        for (candidate, existed_before_recording) in candidates_with_hits {
            let op_count = candidate.entry.len();
            if existed_before_recording {
                self.stats.resource_summary_value_hits += op_count;
                self.stats.resource_summary_value_drop_traversal_forall_hits += op_count;
                continue;
            }

            self.stats.resource_summary_value_misses += op_count;
            self.drop_traversal_forall_leaf_entries
                .insert(candidate.key, candidate.entry);
            self.stats.resource_summary_value_stores += op_count;
            self.stats
                .resource_summary_value_drop_traversal_forall_stores += op_count;
        }
    }
}
