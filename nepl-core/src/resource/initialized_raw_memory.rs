use crate::span::Span;
use crate::types::TypeKind;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, RawMemoryOp};
use super::place_utils::{raw_memory_cell_place, raw_memory_unknown_offset_cell_place};
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
                let Some(address_arg) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let address_available = self.ensure_available(
                    cells,
                    address_arg,
                    ResourceCheckOperation::RawMemoryLoadAddress,
                    span,
                );
                let address = raw_aliases.canonicalize(address_arg);
                let cell = raw_memory_cell_place(&address, output.ty);
                let loaded_from_untracked_external = raw_aliases
                    .aliases_for(&address)
                    .iter()
                    .any(|alias| cells.raw_cell_is_untracked_external(alias));
                let cell_available = loaded_from_untracked_external
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
                        raw_aliases.clear(output);
                    }
                }
            }
            RawMemoryOp::Store => {
                pending_reallocs.clear_result(output);
                let Some(address_arg) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let address_available = self.ensure_available(
                    cells,
                    address_arg,
                    ResourceCheckOperation::RawMemoryStoreAddress,
                    span,
                );
                let address = raw_aliases.canonicalize(address_arg);
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
                        let value_is_known_raw_address =
                            raw_aliases.value_is_known_raw_address(value);
                        cells.clear_raw_cells_under(&address);
                        cells.mark_initialized(&cell);
                        raw_aliases.clear(&cell);
                        if value_is_known_raw_address {
                            raw_aliases.copy_alias_or_seed(value, &cell);
                        }
                    }
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                }
            }
            RawMemoryOp::Dealloc => {
                pending_reallocs.clear_result(output);
                let Some(address_arg) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let address_available = self.ensure_available(
                    cells,
                    address_arg,
                    ResourceCheckOperation::RawMemoryDeallocAddress,
                    span,
                );
                let address = raw_aliases.canonicalize(address_arg);
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
                let Some(address_arg) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let address_available = self.ensure_available(
                    cells,
                    address_arg,
                    ResourceCheckOperation::RawMemoryReallocAddress,
                    span,
                );
                let address = raw_aliases.canonicalize(address_arg);
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
            RawMemoryOp::Fill { unit } => {
                pending_reallocs.clear_result(output);
                let Some(address_arg) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let address_available = self.ensure_available(
                    cells,
                    address_arg,
                    ResourceCheckOperation::RawMemoryFillAddress,
                    span,
                );
                let address = raw_aliases.canonicalize(address_arg);
                let cells_released = self.ensure_no_live_non_copy_raw_cells(
                    cells,
                    &address,
                    ResourceCheckOperation::RawMemoryFillCell,
                    span,
                );
                if address_available && cells_released {
                    let address_aliases = raw_aliases.aliases_for(&address);
                    for alias in &address_aliases {
                        cells.clear_raw_cells_under(alias);
                    }
                    if let Some(value) = args.get(2) {
                        let cell = raw_memory_unknown_offset_cell_place(&address, value.ty);
                        cells.mark_initialized(&cell);
                        if let Some(count) =
                            args.get(1).and_then(|count| raw_aliases.i32_value(count))
                        {
                            for alias in &address_aliases {
                                cells
                                    .mark_raw_fill_range_initialized(alias, *unit, count, value.ty);
                            }
                        }
                    }
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                }
            }
            RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => {
                pending_reallocs.clear_result(output);
                let Some(destination_arg) = args.first() else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let Some(source_arg) = args.get(1) else {
                    cells.mark_initialized(output);
                    raw_aliases.clear(output);
                    return;
                };
                let destination_available = self.ensure_available(
                    cells,
                    destination_arg,
                    ResourceCheckOperation::RawMemoryBulkDestinationAddress,
                    span,
                );
                let source_available = self.ensure_available(
                    cells,
                    source_arg,
                    ResourceCheckOperation::RawMemoryBulkSourceAddress,
                    span,
                );
                let destination = raw_aliases.canonicalize(destination_arg);
                let source = raw_aliases.canonicalize(source_arg);
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
