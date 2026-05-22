use alloc::string::ToString;

use crate::span::Span;

use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::report::ResourceCheckDiagnostic;

impl ResourceCheckEngine<'_> {
    pub(super) fn release_collection_slots_for_raw_dealloc(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        span: Span,
    ) -> bool {
        match collection_slots
            .release_storage_if_collection_tracked_with_aliases(storage, raw_aliases)
        {
            Ok(()) => true,
            Err(refutation) => {
                self.report_raw_dealloc_collection_slot_refutation(refutation, span);
                false
            }
        }
    }

    fn report_raw_dealloc_collection_slot_refutation(
        &mut self,
        refutation: CollectionSlotTableRefutation,
        span: Span,
    ) {
        self.diagnostics
            .push(ResourceCheckDiagnostic::CollectionSlotRefuted {
                function: self.function.to_string(),
                target: refutation.slot,
                reason: refutation.reason,
                span,
            });
    }
}
