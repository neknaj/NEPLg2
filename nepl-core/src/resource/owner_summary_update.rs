extern crate alloc;

use alloc::vec::Vec;

use super::owner_summary_canonicalize::canonicalize_owner_return_summary;
use super::summary::OwnerReturnSummary;
use super::summary_index::SummaryNameIndex;

#[cfg(test)]
pub(super) fn update_owner_return_summary(
    summaries: &mut Vec<OwnerReturnSummary>,
    summary: OwnerReturnSummary,
) -> bool {
    let mut summary_name_index = SummaryNameIndex::from_entries(summaries);
    update_owner_return_summary_with_index(summaries, &mut summary_name_index, summary)
}

pub(super) fn update_owner_return_summary_with_index(
    summaries: &mut Vec<OwnerReturnSummary>,
    summary_name_index: &mut SummaryNameIndex,
    mut summary: OwnerReturnSummary,
) -> bool {
    canonicalize_owner_return_summary(&mut summary);
    let function = summary.function.clone();
    let position = summary_name_index.position(&function);
    match (owner_return_summary_has_facts(&summary), position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
        }
        (true, None) => {
            summary_name_index.insert_at_end(&function, summaries.len());
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
            summary_name_index.remove_and_shift(&function, index);
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
        || !summary.parameter_return_extents.is_empty()
        || !summary.consumed_parameter_indices.is_empty()
        || !summary.consumed_parameter_sources.is_empty()
        || !summary.consumed_extent_requirements.is_empty()
        || !summary.memory_span_requirements.is_empty()
        || !summary.host_size_returns.is_empty()
        || !summary.type_size_returns.is_empty()
        || !summary.variant_consumed_parameter_indices.is_empty()
        || !summary.variant_consumed_parameter_sources.is_empty()
        || !summary.variant_consumed_extent_requirements.is_empty()
        || !summary.variant_projection_returns.is_empty()
        || !summary.resolved_parameter_variants.is_empty()
        || !summary.variant_conditions.is_empty()
        || !summary.variant_payload_conditions.is_empty()
        || !summary.projection_returns.is_empty()
        || !summary.projection_markers.is_empty()
        || !summary.storage_origin_markers.is_empty()
}

#[cfg(test)]
#[path = "owner_summary_update_tests.rs"]
mod tests;
