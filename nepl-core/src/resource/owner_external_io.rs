use crate::span::Span;

use super::host_memory_contract::host_memory_spans;
use super::host_size_contract::dependent_host_memory_spans;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;

impl ResourceOwnerCheckEngine<'_> {
    pub(super) fn ensure_external_io_owner_spans_available(
        &mut self,
        owners: &OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        effect: &super::model::EffectOp,
        args: &[Place],
        span: Span,
    ) -> bool {
        let mut available = true;
        for contract in host_memory_spans(effect) {
            available &= self.ensure_host_memory_contract_owner_span_available(
                owners,
                raw_aliases,
                raw_views,
                contract,
                args,
                span,
            );
        }
        for contract in dependent_host_memory_spans(effect) {
            available &= self.ensure_dependent_host_memory_owner_span_available(
                owners,
                raw_aliases,
                contract,
                args,
                span,
            );
        }
        if available {
            self.record_host_size_outputs(raw_aliases, effect, args);
        }
        available
    }
}
