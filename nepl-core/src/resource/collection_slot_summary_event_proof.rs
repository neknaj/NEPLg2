use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::{
    collection_slot_drop_obligation, collection_slot_drop_proof_available,
};
use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_owner_transfer::collection_slot_owner_transfer_obligation;
use super::collection_slot_owner_transfer_proof::collection_slot_owner_transfer_proof_available;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryDropProof, CollectionSlotLifecycleSummaryEventProof,
    CollectionSlotLifecycleSummaryOwnerTransferProof,
};
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

pub(super) fn summary_event_proof(
    types: &TypeCtx,
    cells: &CellTable,
    raw_aliases: Option<&RawCellAddressAliases>,
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
    Some(CollectionSlotLifecycleSummaryEventProof {
        owner_transfer,
        slot_drop,
    })
}

pub(super) fn summary_event_proof_with_aliases(
    types: &TypeCtx,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    target: &Place,
    event: CollectionSlotLifecycleEvent,
) -> Option<CollectionSlotLifecycleSummaryEventProof> {
    summary_event_proof(types, cells, Some(raw_aliases), target, event)
}
