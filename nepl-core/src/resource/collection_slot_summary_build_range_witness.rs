extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::layout::storage_size_bytes;
use crate::types::TypeId;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::drop_point_path::ResourceDropPointPath;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    Place, PlaceProjection, RawMemoryOp, ResourceBlockId, ResourceOffset, ResourceOp,
};
use super::place_utils::raw_memory_cell_place;

pub(super) fn loop_body_candidate_slots(
    ops: &[ResourceOp],
    index: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Vec<(Place, TypeId, usize)> {
    let mut out = Vec::new();
    for op in ops {
        let ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            output,
            args,
            ..
        } = op
        else {
            continue;
        };
        let Some(address) = args.first() else {
            continue;
        };
        let address = raw_aliases.canonicalize_owner_cell_address(address);
        let Some((storage, element_stride)) = storage_scaled_by_index(&address, index, raw_aliases)
        else {
            continue;
        };
        push_candidate_slot(&mut out, storage, output.ty, element_stride);
    }
    out
}

pub(super) fn loop_body_drops_symbolic_slot(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    body_prefix: &[ResourceOp],
    storage: &Place,
    index: &Place,
    initialized_count: &Place,
    expected_ty: TypeId,
    element_stride: usize,
) -> bool {
    if element_stride != storage_size_bytes(engine.types, expected_ty) || element_stride == 0 {
        return false;
    }
    let slot_address = storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
            place: Box::new(index.clone()),
            scale: element_stride,
        }),
        engine.types.i32(),
    );
    let slot = slot_address
        .clone()
        .with_projection(PlaceProjection::Deref, expected_ty);
    let mut probe = state.clone();
    probe
        .cells
        .mark_initialized(&raw_memory_cell_place(&slot_address, expected_ty));
    probe
        .collection_slots
        .set_slot_state(&slot, CollectionSlotState::Initialized(expected_ty));
    let mut probe = state_after_ops(engine, probe, body_prefix);
    engine
        .collection_slot_drop_traversal_result(
            &mut probe.cells,
            &mut probe.collection_slots,
            &probe.raw_aliases,
            storage,
            initialized_count,
            expected_ty,
        )
        .is_ok()
}

pub(super) fn state_after_ops(
    engine: &ResourceCheckEngine<'_>,
    mut state: CollectionSlotSummaryBuildState,
    ops: &[ResourceOp],
) -> CollectionSlotSummaryBuildState {
    let mut probe = ResourceCheckEngine {
        function: engine.function,
        types: engine.types,
        raw_alias_summaries: engine.raw_alias_summaries,
        i32_scalar_summaries: engine.i32_scalar_summaries,
        raw_init_summaries: engine.raw_init_summaries,
        collection_slot_summaries: engine.collection_slot_summaries,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: Default::default(),
        path_alternatives: Default::default(),
    };
    probe.check_ops(
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
    state
}

fn storage_scaled_by_index(
    address: &Place,
    index: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Option<(Place, usize)> {
    let mut storage = address.clone();
    let projection = storage.projections.pop()?;
    match projection {
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic { place, scale }) => {
            let offset_place = raw_aliases.canonicalize_scalar(&place);
            let index = raw_aliases.canonicalize_scalar(index);
            (offset_place == index).then_some((storage, scale))
        }
        _ => None,
    }
}

fn push_candidate_slot(
    out: &mut Vec<(Place, TypeId, usize)>,
    storage: Place,
    expected_ty: TypeId,
    element_stride: usize,
) {
    if !out.iter().any(|existing| {
        existing.0 == storage && existing.1 == expected_ty && existing.2 == element_stride
    }) {
        out.push((storage, expected_ty, element_stride));
    }
}
