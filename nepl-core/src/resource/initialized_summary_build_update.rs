extern crate alloc;

use alloc::vec::Vec;

use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::summary_index::SummaryNameIndex;

pub(super) fn update_raw_cell_initialization_summary(
    summaries: &mut Vec<RawCellInitializationFunctionSummary>,
    summary_name_index: &mut SummaryNameIndex,
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
    let function = summary.function.clone();
    let position = summary_name_index.position(&function);
    match (has_facts, position) {
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
