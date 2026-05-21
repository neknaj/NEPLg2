use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::{
    collection_slot_drop_obligation, collection_slot_drop_proof_available,
};
use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_owner_transfer::collection_slot_owner_transfer_obligation;
use super::collection_slot_owner_transfer_proof::collection_slot_owner_transfer_proof_available;
use super::collection_slot_storage_release_proof::{
    collection_slot_storage_release_obligation, collection_slot_storage_release_proof_available,
};
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryDropProof, CollectionSlotLifecycleSummaryEventProof,
    CollectionSlotLifecycleSummaryOwnerTransferProof,
    CollectionSlotLifecycleSummaryStorageReleaseProof,
};
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::raw_realloc::PendingRawReallocs;

pub(super) fn summary_event_proof(
    types: &TypeCtx,
    cells: &CellTable,
    raw_aliases: Option<&RawCellAddressAliases>,
    pending_raw_storage: &PendingRawReallocs,
    target: &Place,
    event: CollectionSlotLifecycleEvent,
) -> Option<CollectionSlotLifecycleSummaryEventProof> {
    let owner_transfer = match collection_slot_owner_transfer_obligation(types, event) {
        Some(obligation) => {
            if collection_slot_owner_transfer_proof_available(
                cells,
                raw_aliases,
                target,
                obligation,
                types,
            ) {
                CollectionSlotLifecycleSummaryOwnerTransferProof::ValueFlow(obligation)
            } else {
                return None;
            }
        }
        None => CollectionSlotLifecycleSummaryOwnerTransferProof::StateOnly,
    };
    let slot_drop = match collection_slot_drop_obligation(types, event) {
        Some(obligation) => {
            if collection_slot_drop_proof_available(cells, raw_aliases, target, obligation, types) {
                CollectionSlotLifecycleSummaryDropProof::LoadedValueDrop(obligation)
            } else {
                return None;
            }
        }
        None => CollectionSlotLifecycleSummaryDropProof::StateOnly,
    };
    let storage_release = if collection_slot_storage_release_obligation(event) {
        if collection_slot_storage_release_proof_available(pending_raw_storage, target) {
            CollectionSlotLifecycleSummaryStorageReleaseProof::RawStorageRelease
        } else {
            return None;
        }
    } else {
        CollectionSlotLifecycleSummaryStorageReleaseProof::StateOnly
    };
    Some(CollectionSlotLifecycleSummaryEventProof {
        owner_transfer,
        slot_drop,
        storage_release,
    })
}

pub(super) fn summary_event_proof_with_aliases(
    types: &TypeCtx,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    pending_raw_storage: &PendingRawReallocs,
    target: &Place,
    event: CollectionSlotLifecycleEvent,
) -> Option<CollectionSlotLifecycleSummaryEventProof> {
    summary_event_proof(
        types,
        cells,
        Some(raw_aliases),
        pending_raw_storage,
        target,
        event,
    )
}
