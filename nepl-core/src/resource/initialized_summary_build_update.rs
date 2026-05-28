extern crate alloc;

use alloc::vec::Vec;

use super::initialized_summary::RawCellInitializationFunctionSummary;

pub(super) fn update_raw_cell_initialization_summary(
    summaries: &mut Vec<RawCellInitializationFunctionSummary>,
    summary: RawCellInitializationFunctionSummary,
) -> bool {
    let has_facts = !summary.return_cells.is_empty()
        || !summary.return_byte_ranges.is_empty()
        || !summary.param_cells.is_empty()
        || !summary.param_byte_ranges.is_empty()
        || !summary.param_release_requirements.is_empty()
        || !summary.variant_param_cells.is_empty()
        || !summary.variant_param_byte_ranges.is_empty()
        || !summary.variant_required_param_cells.is_empty()
        || !summary.variant_conditions.is_empty();
    let position = summaries
        .iter()
        .position(|existing| existing.function == summary.function);
    match (has_facts, position) {
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
