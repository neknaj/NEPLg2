use alloc::vec::Vec;

use super::model::{Place, ResourceOp};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_summary_raw_alias_walk::collect_raw_owner_aliases_with_views;
use super::summary::OwnerReturnSummaryIndex;

pub(super) fn collect_raw_owner_aliases(
    ops: &[ResourceOp],
    aliases: &mut Vec<Place>,
    summaries: &OwnerReturnSummaryIndex<'_>,
) {
    let mut raw_views = RawAddressViewTable::default();
    collect_raw_owner_aliases_with_views(ops, aliases, &mut raw_views, summaries);
}
