use alloc::vec::Vec;

use super::model::{Place, ResourceOp};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_summary_raw_use_walk::ops_use_raw_owner_alias_with_views;
use super::summary::OwnerReturnSummaryIndex;

pub(super) fn ops_use_raw_owner_alias(
    ops: &[ResourceOp],
    aliases: &mut Vec<Place>,
    summaries: &OwnerReturnSummaryIndex<'_>,
) -> bool {
    let mut raw_views = RawAddressViewTable::default();
    ops_use_raw_owner_alias_with_views(ops, aliases, &mut raw_views, summaries)
}
