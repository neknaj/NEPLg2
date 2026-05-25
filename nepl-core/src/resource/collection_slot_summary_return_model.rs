extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryPlace;
use super::collection_slot_summary_projection::CollectionSlotLifecycleSummaryProjection;
use super::model::{I32ValueCondition, ResourceI32RelationOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CollectionSlotLifecyclePathPreconditionOperand {
    Place(CollectionSlotLifecycleSummaryPlace),
    KnownI32 { value: i32, ty: TypeId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CollectionSlotLifecyclePathPrecondition {
    I32Condition {
        operand: CollectionSlotLifecyclePathPreconditionOperand,
        condition: I32ValueCondition,
    },
    I32Relation {
        left: CollectionSlotLifecyclePathPreconditionOperand,
        op: ResourceI32RelationOp,
        right: CollectionSlotLifecyclePathPreconditionOperand,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CollectionSlotLifecycleReturnSlot {
    pub(super) suffix: Vec<CollectionSlotLifecycleSummaryProjection>,
    pub(super) ty: TypeId,
    pub(super) state: CollectionSlotState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CollectionSlotLifecycleReturnRange {
    pub(super) storage_suffix: Vec<CollectionSlotLifecycleSummaryProjection>,
    pub(super) storage_ty: TypeId,
    pub(super) initialized_count: CollectionSlotLifecycleReturnRangeCount,
    pub(super) value_ty: TypeId,
    pub(super) element_stride: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CollectionSlotLifecycleReturnRangeCount {
    ReturnValueProjection {
        suffix: Vec<CollectionSlotLifecycleSummaryProjection>,
        ty: TypeId,
    },
    KnownI32 {
        value: i32,
        ty: TypeId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CollectionSlotLifecycleReturnTransfer {
    pub(super) source: CollectionSlotLifecycleSummaryPlace,
    pub(super) target_suffix: Vec<CollectionSlotLifecycleSummaryProjection>,
    pub(super) target_ty: TypeId,
}
