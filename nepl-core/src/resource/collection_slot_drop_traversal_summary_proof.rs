use crate::types::TypeId;

use super::collection_slot_drop_proof::{CollectionSlotDropObligation, CollectionSlotDropProof};
use super::collection_slot_lifecycle::CollectionSlotLifecycleOp;

pub(super) fn summary_certified_drop_traversal_proof(
    expected_ty: TypeId,
) -> CollectionSlotDropProof {
    CollectionSlotDropProof::SummaryCertified(CollectionSlotDropObligation::DropLoadedValue {
        operation: CollectionSlotLifecycleOp::DropInitialized,
        value_ty: expected_ty,
    })
}
