use crate::types::{TypeCtx, TypeKind};

use super::cell_state::CellTable;
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{I32ValueCondition, Place, PlaceProjection, ResourceOffset};
use super::place_utils::raw_memory_cell_place;

pub(super) fn seed_str_storage_layout(
    types: &TypeCtx,
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    place: &Place,
) {
    if !type_is_str(types, place.ty) {
        return;
    }
    if !raw_aliases.contains_exact(place) {
        raw_aliases.mark(place);
    }
    let header_cell = raw_memory_cell_place(place, types.i32());
    cells.mark_initialized(&header_cell);
    raw_aliases.add_i32_condition(&header_cell, I32ValueCondition::NonNegative);

    let data = place.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        types.i32(),
    );
    cells.add_initialized_raw_byte_range(
        &data,
        &header_cell,
        InitializedRawRangeUnit::Bytes,
        types.u8(),
    );
    cells.add_initialized_raw_byte_range(
        &data,
        &header_cell,
        InitializedRawRangeUnit::Bytes,
        types.i32(),
    );
}

fn type_is_str(types: &TypeCtx, ty: crate::types::TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    matches!(types.get_ref(resolved), TypeKind::Str) || resolved == types.str()
}
