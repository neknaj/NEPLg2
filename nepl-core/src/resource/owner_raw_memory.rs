use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, OwnerStorageExtent, Place, RawMemoryOp};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::raw_memory_cell_place;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn check_raw_memory_op(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        operation: RawMemoryOp,
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        match operation {
            RawMemoryOp::Alloc => {
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
                let extent = args
                    .first()
                    .map(OwnerStorageExtent::payload_bytes)
                    .unwrap_or(OwnerStorageExtent::Unknown);
                owners.allocate_with_extent(output, extent);
                raw_aliases.mark(output);
                raw_views.clear(output);
                storage_origins.mark_owned(output);
            }
            RawMemoryOp::Dealloc => {
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
                if let [ptr, size, ..] = args {
                    if !variant_owner_effects.reject_reserved_source_use(
                        self,
                        owners,
                        raw_aliases,
                        ptr,
                        ResourceOwnerOperation::Dealloc,
                        span,
                    ) {
                        self.release_owner_with_extent(
                            owners,
                            raw_aliases,
                            raw_views,
                            storage_origins,
                            ptr,
                            size,
                            ResourceOwnerOperation::Dealloc,
                            ResourceOwnerOperation::DeallocExtent,
                            span,
                        );
                    }
                } else if let Some(ptr) = args.first() {
                    if !variant_owner_effects.reject_reserved_source_use(
                        self,
                        owners,
                        raw_aliases,
                        ptr,
                        ResourceOwnerOperation::Dealloc,
                        span,
                    ) {
                        self.release_owner(
                            owners,
                            raw_aliases,
                            raw_views,
                            storage_origins,
                            ptr,
                            ResourceOwnerOperation::Dealloc,
                            span,
                        );
                    }
                }
            }
            RawMemoryOp::Realloc => {
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
                if let [ptr, old_size, new_size, ..] = args {
                    if !variant_owner_effects.reject_reserved_source_use(
                        self,
                        owners,
                        raw_aliases,
                        ptr,
                        ResourceOwnerOperation::ReallocInput,
                        span,
                    ) && self.ensure_owner_available_with_extent(
                        owners,
                        raw_aliases,
                        raw_views,
                        storage_origins,
                        ptr,
                        old_size,
                        ResourceOwnerOperation::ReallocInput,
                        ResourceOwnerOperation::ReallocExtent,
                        span,
                    ) {
                        owners.set_state(output, OwnerState::MaybeFreed { storage: None });
                        raw_aliases.mark(output);
                        raw_views.clear(output);
                        pending_reallocs.mark(
                            ptr,
                            output,
                            OwnerStorageExtent::payload_bytes(new_size),
                        );
                    }
                } else if let Some(ptr) = args.first() {
                    if !variant_owner_effects.reject_reserved_source_use(
                        self,
                        owners,
                        raw_aliases,
                        ptr,
                        ResourceOwnerOperation::ReallocInput,
                        span,
                    ) && self.ensure_owner_available(
                        owners,
                        raw_aliases,
                        raw_views,
                        storage_origins,
                        ptr,
                        ResourceOwnerOperation::ReallocInput,
                        span,
                    ) {
                        owners.set_state(output, OwnerState::MaybeFreed { storage: None });
                        raw_aliases.mark(output);
                        raw_views.clear(output);
                        pending_reallocs.mark(ptr, output, OwnerStorageExtent::Unknown);
                    }
                }
            }
            RawMemoryOp::Load | RawMemoryOp::LoadU8 => {
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
                if let Some(address) = args.first() {
                    variant_owner_effects.reject_reserved_source_use(
                        self,
                        owners,
                        raw_aliases,
                        address,
                        ResourceOwnerOperation::RawMemoryLoadCell,
                        span,
                    );
                    let address = raw_aliases.canonicalize_owner_cell_address(address);
                    let cell_ty = match operation {
                        RawMemoryOp::Load => output.ty,
                        RawMemoryOp::LoadU8 => self.types.u8(),
                        _ => unreachable!("load branch contains only raw load operations"),
                    };
                    let cell = raw_memory_cell_place(&address, cell_ty);
                    if self.raw_memory_load_is_non_owning_raw_address_view(
                        owners,
                        raw_aliases,
                        &cell,
                        output.ty,
                    ) {
                        raw_aliases.copy_alias_if_tracked(&cell, output);
                        storage_origins.copy_origin(&cell, output);
                        raw_views.mark_non_owning(output);
                    } else {
                        raw_aliases.copy_scalar_facts_if_tracked(&cell, output);
                        self.transfer_owner(
                            owners,
                            raw_aliases,
                            raw_views,
                            storage_origins,
                            &cell,
                            output,
                            ResourceOwnerOperation::RawMemoryLoadCell,
                            span,
                        );
                        raw_views.clear(output);
                    }
                }
            }
            RawMemoryOp::Store | RawMemoryOp::StoreU8 => {
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
                if let [address, value, ..] = args {
                    variant_owner_effects.reject_reserved_source_use(
                        self,
                        owners,
                        raw_aliases,
                        address,
                        ResourceOwnerOperation::CallArgument,
                        span,
                    );
                    let address = raw_aliases.canonicalize_owner_cell_address(address);
                    let cell_ty = match operation {
                        RawMemoryOp::Store => value.ty,
                        RawMemoryOp::StoreU8 => self.types.u8(),
                        _ => unreachable!("store branch contains only raw store operations"),
                    };
                    let cell = raw_memory_cell_place(&address, cell_ty);
                    self.report_overwritten_owners(
                        owners,
                        raw_aliases,
                        storage_origins,
                        &cell,
                        value,
                        span,
                    );
                    let value_reserved = variant_owner_effects.reject_reserved_source_use(
                        self,
                        owners,
                        raw_aliases,
                        value,
                        ResourceOwnerOperation::RawMemoryStoreValue,
                        span,
                    );
                    if !value_reserved
                        && self.raw_store_value_is_non_owning_raw_address_view(
                            owners,
                            raw_aliases,
                            raw_views,
                            value,
                        )
                    {
                        raw_aliases.copy_scalar_facts_if_tracked(value, &cell);
                        raw_aliases.copy_alias_if_tracked(value, &cell);
                        storage_origins.copy_origin(value, &cell);
                    } else if !value_reserved {
                        raw_aliases.copy_scalar_facts_if_tracked(value, &cell);
                        self.transfer_owner(
                            owners,
                            raw_aliases,
                            raw_views,
                            storage_origins,
                            value,
                            &cell,
                            ResourceOwnerOperation::RawMemoryStoreValue,
                            span,
                        );
                    }
                }
            }
            RawMemoryOp::BulkCopy
            | RawMemoryOp::BulkMove
            | RawMemoryOp::MemorySize
            | RawMemoryOp::MemoryGrow
            | RawMemoryOp::FillBytes
            | RawMemoryOp::Fill => {
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
            }
        }
    }
}
