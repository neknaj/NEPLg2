use super::cell_state::CellTable;
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::host_memory_contract::HostMemoryDirection;
use super::host_size_contract::{HostDependentLength, HostDependentMemorySpan, HostSizeOutput};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::raw_memory_cell_place;

impl ResourceCheckEngine<'_> {
    pub(super) fn record_host_size_output(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        output: &HostSizeOutput,
        args: &[Place],
    ) {
        let Some(address) = args.get(output.address_arg) else {
            return;
        };
        let address = raw_aliases.canonicalize(address);
        let cell = raw_memory_cell_place(&address, self.types.i32());
        raw_aliases.set_host_size_kind(&cell, output.kind);
    }

    pub(super) fn apply_dependent_host_memory_initialized_output(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        contract: &HostDependentMemorySpan,
        args: &[Place],
    ) {
        let Some(address) = args.get(contract.address_arg) else {
            return;
        };
        if contract.direction != HostMemoryDirection::Output {
            return;
        }
        let address = raw_aliases.canonicalize(address);
        match contract.length {
            HostDependentLength::HostSize(kind) => {
                for count in raw_aliases.host_size_places(kind) {
                    for alias in raw_aliases.aliases_for(&address) {
                        cells.add_initialized_raw_byte_range(
                            &alias,
                            &count,
                            InitializedRawRangeUnit::Bytes,
                            self.types.i32(),
                        );
                    }
                }
            }
            HostDependentLength::HostSizeScaled {
                kind,
                bytes_per_item,
            } => {
                for count in raw_aliases.host_size_places(kind) {
                    for alias in raw_aliases.aliases_for(&address) {
                        cells.add_initialized_raw_byte_range(
                            &alias,
                            &count,
                            InitializedRawRangeUnit::Elements {
                                stride: bytes_per_item,
                            },
                            self.types.i32(),
                        );
                    }
                }
            }
        }
    }
}
