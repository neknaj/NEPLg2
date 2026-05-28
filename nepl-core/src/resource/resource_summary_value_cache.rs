extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryOp;
use super::model::ResourceFunction;

mod body_hash;
mod candidate_key;
mod context;
mod key;
mod stable_hash;
mod stable_mirror;
mod stable_type_key;
mod type_boundary;

pub use self::context::ResourceSummaryValueCacheContext;

use self::candidate_key::{
    drop_traversal_forall_leaf_entry_candidate_key_and_entry,
    ResourceSummaryGenericTypeArgumentKeyInput,
};
use self::key::ResourceSummaryValueCacheKey;
use self::stable_mirror::{
    reproject_drop_traversal_forall_leaf_entry, ResourceSummaryStableDropTraversalForallLeafEntry,
    ResourceSummaryTypeReprojection,
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
    pub resource_summary_value_hits: usize,
    pub resource_summary_value_misses: usize,
    pub resource_summary_value_stores: usize,
    pub resource_summary_value_bypasses: usize,
    pub resource_summary_value_replay_hits: usize,
    pub resource_summary_value_replay_bypasses: usize,
    pub resource_summary_value_replayed_ops: usize,
    pub resource_summary_value_recomputed_ops: usize,
    pub resource_summary_value_drop_traversal_forall_hits: usize,
    pub resource_summary_value_drop_traversal_forall_stores: usize,
    pub resource_summary_value_drop_traversal_forall_bypasses: usize,
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
}

#[derive(Debug, Clone)]
pub(super) struct ResourceSummaryDropTraversalForallLeafEntryCandidate {
    key: ResourceSummaryValueCacheKey,
    entry: ResourceSummaryStableDropTraversalForallLeafEntry,
}

impl ResourceSummaryValueCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.stats = ResourceSummaryValueCacheStats::default();
        self.drop_traversal_forall_leaf_entries.clear();
    }

    pub fn stats(&self) -> ResourceSummaryValueCacheStats {
        self.stats
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
