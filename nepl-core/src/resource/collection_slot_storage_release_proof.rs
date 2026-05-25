use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
};
use super::collection_slot_state_alias::storage_aliases_for_place;
use super::collection_slot_state_table::CollectionSlotTableRefutation;
use super::initialized_alias::RawCellAddressAliases;
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
    raw_aliases: Option<&RawCellAddressAliases>,
    storage: &Place,
) -> bool {
    storage_release_candidates(raw_aliases, storage)
        .iter()
        .any(|storage| pending_raw_storage.certified_storage_release_available(storage))
}

pub(super) fn consume_collection_slot_storage_release_proof(
    pending_raw_storage: Option<&mut PendingRawReallocs>,
    raw_aliases: Option<&RawCellAddressAliases>,
    storage: &Place,
    proof: CollectionSlotStorageReleaseProof,
) -> bool {
    match proof {
        CollectionSlotStorageReleaseProof::LocalRawStorageRelease => {
            let Some(pending_raw_storage) = pending_raw_storage else {
                return false;
            };
            storage_release_candidates(raw_aliases, storage)
                .iter()
                .any(|storage| pending_raw_storage.consume_certified_storage_release(storage))
        }
        CollectionSlotStorageReleaseProof::SummaryStateOnly => false,
        CollectionSlotStorageReleaseProof::SummaryCertified => true,
    }
}

fn storage_release_candidates(
    raw_aliases: Option<&RawCellAddressAliases>,
    storage: &Place,
) -> alloc::vec::Vec<Place> {
    raw_aliases
        .map(|raw_aliases| storage_aliases_for_place(storage, raw_aliases))
        .unwrap_or_else(|| alloc::vec![storage.clone()])
}

pub(super) fn storage_release_refutation(storage: &Place) -> CollectionSlotTableRefutation {
    CollectionSlotTableRefutation {
        slot: storage.clone(),
        reason: CollectionSlotLifecycleRefutation::StorageDeallocRequiresRawReleaseProof {
            operation: CollectionSlotLifecycleOp::StorageDealloc,
        },
    }
}
