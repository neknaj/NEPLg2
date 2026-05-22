extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::collection_slot_drop_proof::CollectionSlotDropObligation;
use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_owner_transfer::CollectionSlotOwnerTransferObligation;
use super::collection_slot_summary_return_model::{
    CollectionSlotLifecycleReturnSlot, CollectionSlotLifecycleReturnTransfer,
};
use super::summary_index::{FunctionSummary, SummaryIndex};
use super::summary_projection::SummaryPlace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CollectionSlotLifecycleFunctionSummary {
    pub(super) function: String,
    pub(super) ops: Vec<CollectionSlotLifecycleSummaryOp>,
    pub(super) return_transfers: Vec<CollectionSlotLifecycleReturnTransfer>,
    pub(super) return_slots: Vec<CollectionSlotLifecycleReturnSlot>,
    pub(super) return_paths: Vec<CollectionSlotLifecycleReturnPath>,
}

pub(super) type CollectionSlotLifecycleFunctionSummaryIndex<'a> =
    SummaryIndex<'a, CollectionSlotLifecycleFunctionSummary>;

impl FunctionSummary for CollectionSlotLifecycleFunctionSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CollectionSlotLifecycleSummaryOp {
    Event {
        target: CollectionSlotLifecycleSummaryPlace,
        event: CollectionSlotLifecycleEvent,
        proof: CollectionSlotLifecycleSummaryEventProof,
    },
    Relocate {
        old_storage: CollectionSlotLifecycleSummaryPlace,
        new_storage: CollectionSlotLifecycleSummaryPlace,
        proof: CollectionSlotLifecycleSummaryRelocateProof,
    },
    DropTraversal {
        storage: CollectionSlotLifecycleSummaryPlace,
        initialized_count: CollectionSlotLifecycleSummaryPlace,
        expected_ty: TypeId,
        coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage,
    },
    Merge {
        paths: Vec<Vec<CollectionSlotLifecycleSummaryOp>>,
    },
    Loop {
        condition_ops: Vec<CollectionSlotLifecycleSummaryOp>,
        body_ops: Vec<CollectionSlotLifecycleSummaryOp>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum CollectionSlotLifecycleSummaryDropTraversalCoverage {
    CertifiedSlots(Vec<CollectionSlotLifecycleSummaryPlace>),
    ForallInitializedRange(CollectionSlotInitializedRangeDropTraversalCertificate),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CollectionSlotInitializedRangeDropTraversalCertificate {
    pub(super) element_stride: usize,
    pub(super) drop_proof: CollectionSlotInitializedRangeDropTraversalProof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CollectionSlotInitializedRangeDropTraversalProof {
    StateOnly,
    LoadedValueDrop(CollectionSlotDropObligation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CollectionSlotLifecycleSummaryEventProof {
    pub(super) owner_transfer: CollectionSlotLifecycleSummaryOwnerTransferProof,
    pub(super) slot_drop: CollectionSlotLifecycleSummaryDropProof,
    pub(super) storage_release: CollectionSlotLifecycleSummaryStorageReleaseProof,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CollectionSlotLifecycleSummaryOwnerTransferProof {
    StateOnly,
    ValueFlow(CollectionSlotOwnerTransferObligation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CollectionSlotLifecycleSummaryDropProof {
    StateOnly,
    LoadedValueDrop(CollectionSlotDropObligation),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CollectionSlotLifecycleSummaryStorageReleaseProof {
    StateOnly,
    RawStorageRelease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CollectionSlotLifecycleSummaryRelocateProof {
    RawStorageRelocation,
}

pub(super) type CollectionSlotLifecycleSummaryPlace = SummaryPlace;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CollectionSlotLifecycleReturnPath {
    pub(super) ops: Vec<CollectionSlotLifecycleSummaryOp>,
    pub(super) return_transfers: Vec<CollectionSlotLifecycleReturnTransfer>,
    pub(super) return_slots: Vec<CollectionSlotLifecycleReturnSlot>,
}
