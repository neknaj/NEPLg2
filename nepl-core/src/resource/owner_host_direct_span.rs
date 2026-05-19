use crate::span::Span;

use super::host_memory_contract::HostMemorySpan;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::report::ResourceOwnerOperation;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn ensure_direct_memory_contract_owner_span_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        contract: &HostMemorySpan,
        args: &[Place],
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        let HostMemorySpan::Direct {
            address_arg,
            length,
            direction,
            ..
        } = *contract
        else {
            return true;
        };

        let Some(address) = args.get(address_arg) else {
            return true;
        };
        let Some(length) = length.resolve(args, self.types.i32(), raw_aliases) else {
            if self.try_record_deferred_memory_span_requirement(
                raw_aliases,
                contract,
                args,
                operation,
            ) {
                return true;
            }
            return true;
        };
        self.ensure_memory_payload_extent_available(
            owners,
            raw_aliases,
            raw_views,
            address,
            &length,
            direction,
            operation,
            span,
        )
    }
}
