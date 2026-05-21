extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleReturnPath, CollectionSlotLifecycleSummaryOp,
};

#[derive(Clone)]
pub(super) struct ReturnPathBuildState {
    pub(super) state: CollectionSlotSummaryBuildState,
    pub(super) ops: Vec<CollectionSlotLifecycleSummaryOp>,
}

pub(super) fn push_return_path(
    out: &mut Vec<CollectionSlotLifecycleReturnPath>,
    path: CollectionSlotLifecycleReturnPath,
) {
    if !out.contains(&path) {
        out.push(path);
    }
}
