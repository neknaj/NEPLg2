use crate::types::{TypeCtx, TypeId};

use super::cell_state::CellTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::type_can_seed_raw_address_alias;

pub(super) fn seed_summary_input_place(
    types: &TypeCtx,
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    place: &Place,
) {
    cells.mark_initialized(place);
    if summary_input_type_may_seed_raw_address_alias(types, place.ty) {
        raw_aliases.mark(place);
    }
}

pub(super) fn summary_input_type_may_seed_raw_address_alias(types: &TypeCtx, ty: TypeId) -> bool {
    types.resolve_id(ty) == types.i32() || type_can_seed_raw_address_alias(types, ty)
}
