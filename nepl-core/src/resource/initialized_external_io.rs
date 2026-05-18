use super::cell_state::CellTable;
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::host_memory_contract::{
    host_memory_spans, HostMemoryDirectUnit, HostMemoryDirection, HostMemorySpan,
};
use super::host_size_contract::{dependent_host_memory_spans, host_size_outputs};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{EffectOp, Place};

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_external_io_initialized_effect(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        effect: &EffectOp,
        args: &[Place],
    ) {
        for contract in host_memory_spans(effect) {
            self.apply_host_memory_initialized_output(cells, raw_aliases, contract, args);
        }
        for output in host_size_outputs(effect) {
            self.record_host_size_output(raw_aliases, output, args);
        }
        for contract in dependent_host_memory_spans(effect) {
            self.apply_dependent_host_memory_initialized_output(cells, raw_aliases, contract, args);
        }
    }

    fn apply_host_memory_initialized_output(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        contract: &HostMemorySpan,
        args: &[Place],
    ) {
        match *contract {
            HostMemorySpan::Direct {
                address_arg,
                length,
                unit: HostMemoryDirectUnit::I32Cell,
                direction: HostMemoryDirection::Output,
            } => {
                if let Some(address) = args.get(address_arg) {
                    self.mark_raw_cell_initialized(cells, raw_aliases, address, self.types.i32());
                }
                let _ = length;
            }
            HostMemorySpan::Direct {
                address_arg,
                length,
                unit: HostMemoryDirectUnit::Bytes,
                direction: HostMemoryDirection::Output,
            } => {
                let Some(address) = args.get(address_arg) else {
                    return;
                };
                let Some(length) = length.resolve(args, self.types.i32(), raw_aliases) else {
                    return;
                };
                let address = raw_aliases.canonicalize(address);
                for alias in raw_aliases.aliases_for(&address) {
                    cells.add_initialized_raw_byte_range(
                        &alias,
                        &length,
                        InitializedRawRangeUnit::Bytes,
                        self.types.i32(),
                    );
                }
            }
            HostMemorySpan::IovPayload {
                iovs_arg,
                iov_count_arg,
                transferred_count_arg,
                direction: HostMemoryDirection::Output,
            } => self.apply_iov_read_buffers_initialized(
                cells,
                raw_aliases,
                args.get(iovs_arg),
                args.get(iov_count_arg),
                transferred_count_arg.and_then(|index| args.get(index)),
            ),
            HostMemorySpan::Direct {
                direction: HostMemoryDirection::Input,
                ..
            }
            | HostMemorySpan::IovPayload {
                direction: HostMemoryDirection::Input,
                ..
            }
            | HostMemorySpan::IovDescriptor { .. } => {}
        }
    }
}
