use crate::span::Span;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::raw_cell_lifecycle::RawCellLifecycleEvent;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckOperation;

impl ResourceCheckEngine<'_> {
    pub(super) fn check_raw_memory_bulk_copy_or_move(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        pending_reallocs: &mut PendingRawReallocs,
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        pending_reallocs.clear_result(output);
        let Some(destination) = args.first() else {
            cells.mark_initialized(output);
            raw_aliases.clear(output);
            return;
        };
        let Some(source) = args.get(1) else {
            cells.mark_initialized(output);
            raw_aliases.clear(output);
            return;
        };
        let destination = raw_aliases.canonicalize(destination);
        let source = raw_aliases.canonicalize(source);
        let count = args
            .get(2)
            .map(|count| raw_aliases.canonicalize_scalar(count));
        let destination_available = self.ensure_available(
            cells,
            &destination,
            ResourceCheckOperation::RawMemoryBulkDestinationAddress,
            span,
        );
        let source_available = self.ensure_available(
            cells,
            &source,
            ResourceCheckOperation::RawMemoryBulkSourceAddress,
            span,
        );
        let count_available = count.as_ref().is_none_or(|count| {
            self.ensure_available(
                cells,
                count,
                ResourceCheckOperation::RawMemoryBulkCount,
                span,
            )
        });
        let destination_cells_released = self.ensure_no_live_non_copy_raw_cells(
            cells,
            &destination,
            ResourceCheckOperation::RawMemoryBulkDestinationCell,
            span,
        );
        let source_cells_copyable = self.ensure_no_live_non_copy_raw_cells(
            cells,
            &source,
            ResourceCheckOperation::RawMemoryBulkSourceCell,
            span,
        );
        if destination_available
            && source_available
            && count_available
            && destination_cells_released
            && source_cells_copyable
        {
            cells.apply_raw_cell_lifecycle_event(
                RawCellLifecycleEvent::BulkCopyInitializedRawState {
                    source: &source,
                    destination: &destination,
                    count: count.as_ref(),
                },
                raw_aliases,
                self.types,
            );
            cells.mark_initialized(output);
            raw_aliases.clear(output);
        }
    }
}
