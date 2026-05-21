use crate::span::Span;

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::CollectionSlotDropProof;
use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_owner_transfer_proof::CollectionSlotOwnerTransferProof;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryDropProof, CollectionSlotLifecycleSummaryEventProof,
    CollectionSlotLifecycleSummaryOwnerTransferProof,
};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_lifecycle_with_aliases(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
        span: Span,
    ) {
        let target = raw_aliases.canonicalize_owner_cell_address(target);
        self.apply_collection_slot_lifecycle(cells, collection_slots, &target, event, span);
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
            &target,
            event,
            summary_owner_transfer_proof(proof),
            summary_drop_proof(proof),
            span,
        );
    }
}

fn summary_owner_transfer_proof(
    proof: CollectionSlotLifecycleSummaryEventProof,
) -> CollectionSlotOwnerTransferProof {
    match proof.owner_transfer {
        CollectionSlotLifecycleSummaryOwnerTransferProof::StateOnly => {
            CollectionSlotOwnerTransferProof::SummaryStateOnly
        }
        CollectionSlotLifecycleSummaryOwnerTransferProof::ValueFlow(obligation) => {
            CollectionSlotOwnerTransferProof::SummaryCertified(obligation)
        }
    }
}

fn summary_drop_proof(proof: CollectionSlotLifecycleSummaryEventProof) -> CollectionSlotDropProof {
    match proof.slot_drop {
        CollectionSlotLifecycleSummaryDropProof::StateOnly => {
            CollectionSlotDropProof::SummaryStateOnly
        }
        CollectionSlotLifecycleSummaryDropProof::LoadedValueDrop(obligation) => {
            CollectionSlotDropProof::SummaryCertified(obligation)
        }
    }
}
