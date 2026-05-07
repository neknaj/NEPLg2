use super::cell_state::CellTable;
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_external_io_iov_layout::{
    iov_buffer_pointer_cells, raw_cell_is_under_any_address,
};
use super::model::Place;
use super::place_utils::raw_memory_cell_place;

impl ResourceCheckEngine<'_> {
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
        if raw_aliases.i32_value(iov_count) != Some(1) {
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
}
