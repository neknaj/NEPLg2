use alloc::vec::Vec;

use super::model::{Place, ResourceOp};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_summary_raw_transfer::push_transferred_value_aliases_from;
use super::owner_summary_raw_use_walk::ops_use_raw_owner_alias_with_views;
use super::summary::OwnerReturnSummaryIndex;

pub(super) fn branch_uses_raw_owner_alias(
    aliases: &mut Vec<Place>,
    raw_views: &mut RawAddressViewTable,
    output: &Place,
    then_ops: &[ResourceOp],
    then_value: &Place,
    else_ops: &[ResourceOp],
    else_value: &Place,
    summaries: &OwnerReturnSummaryIndex<'_>,
) -> bool {
    let mut then_aliases = aliases.clone();
    let mut then_raw_views = raw_views.clone();
    if ops_use_raw_owner_alias_with_views(
        then_ops,
        &mut then_aliases,
        &mut then_raw_views,
        summaries,
    ) {
        return true;
    }
    let mut else_aliases = aliases.clone();
    let mut else_raw_views = raw_views.clone();
    if ops_use_raw_owner_alias_with_views(
        else_ops,
        &mut else_aliases,
        &mut else_raw_views,
        summaries,
    ) {
        return true;
    }
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
    false
}
