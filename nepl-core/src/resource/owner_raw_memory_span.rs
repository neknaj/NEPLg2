use crate::layout::storage_size_bytes;
use crate::span::Span;

use super::host_memory_contract::{
    HostMemoryDirectUnit, HostMemoryDirection, HostMemoryInitializedLength, HostMemoryLength,
    HostMemorySpan,
};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, RawMemoryOp};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::report::ResourceOwnerOperation;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn ensure_raw_memory_cell_extent_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        operation: RawMemoryOp,
        cell: &Place,
        address: &Place,
        direction: HostMemoryDirection,
        span: Span,
    ) -> bool {
        let bytes = match operation {
            RawMemoryOp::LoadU8 | RawMemoryOp::StoreU8 => 1,
            RawMemoryOp::Load | RawMemoryOp::Store => storage_size_bytes(self.types, cell.ty),
            _ => return true,
        };
        let Some(bytes) = i32::try_from(bytes).ok().filter(|bytes| *bytes > 0) else {
            return true;
        };
        self.ensure_raw_memory_byte_span_available(
            owners,
            raw_aliases,
            raw_views,
            HostMemoryLength::ConstI32(bytes),
            &[address.clone()],
            direction,
            span,
        )
    }

    pub(super) fn ensure_raw_memory_byte_span_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        length: HostMemoryLength,
        args: &[Place],
        direction: HostMemoryDirection,
        span: Span,
    ) -> bool {
        let contract = HostMemorySpan::Direct {
            address_arg: 0,
            length,
            initialized_length: HostMemoryInitializedLength::SameAsLength,
            unit: HostMemoryDirectUnit::Bytes,
            direction,
        };
        self.ensure_memory_contract_owner_span_available(
            owners,
            raw_aliases,
            raw_views,
            &contract,
            args,
            ResourceOwnerOperation::RawMemoryPayloadExtent,
            span,
        )
    }
}
