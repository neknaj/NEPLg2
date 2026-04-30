use crate::types::TypeId;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{EffectOp, Place, PlaceProjection, ResourceOffset};
use super::place_utils::raw_memory_cell_place;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_external_io_initialized_effect(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        effect: &EffectOp,
        args: &[Place],
    ) {
        let EffectOp::ExternalIo { operation } = effect else {
            return;
        };
        match operation.as_str() {
            "fd_read" => self.apply_fd_read_initialized_effect(cells, raw_aliases, args),
            _ => {}
        }
    }

    fn apply_fd_read_initialized_effect(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        args: &[Place],
    ) {
        if let Some(nread) = args.get(3) {
            self.mark_raw_cell_initialized(cells, raw_aliases, nread, self.types.i32());
        }

        let Some(iovs) = args.get(1) else {
            return;
        };
        let iovs = raw_aliases.canonicalize(iovs);
        let iov_buffer_cell = raw_memory_cell_place(&iovs, self.types.i32());
        for buffer in raw_aliases.aliases_for(&iov_buffer_cell) {
            self.mark_unknown_offset_raw_cell_initialized(
                cells,
                raw_aliases,
                &buffer,
                self.types.i32(),
            );
        }
    }

    fn mark_raw_cell_initialized(
        &self,
        cells: &mut CellTable,
        raw_aliases: &RawCellAddressAliases,
        address: &Place,
        ty: TypeId,
    ) {
        let address = raw_aliases.canonicalize(address);
        let cell = raw_memory_cell_place(&address, ty);
        cells.mark_initialized(&cell);
    }

    fn mark_unknown_offset_raw_cell_initialized(
        &self,
        cells: &mut CellTable,
        raw_aliases: &RawCellAddressAliases,
        address: &Place,
        ty: TypeId,
    ) {
        let address = raw_aliases.canonicalize(address);
        let address = address.with_projection(
            PlaceProjection::StorageOffset(ResourceOffset { bytes: None }),
            ty,
        );
        let cell = raw_memory_cell_place(&address, ty);
        cells.mark_initialized(&cell);
    }
}
