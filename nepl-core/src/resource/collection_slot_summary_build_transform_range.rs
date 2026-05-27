extern crate alloc;

use alloc::{vec, vec::Vec};

use crate::layout::storage_size_bytes;

use super::collection_slot_drop_proof::CollectionSlotDropObligation;
use super::collection_slot_lifecycle::CollectionSlotLifecycleOp;
use super::collection_slot_owner_transfer::CollectionSlotOwnerTransferObligation;
use super::collection_slot_payload_tracking::collection_slot_payload_type_needs_tracking;
use super::collection_slot_summary_build_nested::apply_summary_condition_fact;
use super::collection_slot_summary_build_range_bound::initialized_range_loop_bound;
use super::collection_slot_summary_build_range_step::loop_body_increment_step;
use super::collection_slot_summary_build_state::{
    CollectionSlotSummaryBuildState, CollectionSlotTransformRangeCertificateCandidate,
};
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryOp, CollectionSlotTransformRangeCertificate,
    CollectionSlotTransformRangeDiscardProof, CollectionSlotTransformRangeOutputProof,
    CollectionSlotTransformRangeSourceProof,
};
use super::collection_slot_summary_return_state::collection_slot_summary_state_after_ops;
use super::collection_slot_summary_target::summary_place_for_params_with_aliases;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_scalar_flow_ops::propagate_i32_scalar_ops;
use super::model::{
    EffectOp, Place, PlaceProjection, RawMemoryOp, ResourceLocal, ResourceOffset, ResourceOp,
};
use super::raw_cell_value_flow_alias::raw_cell_place_with_canonical_symbolic_offsets;

pub(super) fn loop_transform_range_certificates(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    condition_ops: &[ResourceOp],
    condition_fact: Option<&super::model::ResourceConditionFact>,
    body_ops: &[ResourceOp],
) -> Vec<CollectionSlotTransformRangeCertificateCandidate> {
    let Some((read_index, source_initialized_count)) =
        initialized_range_loop_bound(&state.raw_aliases, condition_fact)
    else {
        return Vec::new();
    };
    if state.raw_aliases.i32_value(&read_index) != Some(0) {
        return Vec::new();
    }
    let Some(step_index) = loop_body_tail_increment_step(body_ops, &read_index) else {
        return Vec::new();
    };
    let body_prefix = &body_ops[..step_index];
    let mut condition_state = collection_slot_summary_state_after_ops(engine, state, condition_ops);
    apply_summary_condition_fact(&mut condition_state, condition_fact, true);
    transform_candidates_from_body(
        engine,
        &condition_state,
        body_prefix,
        &read_index,
        &source_initialized_count,
    )
}

fn loop_body_tail_increment_step(ops: &[ResourceOp], index: &Place) -> Option<usize> {
    for start in 0..ops.len() {
        if let Some(relative_step) = loop_body_increment_step(&ops[start..], index) {
            return Some(start + relative_step);
        }
    }
    None
}

pub(super) fn collect_summary_transform_range_op(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    state: &CollectionSlotSummaryBuildState,
    params: &[ResourceLocal],
    source_storage: &Place,
    source_initialized_count: &Place,
    output_storage: &Place,
    output_initialized_count: &Place,
    expected_ty: crate::types::TypeId,
) {
    let Some(candidate) = transform_range_certificate_candidate_for_op(
        state,
        source_storage,
        source_initialized_count,
        output_storage,
        output_initialized_count,
        expected_ty,
    ) else {
        return;
    };
    let Some(source_storage) = summary_place_for_params_with_aliases(
        params,
        &state.raw_aliases,
        &candidate.source_storage,
    ) else {
        return;
    };
    let Some(source_initialized_count) = summary_place_for_params_with_aliases(
        params,
        &state.raw_aliases,
        &candidate.source_initialized_count,
    ) else {
        return;
    };
    let output_storage = summary_place_for_params_with_aliases(
        params,
        &state.raw_aliases,
        &candidate.output_storage,
    );
    let output_initialized_count = summary_place_for_params_with_aliases(
        params,
        &state.raw_aliases,
        &candidate.output_initialized_count,
    );
    match (output_storage, output_initialized_count) {
        (Some(output_storage), Some(output_initialized_count)) => {
            out.push(CollectionSlotLifecycleSummaryOp::TransformRange {
                source_storage,
                source_initialized_count,
                output_storage,
                output_initialized_count,
                expected_ty,
                certificate: candidate.certificate,
            });
        }
        (None, _) | (Some(_), None) => {
            out.push(
                CollectionSlotLifecycleSummaryOp::TransformRangeSourceDrain {
                    source_storage,
                    source_initialized_count,
                    expected_ty,
                    certificate: candidate.certificate,
                },
            );
        }
    }
}

pub(super) fn transform_range_certificate_candidate_for_op(
    state: &CollectionSlotSummaryBuildState,
    source_storage: &Place,
    source_initialized_count: &Place,
    output_storage: &Place,
    output_initialized_count: &Place,
    expected_ty: crate::types::TypeId,
) -> Option<CollectionSlotTransformRangeCertificateCandidate> {
    let source_storage = state
        .raw_aliases
        .canonicalize_owner_cell_address(source_storage);
    let source_initialized_count = state
        .raw_aliases
        .canonicalize_scalar(source_initialized_count);
    let output_storage = state
        .raw_aliases
        .canonicalize_owner_cell_address(output_storage);
    let output_initialized_count = state
        .raw_aliases
        .canonicalize_scalar(output_initialized_count);
    state
        .transform_range_certificates
        .iter()
        .rev()
        .find(|candidate| {
            candidate.source_storage == source_storage
                && candidate.source_initialized_count == source_initialized_count
                && candidate.output_storage == output_storage
                && candidate.output_initialized_count == output_initialized_count
                && candidate.expected_ty == expected_ty
        })
        .cloned()
}

fn transform_candidates_from_body(
    engine: &ResourceCheckEngine<'_>,
    state: &CollectionSlotSummaryBuildState,
    body_ops: &[ResourceOp],
    read_index: &Place,
    source_initialized_count: &Place,
) -> Vec<CollectionSlotTransformRangeCertificateCandidate> {
    let mut aliases = state.raw_aliases.clone();
    let mut function_aliases = state.function_aliases.clone();
    let mut out = Vec::new();
    for (load_index, op) in body_ops.iter().enumerate() {
        let ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            output: loaded,
            args,
            ..
        } = op
        else {
            propagate_transform_alias_facts(engine, &mut aliases, &mut function_aliases, op);
            continue;
        };
        let Some(address) = args.first() else {
            continue;
        };
        if !collection_slot_payload_type_needs_tracking(engine.types, loaded.ty) {
            propagate_transform_alias_facts(engine, &mut aliases, &mut function_aliases, op);
            continue;
        }
        for address in aliases.raw_address_aliases_for_value(address) {
            let Some((source_storage, element_stride)) =
                storage_scaled_by_index(&address, read_index, &aliases)
            else {
                continue;
            };
            if element_stride != storage_size_bytes(engine.types, loaded.ty) || element_stride == 0
            {
                continue;
            }
            if let Some(candidate) = transform_candidate_after_load(
                engine,
                body_ops,
                load_index,
                loaded,
                source_storage,
                source_initialized_count,
                element_stride,
                &aliases,
            ) {
                push_transform_candidate(&mut out, candidate);
            }
        }
        propagate_transform_alias_facts(engine, &mut aliases, &mut function_aliases, op);
    }
    out
}

fn transform_candidate_after_load(
    engine: &ResourceCheckEngine<'_>,
    body_ops: &[ResourceOp],
    load_index: usize,
    loaded: &Place,
    source_storage: Place,
    source_initialized_count: &Place,
    element_stride: usize,
    aliases: &RawCellAddressAliases,
) -> Option<CollectionSlotTransformRangeCertificateCandidate> {
    let mut loaded_aliases = vec![loaded.clone()];
    for op in &body_ops[load_index + 1..] {
        if let ResourceOp::Branch {
            then_ops, else_ops, ..
        } = op
        {
            if let Some((output_storage, output_initialized_count)) =
                branch_stores_loaded_to_output(engine, then_ops, &loaded_aliases, aliases)
            {
                if aliases.i32_value(&output_initialized_count) != Some(0) {
                    propagate_loaded_value_aliases(&mut loaded_aliases, op);
                    continue;
                }
                if branch_drops_loaded(else_ops, &loaded_aliases) {
                    return Some(transform_range_candidate(
                        engine,
                        source_storage,
                        source_initialized_count.clone(),
                        output_storage,
                        output_initialized_count,
                        loaded.ty,
                        element_stride,
                        true,
                    ));
                }
            }
            if let Some((output_storage, output_initialized_count)) =
                branch_stores_loaded_to_output(engine, else_ops, &loaded_aliases, aliases)
            {
                if aliases.i32_value(&output_initialized_count) != Some(0) {
                    propagate_loaded_value_aliases(&mut loaded_aliases, op);
                    continue;
                }
                if branch_drops_loaded(then_ops, &loaded_aliases) {
                    return Some(transform_range_candidate(
                        engine,
                        source_storage,
                        source_initialized_count.clone(),
                        output_storage,
                        output_initialized_count,
                        loaded.ty,
                        element_stride,
                        true,
                    ));
                }
            }
        }
        propagate_loaded_value_aliases(&mut loaded_aliases, op);
    }
    None
}

fn branch_stores_loaded_to_output(
    engine: &ResourceCheckEngine<'_>,
    ops: &[ResourceOp],
    loaded_aliases: &[Place],
    aliases: &RawCellAddressAliases,
) -> Option<(Place, Place)> {
    let mut out = None;
    let mut branch_aliases = aliases.clone();
    let mut branch_function_aliases = super::function_alias::FunctionAliasTable::default();
    let mut loaded_aliases = loaded_aliases.to_vec();
    for (op_index, op) in ops.iter().enumerate() {
        if let ResourceOp::RawMemory {
            operation: RawMemoryOp::Store,
            args,
            ..
        } = op
        {
            if let (Some(address), Some(value)) = (args.first(), args.get(1)) {
                if loaded_value_aliases_contain(&loaded_aliases, value) {
                    for address in branch_aliases.raw_address_aliases_for_value(address) {
                        if let Some((storage, write_index)) = storage_scaled_by_any_index(
                            &address,
                            engine.types.i32(),
                            &branch_aliases,
                        ) {
                            if branch_has_single_output_increment_after_store(
                                ops,
                                op_index,
                                &write_index,
                            ) {
                                let candidate = (
                                    branch_aliases.canonicalize_owner_cell_address(&storage),
                                    branch_aliases.canonicalize_scalar(&write_index),
                                );
                                match &out {
                                    Some(existing) if existing != &candidate => return None,
                                    Some(_) => {}
                                    None => out = Some(candidate),
                                }
                            }
                        }
                    }
                }
            }
        }
        propagate_loaded_value_aliases(&mut loaded_aliases, op);
        propagate_transform_alias_facts(
            engine,
            &mut branch_aliases,
            &mut branch_function_aliases,
            op,
        );
    }
    out
}

fn propagate_loaded_value_aliases(aliases: &mut Vec<Place>, op: &ResourceOp) {
    match op {
        ResourceOp::DeclareLocal {
            place,
            initializer: Some(initializer),
            ..
        } => {
            remove_loaded_value_alias(aliases, place);
            if loaded_value_aliases_contain(aliases, initializer) {
                push_unique_place_alias(aliases, place);
            }
        }
        ResourceOp::Read { source, output, .. } => {
            remove_loaded_value_alias(aliases, output);
            if loaded_value_aliases_contain(aliases, source) {
                push_unique_place_alias(aliases, output);
            }
        }
        ResourceOp::Move { source, output, .. } => {
            let source_was_loaded = loaded_value_aliases_contain(aliases, source);
            remove_loaded_value_alias(aliases, source);
            remove_loaded_value_alias(aliases, output);
            if source_was_loaded {
                push_unique_place_alias(aliases, output);
            }
        }
        ResourceOp::Assign { target, value, .. } => {
            let value_was_loaded = loaded_value_aliases_contain(aliases, value);
            remove_loaded_value_alias(aliases, target);
            if value_was_loaded {
                push_unique_place_alias(aliases, target);
            }
        }
        ResourceOp::Drop { place, .. } => {
            remove_loaded_value_alias(aliases, place);
        }
        ResourceOp::EndScope { locals, .. } => {
            for local in locals {
                remove_loaded_value_alias(aliases, local);
            }
        }
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Store,
            args,
            ..
        } => {
            if let Some(value) = args.get(1) {
                remove_loaded_value_alias(aliases, value);
            }
        }
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            let mut then_aliases = aliases.clone();
            let mut else_aliases = aliases.clone();
            for op in then_ops {
                propagate_loaded_value_aliases(&mut then_aliases, op);
            }
            for op in else_ops {
                propagate_loaded_value_aliases(&mut else_aliases, op);
            }
            *aliases = intersect_loaded_value_aliases(&then_aliases, &else_aliases);
        }
        ResourceOp::Call {
            output,
            args,
            effect,
            ..
        } => {
            remove_loaded_value_alias(aliases, output);
            if matches!(
                effect,
                EffectOp::InternalAlloc { .. } | EffectOp::UnsafeMemory { .. }
            ) {
                return;
            }
            if args
                .iter()
                .any(|arg| loaded_value_aliases_contain(aliases, arg))
            {
                aliases.clear();
            }
        }
        ResourceOp::IndirectCall { output, args, .. } => {
            remove_loaded_value_alias(aliases, output);
            if args
                .iter()
                .any(|arg| loaded_value_aliases_contain(aliases, arg))
            {
                aliases.clear();
            }
        }
        ResourceOp::FunctionValue { output, .. } | ResourceOp::Construct { output, .. } => {
            remove_loaded_value_alias(aliases, output);
        }
        ResourceOp::Loop { .. } | ResourceOp::Match { .. } => {
            aliases.clear();
        }
        ResourceOp::DeclareLocal {
            initializer: None, ..
        } => {}
        ResourceOp::RawMemory { .. }
        | ResourceOp::Expr { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::CollectionSlotTransformRange { .. } => {}
    }
}

fn loaded_value_aliases_contain(aliases: &[Place], value: &Place) -> bool {
    aliases.iter().any(|alias| alias == value)
}

fn remove_loaded_value_alias(aliases: &mut Vec<Place>, place: &Place) {
    aliases.retain(|alias| alias != place);
}

fn intersect_loaded_value_aliases(left: &[Place], right: &[Place]) -> Vec<Place> {
    let mut out = Vec::new();
    for alias in left {
        if right.iter().any(|right_alias| right_alias == alias) {
            push_unique_place_alias(&mut out, alias);
        }
    }
    out
}

fn push_unique_place_alias(out: &mut Vec<Place>, place: &Place) {
    if !out.iter().any(|existing| existing == place) {
        out.push(place.clone());
    }
}

fn branch_has_single_output_increment_after_store(
    ops: &[ResourceOp],
    store_index: usize,
    write_index: &Place,
) -> bool {
    let after_store = store_index.saturating_add(1);
    let Some(relative_step) = loop_body_increment_step(&ops[after_store..], write_index) else {
        return false;
    };
    let step_index = after_store + relative_step;
    loop_body_increment_step(&ops[step_index + 1..], write_index).is_none()
}

fn branch_drops_loaded(ops: &[ResourceOp], loaded_aliases: &[Place]) -> bool {
    let mut loaded_aliases = loaded_aliases.to_vec();
    for op in ops {
        if matches!(op, ResourceOp::Drop { place, .. } if loaded_value_aliases_contain(&loaded_aliases, place))
        {
            return true;
        }
        propagate_loaded_value_aliases(&mut loaded_aliases, op);
    }
    false
}

fn transform_range_candidate(
    engine: &ResourceCheckEngine<'_>,
    source_storage: Place,
    source_initialized_count: Place,
    output_storage: Place,
    output_initialized_count: Place,
    expected_ty: crate::types::TypeId,
    element_stride: usize,
    has_discard: bool,
) -> CollectionSlotTransformRangeCertificateCandidate {
    CollectionSlotTransformRangeCertificateCandidate {
        source_storage,
        source_initialized_count,
        output_storage,
        output_initialized_count,
        expected_ty,
        certificate: CollectionSlotTransformRangeCertificate {
            element_stride,
            source_move_proof: if engine.types.is_copy(expected_ty) {
                CollectionSlotTransformRangeSourceProof::StateOnly
            } else {
                CollectionSlotTransformRangeSourceProof::LoadedValueMove(
                    CollectionSlotOwnerTransferObligation::MoveOutValue {
                        operation: CollectionSlotLifecycleOp::MoveOut,
                        value_ty: expected_ty,
                    },
                )
            },
            output_store_proof: if engine.types.is_copy(expected_ty) {
                CollectionSlotTransformRangeOutputProof::StateOnly
            } else {
                CollectionSlotTransformRangeOutputProof::StoredValue(
                    CollectionSlotOwnerTransferObligation::StoreValue {
                        operation: CollectionSlotLifecycleOp::InitializeEmpty,
                        value_ty: expected_ty,
                    },
                )
            },
            discard_drop_proof: if has_discard && !engine.types.is_copy(expected_ty) {
                CollectionSlotTransformRangeDiscardProof::LoadedValueDrop(
                    CollectionSlotDropObligation::DropLoadedValue {
                        operation: CollectionSlotLifecycleOp::DropInitialized,
                        value_ty: expected_ty,
                    },
                )
            } else {
                CollectionSlotTransformRangeDiscardProof::NoDiscard
            },
        },
    }
}

fn storage_scaled_by_index(
    address: &Place,
    index: &Place,
    aliases: &RawCellAddressAliases,
) -> Option<(Place, usize)> {
    let address = raw_cell_place_with_canonical_symbolic_offsets(address, aliases);
    let (storage, offset) = storage_and_offset(&address)?;
    match offset {
        ResourceOffset::ScaledSymbolic { place, scale } => {
            let offset_place = aliases.canonicalize_scalar(&place);
            let index = aliases.canonicalize_scalar(index);
            (offset_place == index).then_some((storage, scale))
        }
        ResourceOffset::Symbolic { place } => {
            let scale = symbolic_offset_scale_for_index(&place, index, aliases)?;
            Some((storage, scale))
        }
        _ => None,
    }
}

fn storage_scaled_by_any_index(
    address: &Place,
    i32_ty: crate::types::TypeId,
    aliases: &RawCellAddressAliases,
) -> Option<(Place, Place)> {
    let address = raw_cell_place_with_canonical_symbolic_offsets(address, aliases);
    let (storage, offset) = storage_and_offset(&address)?;
    match offset {
        ResourceOffset::ScaledSymbolic { place, .. } => {
            Some((storage, aliases.canonicalize_scalar(&place)))
        }
        ResourceOffset::Symbolic { place } => {
            Some((storage, symbolic_offset_index_place(&place, aliases)))
        }
        ResourceOffset::Known(_)
        | ResourceOffset::Offset { .. }
        | ResourceOffset::ScaledOffset { .. }
        | ResourceOffset::Unknown => Some((storage, Place::unknown(i32_ty))),
    }
}

fn symbolic_offset_index_place(offset: &Place, aliases: &RawCellAddressAliases) -> Place {
    let offset = aliases.canonicalize_scalar(offset);
    aliases
        .i32_scaled_source(&offset)
        .map(|(source, _)| aliases.canonicalize_scalar(&source))
        .unwrap_or(offset)
}

fn symbolic_offset_scale_for_index(
    offset: &Place,
    index: &Place,
    aliases: &RawCellAddressAliases,
) -> Option<usize> {
    let offset = aliases.canonicalize_scalar(offset);
    let index = aliases.canonicalize_scalar(index);
    if offset == index {
        return Some(1);
    }
    let (source, scale) = aliases.i32_scaled_source(&offset)?;
    let source = aliases.canonicalize_scalar(&source);
    (source == index).then_some(scale)
}

fn storage_and_offset(address: &Place) -> Option<(Place, ResourceOffset)> {
    let mut storage = address.clone();
    let projection = storage.projections.pop()?;
    match projection {
        PlaceProjection::StorageOffset(offset) => Some((storage, offset)),
        _ => None,
    }
}

fn push_transform_candidate(
    out: &mut Vec<CollectionSlotTransformRangeCertificateCandidate>,
    candidate: CollectionSlotTransformRangeCertificateCandidate,
) {
    if !out.iter().any(|existing| {
        existing.source_storage == candidate.source_storage
            && existing.source_initialized_count == candidate.source_initialized_count
            && existing.output_storage == candidate.output_storage
            && existing.output_initialized_count == candidate.output_initialized_count
            && existing.expected_ty == candidate.expected_ty
            && existing.certificate == candidate.certificate
    }) {
        out.push(candidate);
    }
}

fn propagate_transform_alias_facts(
    engine: &ResourceCheckEngine<'_>,
    aliases: &mut RawCellAddressAliases,
    function_aliases: &mut super::function_alias::FunctionAliasTable,
    op: &ResourceOp,
) {
    propagate_i32_scalar_ops(
        aliases,
        function_aliases,
        core::slice::from_ref(op),
        engine.i32_scalar_summaries,
        engine.raw_alias_summaries,
        engine.types,
    );
}
