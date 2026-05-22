extern crate alloc;

use alloc::vec::Vec;

use super::initialized_summary_byte_range_model::RawCellInitializationVariantParamByteRange;
use super::initialized_summary_variant_model::RawCellInitializationVariantParamCell;

pub(super) fn push_unique_variant_param_cell(
    cells: &mut Vec<RawCellInitializationVariantParamCell>,
    cell: RawCellInitializationVariantParamCell,
) {
    if !cells.iter().any(|existing| existing == &cell) {
        cells.push(cell);
    }
}

pub(super) fn push_unique_variant_param_byte_range(
    ranges: &mut Vec<RawCellInitializationVariantParamByteRange>,
    range: RawCellInitializationVariantParamByteRange,
) {
    if !ranges.iter().any(|existing| existing == &range) {
        ranges.push(range);
    }
}
