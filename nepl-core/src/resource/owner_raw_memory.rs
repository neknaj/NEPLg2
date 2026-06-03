use alloc::vec::Vec;

use crate::span::Span;

use super::host_memory_contract::{HostMemoryDirection, HostMemoryLength};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, OwnerStorageExtent, Place, RawMemoryOp};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
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
                        let storage_source = storage_origins
                            .origin_source(ptr)
                            .unwrap_or_else(|| raw_aliases.canonicalize_owner_cell_address(ptr));
                        pending_reallocs.mark(
                            ptr,
                            &storage_source,
                            output,
                            OwnerStorageExtent::payload_bytes(new_size),
                            Vec::new(),
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
                        let storage_source = storage_origins
                            .origin_source(ptr)
                            .unwrap_or_else(|| raw_aliases.canonicalize_owner_cell_address(ptr));
                        pending_reallocs.mark(
                            ptr,
                            &storage_source,
                            output,
                            OwnerStorageExtent::Unknown,
                            Vec::new(),
                        );
                    }
                }
            }
            RawMemoryOp::Load | RawMemoryOp::LoadU8 => {
                self.check_raw_memory_cell_op(
                    owners,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    pending_reallocs,
                    variant_owner_effects,
                    operation,
                    output,
                    args,
                    span,
                );
            }
            RawMemoryOp::Store | RawMemoryOp::StoreU8 => {
                self.check_raw_memory_cell_op(
                    owners,
                    raw_aliases,
                    raw_views,
                    storage_origins,
                    pending_reallocs,
                    variant_owner_effects,
                    operation,
                    output,
                    args,
                    span,
                );
            }
            RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => {
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
                if let [destination, source, length, ..] = args {
                    self.ensure_raw_memory_byte_span_available(
                        owners,
                        raw_aliases,
                        raw_views,
                        HostMemoryLength::Arg(1),
                        &[destination.clone(), length.clone()],
                        HostMemoryDirection::Output,
                        span,
                    );
                    self.ensure_raw_memory_byte_span_available(
                        owners,
                        raw_aliases,
                        raw_views,
                        HostMemoryLength::Arg(1),
                        &[source.clone(), length.clone()],
                        HostMemoryDirection::Input,
                        span,
                    );
                }
            }
            RawMemoryOp::FillBytes => {
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
                if let [destination, length, ..] = args {
                    self.ensure_raw_memory_byte_span_available(
                        owners,
                        raw_aliases,
                        raw_views,
                        HostMemoryLength::Arg(1),
                        &[destination.clone(), length.clone()],
                        HostMemoryDirection::Output,
                        span,
                    );
                }
            }
            RawMemoryOp::Fill => {
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
                if let [destination, count, ..] = args {
                    self.ensure_raw_memory_byte_span_available(
                        owners,
                        raw_aliases,
                        raw_views,
                        HostMemoryLength::ArgScaled {
                            arg: 1,
                            bytes_per_item: 4,
                        },
                        &[destination.clone(), count.clone()],
                        HostMemoryDirection::Output,
                        span,
                    );
                }
            }
            RawMemoryOp::MemorySize | RawMemoryOp::MemoryGrow => {
                pending_reallocs.clear_result(output);
                variant_owner_effects.clear_result(output);
            }
        }
    }
}
