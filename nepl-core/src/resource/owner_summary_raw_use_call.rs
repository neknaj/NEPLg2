use alloc::vec::Vec;

use super::model::{Place, ResourceCallTarget};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_return_apply_source::owner_projection_source_place_for_arg;
use super::owner_summary_raw_transfer::{place_matches_any_alias, push_transferred_value_aliases};
use super::place_utils::place_with_suffix;
use super::summary::{
    OwnerProjectionReturnOwner, OwnerProjectionSource, OwnerReturnSummary, OwnerReturnSummaryIndex,
};

pub(super) fn direct_call_consumes_raw_owner_alias(
    target: &ResourceCallTarget,
    args: &[Place],
    aliases: &[Place],
    summaries: &OwnerReturnSummaryIndex<'_>,
) -> bool {
    let ResourceCallTarget::User { name, .. } = target else {
        return false;
    };
    summaries
        .get(name)
        .is_some_and(|summary| summary_consumes_raw_owner_alias(summary, args, aliases))
}

pub(super) fn push_direct_call_returned_raw_owner_aliases(
    output: &Place,
    target: &ResourceCallTarget,
    args: &[Place],
    aliases: &mut Vec<Place>,
    raw_views: &mut RawAddressViewTable,
    summaries: &OwnerReturnSummaryIndex<'_>,
) {
    let ResourceCallTarget::User { name, .. } = target else {
        return;
    };
    let Some(summary) = summaries.get(name) else {
        return;
    };
    push_summary_returned_raw_owner_aliases(output, args, aliases, raw_views, summary);
}

fn summary_consumes_raw_owner_alias(
    summary: &OwnerReturnSummary,
    args: &[Place],
    aliases: &[Place],
) -> bool {
    summary
        .consumed_parameter_indices
        .iter()
        .filter_map(|index| args.get(*index))
        .any(|arg| place_matches_any_alias(arg, aliases))
        || summary
            .consumed_parameter_sources
            .iter()
            .any(|source| source_alias_matches_arg(args, source, aliases))
        || summary
            .variant_consumed_parameter_indices
            .iter()
            .filter_map(|entry| args.get(entry.parameter_index))
            .any(|arg| place_matches_any_alias(arg, aliases))
        || summary
            .variant_consumed_parameter_sources
            .iter()
            .any(|entry| source_alias_matches_arg(args, &entry.source, aliases))
}

fn source_alias_matches_arg(
    args: &[Place],
    source: &OwnerProjectionSource,
    aliases: &[Place],
) -> bool {
    let Some(arg) = args.get(source.parameter_index) else {
        return false;
    };
    let source_place = owner_projection_source_place_for_arg(arg, source);
    place_matches_any_alias(&source_place, aliases)
}

fn push_summary_returned_raw_owner_aliases(
    output: &Place,
    args: &[Place],
    aliases: &mut Vec<Place>,
    raw_views: &mut RawAddressViewTable,
    summary: &OwnerReturnSummary,
) {
    for parameter_index in &summary.parameter_indices {
        let Some(arg) = args.get(*parameter_index) else {
            continue;
        };
        push_transferred_value_aliases(aliases, raw_views, arg, output);
    }
    for source in &summary.parameter_sources {
        push_parameter_source_returned_alias(output, args, aliases, raw_views, source);
    }
    for projection in &summary.projection_returns {
        let output_projection = place_with_suffix(output, &projection.suffix, projection.ty);
        for parameter_index in &projection.parameter_indices {
            let Some(arg) = args.get(*parameter_index) else {
                continue;
            };
            push_transferred_value_aliases(aliases, raw_views, arg, &output_projection);
        }
        for source in &projection.parameter_sources {
            push_parameter_source_returned_alias(
                &output_projection,
                args,
                aliases,
                raw_views,
                source,
            );
        }
    }
    for projection in &summary.variant_projection_returns {
        let OwnerProjectionReturnOwner::Parameter { source, .. } = &projection.owner else {
            continue;
        };
        let output_projection = place_with_suffix(output, &projection.suffix, projection.ty);
        push_parameter_source_returned_alias(&output_projection, args, aliases, raw_views, source);
    }
}

fn push_parameter_source_returned_alias(
    output: &Place,
    args: &[Place],
    aliases: &mut Vec<Place>,
    raw_views: &mut RawAddressViewTable,
    source: &OwnerProjectionSource,
) {
    let Some(arg) = args.get(source.parameter_index) else {
        return;
    };
    let source_place = owner_projection_source_place_for_arg(arg, source);
    push_transferred_value_aliases(aliases, raw_views, &source_place, output);
}
