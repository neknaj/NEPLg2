use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary_apply_return::mark_known_raw_address;
use super::initialized_summary_byte_range_model::RawCellInitializationParamCount;
use super::model::Place;
use super::owner_extent_summary::instantiate_summary_type;
use super::summary_projection::instantiate_summary_suffix_on_base_with_types;

pub(super) fn apply_param_initialization_summary(
    types: &TypeCtx,
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    args: &[Place],
    type_args: &[crate::types::TypeId],
    summary: &RawCellInitializationFunctionSummary,
) {
    for cell in &summary.param_cells {
        let Some(arg) = args.get(cell.param_index) else {
            continue;
        };
        let arg = raw_aliases.canonicalize(arg);
        let cell_ty = instantiate_summary_type(&summary.type_params, type_args, cell.ty);
        let Some(place) = instantiate_summary_suffix_on_base_with_types(
            types,
            args,
            None,
            &arg,
            &cell.suffix,
            cell_ty,
        ) else {
            continue;
        };
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
        let Some(address) = instantiate_summary_suffix_on_base_with_types(
            types,
            args,
            None,
            &address_arg,
            &range.address_suffix,
            instantiate_summary_type(&summary.type_params, type_args, range.address_ty),
        ) else {
            continue;
        };
        let Some(count) = param_count_source_place(
            types,
            raw_aliases,
            args,
            &summary.type_params,
            type_args,
            &range.count,
        ) else {
            continue;
        };
        let count = raw_aliases.canonicalize_scalar(&count);
        cells.mark_initialized_raw_byte_range(
            &address,
            &count,
            range.unit,
            instantiate_summary_type(&summary.type_params, type_args, range.ty),
        );
    }
}

fn param_count_source_place(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    summary_type_params: &[crate::types::TypeId],
    type_args: &[crate::types::TypeId],
    count: &RawCellInitializationParamCount,
) -> Option<Place> {
    match count {
        RawCellInitializationParamCount::ParamProjection {
            param_index,
            suffix,
            ty,
        } => {
            let count_arg = raw_aliases.canonicalize_scalar(args.get(*param_index)?);
            instantiate_summary_suffix_on_base_with_types(
                types,
                args,
                None,
                &count_arg,
                suffix,
                instantiate_summary_type(summary_type_params, type_args, *ty),
            )
        }
        RawCellInitializationParamCount::KnownI32 { value, ty } => Some(Place::i32_constant(
            *value,
            instantiate_summary_type(summary_type_params, type_args, *ty),
        )),
    }
}
