use crate::span::Span;

use super::host_memory_contract::{HostMemoryDirection, HostMemoryLength, HostMemorySpan};
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_state::OwnerTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn ensure_iov_descriptor_owner_extent_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        args: &[Place],
        iovs_arg: usize,
        iov_count_arg: usize,
        contract: &HostMemorySpan,
        span: Span,
    ) -> bool {
        let Some(iovs) = args.get(iovs_arg) else {
            return true;
        };
        let length = HostMemoryLength::ArgScaled {
            arg: iov_count_arg,
            bytes_per_item: 8,
        };
        let Some(length) = length.resolve(args, self.types.i32(), raw_aliases) else {
            if self.try_record_deferred_host_memory_span_requirement(raw_aliases, contract, args) {
                return true;
            }
            return true;
        };
        self.ensure_external_io_payload_extent_available(
            owners,
            raw_aliases,
            iovs,
            &length,
            HostMemoryDirection::Input,
            span,
        )
    }
}
