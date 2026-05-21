use alloc::string::ToString;

use crate::span::Span;

use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::report::ResourceCheckDiagnostic;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_storage_relocate(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        old_storage: &Place,
        new_storage: &Place,
        span: Span,
    ) {
        if let Err(refutation) = collection_slots.relocate_storage(old_storage, new_storage) {
            self.diagnostics
                .push(ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: self.function.to_string(),
                    target: refutation.slot,
                    reason: refutation.reason,
                    span,
                });
        }
    }

    pub(super) fn apply_collection_storage_relocate_with_aliases(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        old_storage: &Place,
        new_storage: &Place,
        span: Span,
    ) {
        let old_storage = raw_aliases.canonicalize_owner_cell_address(old_storage);
        let new_storage = raw_aliases.canonicalize_owner_cell_address(new_storage);
        self.apply_collection_storage_relocate(collection_slots, &old_storage, &new_storage, span);
    }
}
