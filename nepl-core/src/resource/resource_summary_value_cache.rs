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
    drop_traversal_forall_candidate_key_and_value, ResourceSummaryGenericTypeArgumentKeyInput,
};
use self::key::ResourceSummaryValueCacheKey;
use self::stable_mirror::ResourceSummaryStableDropTraversalForallValue;

/// Resource IR summary value cache の累積統計。
///
/// この統計は compiled-output cache とは別に、Resource IR の証明結果そのものが
/// 再利用されたかを観測するために使う。初期実装では安全に保存できる stable
/// mirror がまだ限定されるため、hit/store だけでなく bypass も明示的に数える。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ResourceSummaryValueCacheStats {
    pub resource_summary_value_hits: usize,
    pub resource_summary_value_misses: usize,
    pub resource_summary_value_stores: usize,
    pub resource_summary_value_bypasses: usize,
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
    drop_traversal_forall_values:
        BTreeMap<ResourceSummaryValueCacheKey, Vec<ResourceSummaryStableDropTraversalForallValue>>,
}

#[derive(Debug, Clone)]
pub(super) struct ResourceSummaryDropTraversalForallCandidate {
    key: ResourceSummaryValueCacheKey,
    value: ResourceSummaryStableDropTraversalForallValue,
}

impl ResourceSummaryValueCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.stats = ResourceSummaryValueCacheStats::default();
        self.drop_traversal_forall_values.clear();
    }

    pub fn stats(&self) -> ResourceSummaryValueCacheStats {
        self.stats
    }

    /// `DropTraversal + ForallInitializedRange` の store/hit 候補を作る。
    ///
    /// hit した stable value を現在の `CollectionSlotLifecycleSummaryOp` へ戻す逆投影は
    /// まだ実装しない。この段階では、session をまたいで同じ pure summary query が
    /// 再利用可能だったことを統計として観測し、stale hit になり得る value は保存しない。
    pub(super) fn drop_traversal_forall_candidate(
        &self,
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
        type_params: &[TypeId],
        op: &CollectionSlotLifecycleSummaryOp,
    ) -> Option<ResourceSummaryDropTraversalForallCandidate> {
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
        let (key, value) = drop_traversal_forall_candidate_key_and_value(
            types,
            context.namespace_hash(),
            source_capability_policy_hash,
            function,
            type_params,
            generic_type_args,
            op,
        )?;
        Some(ResourceSummaryDropTraversalForallCandidate { key, value })
    }

    pub(super) fn record_drop_traversal_forall_bypass(&mut self) {
        self.stats.resource_summary_value_bypasses += 1;
        self.stats
            .resource_summary_value_drop_traversal_forall_bypasses += 1;
    }

    /// keyable な `DropTraversal + ForallInitializedRange` 候補を session cache に記録する。
    ///
    /// hit 判定は、この呼び出しが始まる前から cache に存在した value だけを対象にする。
    /// 同じ summary build pass 内で先に store した value を即 hit と数えると、微小変更時の
    /// 再利用可能性を過大評価するためである。
    pub(super) fn record_drop_traversal_forall_candidates(
        &mut self,
        candidates: Vec<ResourceSummaryDropTraversalForallCandidate>,
    ) {
        let candidates_with_hits = candidates
            .into_iter()
            .map(|candidate| {
                let existed_before_recording = self
                    .drop_traversal_forall_values
                    .get(&candidate.key)
                    .is_some_and(|values| values.contains(&candidate.value));
                (candidate, existed_before_recording)
            })
            .collect::<Vec<_>>();

        for (candidate, existed_before_recording) in candidates_with_hits {
            if existed_before_recording {
                self.stats.resource_summary_value_hits += 1;
                self.stats.resource_summary_value_drop_traversal_forall_hits += 1;
                continue;
            }

            self.stats.resource_summary_value_misses += 1;
            let values = self
                .drop_traversal_forall_values
                .entry(candidate.key)
                .or_default();
            if !values.contains(&candidate.value) {
                values.push(candidate.value);
                self.stats.resource_summary_value_stores += 1;
                self.stats
                    .resource_summary_value_drop_traversal_forall_stores += 1;
            }
        }
    }
}
