use crate::types::TypeId;

use super::cell_state::CellTable;
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_external_io_iov_layout::{
    iov_buffer_pointer_cells, raw_cell_is_under_any_address,
};
use super::model::Place;
use super::place_utils::{raw_memory_cell_place, raw_memory_unknown_offset_cell_place};

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_fd_read_initialized_effect(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        args: &[Place],
    ) {
        if let Some(nread) = args.get(3) {
            self.mark_raw_cell_initialized(cells, raw_aliases, nread, self.types.i32());
        }

        self.apply_iov_read_buffers_initialized(
            cells,
            raw_aliases,
            args.get(1),
            args.get(2),
            args.get(3),
        );
    }

    pub(super) fn apply_iov_read_buffers_initialized(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        iovs: Option<&Place>,
        iov_count: Option<&Place>,
        nread: Option<&Place>,
    ) {
        let Some(iovs) = iovs else {
            return;
        };
        let iovs = raw_aliases.canonicalize(iovs);
        let Some(iov_count) = iov_count else {
            return;
        };
        if raw_aliases.i32_value(&raw_aliases.canonicalize(iov_count)) != Some(1) {
            return;
        }
        let Some(nread) = nread else {
            return;
        };
        let nread_count = raw_memory_cell_place(&raw_aliases.canonicalize(nread), self.types.i32());
        let iov_aliases = raw_aliases.aliases_for(&iovs);
        let Some(iov_buffer_cell) = iov_buffer_pointer_cells(raw_aliases, &iovs, self.types.i32())
            .into_iter()
            .find(|cell| *cell == raw_memory_cell_place(&iovs, self.types.i32()))
        else {
            return;
        };
        for buffer in raw_aliases.aliases_for(&iov_buffer_cell) {
            if buffer == iov_buffer_cell || raw_cell_is_under_any_address(&buffer, &iov_aliases) {
                continue;
            }
            cells.add_initialized_raw_byte_range(
                &buffer,
                &nread_count,
                InitializedRawRangeUnit::Bytes,
                self.types.i32(),
            );
        }
    }

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
