use crate::span::Span;
use crate::types::TypeKind;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, RawMemoryOp};
use super::place_utils::raw_memory_cell_place;
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
                let Some(address) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let address = raw_aliases.canonicalize(address);
                let address_available = self.ensure_available(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryLoadAddress,
                    span,
                );
                let cell = raw_memory_cell_place(&address, output.ty);
                let loaded_from_untracked_external = raw_aliases
                    .aliases_for(&address)
                    .iter()
                    .any(|alias| cells.raw_cell_is_untracked_external(alias));
                let cell_available = loaded_from_untracked_external
                    || cells.raw_cell_initialized_by_byte_range(&address, output.ty, raw_aliases)
                    || self.ensure_available(
                        cells,
                        &cell,
                        ResourceCheckOperation::RawMemoryLoadCell,
                        span,
                    );
                if address_available && cell_available {
                    if !self.types.is_copy(output.ty) {
                        cells.mark_raw_cell_moved(&address, output.ty);
                    }
                    cells.mark_initialized(output);
                    cells.copy_initialized_raw_byte_range_counts(&cell, output);
                    if raw_aliases.value_is_known_raw_address(&cell) {
                        self.copy_raw_alias_and_rekey_cells_preferring_target(
                            cells,
                            raw_aliases,
                            &cell,
                            output,
                        );
                    } else if loaded_from_untracked_external
                        && self.output_can_hold_raw_address(output.ty)
                    {
                        cells.mark_external_raw_storage_root(output);
                        raw_aliases.mark(output);
                    } else {
                        raw_aliases.copy_alias_if_tracked(&cell, output);
                    }
                }
            }
            RawMemoryOp::Store => {
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
                    ResourceCheckOperation::RawMemoryStoreAddress,
                    span,
                );
                let cell_available = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryStoreCell,
                    span,
                );
                let value_available = if address_available && cell_available {
                    args.get(1).is_none_or(|value| {
                        self.consume_by_value(
                            cells,
                            value,
                            ResourceCheckOperation::RawMemoryStoreValue,
                            span,
                        )
                    })
                } else {
                    false
                };
                if address_available && cell_available && value_available {
                    if let Some(value) = args.get(1) {
                        let cell = raw_memory_cell_place(&address, value.ty);
                        cells.clear_raw_cells_overwritten_by_store(&address, value.ty, self.types);
                        cells.mark_initialized(&cell);
                        cells.copy_initialized_raw_byte_range_counts(value, &cell);
                        raw_aliases.clear(&cell);
                        raw_aliases.copy_alias_if_tracked(value, &cell);
                    }
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                }
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
                    pending_reallocs.mark(&address, output);
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

    fn output_can_hold_raw_address(&self, ty: crate::types::TypeId) -> bool {
        matches!(self.types.get_ref(self.types.resolve_id(ty)), TypeKind::I32)
    }
}
