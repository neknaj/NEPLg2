use crate::span::Span;

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::CollectionSlotDropProof;
use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_owner_transfer_proof::CollectionSlotOwnerTransferProof;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_storage_release_proof::CollectionSlotStorageReleaseProof;
use super::collection_slot_summary_event_apply_proof::{
    summary_drop_proof, summary_owner_transfer_proof, summary_storage_release_proof,
};
use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryEventProof;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::raw_realloc::PendingRawReallocs;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_lifecycle_with_aliases(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        pending_raw_storage: &mut PendingRawReallocs,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
        span: Span,
    ) {
        let target = raw_aliases.canonicalize_owner_cell_address(target);
        self.apply_collection_slot_lifecycle_with_proofs(
            cells,
            collection_slots,
            Some(raw_aliases),
            Some(pending_raw_storage),
            &target,
            event,
            CollectionSlotOwnerTransferProof::LocalRawValueFlow,
            CollectionSlotDropProof::LocalLoadedValueDrop,
            CollectionSlotStorageReleaseProof::LocalRawStorageRelease,
            span,
        );
    }

    pub(super) fn apply_collection_slot_lifecycle_summary_event_with_aliases(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
        proof: CollectionSlotLifecycleSummaryEventProof,
        span: Span,
    ) {
        let target = raw_aliases.canonicalize_owner_cell_address(target);
        self.apply_collection_slot_lifecycle_with_proofs(
            cells,
            collection_slots,
            Some(raw_aliases),
            None,
            &target,
            event,
            summary_owner_transfer_proof(proof),
            summary_drop_proof(proof),
            summary_storage_release_proof(proof),
            span,
        );
    }
}
