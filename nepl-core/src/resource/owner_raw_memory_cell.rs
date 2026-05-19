use crate::span::Span;

use super::host_memory_contract::HostMemoryDirection;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, RawMemoryOp};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::raw_memory_cell_place;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn check_raw_memory_cell_op(
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
        pending_reallocs.clear_result(output);
        variant_owner_effects.clear_result(output);
        match operation {
            RawMemoryOp::Load | RawMemoryOp::LoadU8 => self.check_raw_memory_load_cell(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                variant_owner_effects,
                operation,
                output,
                args,
                span,
            ),
            RawMemoryOp::Store | RawMemoryOp::StoreU8 => self.check_raw_memory_store_cell(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                variant_owner_effects,
                operation,
                output,
                args,
                span,
            ),
            _ => {}
        }
    }

    fn check_raw_memory_load_cell(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        operation: RawMemoryOp,
        output: &Place,
        args: &[Place],
        span: Span,
    ) {
        let Some(address) = args.first() else {
            return;
        };
        self.ensure_raw_memory_cell_extent_available(
            owners,
            raw_aliases,
            raw_views,
            operation,
            output,
            address,
            HostMemoryDirection::Input,
            span,
        );
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

    fn check_raw_memory_store_cell(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        operation: RawMemoryOp,
        _output: &Place,
        args: &[Place],
        span: Span,
    ) {
        let [address, value, ..] = args else {
            return;
        };
        self.ensure_raw_memory_cell_extent_available(
            owners,
            raw_aliases,
            raw_views,
            operation,
            value,
            address,
            HostMemoryDirection::Output,
            span,
        );
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
        self.report_overwritten_owners(owners, raw_aliases, storage_origins, &cell, value, span);
        let value_reserved = variant_owner_effects.reject_reserved_source_use(
            self,
            owners,
            raw_aliases,
            value,
            ResourceOwnerOperation::RawMemoryStoreValue,
            span,
        );
        if value_reserved {
            return;
        }
        if self.raw_store_value_is_non_owning_raw_address_view(
            owners,
            raw_aliases,
            raw_views,
            value,
        ) {
            raw_aliases.copy_scalar_facts_if_tracked(value, &cell);
            raw_aliases.copy_alias_if_tracked(value, &cell);
            storage_origins.copy_origin(value, &cell);
        } else {
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
