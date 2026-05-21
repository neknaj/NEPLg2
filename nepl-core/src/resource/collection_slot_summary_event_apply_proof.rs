use super::collection_slot_drop_proof::CollectionSlotDropProof;
use super::collection_slot_owner_transfer_proof::CollectionSlotOwnerTransferProof;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryDropProof, CollectionSlotLifecycleSummaryEventProof,
    CollectionSlotLifecycleSummaryOwnerTransferProof,
};

pub(super) fn summary_owner_transfer_proof(
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

pub(super) fn summary_drop_proof(
    proof: CollectionSlotLifecycleSummaryEventProof,
) -> CollectionSlotDropProof {
    match proof.slot_drop {
        CollectionSlotLifecycleSummaryDropProof::StateOnly => {
            CollectionSlotDropProof::SummaryStateOnly
        }
        CollectionSlotLifecycleSummaryDropProof::LoadedValueDrop(obligation) => {
            CollectionSlotDropProof::SummaryCertified(obligation)
        }
    }
}
