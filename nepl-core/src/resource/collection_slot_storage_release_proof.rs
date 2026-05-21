use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
};
use super::collection_slot_state_table::CollectionSlotTableRefutation;
use super::model::Place;
use super::raw_realloc::PendingRawReallocs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CollectionSlotStorageReleaseProof {
    LocalRawStorageRelease,
    SummaryStateOnly,
    SummaryCertified,
}

pub(super) fn collection_slot_storage_release_obligation(
    event: CollectionSlotLifecycleEvent,
) -> bool {
    match event {
        CollectionSlotLifecycleEvent::StorageDealloc => true,
        CollectionSlotLifecycleEvent::InitializeEmpty { .. }
        | CollectionSlotLifecycleEvent::BorrowRead { .. }
        | CollectionSlotLifecycleEvent::MoveOut { .. }
        | CollectionSlotLifecycleEvent::ReplaceInitialized { .. }
        | CollectionSlotLifecycleEvent::DropInitialized { .. } => false,
    }
}

pub(super) fn collection_slot_storage_release_proof_available(
    pending_raw_storage: &PendingRawReallocs,
    storage: &Place,
) -> bool {
    pending_raw_storage.certified_storage_release_available(storage)
}

pub(super) fn consume_collection_slot_storage_release_proof(
    pending_raw_storage: Option<&mut PendingRawReallocs>,
    storage: &Place,
    proof: CollectionSlotStorageReleaseProof,
) -> bool {
    match proof {
        CollectionSlotStorageReleaseProof::LocalRawStorageRelease => pending_raw_storage
            .map(|pending_raw_storage| {
                pending_raw_storage.consume_certified_storage_release(storage)
            })
            .unwrap_or(false),
        CollectionSlotStorageReleaseProof::SummaryStateOnly => false,
        CollectionSlotStorageReleaseProof::SummaryCertified => true,
    }
}

pub(super) fn storage_release_refutation(storage: &Place) -> CollectionSlotTableRefutation {
    CollectionSlotTableRefutation {
        slot: storage.clone(),
        reason: CollectionSlotLifecycleRefutation::StorageDeallocRequiresRawReleaseProof {
            operation: CollectionSlotLifecycleOp::StorageDealloc,
        },
    }
}
