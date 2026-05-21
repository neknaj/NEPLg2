use crate::span::Span;

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::CollectionSlotDropProof;
use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_owner_transfer_proof::CollectionSlotOwnerTransferProof;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_storage_release_proof::CollectionSlotStorageReleaseProof;
use super::initialized::ResourceCheckEngine;
use super::model::Place;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_lifecycle(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
        span: Span,
    ) {
        self.apply_collection_slot_lifecycle_with_proofs(
            cells,
            collection_slots,
            None,
            None,
            target,
            event,
            CollectionSlotOwnerTransferProof::LocalRawValueFlow,
            CollectionSlotDropProof::LocalLoadedValueDrop,
            CollectionSlotStorageReleaseProof::LocalRawStorageRelease,
            span,
        );
    }
}
