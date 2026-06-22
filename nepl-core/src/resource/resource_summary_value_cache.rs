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
mod function_fingerprint;
mod i32_scalar;
mod initialized_check;
mod key;
mod owner_obligation;
mod pass_plan;
mod raw_alias;
mod raw_init;
mod stable_hash;
mod stable_mirror;
mod stable_type_key;
mod summary_plan;
mod type_boundary;

pub use self::context::ResourceSummaryValueCacheContext;
pub(super) use self::dependency_hash::{
    i32_scalar_dependency_closure_hash, initialized_function_check_dependency_closure_hash,
    owner_obligation_check_dependency_closure_hash, raw_alias_dependency_closure_hash,
    raw_init_dependency_closure_hash,
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
    ResourceSummaryStableInitializedFunctionCheckEntry,
    ResourceSummaryStableOwnerObligationCheckEntry, ResourceSummaryStableRawAliasReturnEntry,
    ResourceSummaryStableRawInitCompleteLeafEntry, ResourceSummaryTypeReprojection,
};
pub(in crate::resource) use self::summary_plan::ResourceSummaryReplayPlan;

/// Resource IR 関数本文を artifact key 用の安定 hash へ変換する。
///
/// `.neplproof` と `.neplobj` はどちらも、`TypeId`、`Span`、一時値 ID、storage ID のような
/// session-local 値を永続 key に入れてはならない。この関数は Resource summary cache が使う
/// body hash と同じ authority を公開し、direct-call `.neplobj` が selected callable body hash を
/// 再発明しないようにする。
///
/// raw wasm / LLVM body の本文文字列は `ResourceFunction` に残らないため、この hash だけで
/// raw body の再利用可否を決めてはならない。caller は source capability policy hash や
/// backend feature set と組み合わせて fail-closed に扱う。
pub fn resource_function_body_stable_hash(
    types: &TypeCtx,
    function: &ResourceFunction,
) -> Option<u64> {
    body_hash::resource_function_body_hash(types, function)
}

/// Resource IR summary value cache の累積統計。
///
/// この統計は compiled-output cache とは別に、Resource IR の証明結果を stable value
/// として保存・再投影できるかを観測するために使う。`resource_summary_value_hits` は
/// 既存 stable value が再投影可能だった候補 hit であり、fixed-point worklist の skip
/// までは意味しない。実際に summary op を replay して compile work を減らす段階では、
/// `resource_summary_value_replay_*` を別 counter として増やす。
///
/// `*_plan_skip_functions` は changed-function plan により dependency closure hash の
/// 再構築を省けた関数数である。対応する `*_plan_skip_ops` は、その関数で再投影した
/// summary fact 数を示す補助指標であり、`resource_summary_value_replayed_ops` へ加算済みの
/// materialized fact 数と合算して「削減 op 数」と読んではならない。
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
    pub resource_owner_obligation_function_checks: usize,
    pub resource_owner_obligation_function_check_ops: usize,
    pub resource_owner_return_summary_recomputations: usize,
    pub resource_owner_return_summary_count: usize,
    pub resource_owner_return_summary_pass_cache_skip_functions: usize,
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
    pub resource_summary_value_raw_alias_return_entry_plan_skip_functions: usize,
    pub resource_summary_value_raw_alias_return_entry_plan_skip_ops: usize,
    pub resource_summary_value_i32_scalar_return_facts_plan_skip_functions: usize,
    pub resource_summary_value_i32_scalar_return_facts_plan_skip_ops: usize,
    pub resource_summary_value_raw_init_param_facts_plan_skip_functions: usize,
    pub resource_summary_value_raw_init_param_facts_plan_skip_ops: usize,
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
    pub resource_summary_value_initialized_function_check_plan_skip_functions: usize,
    pub resource_summary_value_initialized_function_check_plan_skip_ops: usize,
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
    pub resource_summary_value_owner_obligation_check_hits: usize,
    pub resource_summary_value_owner_obligation_check_stores: usize,
    pub resource_summary_value_owner_obligation_check_bypasses: usize,
    pub resource_summary_value_owner_obligation_check_plan_skip_functions: usize,
    pub resource_summary_value_owner_obligation_check_plan_skip_ops: usize,
    pub resource_summary_value_owner_obligation_check_replay_probe_functions: usize,
    pub resource_summary_value_owner_obligation_check_replay_hit_functions: usize,
    pub resource_summary_value_owner_obligation_check_replay_miss_functions: usize,
    pub resource_summary_value_owner_obligation_check_dependency_bypasses: usize,
    pub resource_summary_value_owner_obligation_check_diagnostic_bypasses: usize,
    pub resource_summary_value_owner_obligation_check_missing_source_policy_bypasses: usize,
    pub resource_summary_value_owner_obligation_check_unstable_key_bypasses: usize,
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
#[derive(Debug)]
pub struct ResourceSummaryValueCache {
    stats: ResourceSummaryValueCacheStats,
    stable_entry_collection_enabled: bool,
    same_session_pass_snapshot_collection_enabled: bool,
    raw_alias_return_entry_collection_enabled: bool,
    drop_traversal_forall_leaf_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableDropTraversalForallLeafEntry>,
    raw_alias_return_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableRawAliasReturnEntry>,
    raw_alias_return_summary_snapshot: Option<summary_plan::ResourceSummaryReplaySnapshot>,
    i32_scalar_return_facts_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableI32ScalarReturnFactsEntry>,
    i32_scalar_return_summary_snapshot: Option<summary_plan::ResourceSummaryReplaySnapshot>,
    initialized_function_check_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableInitializedFunctionCheckEntry>,
    initialized_function_check_pass_snapshot:
        Option<pass_plan::InitializedFunctionCheckPassSnapshot>,
    owner_obligation_check_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableOwnerObligationCheckEntry>,
    owner_obligation_check_pass_snapshot: Option<pass_plan::OwnerObligationCheckPassSnapshot>,
    raw_init_complete_leaf_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableRawInitCompleteLeafEntry>,
    raw_init_complete_leaf_summary_snapshot: Option<summary_plan::ResourceSummaryReplaySnapshot>,
}

impl Default for ResourceSummaryValueCache {
    fn default() -> Self {
        Self {
            stats: ResourceSummaryValueCacheStats::default(),
            stable_entry_collection_enabled: true,
            same_session_pass_snapshot_collection_enabled: true,
            raw_alias_return_entry_collection_enabled: true,
            drop_traversal_forall_leaf_entries: BTreeMap::new(),
            raw_alias_return_entries: BTreeMap::new(),
            raw_alias_return_summary_snapshot: None,
            i32_scalar_return_facts_entries: BTreeMap::new(),
            i32_scalar_return_summary_snapshot: None,
            initialized_function_check_entries: BTreeMap::new(),
            initialized_function_check_pass_snapshot: None,
            owner_obligation_check_entries: BTreeMap::new(),
            owner_obligation_check_pass_snapshot: None,
            raw_init_complete_leaf_entries: BTreeMap::new(),
            raw_init_complete_leaf_summary_snapshot: None,
        }
    }
}

/// `.neplproof` へ進める Resource proof snapshot の stable entry 数。
///
/// これは永続化形式そのものではなく、`ResourceSummaryValueCache` が外へ出せる証明情報の
/// public observation surface である。各 field は保存可能な entry 数だけを数え、
/// changed-function replay plan や診断 span などの compile session に閉じた情報は含めない。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceSummaryProofSnapshotCounts {
    pub drop_traversal_forall_leaf_entries: usize,
    pub raw_alias_return_entries: usize,
    pub i32_scalar_return_facts_entries: usize,
    pub initialized_function_check_entries: usize,
    pub owner_obligation_check_entries: usize,
    pub raw_init_complete_leaf_entries: usize,
}

impl ResourceSummaryProofSnapshotCounts {
    pub fn total_entries(self) -> usize {
        self.drop_traversal_forall_leaf_entries
            + self.raw_alias_return_entries
            + self.i32_scalar_return_facts_entries
            + self.initialized_function_check_entries
            + self.owner_obligation_check_entries
            + self.raw_init_complete_leaf_entries
    }
}

/// `.neplproof` preseed 時の kind 別 merge 結果。
///
/// snapshot は古い compile session や disk artifact から来る前提なので、現在の cache に
/// 同じ key で異なる value がある場合は古い値を上書きしない。`rejected_conflict_entries`
/// はその fail-closed 分岐を数え、次回の通常 Resource check が authority になることを
/// 呼び出し側が観測できるようにする。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceSummaryProofSnapshotMergeStats {
    pub accepted_entries: usize,
    pub existing_matching_entries: usize,
    pub rejected_conflict_entries: usize,
}

impl ResourceSummaryProofSnapshotMergeStats {
    pub fn input_entries(self) -> usize {
        self.accepted_entries + self.existing_matching_entries + self.rejected_conflict_entries
    }

    pub fn usable_entries(self) -> usize {
        self.accepted_entries + self.existing_matching_entries
    }
}

/// `.neplproof` snapshot を `ResourceSummaryValueCache` へ preseed した結果。
///
/// これは高速化の観測値であり、安全性の根拠ではない。実際に保存 entry を使うかどうかは、
/// 後続の replay API が現在の `TypeCtx` / function signature / source capability policy へ
/// 再投影できるかを再度検査して決める。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceSummaryProofSnapshotPreseedStats {
    pub drop_traversal_forall_leaf_entries: ResourceSummaryProofSnapshotMergeStats,
    pub raw_alias_return_entries: ResourceSummaryProofSnapshotMergeStats,
    pub i32_scalar_return_facts_entries: ResourceSummaryProofSnapshotMergeStats,
    pub initialized_function_check_entries: ResourceSummaryProofSnapshotMergeStats,
    pub owner_obligation_check_entries: ResourceSummaryProofSnapshotMergeStats,
    pub raw_init_complete_leaf_entries: ResourceSummaryProofSnapshotMergeStats,
}

impl ResourceSummaryProofSnapshotPreseedStats {
    pub fn accepted_entries(self) -> usize {
        self.drop_traversal_forall_leaf_entries.accepted_entries
            + self.raw_alias_return_entries.accepted_entries
            + self.i32_scalar_return_facts_entries.accepted_entries
            + self.initialized_function_check_entries.accepted_entries
            + self.owner_obligation_check_entries.accepted_entries
            + self.raw_init_complete_leaf_entries.accepted_entries
    }

    pub fn existing_matching_entries(self) -> usize {
        self.drop_traversal_forall_leaf_entries
            .existing_matching_entries
            + self.raw_alias_return_entries.existing_matching_entries
            + self
                .i32_scalar_return_facts_entries
                .existing_matching_entries
            + self
                .initialized_function_check_entries
                .existing_matching_entries
            + self
                .owner_obligation_check_entries
                .existing_matching_entries
            + self
                .raw_init_complete_leaf_entries
                .existing_matching_entries
    }

    pub fn rejected_conflict_entries(self) -> usize {
        self.drop_traversal_forall_leaf_entries
            .rejected_conflict_entries
            + self.raw_alias_return_entries.rejected_conflict_entries
            + self
                .i32_scalar_return_facts_entries
                .rejected_conflict_entries
            + self
                .initialized_function_check_entries
                .rejected_conflict_entries
            + self
                .owner_obligation_check_entries
                .rejected_conflict_entries
            + self
                .raw_init_complete_leaf_entries
                .rejected_conflict_entries
    }
}

/// `.neplproof` artifact header の現行 schema version。
///
/// stable mirror entry の形や envelope に含める invalidation 入力が変わる場合は、この値を
/// 上げる。古い artifact は読み込めても preseed 前の互換性検査で拒否し、通常の Resource
/// check へ戻す。
pub const RESOURCE_SUMMARY_PROOF_ARTIFACT_SCHEMA_VERSION: u32 = 2;

/// `.neplproof` artifact の invalidation envelope。
///
/// この header は disk format そのものではなく、artifact を cache へ preseed してよいかを
/// 判定する typed boundary である。`target_hash`、`profile_hash`、`compiler_identity_hash`
/// は呼び出し側が canonical な compiler version / target / profile 文字列から作る安定 hash
/// を渡す。`resource_summary_namespace_hash` は `ResourceSummaryCacheNamespaceKey::stable_hash`
/// と一致させ、per-entry key の namespace と artifact 全体の namespace を二重に照合する。
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResourceSummaryProofArtifactHeader {
    pub schema_version: u32,
    pub compiler_identity_hash: u64,
    pub target_hash: u64,
    pub profile_hash: u64,
    pub stdlib_content_hash: Option<u64>,
    pub dependency_public_surface_hash: Option<u64>,
    pub resource_summary_namespace_hash: u64,
    pub source_capability_policy_set_hash: Option<u64>,
    pub private_effect_policy_hash: Option<u64>,
}

impl ResourceSummaryProofArtifactHeader {
    pub fn new(
        compiler_identity_hash: u64,
        target_hash: u64,
        profile_hash: u64,
        stdlib_content_hash: Option<u64>,
        dependency_public_surface_hash: Option<u64>,
        resource_summary_namespace_hash: u64,
        source_capability_policy_set_hash: Option<u64>,
        private_effect_policy_hash: Option<u64>,
    ) -> Self {
        Self {
            schema_version: RESOURCE_SUMMARY_PROOF_ARTIFACT_SCHEMA_VERSION,
            compiler_identity_hash,
            target_hash,
            profile_hash,
            stdlib_content_hash,
            dependency_public_surface_hash,
            resource_summary_namespace_hash,
            source_capability_policy_set_hash,
            private_effect_policy_hash,
        }
    }

    pub fn compatibility_reject(
        self,
        expected: Self,
    ) -> Option<ResourceSummaryProofArtifactCompatibilityReject> {
        if self.schema_version != expected.schema_version {
            return Some(ResourceSummaryProofArtifactCompatibilityReject::SchemaVersion);
        }
        if self.compiler_identity_hash != expected.compiler_identity_hash {
            return Some(ResourceSummaryProofArtifactCompatibilityReject::CompilerIdentity);
        }
        if self.target_hash != expected.target_hash {
            return Some(ResourceSummaryProofArtifactCompatibilityReject::Target);
        }
        if self.profile_hash != expected.profile_hash {
            return Some(ResourceSummaryProofArtifactCompatibilityReject::Profile);
        }
        if self.stdlib_content_hash != expected.stdlib_content_hash {
            return Some(ResourceSummaryProofArtifactCompatibilityReject::StdlibContent);
        }
        if self.dependency_public_surface_hash != expected.dependency_public_surface_hash {
            return Some(ResourceSummaryProofArtifactCompatibilityReject::DependencyPublicSurface);
        }
        if self.resource_summary_namespace_hash != expected.resource_summary_namespace_hash {
            return Some(ResourceSummaryProofArtifactCompatibilityReject::ResourceSummaryNamespace);
        }
        if self.source_capability_policy_set_hash != expected.source_capability_policy_set_hash {
            return Some(
                ResourceSummaryProofArtifactCompatibilityReject::SourceCapabilityPolicySet,
            );
        }
        if self.private_effect_policy_hash != expected.private_effect_policy_hash {
            return Some(ResourceSummaryProofArtifactCompatibilityReject::PrivateEffectPolicy);
        }
        None
    }
}

/// `.neplproof` artifact header が現在 compile と一致しない理由。
///
/// reject はすべて fail-closed であり、該当 artifact の proof entry は cache へ preseed しない。
/// どの入力で拒否したかを enum に分け、Web / CLI 側が将来 JSON counter へ出せるようにする。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceSummaryProofArtifactCompatibilityReject {
    SchemaVersion,
    CompilerIdentity,
    Target,
    Profile,
    StdlibContent,
    DependencyPublicSurface,
    ResourceSummaryNamespace,
    SourceCapabilityPolicySet,
    PrivateEffectPolicy,
}

/// `.neplproof` artifact の in-memory 表現。
///
/// payload は `ResourceSummaryProofSnapshot`、invalidation authority は
/// `ResourceSummaryProofArtifactHeader` に分ける。これにより、disk / IndexedDB / bundled
/// stdlib artifact の保存方法を後から選んでも、`core` 側の preseed 判定は同じになる。
#[derive(Debug, Clone)]
pub struct ResourceSummaryProofArtifact {
    header: ResourceSummaryProofArtifactHeader,
    snapshot: ResourceSummaryProofSnapshot,
}

impl ResourceSummaryProofArtifact {
    pub fn new(
        header: ResourceSummaryProofArtifactHeader,
        snapshot: ResourceSummaryProofSnapshot,
    ) -> Self {
        Self { header, snapshot }
    }

    pub fn header(&self) -> ResourceSummaryProofArtifactHeader {
        self.header
    }

    pub fn snapshot(&self) -> &ResourceSummaryProofSnapshot {
        &self.snapshot
    }

    pub fn counts(&self) -> ResourceSummaryProofSnapshotCounts {
        self.snapshot.counts()
    }

    pub fn compatibility_reject(
        &self,
        expected_header: ResourceSummaryProofArtifactHeader,
    ) -> Option<ResourceSummaryProofArtifactCompatibilityReject> {
        self.header.compatibility_reject(expected_header)
    }

    /// `.neplproof` 用の header-first binary payload へ変換する。
    ///
    /// container header は postcard payload の外に固定長で書き出す。これにより、読み込み側は
    /// `schema_version`、compiler、target/profile、stdlib hash、Resource namespace、
    /// source capability policy などを先に照合し、不一致なら snapshot payload を
    /// deserialize せずに通常の Resource static check へ戻せる。
    pub fn to_neplproof_bytes(&self) -> Result<Vec<u8>, ResourceSummaryProofArtifactCodecError> {
        let payload = postcard::to_allocvec(&self.snapshot)
            .map_err(|_| ResourceSummaryProofArtifactCodecError::Encode)?;
        let mut out = Vec::with_capacity(NEPLPROOF_FIXED_HEADER_LEN + payload.len());
        write_neplproof_header_bytes(&mut out, self.header, neplproof_payload_hash(&payload));
        out.extend(payload);
        Ok(out)
    }

    /// `.neplproof` binary payload の header だけを読む。
    ///
    /// payload は一切 deserialize しない。CLI や Web host は、この値を観測して
    /// cache hit/miss の理由を出せるが、実際の preseed には必ず
    /// `from_neplproof_bytes_with_expected_header` を使う。
    pub fn header_from_neplproof_bytes(
        bytes: &[u8],
    ) -> Result<ResourceSummaryProofArtifactHeader, ResourceSummaryProofArtifactCodecError> {
        read_neplproof_header_bytes(bytes).map(|(header, _, _)| header)
    }

    /// expected header と一致する `.neplproof` だけを in-memory artifact へ戻す。
    ///
    /// header mismatch は `Compatibility` として返し、payload decode は実行しない。
    /// そのため、古い artifact や別 target の artifact に壊れた payload が付いていても、
    /// Resource proof cache は fail-closed に通常検査へ戻れる。
    pub fn from_neplproof_bytes_with_expected_header(
        bytes: &[u8],
        expected_header: ResourceSummaryProofArtifactHeader,
    ) -> Result<Self, ResourceSummaryProofArtifactByteReject> {
        let (header, payload_offset, expected_payload_hash) = read_neplproof_header_bytes(bytes)
            .map_err(ResourceSummaryProofArtifactByteReject::Codec)?;
        if let Some(reject) = header.compatibility_reject(expected_header) {
            return Err(ResourceSummaryProofArtifactByteReject::Compatibility(
                reject,
            ));
        }
        let payload =
            bytes
                .get(payload_offset..)
                .ok_or(ResourceSummaryProofArtifactByteReject::Codec(
                    ResourceSummaryProofArtifactCodecError::Decode,
                ))?;
        if neplproof_payload_hash(payload) != expected_payload_hash {
            return Err(ResourceSummaryProofArtifactByteReject::Codec(
                ResourceSummaryProofArtifactCodecError::Decode,
            ));
        }
        let snapshot = postcard::from_bytes(payload).map_err(|_| {
            ResourceSummaryProofArtifactByteReject::Codec(
                ResourceSummaryProofArtifactCodecError::Decode,
            )
        })?;
        Ok(Self { header, snapshot })
    }
}

/// `.neplproof` codec の失敗理由。
///
/// 詳細な postcard error は永続 format の public contract にしない。CLI/Web 側は
/// `Encode` / `Decode` のどちらで失敗したかだけを観測し、失敗時は cache を使わず
/// 通常の静的検査へ戻す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceSummaryProofArtifactCodecError {
    Encode,
    Decode,
}

/// `.neplproof` bytes を現在 compile へ preseed できない理由。
///
/// `Compatibility` は header の不一致なので payload を読まずに拒否できる。`Codec` は
/// container header または compatible header 後の payload が壊れている場合で、どちらも
/// fail-closed に通常検査へ戻す。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceSummaryProofArtifactByteReject {
    Codec(ResourceSummaryProofArtifactCodecError),
    Compatibility(ResourceSummaryProofArtifactCompatibilityReject),
}

const NEPLPROOF_MAGIC: &[u8; NEPLPROOF_MAGIC_LEN] = b"NEPLPROOF2";
const NEPLPROOF_MAGIC_LEN: usize = 10;
const NEPLPROOF_CONTAINER_SCHEMA_VERSION: u32 = 2;
const NEPLPROOF_FIXED_HEADER_LEN: usize =
    NEPLPROOF_MAGIC_LEN + 4 + 4 + 8 + 8 + 8 + (1 + 8) + (1 + 8) + 8 + (1 + 8) + (1 + 8) + 8;

fn write_neplproof_header_bytes(
    out: &mut Vec<u8>,
    header: ResourceSummaryProofArtifactHeader,
    payload_hash: u64,
) {
    out.extend_from_slice(NEPLPROOF_MAGIC);
    write_u32(out, NEPLPROOF_CONTAINER_SCHEMA_VERSION);
    write_u32(out, header.schema_version);
    write_u64(out, header.compiler_identity_hash);
    write_u64(out, header.target_hash);
    write_u64(out, header.profile_hash);
    write_option_u64(out, header.stdlib_content_hash);
    write_option_u64(out, header.dependency_public_surface_hash);
    write_u64(out, header.resource_summary_namespace_hash);
    write_option_u64(out, header.source_capability_policy_set_hash);
    write_option_u64(out, header.private_effect_policy_hash);
    write_u64(out, payload_hash);
}

fn read_neplproof_header_bytes(
    bytes: &[u8],
) -> Result<(ResourceSummaryProofArtifactHeader, usize, u64), ResourceSummaryProofArtifactCodecError>
{
    if bytes.len() < NEPLPROOF_FIXED_HEADER_LEN {
        return Err(ResourceSummaryProofArtifactCodecError::Decode);
    }
    if &bytes[..NEPLPROOF_MAGIC_LEN] != NEPLPROOF_MAGIC {
        return Err(ResourceSummaryProofArtifactCodecError::Decode);
    }
    let mut cursor = NEPLPROOF_MAGIC_LEN;
    let container_schema = read_u32(bytes, &mut cursor)?;
    if container_schema != NEPLPROOF_CONTAINER_SCHEMA_VERSION {
        return Err(ResourceSummaryProofArtifactCodecError::Decode);
    }
    let header = ResourceSummaryProofArtifactHeader {
        schema_version: read_u32(bytes, &mut cursor)?,
        compiler_identity_hash: read_u64(bytes, &mut cursor)?,
        target_hash: read_u64(bytes, &mut cursor)?,
        profile_hash: read_u64(bytes, &mut cursor)?,
        stdlib_content_hash: read_option_u64(bytes, &mut cursor)?,
        dependency_public_surface_hash: read_option_u64(bytes, &mut cursor)?,
        resource_summary_namespace_hash: read_u64(bytes, &mut cursor)?,
        source_capability_policy_set_hash: read_option_u64(bytes, &mut cursor)?,
        private_effect_policy_hash: read_option_u64(bytes, &mut cursor)?,
    };
    let payload_hash = read_u64(bytes, &mut cursor)?;
    Ok((header, cursor, payload_hash))
}

fn neplproof_payload_hash(payload: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in b"neplg2-neplproof-payload-v1\0"
        .iter()
        .chain(payload.iter())
    {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn write_u32(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_u64(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn write_option_u64(out: &mut Vec<u8>, value: Option<u64>) {
    match value {
        Some(value) => {
            out.push(1);
            write_u64(out, value);
        }
        None => {
            out.push(0);
            write_u64(out, 0);
        }
    }
}

fn read_u32(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<u32, ResourceSummaryProofArtifactCodecError> {
    let raw = read_exact::<4>(bytes, cursor)?;
    Ok(u32::from_le_bytes(raw))
}

fn read_u64(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<u64, ResourceSummaryProofArtifactCodecError> {
    let raw = read_exact::<8>(bytes, cursor)?;
    Ok(u64::from_le_bytes(raw))
}

fn read_option_u64(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<Option<u64>, ResourceSummaryProofArtifactCodecError> {
    let tag = *bytes
        .get(*cursor)
        .ok_or(ResourceSummaryProofArtifactCodecError::Decode)?;
    *cursor += 1;
    let value = read_u64(bytes, cursor)?;
    match tag {
        0 => Ok(None),
        1 => Ok(Some(value)),
        _ => Err(ResourceSummaryProofArtifactCodecError::Decode),
    }
}

fn read_exact<const N: usize>(
    bytes: &[u8],
    cursor: &mut usize,
) -> Result<[u8; N], ResourceSummaryProofArtifactCodecError> {
    let end = cursor
        .checked_add(N)
        .ok_or(ResourceSummaryProofArtifactCodecError::Decode)?;
    let raw = bytes
        .get(*cursor..end)
        .ok_or(ResourceSummaryProofArtifactCodecError::Decode)?;
    *cursor = end;
    let mut out = [0_u8; N];
    out.copy_from_slice(raw);
    Ok(out)
}

/// `.neplproof` の Resource proof payload に相当する in-memory snapshot。
///
/// この型は serialization schema ではない。`TypeId`、`Span`、`SourceMap`、diagnostic は
/// 含めず、現行 `ResourceSummaryValueCache` が既に持つ stable mirror entry と、
/// stable key/fingerprint だけで構成された replay snapshot をまとめる。disk-backed
/// artifact は、この snapshot に compiler version、schema version、target/profile、
/// stdlib hash、dependency public surface hash などの envelope を付け、読み込み時には
/// 必ず再投影できる entry だけを使う。
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ResourceSummaryProofSnapshot {
    drop_traversal_forall_leaf_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableDropTraversalForallLeafEntry>,
    raw_alias_return_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableRawAliasReturnEntry>,
    raw_alias_return_summary_snapshot: Option<summary_plan::ResourceSummaryReplaySnapshot>,
    i32_scalar_return_facts_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableI32ScalarReturnFactsEntry>,
    i32_scalar_return_summary_snapshot: Option<summary_plan::ResourceSummaryReplaySnapshot>,
    initialized_function_check_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableInitializedFunctionCheckEntry>,
    initialized_function_check_pass_snapshot:
        Option<pass_plan::InitializedFunctionCheckPassSnapshot>,
    owner_obligation_check_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableOwnerObligationCheckEntry>,
    owner_obligation_check_pass_snapshot: Option<pass_plan::OwnerObligationCheckPassSnapshot>,
    raw_init_complete_leaf_entries:
        BTreeMap<ResourceSummaryValueCacheKey, ResourceSummaryStableRawInitCompleteLeafEntry>,
    raw_init_complete_leaf_summary_snapshot: Option<summary_plan::ResourceSummaryReplaySnapshot>,
}

impl ResourceSummaryProofSnapshot {
    pub fn counts(&self) -> ResourceSummaryProofSnapshotCounts {
        ResourceSummaryProofSnapshotCounts {
            drop_traversal_forall_leaf_entries: self.drop_traversal_forall_leaf_entries.len(),
            raw_alias_return_entries: self.raw_alias_return_entries.len(),
            i32_scalar_return_facts_entries: self.i32_scalar_return_facts_entries.len(),
            initialized_function_check_entries: self.initialized_function_check_entries.len(),
            owner_obligation_check_entries: self.owner_obligation_check_entries.len(),
            raw_init_complete_leaf_entries: self.raw_init_complete_leaf_entries.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.counts().total_entries() == 0
    }
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

#[derive(Debug, Clone)]
pub(super) struct ResourceSummaryOwnerObligationCheckEntryCandidate {
    key: ResourceSummaryValueCacheKey,
    entry: ResourceSummaryStableOwnerObligationCheckEntry,
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
        self.raw_alias_return_summary_snapshot = None;
        self.i32_scalar_return_facts_entries.clear();
        self.i32_scalar_return_summary_snapshot = None;
        self.initialized_function_check_entries.clear();
        self.initialized_function_check_pass_snapshot = None;
        self.owner_obligation_check_entries.clear();
        self.owner_obligation_check_pass_snapshot = None;
        self.raw_init_complete_leaf_entries.clear();
        self.raw_init_complete_leaf_summary_snapshot = None;
    }

    /// `.neplproof` へ永続化できる stable entry の収集を止める。
    ///
    /// 同一 `CompilerSession` 内の微小編集には changed-function pass snapshot を使い、
    /// Web playground の通常 compile では `TypeId` を含む検査結果を stable mirror へ
    /// 変換する固定費だけを払わない。disk-backed `.neplproof` を作る CLI 経路や、
    /// 明示的に proof artifact を扱う検証では default の有効状態を使う。
    pub fn disable_stable_entry_collection(&mut self) {
        self.stable_entry_collection_enabled = false;
        self.drop_traversal_forall_leaf_entries.clear();
        self.raw_alias_return_entries.clear();
        self.i32_scalar_return_facts_entries.clear();
        self.initialized_function_check_entries.clear();
        self.owner_obligation_check_entries.clear();
        self.raw_init_complete_leaf_entries.clear();
        self.raw_alias_return_summary_snapshot = None;
        self.i32_scalar_return_summary_snapshot = None;
        self.raw_init_complete_leaf_summary_snapshot = None;
    }

    pub(in crate::resource) fn stable_entry_collection_enabled(&self) -> bool {
        self.stable_entry_collection_enabled
    }

    pub(in crate::resource) fn same_session_pass_snapshot_collection_enabled(&self) -> bool {
        self.same_session_pass_snapshot_collection_enabled
    }

    /// raw-alias return summary の stable entry 収集を止める。
    ///
    /// RPN cold base の計測では、raw-alias summary 本体の再計算は十分軽い一方で、
    /// stable entry 化と replay key 探索は永続 proof からの preseed 時に再計算より
    /// 高い固定費を持つ。CLI の disk-backed `.neplproof` は i32/raw-init/owner など
    /// 重い proof kind だけを使うため、この policy で raw-alias kind を明示的に外す。
    /// same-session cache は既定で有効なままなので、長寿命 session での既存挙動は保つ。
    pub fn disable_raw_alias_return_entry_collection(&mut self) {
        self.raw_alias_return_entry_collection_enabled = false;
        self.raw_alias_return_entries.clear();
        self.raw_alias_return_summary_snapshot = None;
    }

    pub(in crate::resource) fn raw_alias_return_entry_collection_enabled(&self) -> bool {
        self.raw_alias_return_entry_collection_enabled
    }

    /// Raw-alias summary replay が現在の cache 内容から hit し得るかを返す。
    ///
    /// 空の entry kind に対して stable key や dependency closure を構成しても、結果は
    /// 必ず miss になる。cold base compile ではその探索自体が WebAssembly 上の固定費に
    /// なるため、各 checker は replay 可能な kind だけを probing する。一方で store 側は
    /// この判定とは独立に動かし、次の微小編集で使う proof surface を残す。
    pub(in crate::resource) fn has_raw_alias_return_replay_entries(
        &self,
        context: &ResourceSummaryValueCacheContext,
    ) -> bool {
        let namespace_hash = context.namespace_hash().as_u64();
        self.raw_alias_return_entry_collection_enabled
            && self
                .raw_alias_return_entries
                .keys()
                .any(|key| key.namespace_hash() == namespace_hash)
    }

    /// i32 scalar summary replay が現在の cache 内容から hit し得るかを返す。
    pub(in crate::resource) fn has_i32_scalar_return_replay_entries(
        &self,
        context: &ResourceSummaryValueCacheContext,
    ) -> bool {
        let namespace_hash = context.namespace_hash().as_u64();
        self.i32_scalar_return_facts_entries
            .keys()
            .any(|key| key.namespace_hash() == namespace_hash)
    }

    /// raw-init complete leaf summary replay が現在の cache 内容から hit し得るかを返す。
    pub(in crate::resource) fn has_raw_init_complete_leaf_replay_entries(
        &self,
        context: &ResourceSummaryValueCacheContext,
    ) -> bool {
        let namespace_hash = context.namespace_hash().as_u64();
        self.raw_init_complete_leaf_entries
            .keys()
            .any(|key| key.namespace_hash() == namespace_hash)
    }

    /// collection slot drop traversal replay が現在の cache 内容から hit し得るかを返す。
    pub(in crate::resource) fn has_drop_traversal_forall_replay_entries(
        &self,
        context: &ResourceSummaryValueCacheContext,
    ) -> bool {
        let namespace_hash = context.namespace_hash().as_u64();
        self.drop_traversal_forall_leaf_entries
            .keys()
            .any(|key| key.namespace_hash() == namespace_hash)
    }

    /// final initialized check の stable entry replay が hit し得るかを返す。
    pub(in crate::resource) fn has_initialized_function_check_replay_entries(
        &self,
        context: &ResourceSummaryValueCacheContext,
    ) -> bool {
        let namespace_hash = context.namespace_hash().as_u64();
        self.initialized_function_check_entries
            .keys()
            .any(|key| key.namespace_hash() == namespace_hash)
    }

    /// owner obligation check の stable entry replay が hit し得るかを返す。
    pub(in crate::resource) fn has_owner_obligation_check_replay_entries(
        &self,
        context: &ResourceSummaryValueCacheContext,
    ) -> bool {
        let namespace_hash = context.namespace_hash().as_u64();
        self.owner_obligation_check_entries
            .keys()
            .any(|key| key.namespace_hash() == namespace_hash)
    }

    pub fn stats(&self) -> ResourceSummaryValueCacheStats {
        self.stats
    }

    /// 現在の Resource proof cache から `.neplproof` 用 snapshot を作る。
    ///
    /// export 対象は stable mirror entry の map と stable key/fingerprint だけの replay
    /// snapshot である。summary 本体や final cell state は保存せず、次回 compile では
    /// snapshot が指す key の entry を現在の `TypeCtx` へ再投影できた場合だけ使う。
    pub fn export_neplproof_snapshot(&self) -> ResourceSummaryProofSnapshot {
        let raw_alias_return_entries = if self.raw_alias_return_entry_collection_enabled {
            self.raw_alias_return_entries.clone()
        } else {
            BTreeMap::new()
        };
        let raw_alias_return_summary_snapshot = if self.raw_alias_return_entry_collection_enabled {
            self.raw_alias_return_summary_snapshot.clone()
        } else {
            None
        };
        ResourceSummaryProofSnapshot {
            drop_traversal_forall_leaf_entries: self.drop_traversal_forall_leaf_entries.clone(),
            raw_alias_return_entries,
            raw_alias_return_summary_snapshot,
            i32_scalar_return_facts_entries: self.i32_scalar_return_facts_entries.clone(),
            i32_scalar_return_summary_snapshot: self.i32_scalar_return_summary_snapshot.clone(),
            initialized_function_check_entries: self.initialized_function_check_entries.clone(),
            initialized_function_check_pass_snapshot: self
                .initialized_function_check_pass_snapshot
                .clone(),
            owner_obligation_check_entries: self.owner_obligation_check_entries.clone(),
            owner_obligation_check_pass_snapshot: self.owner_obligation_check_pass_snapshot.clone(),
            raw_init_complete_leaf_entries: self.raw_init_complete_leaf_entries.clone(),
            raw_init_complete_leaf_summary_snapshot: self
                .raw_init_complete_leaf_summary_snapshot
                .clone(),
        }
    }

    /// 現在の Resource proof cache から `.neplproof` in-memory artifact を作る。
    ///
    /// header は呼び出し側で作らせず、通常は `ResourceSummaryCacheNamespaceKey` から作った
    /// compile-context header を渡す。payload は stable mirror snapshot だけであり、disk
    /// codec はこの API の外側で header を先に照合してから payload を読む二段階にする。
    pub fn export_neplproof_artifact(
        &self,
        header: ResourceSummaryProofArtifactHeader,
    ) -> ResourceSummaryProofArtifact {
        ResourceSummaryProofArtifact::new(header, self.export_neplproof_snapshot())
    }

    /// `.neplproof` 用 snapshot の stable entry を現在の cache へ preseed する。
    ///
    /// preseed は Resource proof を確定させる操作ではない。ここでは stable key/value を
    /// cache map へ足すだけに留め、実際の replay 時に現在の型境界と source capability
    /// policy へ fail-closed に再投影する。replay snapshot は stable key と fingerprint
    /// だけを保持し、次の plan 作成時に現在 module の namespace / 関数順序 / 関数本体 /
    /// 依存閉包を再照合できる場合だけ dependency key の再構築を省く。
    pub fn preseed_neplproof_snapshot(
        &mut self,
        snapshot: &ResourceSummaryProofSnapshot,
    ) -> ResourceSummaryProofSnapshotPreseedStats {
        self.clear_compile_local_replay_snapshots();

        let stats = ResourceSummaryProofSnapshotPreseedStats {
            drop_traversal_forall_leaf_entries: preseed_neplproof_entry_map(
                &mut self.drop_traversal_forall_leaf_entries,
                &snapshot.drop_traversal_forall_leaf_entries,
            ),
            raw_alias_return_entries: if self.raw_alias_return_entry_collection_enabled {
                preseed_neplproof_entry_map(
                    &mut self.raw_alias_return_entries,
                    &snapshot.raw_alias_return_entries,
                )
            } else {
                ResourceSummaryProofSnapshotMergeStats::default()
            },
            i32_scalar_return_facts_entries: preseed_neplproof_entry_map(
                &mut self.i32_scalar_return_facts_entries,
                &snapshot.i32_scalar_return_facts_entries,
            ),
            initialized_function_check_entries: preseed_neplproof_entry_map(
                &mut self.initialized_function_check_entries,
                &snapshot.initialized_function_check_entries,
            ),
            owner_obligation_check_entries: preseed_neplproof_entry_map(
                &mut self.owner_obligation_check_entries,
                &snapshot.owner_obligation_check_entries,
            ),
            raw_init_complete_leaf_entries: preseed_neplproof_entry_map(
                &mut self.raw_init_complete_leaf_entries,
                &snapshot.raw_init_complete_leaf_entries,
            ),
        };

        if self.raw_alias_return_entry_collection_enabled {
            self.raw_alias_return_summary_snapshot =
                snapshot.raw_alias_return_summary_snapshot.clone();
        }
        self.i32_scalar_return_summary_snapshot =
            snapshot.i32_scalar_return_summary_snapshot.clone();
        self.initialized_function_check_pass_snapshot =
            snapshot.initialized_function_check_pass_snapshot.clone();
        self.owner_obligation_check_pass_snapshot =
            snapshot.owner_obligation_check_pass_snapshot.clone();
        self.raw_init_complete_leaf_summary_snapshot =
            snapshot.raw_init_complete_leaf_summary_snapshot.clone();

        stats
    }

    /// `.neplproof` artifact を header 照合後に現在の cache へ preseed する。
    ///
    /// header が 1 field でも一致しない場合は snapshot payload を見ずに拒否する。これは
    /// stale hit を避けるための artifact-level gate であり、成功しても各 entry の再投影検査は
    /// 後続の replay API で維持する。
    pub fn preseed_neplproof_artifact(
        &mut self,
        artifact: &ResourceSummaryProofArtifact,
        expected_header: ResourceSummaryProofArtifactHeader,
    ) -> Result<
        ResourceSummaryProofSnapshotPreseedStats,
        ResourceSummaryProofArtifactCompatibilityReject,
    > {
        if let Some(reason) = artifact.compatibility_reject(expected_header) {
            return Err(reason);
        }
        Ok(self.preseed_neplproof_snapshot(artifact.snapshot()))
    }

    fn clear_compile_local_replay_snapshots(&mut self) {
        self.raw_alias_return_summary_snapshot = None;
        self.i32_scalar_return_summary_snapshot = None;
        self.initialized_function_check_pass_snapshot = None;
        self.owner_obligation_check_pass_snapshot = None;
        self.raw_init_complete_leaf_summary_snapshot = None;
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

    pub(super) fn record_owner_obligation_check_replay_probe_function(&mut self) {
        self.stats
            .resource_summary_value_owner_obligation_check_replay_probe_functions += 1;
    }

    pub(super) fn record_owner_obligation_check_replay_hit_function(&mut self) {
        self.stats
            .resource_summary_value_owner_obligation_check_replay_hit_functions += 1;
    }

    pub(super) fn record_owner_obligation_check_replay_miss_function(&mut self) {
        self.stats
            .resource_summary_value_owner_obligation_check_replay_miss_functions += 1;
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

    /// owner obligation checker の実行量を session 統計へ記録する。
    ///
    /// owner obligation は Resource static check の後段で秒単位の固定費になり得る。pass-only
    /// replay が効いたかを Web / Node の同一 session 測定で判断するため、実際に checker
    /// 本体を起動した関数数と op 数を initialized check とは別に保持する。
    pub(super) fn record_owner_obligation_function_check(&mut self, op_count: usize) {
        self.stats.resource_owner_obligation_function_checks += 1;
        self.stats.resource_owner_obligation_function_check_ops += op_count;
    }

    /// owner return summary 固定点計算の実行量を session 統計へ記録する。
    ///
    /// owner obligation pass cache が checker 本体を skip しても、pass replay 前に
    /// owner return summary を全関数分作ると edit compile の固定費が残る。この counter は
    /// summary 固定点が本当に必要な miss path だけで走っているかを Web / Node の測定 JSON で
    /// 確認するために保持する。
    pub(super) fn record_owner_return_summary_stage(
        &mut self,
        recomputations: usize,
        summary_count: usize,
    ) {
        self.stats.resource_owner_return_summary_recomputations += recomputations;
        self.stats.resource_owner_return_summary_count += summary_count;
    }

    /// owner obligation pass cache により summary 構築そのものを省けた関数数を記録する。
    ///
    /// これは安全性判定ではなく観測用の counter である。全関数の pass entry が現在の
    /// body / dependency / source policy / type boundary に一致した場合だけ増やし、miss が
    /// ある compile では従来どおり owner summary を構築して checker へ渡す。
    pub(super) fn record_owner_return_summary_pass_cache_skip(&mut self, function_count: usize) {
        self.stats
            .resource_owner_return_summary_pass_cache_skip_functions += function_count;
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

fn preseed_neplproof_entry_map<K, V>(
    current: &mut BTreeMap<K, V>,
    snapshot: &BTreeMap<K, V>,
) -> ResourceSummaryProofSnapshotMergeStats
where
    K: Clone + Ord,
    V: Clone + PartialEq,
{
    let mut stats = ResourceSummaryProofSnapshotMergeStats::default();
    for (key, snapshot_value) in snapshot {
        match current.get(key) {
            Some(current_value) if current_value == snapshot_value => {
                stats.existing_matching_entries += 1;
            }
            Some(_) => {
                stats.rejected_conflict_entries += 1;
            }
            None => {
                current.insert(key.clone(), snapshot_value.clone());
                stats.accepted_entries += 1;
            }
        }
    }
    stats
}

#[cfg(test)]
mod tests {
    use super::key::{ResourceSummaryFunctionIdentity, ResourceSummaryValueCacheKey};
    use super::stable_mirror::ResourceSummaryStableRawAliasReturnEntry;
    use super::NEPLPROOF_FIXED_HEADER_LEN;
    use super::{
        preseed_neplproof_entry_map, ResourceSummaryProofArtifact,
        ResourceSummaryProofArtifactByteReject, ResourceSummaryProofArtifactCodecError,
        ResourceSummaryProofArtifactCompatibilityReject, ResourceSummaryProofArtifactHeader,
        ResourceSummaryProofSnapshot, ResourceSummaryProofSnapshotMergeStats,
        ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
    };
    use crate::ast::Effect;
    use crate::resource::model::{
        Place, ResourceBlock, ResourceBlockId, ResourceExprKind, ResourceFunction, ResourceModule,
        ResourceOp, ResourceTerminator,
    };
    use crate::resource::summary_dependency::ResourceSummaryDependencyGraph;
    use crate::span::{FileId, Span};
    use crate::types::TypeCtx;
    use alloc::collections::BTreeMap;
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    fn test_neplproof_header() -> ResourceSummaryProofArtifactHeader {
        ResourceSummaryProofArtifactHeader::new(2, 3, 4, Some(5), Some(6), 7, Some(8), Some(9))
    }

    fn test_context(policy_hash: u64) -> ResourceSummaryValueCacheContext {
        let mut context = ResourceSummaryValueCacheContext::new(7);
        context.insert_source_policy_hash(FileId(0), policy_hash);
        context
    }

    fn summary_key(name: &str) -> ResourceSummaryValueCacheKey {
        ResourceSummaryValueCacheKey::new_i32_scalar_return_facts_entry(
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
            kind: ResourceExprKind::LiteralI32(literal),
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

    fn span() -> Span {
        Span::new(FileId(0), 1, 2)
    }

    #[test]
    fn neplproof_snapshot_preseeds_stable_replay_plan() {
        let types = TypeCtx::new();
        let module =
            module_with_functions(vec![function_with_ops(&types, "stable", Vec::new(), 1)]);
        let graph = ResourceSummaryDependencyGraph::build(&module);
        let context = test_context(11);
        let relevant = vec![true];
        let key = summary_key("stable");
        let mut cache = ResourceSummaryValueCache::new();
        let mut plan = cache
            .begin_i32_scalar_summary_replay_plan(&context, &types, &module, &graph, &relevant);
        plan.record_key(0, key.clone());
        cache.finish_i32_scalar_summary_replay_plan(plan);

        let snapshot = cache.export_neplproof_snapshot();
        let mut preseeded = ResourceSummaryValueCache::new();
        let preseed = preseeded.preseed_neplproof_snapshot(&snapshot);
        let replay_plan = preseeded
            .begin_i32_scalar_summary_replay_plan(&context, &types, &module, &graph, &relevant);

        assert!(snapshot.is_empty());
        assert_eq!(snapshot.counts().total_entries(), 0);
        assert_eq!(preseed.accepted_entries(), 0);
        assert_eq!(preseed.existing_matching_entries(), 0);
        assert_eq!(preseed.rejected_conflict_entries(), 0);
        assert_eq!(replay_plan.previous_key(0), Some(key));
    }

    #[test]
    fn neplproof_preseed_map_never_overwrites_conflicting_current_value() {
        let mut current = BTreeMap::new();
        current.insert(1_u32, 10_u32);
        current.insert(2_u32, 20_u32);

        let mut snapshot = BTreeMap::new();
        snapshot.insert(1_u32, 10_u32);
        snapshot.insert(2_u32, 21_u32);
        snapshot.insert(3_u32, 30_u32);

        let stats = preseed_neplproof_entry_map(&mut current, &snapshot);

        assert_eq!(
            stats,
            ResourceSummaryProofSnapshotMergeStats {
                accepted_entries: 1,
                existing_matching_entries: 1,
                rejected_conflict_entries: 1,
            }
        );
        assert_eq!(current.get(&1), Some(&10));
        assert_eq!(current.get(&2), Some(&20));
        assert_eq!(current.get(&3), Some(&30));
    }

    #[test]
    fn neplproof_preseed_respects_disabled_raw_alias_entry_collection() {
        let mut snapshot = ResourceSummaryProofSnapshot::default();
        let key = ResourceSummaryValueCacheKey::new_raw_alias_return_entry(
            1,
            ResourceSummaryFunctionIdentity::new("f", "f").unwrap(),
            2,
            3,
            4,
            5,
            6,
        );
        snapshot.raw_alias_return_entries.insert(
            key,
            ResourceSummaryStableRawAliasReturnEntry::empty_for_test(),
        );

        let mut cache = ResourceSummaryValueCache::new();
        cache.disable_raw_alias_return_entry_collection();
        let stats = cache.preseed_neplproof_snapshot(&snapshot);

        assert_eq!(
            stats.raw_alias_return_entries,
            ResourceSummaryProofSnapshotMergeStats::default()
        );
    }

    #[test]
    fn neplproof_snapshot_keeps_raw_alias_entries_until_policy_disables_them() {
        let key = ResourceSummaryValueCacheKey::new_raw_alias_return_entry(
            1,
            ResourceSummaryFunctionIdentity::new("f", "f").unwrap(),
            2,
            3,
            4,
            5,
            6,
        );
        let mut cache = ResourceSummaryValueCache::new();
        cache.raw_alias_return_entries.insert(
            key,
            ResourceSummaryStableRawAliasReturnEntry::empty_for_test(),
        );

        let default_snapshot = cache.export_neplproof_snapshot();
        assert_eq!(default_snapshot.counts().raw_alias_return_entries, 1);

        cache.disable_raw_alias_return_entry_collection();
        let disabled_snapshot = cache.export_neplproof_snapshot();
        assert_eq!(disabled_snapshot.counts().raw_alias_return_entries, 0);
    }

    #[test]
    fn neplproof_snapshot_counts_sum_all_stable_entry_kinds() {
        let snapshot = ResourceSummaryProofSnapshot::default();

        assert_eq!(snapshot.counts().total_entries(), 0);
    }

    #[test]
    fn neplproof_artifact_preseed_rejects_schema_mismatch_before_payload_merge() {
        let mut cache = ResourceSummaryValueCache::new();
        let expected = test_neplproof_header();
        let mut stale = expected;
        stale.schema_version += 1;
        let artifact =
            ResourceSummaryProofArtifact::new(stale, ResourceSummaryProofSnapshot::default());

        let result = cache.preseed_neplproof_artifact(&artifact, expected);

        assert_eq!(
            result,
            Err(ResourceSummaryProofArtifactCompatibilityReject::SchemaVersion)
        );
    }

    #[test]
    fn neplproof_artifact_preseed_accepts_matching_header() {
        let mut cache = ResourceSummaryValueCache::new();
        let header = test_neplproof_header();
        let artifact =
            ResourceSummaryProofArtifact::new(header, ResourceSummaryProofSnapshot::default());

        let result = cache
            .preseed_neplproof_artifact(&artifact, header)
            .expect("matching artifact header should allow empty snapshot preseed");

        assert_eq!(result.accepted_entries(), 0);
        assert_eq!(artifact.header(), header);
        assert_eq!(artifact.counts().total_entries(), 0);
    }

    #[test]
    fn neplproof_artifact_export_wraps_current_snapshot_with_header() {
        let cache = ResourceSummaryValueCache::new();
        let header = test_neplproof_header();

        let artifact = cache.export_neplproof_artifact(header);

        assert_eq!(artifact.header(), header);
        assert_eq!(
            artifact.counts(),
            cache.export_neplproof_snapshot().counts()
        );
    }

    #[test]
    fn neplproof_artifact_codec_round_trips_header_and_counts() {
        let cache = ResourceSummaryValueCache::new();
        let header = test_neplproof_header();
        let artifact = cache.export_neplproof_artifact(header);

        let bytes = artifact
            .to_neplproof_bytes()
            .expect("empty proof artifact should encode");
        let decoded_header = ResourceSummaryProofArtifact::header_from_neplproof_bytes(&bytes)
            .expect("encoded proof artifact header should decode");
        let decoded =
            ResourceSummaryProofArtifact::from_neplproof_bytes_with_expected_header(&bytes, header)
                .expect("encoded proof artifact should decode");

        assert_eq!(decoded_header, header);
        assert_eq!(decoded.header(), header);
        assert_eq!(decoded.counts(), artifact.counts());
    }

    #[test]
    fn neplproof_artifact_codec_rejects_malformed_payload() {
        let result = ResourceSummaryProofArtifact::from_neplproof_bytes_with_expected_header(
            b"not a neplproof",
            test_neplproof_header(),
        );

        assert!(matches!(
            result,
            Err(ResourceSummaryProofArtifactByteReject::Codec(
                ResourceSummaryProofArtifactCodecError::Decode
            ))
        ));
    }

    #[test]
    fn neplproof_artifact_codec_rejects_valid_payload_with_wrong_hash() {
        let header = test_neplproof_header();
        let empty =
            ResourceSummaryProofArtifact::new(header, ResourceSummaryProofSnapshot::default())
                .to_neplproof_bytes()
                .expect("empty proof artifact should encode");

        let mut snapshot = ResourceSummaryProofSnapshot::default();
        let key = ResourceSummaryValueCacheKey::new_raw_alias_return_entry(
            1,
            ResourceSummaryFunctionIdentity::new("f", "f").unwrap(),
            2,
            3,
            4,
            5,
            6,
        );
        snapshot.raw_alias_return_entries.insert(
            key,
            ResourceSummaryStableRawAliasReturnEntry::empty_for_test(),
        );
        let non_empty = ResourceSummaryProofArtifact::new(header, snapshot)
            .to_neplproof_bytes()
            .expect("non-empty proof artifact should encode");

        let mut forged = empty[..NEPLPROOF_FIXED_HEADER_LEN].to_vec();
        forged.extend_from_slice(&non_empty[NEPLPROOF_FIXED_HEADER_LEN..]);

        let result = ResourceSummaryProofArtifact::from_neplproof_bytes_with_expected_header(
            &forged, header,
        );

        assert!(matches!(
            result,
            Err(ResourceSummaryProofArtifactByteReject::Codec(
                ResourceSummaryProofArtifactCodecError::Decode
            ))
        ));
    }

    #[test]
    fn neplproof_artifact_codec_rejects_stale_header_before_payload_decode() {
        let expected = test_neplproof_header();
        let mut stale = expected;
        stale.target_hash += 1;
        let artifact =
            ResourceSummaryProofArtifact::new(stale, ResourceSummaryProofSnapshot::default());
        let mut bytes = artifact
            .to_neplproof_bytes()
            .expect("stale proof artifact should encode");
        bytes.truncate(NEPLPROOF_FIXED_HEADER_LEN);
        bytes.extend_from_slice(b"corrupted payload that must not be decoded");

        let result = ResourceSummaryProofArtifact::from_neplproof_bytes_with_expected_header(
            &bytes, expected,
        );

        assert!(matches!(
            result,
            Err(ResourceSummaryProofArtifactByteReject::Compatibility(
                ResourceSummaryProofArtifactCompatibilityReject::Target
            ))
        ));
    }

    #[test]
    fn neplproof_artifact_header_reports_private_effect_policy_mismatch() {
        let expected = test_neplproof_header();
        let mut stale = expected;
        stale.private_effect_policy_hash = Some(99);

        assert_eq!(
            stale.compatibility_reject(expected),
            Some(ResourceSummaryProofArtifactCompatibilityReject::PrivateEffectPolicy)
        );
    }
}
