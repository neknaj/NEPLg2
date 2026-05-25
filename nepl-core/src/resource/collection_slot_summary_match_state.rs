use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::initialized::ResourceCheckEngine;
use super::initialized_summary_engine::summary_check_engine;
use super::model::{Place, ResourceMatchArm};
use super::place_utils::match_bind_payload_place;

pub(super) fn collection_slot_summary_match_arm_entry_state(
    engine: &ResourceCheckEngine<'_>,
    match_state: &CollectionSlotSummaryBuildState,
    scrutinee: &Place,
    arm: &ResourceMatchArm,
) -> Option<CollectionSlotSummaryBuildState> {
    if !match_state
        .variant_initializations
        .match_arm_reachable(scrutinee, &arm.pattern)
    {
        return None;
    }
    let mut arm_state = match_state.clone();
    let mut arm_engine = summary_check_engine(engine);
    if let Some(bind_local) = &arm.bind_local {
        arm_state.cells.mark_initialized(bind_local);
        if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
            arm_engine.copy_raw_address_alias_and_rekey_cells(
                &mut arm_state.cells,
                &mut arm_state.raw_aliases,
                &source,
                bind_local,
            );
            arm_state
                .cells
                .transfer_raw_cell_loaded_value_origin(&source, bind_local);
            arm_engine.transfer_slot_state_if_moved_with_aliases(
                &arm_state.cells,
                &mut arm_state.collection_slots,
                &source,
                bind_local,
                &arm_state.raw_aliases,
                arm.span,
            );
            arm_state.function_aliases.copy_alias(&source, bind_local);
            arm_state.pending_reallocs.copy_result(&source, bind_local);
            arm_state
                .variant_initializations
                .copy_result(&source, bind_local);
        } else {
            arm_state.raw_aliases.clear(bind_local);
            arm_state.function_aliases.clear_alias(bind_local);
            arm_state.pending_reallocs.clear_result(bind_local);
            arm_state.variant_initializations.clear_result(bind_local);
        }
    }
    arm_state.variant_initializations.apply_match_arm(
        &mut arm_engine,
        &mut arm_state.cells,
        &mut arm_state.raw_aliases,
        scrutinee,
        &arm.pattern,
        arm.span,
    );
    Some(arm_state)
}
