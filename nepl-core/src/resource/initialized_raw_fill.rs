use crate::layout::storage_size_bytes;
use crate::span::Span;

use super::cell_state::CellTable;
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckOperation;

impl ResourceCheckEngine<'_> {
    pub(super) fn check_raw_memory_fill_bytes(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        pending_reallocs: &mut PendingRawReallocs,
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        pending_reallocs.clear_result(output);
        let Some(address) = args.first() else {
            cells.mark_initialized(output);
            raw_aliases.clear(output);
            return;
        };
        let address = raw_aliases.canonicalize(address);
        let address_available = self.ensure_available(
            cells,
            &address,
            ResourceCheckOperation::RawMemoryFillAddress,
            span,
        );
        let cells_released = self.ensure_no_live_non_copy_raw_cells(
            cells,
            &address,
            ResourceCheckOperation::RawMemoryFillCell,
            span,
        );
        if address_available && cells_released {
            cells.clear_raw_cells_under(&address);
            if let (Some(count), Some(value)) = (args.get(1), args.get(2)) {
                cells.mark_initialized_raw_byte_range_extending_appended_difference(
                    &address,
                    count,
                    InitializedRawRangeUnit::Bytes,
                    value.ty,
                    raw_aliases,
                );
            }
            cells.mark_initialized(output);
            raw_aliases.clear(output);
        }
    }

    pub(super) fn check_raw_memory_fill_words(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        pending_reallocs: &mut PendingRawReallocs,
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        pending_reallocs.clear_result(output);
        let Some(address) = args.first() else {
            cells.mark_initialized(output);
            raw_aliases.clear(output);
            return;
        };
        let address = raw_aliases.canonicalize(address);
        let address_available = self.ensure_available(
            cells,
            &address,
            ResourceCheckOperation::RawMemoryFillAddress,
            span,
        );
        let cells_released = self.ensure_no_live_non_copy_raw_cells(
            cells,
            &address,
            ResourceCheckOperation::RawMemoryFillCell,
            span,
        );
        if address_available && cells_released {
            cells.clear_raw_cells_under(&address);
            if let (Some(count), Some(value)) = (args.get(1), args.get(2)) {
                cells.mark_initialized_raw_byte_range(
                    &address,
                    count,
                    InitializedRawRangeUnit::Elements {
                        stride: storage_size_bytes(self.types, value.ty),
                    },
                    value.ty,
                );
            }
            cells.mark_initialized(output);
            raw_aliases.clear(output);
        }
    }
}
