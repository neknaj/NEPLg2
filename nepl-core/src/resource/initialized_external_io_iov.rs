use crate::span::Span;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_external_io_iov_layout::{
    iov_buffer_pointer_cells, iov_length_cell, raw_cell_is_under_any_address,
};
use super::model::{CellState, Place};
use super::place_utils::raw_memory_unknown_offset_cell_place;
use super::report::ResourceCheckOperation;

impl ResourceCheckEngine<'_> {
    pub(super) fn ensure_iov_descriptor_cells_available(
        &mut self,
        cells: &CellTable,
        raw_aliases: &RawCellAddressAliases,
        iovs: Option<&Place>,
        span: Span,
    ) -> bool {
        let Some(iovs) = iovs else {
            return true;
        };
        let iovs = raw_aliases.canonicalize(iovs);
        let mut available = true;
        for buffer_cell in iov_buffer_pointer_cells(raw_aliases, &iovs, self.types.i32()) {
            available &= self.ensure_available(
                cells,
                &buffer_cell,
                ResourceCheckOperation::RawMemoryLoadCell,
                span,
            );
            if let Some(length_cell) = iov_length_cell(&buffer_cell, self.types.i32()) {
                available &= self.ensure_available(
                    cells,
                    &length_cell,
                    ResourceCheckOperation::RawMemoryLoadCell,
                    span,
                );
            }
        }
        available
    }

    pub(super) fn ensure_iov_write_buffers_available(
        &mut self,
        cells: &CellTable,
        raw_aliases: &RawCellAddressAliases,
        iovs: Option<&Place>,
        span: Span,
    ) -> bool {
        let Some(iovs) = iovs else {
            return true;
        };
        let iovs = raw_aliases.canonicalize(iovs);
        let iov_aliases = raw_aliases.aliases_for(&iovs);
        let mut available =
            self.ensure_iov_descriptor_cells_available(cells, raw_aliases, Some(&iovs), span);
        for buffer_cell in iov_buffer_pointer_cells(raw_aliases, &iovs, self.types.i32()) {
            let length_cell = iov_length_cell(&buffer_cell, self.types.i32())
                .map(|cell| raw_aliases.canonicalize(&cell));
            for buffer in raw_aliases.aliases_for(&buffer_cell) {
                if buffer == buffer_cell || raw_cell_is_under_any_address(&buffer, &iov_aliases) {
                    continue;
                }
                available &= self.ensure_iov_payload_buffer_available(
                    cells,
                    raw_aliases,
                    &buffer,
                    length_cell.as_ref(),
                    span,
                );
            }
        }
        available
    }

    fn ensure_iov_payload_buffer_available(
        &mut self,
        cells: &CellTable,
        raw_aliases: &RawCellAddressAliases,
        buffer: &Place,
        length: Option<&Place>,
        span: Span,
    ) -> bool {
        let candidates = raw_aliases.aliases_for(buffer);
        if candidates
            .iter()
            .any(|candidate| cells.raw_cell_is_untracked_external(candidate))
        {
            return true;
        }
        if candidates.iter().any(|candidate| {
            let cell = raw_memory_unknown_offset_cell_place(candidate, self.types.i32());
            matches!(
                cells.availability_state_with_types(self.types, &cell),
                CellState::Initialized(_)
            )
        }) {
            return true;
        }
        if let Some(length) = length {
            if candidates.iter().any(|candidate| {
                cells.raw_range_initialized_for_count(
                    candidate,
                    length,
                    self.types.i32(),
                    raw_aliases,
                )
            }) {
                return true;
            }
        }
        let cell = raw_memory_unknown_offset_cell_place(buffer, self.types.i32());
        self.ensure_available(
            cells,
            &cell,
            ResourceCheckOperation::RawMemoryLoadCell,
            span,
        )
    }
}
