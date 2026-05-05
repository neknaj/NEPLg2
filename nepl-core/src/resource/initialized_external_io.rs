use crate::span::Span;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{EffectOp, ExternalIoOp, NondetOp, Place};

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
            EffectOp::ExternalIo { operation } => *operation,
            _ => return true,
        };
        match operation {
            ExternalIoOp::FdRead | ExternalIoOp::FdPread => {
                self.ensure_iov_descriptor_cells_available(cells, raw_aliases, args.get(1), span)
            }
            ExternalIoOp::FdWrite | ExternalIoOp::FdPwrite => {
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
        match effect {
            EffectOp::ExternalIo {
                operation: ExternalIoOp::FdRead,
            } => self.apply_fd_read_initialized_effect(cells, raw_aliases, args),
            EffectOp::ExternalIo {
                operation: ExternalIoOp::FdPread,
            } => {
                self.apply_iov_read_buffers_initialized(cells, raw_aliases, args.get(1));
                if let Some(nread) = args.get(4) {
                    self.mark_raw_cell_initialized(cells, raw_aliases, nread, self.types.i32());
                }
            }
            EffectOp::ExternalIo {
                operation: ExternalIoOp::FdWrite,
            } => {
                if let Some(nwritten) = args.get(3) {
                    self.mark_raw_cell_initialized(cells, raw_aliases, nwritten, self.types.i32());
                }
            }
            EffectOp::ExternalIo {
                operation: ExternalIoOp::FdPwrite,
            } => {
                if let Some(nwritten) = args.get(4) {
                    self.mark_raw_cell_initialized(cells, raw_aliases, nwritten, self.types.i32());
                }
            }
            EffectOp::ExternalIo {
                operation: ExternalIoOp::FdReaddir,
            } => {
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
            EffectOp::ExternalIo {
                operation: ExternalIoOp::PathOpen,
            } => {
                if let Some(opened_fd) = args.get(8) {
                    self.mark_raw_cell_initialized(cells, raw_aliases, opened_fd, self.types.i32());
                }
            }
            EffectOp::ExternalIo {
                operation: ExternalIoOp::ArgsSizesGet | ExternalIoOp::EnvironSizesGet,
            } => {
                if let Some(count) = args.first() {
                    self.mark_raw_cell_initialized(cells, raw_aliases, count, self.types.i32());
                }
                if let Some(size) = args.get(1) {
                    self.mark_raw_cell_initialized(cells, raw_aliases, size, self.types.i32());
                }
            }
            EffectOp::ExternalIo {
                operation: ExternalIoOp::ArgsGet | ExternalIoOp::EnvironGet,
            } => {
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
            EffectOp::ExternalIo {
                operation:
                    ExternalIoOp::FdFdstatGet | ExternalIoOp::FdFilestatGet | ExternalIoOp::FdPrestatGet,
            } => {
                self.mark_unknown_offset_raw_cell_initialized_for_arg(
                    cells,
                    raw_aliases,
                    args.get(1),
                    self.types.i32(),
                );
            }
            EffectOp::ExternalIo {
                operation: ExternalIoOp::FdPrestatDirName,
            } => {
                self.mark_unknown_offset_raw_cell_initialized_for_arg(
                    cells,
                    raw_aliases,
                    args.get(1),
                    self.types.i32(),
                );
            }
            EffectOp::ExternalIo {
                operation: ExternalIoOp::PathFilestatGet,
            } => {
                self.mark_unknown_offset_raw_cell_initialized_for_arg(
                    cells,
                    raw_aliases,
                    args.get(4),
                    self.types.i32(),
                );
            }
            EffectOp::Nondet {
                operation: NondetOp::RandomGet,
            } => {
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
