use crate::span::Span;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerStorageExtent, Place, RawMemoryOp};
use super::raw_cell_lifecycle::RawCellLifecycleEvent;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckOperation;

impl ResourceCheckEngine<'_> {
    pub(super) fn check_raw_memory(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
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
            RawMemoryOp::Load | RawMemoryOp::LoadU8 => {
                pending_reallocs.clear_result(output);
                let cell_ty = match operation {
                    RawMemoryOp::Load => output.ty,
                    RawMemoryOp::LoadU8 => self.types.u8(),
                    _ => unreachable!("load branch contains only raw load operations"),
                };
                self.check_raw_memory_load(cells, raw_aliases, output, args, cell_ty, span);
            }
            RawMemoryOp::Store | RawMemoryOp::StoreU8 => {
                pending_reallocs.clear_result(output);
                let cell_ty = match operation {
                    RawMemoryOp::Store => args.get(1).map(|value| value.ty),
                    RawMemoryOp::StoreU8 => Some(self.types.u8()),
                    _ => unreachable!("store branch contains only raw store operations"),
                };
                self.check_raw_memory_store(cells, raw_aliases, output, args, cell_ty, span);
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
                let collection_slots_released = self.release_collection_slots_for_raw_dealloc(
                    collection_slots,
                    raw_aliases,
                    &address,
                    span,
                );
                if address_available && cells_released && collection_slots_released {
                    let owner_address = raw_aliases.canonicalize_owner_cell_address(&address);
                    cells.apply_raw_cell_lifecycle_event(
                        RawCellLifecycleEvent::ReleaseStorage { address: &address },
                        raw_aliases,
                        self.types,
                    );
                    pending_reallocs.certify_release(&address);
                    pending_reallocs.certify_release(&owner_address);
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
                let collection_managed_non_copy_cells = self
                    .certified_collection_managed_non_copy_raw_cells_for_realloc(
                        cells,
                        collection_slots,
                        &address,
                        span,
                    );
                if address_available && collection_managed_non_copy_cells.is_some() {
                    cells.mark_initialized(output);
                    raw_aliases.mark(output);
                    pending_reallocs.mark(
                        &address,
                        output,
                        OwnerStorageExtent::Unknown,
                        collection_managed_non_copy_cells.unwrap_or_default(),
                    );
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
                self.check_raw_memory_bulk_copy_or_move(
                    cells,
                    raw_aliases,
                    pending_reallocs,
                    output,
                    args,
                    span,
                );
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
