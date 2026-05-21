extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_build_range_witness_drop::{
    drop_witness_candidate, prefix_drops_symbolic_slot, LoopBodyCandidateSlot,
    LoopBodyDropWitnessCandidate,
};
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_scalar_flow_ops::propagate_i32_scalar_ops;
use super::model::{Place, PlaceProjection, RawMemoryOp, ResourceOffset, ResourceOp};

pub(super) fn loop_body_candidate_slots(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    ops: &[ResourceOp],
    index: &Place,
    initialized_count: &Place,
) -> Vec<LoopBodyDropWitnessCandidate> {
    let mut raw_aliases = state.raw_aliases.clone();
    let mut function_aliases = state.function_aliases.clone();
    let mut out = Vec::new();
    for (op_index, op) in ops.iter().enumerate() {
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
                        push_candidate_slot(
                            &mut out,
                            LoopBodyCandidateSlot {
                                storage,
                                expected_ty: output.ty,
                                element_stride,
                                load_index: op_index,
                                loaded: output.clone(),
                            },
                        );
                    }
                }
            }
        }
        propagate_candidate_alias_facts(engine, &mut raw_aliases, &mut function_aliases, op);
    }
    out.into_iter()
        .filter_map(|candidate| drop_witness_candidate(engine, ops, candidate))
        .filter(|candidate| {
            prefix_drops_symbolic_slot(engine, state, ops, index, initialized_count, candidate)
        })
        .collect()
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

fn push_candidate_slot(out: &mut Vec<LoopBodyCandidateSlot>, candidate: LoopBodyCandidateSlot) {
    if !out.iter().any(|existing| {
        existing.storage == candidate.storage
            && existing.expected_ty == candidate.expected_ty
            && existing.element_stride == candidate.element_stride
            && existing.load_index == candidate.load_index
    }) {
        out.push(candidate);
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
