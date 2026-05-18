use super::model::{Place, ResourceCallTarget};
use super::owner_return_apply_place::owner_projection_source_place_for_arg;
use super::owner_summary_raw_transfer::place_matches_any_alias;
use super::summary::{OwnerProjectionSource, OwnerReturnSummary, OwnerReturnSummaryIndex};

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
