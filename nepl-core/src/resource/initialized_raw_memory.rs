use crate::span::Span;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerStorageExtent, Place, RawMemoryOp};
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckOperation;

impl ResourceCheckEngine<'_> {
    pub(super) fn check_raw_memory(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        pending_reallocs: &mut PendingRawReallocs,
        operation: &RawMemoryOp,
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        match operation {
            RawMemoryOp::Alloc => {
                pending_reallocs.clear_result(output);
                let args_available =
                    self.ensure_args(cells, args, ResourceCheckOperation::RawMemoryArgument, span);
                if args_available {
                    cells.mark_initialized(output);
                    cells.mark_owned_raw_storage_root(output);
                    raw_aliases.mark(output);
                }
            }
            RawMemoryOp::Load => {
                pending_reallocs.clear_result(output);
                self.check_raw_memory_load(cells, raw_aliases, output, args, span);
            }
            RawMemoryOp::Store => {
                pending_reallocs.clear_result(output);
                self.check_raw_memory_store(cells, raw_aliases, output, args, span);
            }
            RawMemoryOp::Dealloc => {
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
                    ResourceCheckOperation::RawMemoryDeallocAddress,
                    span,
                );
                let cells_released = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryDeallocCell,
                    span,
                );
                if address_available && cells_released {
                    cells.clear_raw_cells_under(&address);
                    cells.release_owned_raw_storage_under(&address);
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                }
            }
            RawMemoryOp::Realloc => {
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
                    ResourceCheckOperation::RawMemoryReallocAddress,
                    span,
                );
                let cells_released = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryReallocCell,
                    span,
                );
                if address_available && cells_released {
                    cells.mark_initialized(output);
                    raw_aliases.mark(output);
                    pending_reallocs.mark(&address, output, OwnerStorageExtent::Unknown);
                }
            }
            RawMemoryOp::FillBytes => {
                self.check_raw_memory_fill_bytes(
                    cells,
                    raw_aliases,
                    pending_reallocs,
                    output,
                    args,
                    span,
                );
            }
            RawMemoryOp::Fill => {
                self.check_raw_memory_fill_words(
                    cells,
                    raw_aliases,
                    pending_reallocs,
                    output,
                    args,
                    span,
                );
            }
            RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => {
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
                    && destination_cells_released
                    && source_cells_copyable
                {
                    let copied =
                        cells.copy_initialized_copy_raw_cells(&source, &destination, self.types);
                    cells.clear_raw_cells_under(&destination);
                    cells.extend_entries(copied);
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                }
            }
            _ => {
                pending_reallocs.clear_result(output);
                let args_available =
                    self.ensure_args(cells, args, ResourceCheckOperation::RawMemoryArgument, span);
                if args_available {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                }
            }
        }
    }
}
