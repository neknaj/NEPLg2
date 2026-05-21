extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::cell_state::CellTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::{push_unique_place, raw_memory_cell_place};
use super::raw_cell_value_flow_alias_candidates::raw_cell_alias_candidates;

impl CellTable {
    pub(super) fn mark_raw_cell_moved_with_aliases(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        address: &Place,
        ty: TypeId,
    ) {
        let cell = raw_memory_cell_place(address, ty);
        let mut addresses = Vec::new();
        push_unique_place(&mut addresses, address);
        for candidate in raw_cell_alias_candidates(&cell, raw_aliases) {
            if let Some(address) = super::cell_state::raw_cell_address_prefix(&candidate) {
                push_unique_place(&mut addresses, &address);
            }
        }
        for address in addresses {
            self.mark_raw_cell_moved(&address, ty);
        }
    }
}
