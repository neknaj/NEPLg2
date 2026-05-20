extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::model::PlaceProjection;
use super::summary_index::{FunctionSummary, SummaryIndex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CollectionSlotLifecycleFunctionSummary {
    pub(super) function: String,
    pub(super) ops: Vec<CollectionSlotLifecycleSummaryOp>,
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
pub(super) struct CollectionSlotLifecycleSummaryPlace {
    pub(super) parameter_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
}
