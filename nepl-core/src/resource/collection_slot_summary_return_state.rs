extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::drop_point_path::ResourceDropPointPath;
use super::initialized::ResourceCheckEngine;
use super::initialized_summary_engine::summary_check_engine;
use super::model::{ResourceBlockId, ResourceOp};

pub(super) fn collection_slot_summary_state_after_ops(
    engine: &ResourceCheckEngine<'_>,
    state_at_start: &CollectionSlotSummaryBuildState,
    ops: &[ResourceOp],
) -> CollectionSlotSummaryBuildState {
    let mut engine = summary_check_engine(engine);
    let mut state = state_at_start.clone();
    engine.check_ops(
        &mut state.cells,
        &mut state.collection_slots,
        &mut state.raw_aliases,
        &mut state.function_aliases,
        &mut state.pending_reallocs,
        &mut state.variant_initializations,
        ops,
        ResourceDropPointPath {
            block: ResourceBlockId(usize::MAX),
            steps: Vec::new(),
        },
    );
    engine.auto_drop_points.clear();
    state
}
