use crate::types::TypeId;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::raw_memory_cell_place;

impl ResourceCheckEngine<'_> {
    pub(super) fn mark_raw_cell_initialized(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        address: &Place,
        ty: TypeId,
    ) {
        let address = raw_aliases.canonicalize(address);
        for alias in raw_aliases.raw_address_aliases_for_value(&address) {
            let cell = raw_memory_cell_place(&alias, ty);
            raw_aliases.clear_scalar_facts(&cell);
            cells.mark_initialized(&cell);
        }
    }
}
