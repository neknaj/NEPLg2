use alloc::string::ToString;

use crate::span::Span;

use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDiagnostic;
use super::{
    collection_slot_lifecycle::CollectionSlotLifecycleRefutation,
    collection_slot_state_table::CollectionSlotTableRefutation,
};

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_storage_relocate(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        pending_reallocs: &mut PendingRawReallocs,
        old_storage: &Place,
        new_storage: &Place,
        span: Span,
    ) {
        if !pending_reallocs.certified_storage_relocation_available(old_storage, new_storage) {
            self.report_collection_storage_relocate_refutation(
                CollectionSlotTableRefutation {
                    slot: old_storage.clone(),
                    reason: CollectionSlotLifecycleRefutation::StorageRelocateRequiresRawMoveProof,
                },
                span,
            );
            return;
        }
        if let Err(refutation) = collection_slots.relocate_storage(old_storage, new_storage) {
            self.report_collection_storage_relocate_refutation(refutation, span);
            return;
        }
        let consumed =
            pending_reallocs.consume_certified_storage_relocation(old_storage, new_storage);
        debug_assert!(consumed);
    }

    pub(super) fn apply_certified_collection_storage_relocate(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        old_storage: &Place,
        new_storage: &Place,
        span: Span,
    ) {
        if let Err(refutation) = collection_slots.relocate_storage(old_storage, new_storage) {
            self.report_collection_storage_relocate_refutation(refutation, span);
        }
    }

    pub(super) fn apply_collection_storage_relocate_with_aliases(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        pending_reallocs: &mut PendingRawReallocs,
        old_storage: &Place,
        new_storage: &Place,
        span: Span,
    ) {
        let old_storage = raw_aliases.canonicalize_owner_cell_address(old_storage);
        let new_storage = raw_aliases.canonicalize_owner_cell_address(new_storage);
        self.apply_collection_storage_relocate(
            collection_slots,
            pending_reallocs,
            &old_storage,
            &new_storage,
            span,
        );
    }

    pub(super) fn apply_certified_collection_storage_relocate_with_aliases(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        old_storage: &Place,
        new_storage: &Place,
        span: Span,
    ) {
        let old_storage = raw_aliases.canonicalize_owner_cell_address(old_storage);
        let new_storage = raw_aliases.canonicalize_owner_cell_address(new_storage);
        self.apply_certified_collection_storage_relocate(
            collection_slots,
            &old_storage,
            &new_storage,
            span,
        );
    }

    fn report_collection_storage_relocate_refutation(
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
