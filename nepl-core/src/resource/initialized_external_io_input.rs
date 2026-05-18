use crate::span::Span;

use super::cell_state::CellTable;
use super::host_memory_contract::{
    host_memory_spans, HostMemoryDirectUnit, HostMemoryDirection, HostMemorySpan,
};
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
        let mut available = true;
        for contract in host_memory_spans(effect) {
            match *contract {
                HostMemorySpan::Direct {
                    address_arg,
                    length,
                    unit: HostMemoryDirectUnit::Bytes,
                    direction: HostMemoryDirection::Input,
                } => {
                    if let Some(address) = args.get(address_arg) {
                        available &= self.ensure_raw_payload_buffer_available(
                            cells,
                            raw_aliases,
                            address,
                            length.resolve(args, self.types.i32()).as_ref(),
                            span,
                        );
                    }
                }
                HostMemorySpan::Direct {
                    unit: HostMemoryDirectUnit::I32Cell,
                    direction: HostMemoryDirection::Input,
                    ..
                } => {}
                HostMemorySpan::Direct {
                    direction: HostMemoryDirection::Output,
                    ..
                } => {}
                HostMemorySpan::IovDescriptor { iovs_arg } => {
                    available &= self.ensure_iov_descriptor_cells_available(
                        cells,
                        raw_aliases,
                        args.get(iovs_arg),
                        span,
                    );
                }
                HostMemorySpan::IovPayload {
                    iovs_arg,
                    direction: HostMemoryDirection::Input,
                    ..
                } => {
                    available &= self.ensure_iov_write_buffers_available(
                        cells,
                        raw_aliases,
                        args.get(iovs_arg),
                        span,
                    );
                }
                HostMemorySpan::IovPayload {
                    direction: HostMemoryDirection::Output,
                    ..
                } => {}
            }
        }
        available
    }
}
