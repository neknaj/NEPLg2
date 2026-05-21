extern crate alloc;

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::layout::storage_size_bytes;
use crate::types::TypeId;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_return_state::collection_slot_summary_state_after_ops;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_scalar_flow_ops::propagate_i32_scalar_ops;
use super::model::{Place, PlaceProjection, RawMemoryOp, ResourceOffset, ResourceOp};
use super::place_utils::raw_memory_cell_place;

pub(super) fn loop_body_candidate_slots(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    ops: &[ResourceOp],
    index: &Place,
) -> Vec<(Place, TypeId, usize)> {
    let mut raw_aliases = state.raw_aliases.clone();
    let mut function_aliases = state.function_aliases.clone();
    let mut out = Vec::new();
    for op in ops {
        if let ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            output,
            args,
            ..
        } = op
        {
            if let Some(address) = args.first() {
                for address in raw_aliases.raw_address_aliases_for_value(address) {
                    if let Some((storage, element_stride)) =
                        storage_scaled_by_index(&address, index, &raw_aliases)
                    {
                        push_candidate_slot(&mut out, storage, output.ty, element_stride);
                    }
                }
            }
        }
        propagate_candidate_alias_facts(engine, &mut raw_aliases, &mut function_aliases, op);
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
    let mut probe = collection_slot_summary_state_after_ops(engine, &probe, body_prefix);
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
        PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place }) => {
            let Some(scale) = symbolic_offset_scale_for_index(&place, index, raw_aliases) else {
                return None;
            };
            Some((storage, scale))
        }
        _ => None,
    }
}

fn symbolic_offset_scale_for_index(
    offset: &Place,
    index: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Option<usize> {
    let offset = raw_aliases.canonicalize_scalar(offset);
    let index = raw_aliases.canonicalize_scalar(index);
    if offset == index {
        return Some(1);
    }
    let (source, scale) = raw_aliases.i32_scaled_source(&offset)?;
    (source == index).then_some(scale)
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

fn propagate_candidate_alias_facts(
    engine: &ResourceCheckEngine<'_>,
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut super::function_alias::FunctionAliasTable,
    op: &ResourceOp,
) {
    propagate_i32_scalar_ops(
        raw_aliases,
        function_aliases,
        core::slice::from_ref(op),
        engine.i32_scalar_summaries,
        engine.raw_alias_summaries,
        engine.types,
    );
}
