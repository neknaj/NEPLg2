use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary_apply_return::mark_known_raw_address;
use super::initialized_summary_byte_range_model::RawCellInitializationParamCount;
use super::model::Place;
use super::place_utils::projected_place_with_concrete_type;

pub(super) fn apply_param_initialization_summary(
    types: &TypeCtx,
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    args: &[Place],
    summary: &RawCellInitializationFunctionSummary,
) {
    for cell in &summary.param_cells {
        let Some(arg) = args.get(cell.param_index) else {
            continue;
        };
        let arg = raw_aliases.canonicalize(arg);
        let place = projected_place_with_concrete_type(types, &arg, &cell.suffix, cell.ty);
        cells.mark_initialized(&place);
        if cell.holds_raw_address {
            mark_known_raw_address(raw_aliases, &place);
        }
    }
    for range in &summary.param_byte_ranges {
        let Some(address_arg) = args.get(range.address_param_index) else {
            continue;
        };
        let address_arg = raw_aliases.canonicalize(address_arg);
        let address = projected_place_with_concrete_type(
            types,
            &address_arg,
            &range.address_suffix,
            range.address_ty,
        );
        let Some(count) = param_count_source_place(types, raw_aliases, args, &range.count) else {
            continue;
        };
        let count = raw_aliases.canonicalize_scalar(&count);
        cells.mark_initialized_raw_byte_range(&address, &count, range.unit, range.ty);
    }
}

fn param_count_source_place(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    count: &RawCellInitializationParamCount,
) -> Option<Place> {
    match count {
        RawCellInitializationParamCount::ParamProjection {
            param_index,
            suffix,
            ty,
        } => {
            let count_arg = raw_aliases.canonicalize_scalar(args.get(*param_index)?);
            Some(projected_place_with_concrete_type(
                types, &count_arg, suffix, *ty,
            ))
        }
        RawCellInitializationParamCount::KnownI32 { value, ty } => {
            Some(Place::i32_constant(*value, *ty))
        }
    }
}
