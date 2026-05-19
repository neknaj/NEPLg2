use crate::span::Span;

use super::host_memory_contract::HostMemorySpan;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::report::ResourceOwnerOperation;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn ensure_memory_contract_owner_span_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        contract: &HostMemorySpan,
        args: &[Place],
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        match *contract {
            HostMemorySpan::Direct { .. } => self
                .ensure_direct_memory_contract_owner_span_available(
                    owners,
                    raw_aliases,
                    raw_views,
                    contract,
                    args,
                    operation,
                    span,
                ),
            HostMemorySpan::IovPayload {
                iovs_arg,
                iov_count_arg,
                direction,
                ..
            } => self.ensure_iov_payload_owner_extents_available(
                owners,
                raw_aliases,
                raw_views,
                args.get(iovs_arg),
                args.get(iov_count_arg),
                direction,
                operation,
                span,
            ),
            HostMemorySpan::IovDescriptor {
                iovs_arg,
                iov_count_arg,
            } => self.ensure_iov_descriptor_owner_extent_available(
                owners,
                raw_aliases,
                raw_views,
                args,
                iovs_arg,
                iov_count_arg,
                contract,
                operation,
                span,
            ),
        }
    }
}
