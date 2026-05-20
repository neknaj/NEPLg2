use crate::span::Span;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::raw_cell_lifecycle::RawCellLifecycleEvent;
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
            if let (Some(count), Some(_value)) = (args.get(1), args.get(2)) {
                cells.apply_raw_cell_lifecycle_event(
                    RawCellLifecycleEvent::FillBytes {
                        address: &address,
                        count,
                    },
                    raw_aliases,
                    self.types,
                );
            } else {
                cells.apply_raw_cell_lifecycle_event(
                    RawCellLifecycleEvent::DiscardCellsUnderAddress { address: &address },
                    raw_aliases,
                    self.types,
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
            if let (Some(count), Some(value)) = (args.get(1), args.get(2)) {
                cells.apply_raw_cell_lifecycle_event(
                    RawCellLifecycleEvent::FillCopyElements {
                        address: &address,
                        count,
                        value_ty: value.ty,
                    },
                    raw_aliases,
                    self.types,
                );
            } else {
                cells.apply_raw_cell_lifecycle_event(
                    RawCellLifecycleEvent::DiscardCellsUnderAddress { address: &address },
                    raw_aliases,
                    self.types,
                );
            }
            cells.mark_initialized(output);
            raw_aliases.clear(output);
        }
    }
}
