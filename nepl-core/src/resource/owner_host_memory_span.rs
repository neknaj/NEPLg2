use crate::span::Span;

use super::host_memory_contract::HostMemorySpan;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_state::OwnerTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn ensure_host_memory_contract_owner_span_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        contract: &HostMemorySpan,
        args: &[Place],
        span: Span,
    ) -> bool {
        match *contract {
            HostMemorySpan::Direct {
                address_arg,
                length,
                direction,
                ..
            } => {
                let Some(address) = args.get(address_arg) else {
                    return true;
                };
                let Some(length) = length.resolve(args, self.types.i32(), raw_aliases) else {
                    if self.try_record_deferred_host_memory_span_requirement(
                        raw_aliases,
                        contract,
                        args,
                    ) {
                        return true;
                    }
                    return true;
                };
                self.ensure_external_io_payload_extent_available(
                    owners,
                    raw_aliases,
                    address,
                    &length,
                    direction,
                    span,
                )
            }
            HostMemorySpan::IovPayload {
                iovs_arg,
                iov_count_arg,
                direction,
                ..
            } => self.ensure_iov_payload_owner_extents_available(
                owners,
                raw_aliases,
                args.get(iovs_arg),
                args.get(iov_count_arg),
                direction,
                span,
            ),
            HostMemorySpan::IovDescriptor {
                iovs_arg,
                iov_count_arg,
            } => self.ensure_iov_descriptor_owner_extent_available(
                owners,
                raw_aliases,
                args,
                iovs_arg,
                iov_count_arg,
                contract,
                span,
            ),
        }
    }
}
