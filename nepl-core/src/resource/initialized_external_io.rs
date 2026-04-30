use crate::span::Span;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{EffectOp, Place};

impl ResourceCheckEngine<'_> {
    pub(super) fn ensure_external_io_initialized_inputs(
        &mut self,
        cells: &CellTable,
        raw_aliases: &RawCellAddressAliases,
        effect: &EffectOp,
        args: &[Place],
        span: Span,
    ) -> bool {
        let operation = match effect {
            EffectOp::ExternalIo { operation } | EffectOp::Nondet { operation } => {
                operation.as_str()
            }
            _ => return true,
        };
        match operation {
            "fd_read" | "fd_pread" => {
                self.ensure_iov_descriptor_cells_available(cells, raw_aliases, args.get(1), span)
            }
            "fd_write" | "fd_pwrite" => {
                self.ensure_iov_write_buffers_available(cells, raw_aliases, args.get(1), span)
            }
            _ => true,
        }
    }

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
}
