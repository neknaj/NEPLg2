use crate::span::Span;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{EffectOp, ExternalIoOp, Place};

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
}
