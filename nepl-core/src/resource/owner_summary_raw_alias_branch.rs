use alloc::vec::Vec;

use super::model::{Place, ResourceOp};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_summary_raw_alias_walk::collect_raw_owner_aliases_with_views;
use super::owner_summary_raw_transfer::push_transferred_value_aliases_from;
use super::summary::OwnerReturnSummaryIndex;

pub(super) fn collect_branch_raw_owner_aliases(
    aliases: &mut Vec<Place>,
    raw_views: &mut RawAddressViewTable,
    output: &Place,
    then_ops: &[ResourceOp],
    then_value: &Place,
    else_ops: &[ResourceOp],
    else_value: &Place,
    summaries: &OwnerReturnSummaryIndex<'_>,
) {
    let mut then_aliases = aliases.clone();
    let mut then_raw_views = raw_views.clone();
    collect_raw_owner_aliases_with_views(
        then_ops,
        &mut then_aliases,
        &mut then_raw_views,
        summaries,
    );
    let mut else_aliases = aliases.clone();
    let mut else_raw_views = raw_views.clone();
    collect_raw_owner_aliases_with_views(
        else_ops,
        &mut else_aliases,
        &mut else_raw_views,
        summaries,
    );
    push_transferred_value_aliases_from(
        aliases,
        raw_views,
        then_value,
        output,
        &then_aliases,
        &then_raw_views,
    );
    let mut else_output_raw_views = raw_views.clone();
    push_transferred_value_aliases_from(
        aliases,
        &mut else_output_raw_views,
        else_value,
        output,
        &else_aliases,
        &else_raw_views,
    );
    *raw_views = RawAddressViewTable::merge_paths(&[raw_views.clone(), else_output_raw_views]);
}
