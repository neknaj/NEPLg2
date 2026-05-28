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
/// 依存する Resource IR proof artifact はこの別 cache に置く。現段階では統計と
/// clear 境界を先に固定し、stable mirror value の保存は後続 checkpoint で追加する。
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
}
