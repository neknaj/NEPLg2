extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::CollectionSlotLifecycleReturnPath;
use super::collection_slot_summary_return_path_model::ReturnPathBuildState;
use super::collection_slot_summary_return_path_value::collect_return_paths_from_value_to_suffix;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, ResourceLocal, ResourceOp};

pub(super) fn collect_return_paths_from_ops(
    out: &mut Vec<CollectionSlotLifecycleReturnPath>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    state_at_start: &CollectionSlotSummaryBuildState,
    ops: &[ResourceOp],
    value: &Place,
) {
    let start = ReturnPathBuildState {
        state: state_at_start.clone(),
        preconditions: Vec::new(),
        ops: Vec::new(),
    };
    collect_return_paths_from_value_to_suffix(
        out,
        engine,
        params,
        start,
        ops,
        value,
        &[],
        value.ty,
    );
}
