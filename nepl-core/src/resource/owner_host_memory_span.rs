use crate::span::Span;

use super::host_memory_contract::{HostMemoryLength, HostMemorySpan};
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
                ..
            } => {
                let Some(address) = args.get(address_arg) else {
                    return true;
                };
                let Some(length) = length.resolve(args, self.types.i32(), raw_aliases) else {
                    return true;
                };
                self.ensure_external_io_payload_extent_available(
                    owners,
                    raw_aliases,
                    address,
                    &length,
                    span,
                )
            }
            HostMemorySpan::IovPayload { iovs_arg, .. } => self
                .ensure_iov_payload_owner_extents_available(
                    owners,
                    raw_aliases,
                    args.get(iovs_arg),
                    span,
                ),
            HostMemorySpan::IovDescriptor {
                iovs_arg,
                iov_count_arg,
            } => {
                let Some(iovs) = args.get(iovs_arg) else {
                    return true;
                };
                let length = HostMemoryLength::ArgScaled {
                    arg: iov_count_arg,
                    bytes_per_item: 8,
                };
                let Some(length) = length.resolve(args, self.types.i32(), raw_aliases) else {
                    return true;
                };
                self.ensure_external_io_payload_extent_available(
                    owners,
                    raw_aliases,
                    iovs,
                    &length,
                    span,
                )
            }
        }
    }
}
