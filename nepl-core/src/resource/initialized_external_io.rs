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
        let operation = match effect {
            EffectOp::ExternalIo { operation } | EffectOp::Nondet { operation } => {
                operation.as_str()
            }
            _ => return,
        };
        match operation {
            "fd_read" => self.apply_fd_read_initialized_effect(cells, raw_aliases, args),
            "fd_pread" => {
                self.apply_iov_read_buffers_initialized(cells, raw_aliases, args.get(1));
                if let Some(nread) = args.get(4) {
                    self.mark_raw_cell_initialized(cells, raw_aliases, nread, self.types.i32());
                }
            }
            "fd_write" | "fd_pwrite" => {
                let nwritten_index = if operation == "fd_pwrite" { 4 } else { 3 };
                if let Some(nwritten) = args.get(nwritten_index) {
                    self.mark_raw_cell_initialized(cells, raw_aliases, nwritten, self.types.i32());
                }
            }
            "fd_readdir" => {
                self.mark_unknown_offset_raw_cell_initialized_for_arg(
                    cells,
                    raw_aliases,
                    args.get(1),
                    self.types.i32(),
                );
                if let Some(bufused) = args.get(4) {
                    self.mark_raw_cell_initialized(cells, raw_aliases, bufused, self.types.i32());
                }
            }
            "path_open" => {
                if let Some(opened_fd) = args.get(8) {
                    self.mark_raw_cell_initialized(cells, raw_aliases, opened_fd, self.types.i32());
                }
            }
            "args_sizes_get" | "environ_sizes_get" => {
                if let Some(count) = args.first() {
                    self.mark_raw_cell_initialized(cells, raw_aliases, count, self.types.i32());
                }
                if let Some(size) = args.get(1) {
                    self.mark_raw_cell_initialized(cells, raw_aliases, size, self.types.i32());
                }
            }
            "args_get" | "environ_get" => {
                self.mark_unknown_offset_raw_cell_initialized_for_arg(
                    cells,
                    raw_aliases,
                    args.first(),
                    self.types.i32(),
                );
                self.mark_unknown_offset_raw_cell_initialized_for_arg(
                    cells,
                    raw_aliases,
                    args.get(1),
                    self.types.i32(),
                );
            }
            "fd_fdstat_get" | "fd_filestat_get" | "fd_prestat_get" => {
                self.mark_unknown_offset_raw_cell_initialized_for_arg(
                    cells,
                    raw_aliases,
                    args.get(1),
                    self.types.i32(),
                );
            }
            "fd_prestat_dir_name" => {
                self.mark_unknown_offset_raw_cell_initialized_for_arg(
                    cells,
                    raw_aliases,
                    args.get(1),
                    self.types.i32(),
                );
            }
            "path_filestat_get" => {
                self.mark_unknown_offset_raw_cell_initialized_for_arg(
                    cells,
                    raw_aliases,
                    args.get(4),
                    self.types.i32(),
                );
            }
            "random_get" => {
                self.mark_unknown_offset_raw_cell_initialized_for_arg(
                    cells,
                    raw_aliases,
                    args.first(),
                    self.types.i32(),
                );
            }
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

        self.apply_iov_read_buffers_initialized(cells, raw_aliases, args.get(1));
    }

    fn apply_iov_read_buffers_initialized(
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

    fn mark_raw_cell_initialized(
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

    fn mark_unknown_offset_raw_cell_initialized_for_arg(
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
