use crate::types::TypeId;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::{raw_memory_cell_place, raw_memory_unknown_offset_cell_place};

impl ResourceCheckEngine<'_> {
    pub(super) fn mark_raw_cell_initialized(
        &self,
        cells: &mut CellTable,
        raw_aliases: &RawCellAddressAliases,
        address: &Place,
        ty: TypeId,
    ) {
        let address = raw_aliases.canonicalize(address);
        for alias in raw_aliases.aliases_for(&address) {
            let cell = raw_memory_cell_place(&alias, ty);
            cells.mark_initialized(&cell);
        }
    }

    pub(super) fn mark_unknown_offset_raw_cell_initialized_for_arg(
        &self,
        cells: &mut CellTable,
        raw_aliases: &RawCellAddressAliases,
        address: Option<&Place>,
        ty: TypeId,
    ) {
        if let Some(address) = address {
            self.mark_unknown_offset_raw_cell_initialized(cells, raw_aliases, address, ty);
        }
    }

    fn mark_unknown_offset_raw_cell_initialized(
        &self,
        cells: &mut CellTable,
        raw_aliases: &RawCellAddressAliases,
        address: &Place,
        ty: TypeId,
    ) {
        let address = raw_aliases.canonicalize(address);
        for alias in raw_aliases.aliases_for(&address) {
            let cell = raw_memory_unknown_offset_cell_place(&alias, ty);
            cells.mark_initialized(&cell);
        }
    }
}
