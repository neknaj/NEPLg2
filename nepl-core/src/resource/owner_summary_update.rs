extern crate alloc;

use alloc::vec::Vec;

use super::summary::OwnerReturnSummary;

pub(super) fn update_owner_return_summary(
    summaries: &mut Vec<OwnerReturnSummary>,
    summary: OwnerReturnSummary,
) -> bool {
    let position = summaries
        .iter()
        .position(|existing| existing.function == summary.function);
    match (owner_return_summary_has_facts(&summary), position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
        }
        (true, None) => {
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
            true
        }
        (false, None) => false,
    }
}

fn owner_return_summary_has_facts(summary: &OwnerReturnSummary) -> bool {
    summary.returns_fresh_owner
        || summary.returns_maybe_owner
        || !summary.non_owning_raw_view_returns.is_empty()
        || !summary.parameter_indices.is_empty()
        || !summary.parameter_sources.is_empty()
        || !summary.consumed_parameter_indices.is_empty()
        || !summary.consumed_parameter_sources.is_empty()
        || !summary.variant_consumed_parameter_indices.is_empty()
        || !summary.variant_consumed_parameter_sources.is_empty()
        || !summary.variant_projection_returns.is_empty()
        || !summary.resolved_parameter_variants.is_empty()
        || !summary.variant_conditions.is_empty()
        || !summary.variant_payload_conditions.is_empty()
        || !summary.projection_returns.is_empty()
        || !summary.projection_markers.is_empty()
        || !summary.storage_origin_markers.is_empty()
}
