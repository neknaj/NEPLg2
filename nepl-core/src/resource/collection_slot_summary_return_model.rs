extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryPlace;
use super::collection_slot_summary_projection::CollectionSlotLifecycleSummaryProjection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CollectionSlotLifecycleReturnSlot {
    pub(super) suffix: Vec<CollectionSlotLifecycleSummaryProjection>,
    pub(super) ty: TypeId,
    pub(super) state: CollectionSlotState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CollectionSlotLifecycleReturnTransfer {
    pub(super) source: CollectionSlotLifecycleSummaryPlace,
    pub(super) target_suffix: Vec<CollectionSlotLifecycleSummaryProjection>,
    pub(super) target_ty: TypeId,
}
