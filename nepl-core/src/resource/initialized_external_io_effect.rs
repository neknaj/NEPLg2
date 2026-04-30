use crate::types::TypeId;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection, ResourceOffset};
use super::place_utils::raw_memory_cell_place;

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

        self.apply_iov_read_buffers_initialized(cells, raw_aliases, args.get(1));
    }

    pub(super) fn apply_iov_read_buffers_initialized(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        iovs: Option<&Place>,
    ) {
        let Some(iovs) = iovs else {
            return;
        };
        let iovs = raw_aliases.canonicalize(iovs);
        let iov_buffer_cell = raw_memory_cell_place(&iovs, self.types.i32());
        for buffer in raw_aliases.aliases_for(&iov_buffer_cell) {
            if buffer == iov_buffer_cell {
                continue;
            }
            self.mark_unknown_offset_raw_cell_initialized(
                cells,
                raw_aliases,
                &buffer,
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
            let address = alias.with_projection(
                PlaceProjection::StorageOffset(ResourceOffset { bytes: None }),
                ty,
            );
            let cell = raw_memory_cell_place(&address, ty);
            cells.mark_initialized(&cell);
        }
    }
}
