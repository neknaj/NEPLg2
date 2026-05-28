use crate::types::TypeCtx;

use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryOp;

mod key;
mod stable_mirror;

use self::stable_mirror::stable_drop_traversal_forall_value;

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
}

impl ResourceSummaryValueCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.stats = ResourceSummaryValueCacheStats::default();
    }

    pub fn stats(&self) -> ResourceSummaryValueCacheStats {
        self.stats
    }

    /// `DropTraversal + ForallInitializedRange` は初期 stable mirror の対象である。
    ///
    /// 現 checkpoint では key/value の再投影をまだ実装しないため、stable mirror へ
    /// 変換できる候補だけを bypass として数える。これにより、compiled-output cache
    /// ではなく Resource summary value cache がどの程度効き得るかを timing JSON から
    /// 確認できる。
    pub(super) fn record_drop_traversal_forall_bypass_if_stable(
        &mut self,
        types: &TypeCtx,
        op: &CollectionSlotLifecycleSummaryOp,
    ) {
        if stable_drop_traversal_forall_value(types, op).is_none() {
            return;
        }
        self.stats.resource_summary_value_bypasses += 1;
        self.stats
            .resource_summary_value_drop_traversal_forall_bypasses += 1;
    }
}
