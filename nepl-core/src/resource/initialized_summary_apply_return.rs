use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary_byte_range_model::RawCellInitializationReturnCount;
use super::model::Place;
use super::place_utils::projected_place_with_concrete_type;

pub(super) fn apply_return_initialization_summary(
    types: &TypeCtx,
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    summary: &RawCellInitializationFunctionSummary,
) {
    if !summary.return_cells.is_empty() {
        mark_known_raw_address(raw_aliases, output);
    }
    for cell in &summary.return_cells {
        let place = projected_place_with_concrete_type(types, output, &cell.suffix, cell.ty);
        cells.mark_initialized(&place);
        if cell.holds_raw_address {
            mark_known_raw_address(raw_aliases, &place);
        }
    }
    for range in &summary.return_byte_ranges {
        let address = projected_place_with_concrete_type(
            types,
            output,
            &range.address_suffix,
            range.address_ty,
        );
        let count = match &range.count {
            RawCellInitializationReturnCount::ReturnValueProjection { suffix, ty } => {
                projected_place_with_concrete_type(types, output, suffix, *ty)
            }
            RawCellInitializationReturnCount::KnownI32 { value, ty } => {
                Place::i32_constant(*value, *ty)
            }
        };
        let count = raw_aliases.canonicalize_scalar(&count);
        cells.mark_initialized_raw_byte_range(&address, &count, range.unit, range.ty);
    }
}

pub(super) fn mark_known_raw_address(raw_aliases: &mut RawCellAddressAliases, place: &Place) {
    raw_aliases.ensure_marked(place);
}
