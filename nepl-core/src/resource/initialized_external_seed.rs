use crate::resource_primitives::type_is_raw_pointer;
use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::compiler_memory_place::mem_ptr_raw_field_place;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

impl ResourceCheckEngine<'_> {
    pub(super) fn seed_external_raw_storage_parameter(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        place: &Place,
    ) {
        seed_external_raw_storage_input_place(self.types, cells, raw_aliases, place);
    }
}

pub(super) fn seed_external_raw_storage_input_place(
    types: &TypeCtx,
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    place: &Place,
) {
    cells.mark_external_raw_storage_root(place);
    raw_aliases.mark(place);
    if type_is_raw_pointer(types, place.ty) {
        let raw = mem_ptr_raw_field_place(types, place, types.i32());
        cells.mark_external_raw_storage_root(&raw);
        raw_aliases.mark(&raw);
    }
}
