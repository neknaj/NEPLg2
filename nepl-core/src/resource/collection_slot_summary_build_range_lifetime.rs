use crate::types::TypeCtx;

use super::collection_slot_summary_build_state::{
    CollectionSlotDropTraversalRangeCertificateCandidate,
    CollectionSlotTransformRangeCertificateCandidate,
};
use super::collection_slot_summary_model::{
    CollectionSlotInitializedRangeDropTraversalCertificate,
    CollectionSlotInitializedRangeDropTraversalProof,
};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, RawMemoryOp, ResourceMatchArm, ResourceOp};
use super::place_utils::place_suffix_after_prefix;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DropTraversalRangeCertificateEffect {
    Preserves,
    Invalidates,
}

pub(super) fn drop_traversal_range_certificate_survives_op(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    candidate: &CollectionSlotDropTraversalRangeCertificateCandidate,
    op: &ResourceOp,
) -> bool {
    matches!(
        op_effect(types, raw_aliases, candidate, op),
        DropTraversalRangeCertificateEffect::Preserves
    )
}

pub(super) fn transform_range_certificate_survives_op(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    candidate: &CollectionSlotTransformRangeCertificateCandidate,
    op: &ResourceOp,
) -> bool {
    let source_candidate = transform_anchor_as_drop_traversal_candidate(
        &candidate.source_storage,
        &candidate.source_initialized_count,
        candidate.expected_ty,
    );
    let output_candidate = transform_anchor_as_drop_traversal_candidate(
        &candidate.output_storage,
        &candidate.output_initialized_count,
        candidate.expected_ty,
    );
    drop_traversal_range_certificate_survives_op(types, raw_aliases, &source_candidate, op)
        && drop_traversal_range_certificate_survives_op(types, raw_aliases, &output_candidate, op)
}

fn transform_anchor_as_drop_traversal_candidate(
    storage: &Place,
    initialized_count: &Place,
    expected_ty: crate::types::TypeId,
) -> CollectionSlotDropTraversalRangeCertificateCandidate {
    CollectionSlotDropTraversalRangeCertificateCandidate {
        storage: storage.clone(),
        initialized_count: initialized_count.clone(),
        expected_ty,
        certificate: CollectionSlotInitializedRangeDropTraversalCertificate {
            element_stride: 0,
            drop_proof: CollectionSlotInitializedRangeDropTraversalProof::StateOnly,
        },
    }
}

fn op_effect(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    candidate: &CollectionSlotDropTraversalRangeCertificateCandidate,
    op: &ResourceOp,
) -> DropTraversalRangeCertificateEffect {
    if op_invalidates(types, raw_aliases, candidate, op) {
        DropTraversalRangeCertificateEffect::Invalidates
    } else {
        DropTraversalRangeCertificateEffect::Preserves
    }
}

fn op_invalidates(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    candidate: &CollectionSlotDropTraversalRangeCertificateCandidate,
    op: &ResourceOp,
) -> bool {
    match op {
        ResourceOp::Expr { output, .. } | ResourceOp::Read { output, .. } => {
            writes_candidate_anchor_direct(candidate, output)
        }
        ResourceOp::Borrow { output, .. }
        | ResourceOp::FunctionValue { output, .. }
        | ResourceOp::RawAddressAlias { target: output, .. }
        | ResourceOp::RawAddressView { target: output, .. }
        | ResourceOp::StorageOrigin { target: output, .. } => {
            writes_candidate_anchor(raw_aliases, candidate, output)
        }
        ResourceOp::DeclareLocal {
            place, initializer, ..
        } => {
            writes_candidate_anchor(raw_aliases, candidate, place)
                || initializer.as_ref().is_some_and(|initializer| {
                    consumes_candidate_anchor(types, raw_aliases, candidate, initializer)
                })
        }
        ResourceOp::Assign { target, value, .. } => {
            writes_candidate_anchor(raw_aliases, candidate, target)
                || consumes_candidate_anchor(types, raw_aliases, candidate, value)
        }
        ResourceOp::Move { source, output, .. } => {
            touches_candidate_anchor(raw_aliases, candidate, source)
                || writes_candidate_anchor(raw_aliases, candidate, output)
        }
        ResourceOp::Drop { place, .. } => touches_candidate_anchor(raw_aliases, candidate, place),
        ResourceOp::EndScope { locals, result, .. } => {
            locals
                .iter()
                .any(|local| touches_candidate_anchor(raw_aliases, candidate, local))
                || result.as_ref().is_some_and(|result| {
                    consumes_candidate_anchor(types, raw_aliases, candidate, result)
                })
        }
        ResourceOp::Call {
            output,
            args,
            target: _,
            effect: _,
            ..
        } => {
            writes_candidate_anchor(raw_aliases, candidate, output)
                || args
                    .iter()
                    .any(|arg| touches_candidate_anchor(raw_aliases, candidate, arg))
        }
        ResourceOp::IndirectCall {
            output,
            callee,
            args,
            ..
        } => {
            writes_candidate_anchor(raw_aliases, candidate, output)
                || touches_candidate_anchor(raw_aliases, candidate, callee)
                || args
                    .iter()
                    .any(|arg| touches_candidate_anchor(raw_aliases, candidate, arg))
        }
        ResourceOp::RawMemory {
            operation,
            output,
            args,
            ..
        } => {
            writes_candidate_anchor(raw_aliases, candidate, output)
                || raw_memory_operation_may_change_candidate_range(
                    raw_aliases,
                    candidate,
                    operation,
                    args,
                )
        }
        ResourceOp::CollectionSlotLifecycle { target, .. } => {
            touches_candidate_storage(raw_aliases, candidate, target)
        }
        ResourceOp::CollectionStorageRelocate {
            old_storage,
            new_storage,
            ..
        } => {
            touches_candidate_storage(raw_aliases, candidate, old_storage)
                || touches_candidate_storage(raw_aliases, candidate, new_storage)
        }
        ResourceOp::CollectionSlotDropTraversal {
            storage,
            initialized_count,
            ..
        } => {
            touches_candidate_storage(raw_aliases, candidate, storage)
                || touches_candidate_count(raw_aliases, candidate, initialized_count)
        }
        ResourceOp::CollectionSlotTransformRange {
            source_storage,
            source_initialized_count,
            output_storage,
            output_initialized_count,
            ..
        } => {
            touches_candidate_storage(raw_aliases, candidate, source_storage)
                || touches_candidate_count(raw_aliases, candidate, source_initialized_count)
                || touches_candidate_storage(raw_aliases, candidate, output_storage)
                || touches_candidate_count(raw_aliases, candidate, output_initialized_count)
        }
        ResourceOp::Construct { output, inputs, .. } => {
            writes_candidate_anchor(raw_aliases, candidate, output)
                || inputs
                    .iter()
                    .any(|input| consumes_candidate_anchor(types, raw_aliases, candidate, input))
        }
        ResourceOp::Branch {
            output,
            condition: _,
            condition_fact: _,
            then_ops,
            then_value,
            else_ops,
            else_value,
            ..
        } => {
            writes_candidate_anchor(raw_aliases, candidate, output)
                || consumes_candidate_anchor(types, raw_aliases, candidate, then_value)
                || consumes_candidate_anchor(types, raw_aliases, candidate, else_value)
                || ops_invalidate(types, raw_aliases, candidate, then_ops)
                || ops_invalidate(types, raw_aliases, candidate, else_ops)
        }
        ResourceOp::Loop {
            condition_ops,
            condition: _,
            condition_fact: _,
            body_ops,
            ..
        } => {
            ops_invalidate(types, raw_aliases, candidate, condition_ops)
                || ops_invalidate(types, raw_aliases, candidate, body_ops)
        }
        ResourceOp::Match {
            output,
            scrutinee,
            arms,
            ..
        } => {
            writes_candidate_anchor(raw_aliases, candidate, output)
                || touches_candidate_anchor(raw_aliases, candidate, scrutinee)
                || arms
                    .iter()
                    .any(|arm| match_arm_invalidates(types, raw_aliases, candidate, arm))
        }
        ResourceOp::CallEffect { .. } => false,
    }
}

fn ops_invalidate(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    candidate: &CollectionSlotDropTraversalRangeCertificateCandidate,
    ops: &[ResourceOp],
) -> bool {
    ops.iter()
        .any(|op| op_invalidates(types, raw_aliases, candidate, op))
}

fn match_arm_invalidates(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    candidate: &CollectionSlotDropTraversalRangeCertificateCandidate,
    arm: &ResourceMatchArm,
) -> bool {
    arm.bind_local
        .as_ref()
        .is_some_and(|bind| writes_candidate_anchor(raw_aliases, candidate, bind))
        || ops_invalidate(types, raw_aliases, candidate, &arm.ops)
        || consumes_candidate_anchor(types, raw_aliases, candidate, &arm.value)
}

fn raw_memory_operation_may_change_candidate_range(
    raw_aliases: &RawCellAddressAliases,
    candidate: &CollectionSlotDropTraversalRangeCertificateCandidate,
    operation: &RawMemoryOp,
    args: &[Place],
) -> bool {
    match operation {
        RawMemoryOp::Load => args
            .iter()
            .any(|arg| touches_candidate_storage(raw_aliases, candidate, arg)),
        RawMemoryOp::LoadU8 | RawMemoryOp::MemorySize | RawMemoryOp::MemoryGrow => false,
        RawMemoryOp::Store
        | RawMemoryOp::StoreU8
        | RawMemoryOp::Dealloc
        | RawMemoryOp::Realloc
        | RawMemoryOp::BulkCopy
        | RawMemoryOp::BulkMove
        | RawMemoryOp::FillBytes
        | RawMemoryOp::Fill
        | RawMemoryOp::Alloc => args
            .iter()
            .any(|arg| touches_candidate_storage(raw_aliases, candidate, arg)),
    }
}

fn writes_candidate_anchor(
    raw_aliases: &RawCellAddressAliases,
    candidate: &CollectionSlotDropTraversalRangeCertificateCandidate,
    place: &Place,
) -> bool {
    touches_candidate_storage(raw_aliases, candidate, place)
        || touches_candidate_count(raw_aliases, candidate, place)
}

fn writes_candidate_anchor_direct(
    candidate: &CollectionSlotDropTraversalRangeCertificateCandidate,
    place: &Place,
) -> bool {
    places_touch(place, &candidate.storage) || places_touch(place, &candidate.initialized_count)
}

fn consumes_candidate_anchor(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    candidate: &CollectionSlotDropTraversalRangeCertificateCandidate,
    place: &Place,
) -> bool {
    !types.is_copy(place.ty) && touches_candidate_anchor(raw_aliases, candidate, place)
}

fn touches_candidate_anchor(
    raw_aliases: &RawCellAddressAliases,
    candidate: &CollectionSlotDropTraversalRangeCertificateCandidate,
    place: &Place,
) -> bool {
    touches_candidate_storage(raw_aliases, candidate, place)
        || touches_candidate_count(raw_aliases, candidate, place)
}

fn touches_candidate_storage(
    raw_aliases: &RawCellAddressAliases,
    candidate: &CollectionSlotDropTraversalRangeCertificateCandidate,
    place: &Place,
) -> bool {
    places_touch_with_owner_aliases(raw_aliases, place, &candidate.storage)
}

fn touches_candidate_count(
    raw_aliases: &RawCellAddressAliases,
    candidate: &CollectionSlotDropTraversalRangeCertificateCandidate,
    place: &Place,
) -> bool {
    places_touch_with_scalar_aliases(raw_aliases, place, &candidate.initialized_count)
}

fn places_touch_with_owner_aliases(
    raw_aliases: &RawCellAddressAliases,
    left: &Place,
    right: &Place,
) -> bool {
    places_touch(left, right)
        || places_touch(
            &raw_aliases.canonicalize_owner_cell_address(left),
            &raw_aliases.canonicalize_owner_cell_address(right),
        )
}

fn places_touch_with_scalar_aliases(
    raw_aliases: &RawCellAddressAliases,
    left: &Place,
    right: &Place,
) -> bool {
    places_touch(left, right)
        || places_touch(
            &raw_aliases.canonicalize_scalar(left),
            &raw_aliases.canonicalize_scalar(right),
        )
}

fn places_touch(left: &Place, right: &Place) -> bool {
    place_suffix_after_prefix(left, right).is_some()
        || place_suffix_after_prefix(right, left).is_some()
}
