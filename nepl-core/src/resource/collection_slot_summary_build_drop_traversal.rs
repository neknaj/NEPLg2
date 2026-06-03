extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::collection_slot_payload_tracking::collection_slot_payload_type_needs_tracking;
use super::collection_slot_state_identity::slot_requires_range_proof;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_i32_operand::summary_i32_operand_for_params_with_aliases;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryDropTraversalCoverage, CollectionSlotLifecycleSummaryI32Operand,
    CollectionSlotLifecycleSummaryOp,
};
use super::collection_slot_summary_target::summary_place_for_params_with_aliases;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, ResourceLocal};

pub(super) fn collect_summary_drop_traversal_op(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    storage: &Place,
    initialized_count: &Place,
    expected_ty: TypeId,
) {
    if !collection_slot_payload_type_needs_tracking(engine.types, expected_ty) {
        return;
    }
    let storage_place = state.raw_aliases.canonicalize_owner_cell_address(storage);
    let initialized_count_place = state.raw_aliases.canonicalize_scalar(initialized_count);
    if let Some(certificate) =
        find_range_certificate(state, &storage_place, &initialized_count_place, expected_ty)
    {
        if let (Some(storage), Some(initialized_count)) = (
            summary_place_for_params_with_aliases(params, &state.raw_aliases, &storage_place),
            summary_i32_operand_for_params(state, params, &initialized_count_place),
        ) {
            out.push(CollectionSlotLifecycleSummaryOp::DropTraversal {
                storage,
                initialized_count,
                expected_ty,
                coverage:
                    CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                        certificate,
                    ),
            });
        }
        return;
    }
    let Some(certified_slots) = engine.collection_slot_drop_traversal_certified_slots(
        &state.cells,
        &state.collection_slots,
        &state.raw_aliases,
        &storage_place,
        &initialized_count_place,
        expected_ty,
    ) else {
        return;
    };
    let has_range_checked_symbolic_slot = certified_slots
        .iter()
        .any(|slot| slot_requires_range_proof(slot, &storage_place));
    if has_range_checked_symbolic_slot {
        // A per-slot range proof does not prove that the callee traversed every initialized slot.
        return;
    }
    let mut summary_slots = Vec::new();
    for slot in certified_slots {
        let slot = state.raw_aliases.canonicalize_owner_cell_address(&slot);
        let Some(slot) = summary_place_for_params_with_aliases(params, &state.raw_aliases, &slot)
        else {
            return;
        };
        summary_slots.push(slot);
    }
    if let (Some(storage), Some(initialized_count)) = (
        summary_place_for_params_with_aliases(params, &state.raw_aliases, &storage_place),
        summary_i32_operand_for_params(state, params, &initialized_count_place),
    ) {
        out.push(CollectionSlotLifecycleSummaryOp::DropTraversal {
            storage,
            initialized_count,
            expected_ty,
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::CertifiedSlots(
                summary_slots,
            ),
        });
    }
}

fn summary_i32_operand_for_params(
    state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    place: &Place,
) -> Option<CollectionSlotLifecycleSummaryI32Operand> {
    summary_i32_operand_for_params_with_aliases(params, &state.raw_aliases, place)
}

fn find_range_certificate(
    state: &CollectionSlotSummaryBuildState,
    storage: &Place,
    initialized_count: &Place,
    expected_ty: TypeId,
) -> Option<
    super::collection_slot_summary_model::CollectionSlotInitializedRangeDropTraversalCertificate,
> {
    state
        .drop_traversal_range_certificates
        .iter()
        .rev()
        .find(|candidate| {
            candidate.storage == *storage
                && candidate.initialized_count == *initialized_count
                && candidate.expected_ty == expected_ty
        })
        .map(|candidate| candidate.certificate)
}
