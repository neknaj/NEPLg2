extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_state_merge::merge_collection_slot_states;
use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryPlace;
use super::collection_slot_summary_return_model::{
    CollectionSlotLifecycleReturnSlot, CollectionSlotLifecycleReturnTransfer,
};
use super::model::{Place, ResourceLocal, ResourceOp};
use super::place_utils::{construct_aggregate_field_place, place_suffix_after_prefix};

pub(super) fn collect_return_transfers_from_ops(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    params: &[ResourceLocal],
    ops: &[ResourceOp],
    value: &Place,
) {
    collect_return_transfers_from_value_to_suffix(out, params, ops, value, &[], value.ty);
}

fn collect_return_transfers_from_value_to_suffix(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    params: &[ResourceLocal],
    ops: &[ResourceOp],
    value: &Place,
    target_suffix: &[super::model::PlaceProjection],
    target_ty: crate::types::TypeId,
) {
    for (parameter_index, param) in params.iter().enumerate() {
        let Some(source_suffix) = place_suffix_after_prefix(value, &param.place) else {
            continue;
        };
        push_return_transfer(
            out,
            CollectionSlotLifecycleReturnTransfer {
                source: CollectionSlotLifecycleSummaryPlace {
                    parameter_index,
                    suffix: source_suffix,
                    ty: value.ty,
                },
                target_suffix: target_suffix.to_vec(),
                target_ty,
            },
        );
    }
    collect_return_transfers_from_value_producer(out, params, ops, value, target_suffix, target_ty);
}

fn collect_return_transfers_from_value_producer(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    params: &[ResourceLocal],
    ops: &[ResourceOp],
    value: &Place,
    target_suffix: &[super::model::PlaceProjection],
    target_ty: crate::types::TypeId,
) {
    for index in (0..ops.len()).rev() {
        let prior_ops = &ops[..index];
        match &ops[index] {
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                ..
            } if output == value => {
                for (input_index, input) in inputs.iter().enumerate() {
                    let field = construct_aggregate_field_place(output, kind, input_index, input);
                    let Some(field_suffix) = place_suffix_after_prefix(&field, output) else {
                        continue;
                    };
                    let mut nested_target_suffix = target_suffix.to_vec();
                    nested_target_suffix.extend(field_suffix);
                    collect_return_transfers_from_value_to_suffix(
                        out,
                        params,
                        prior_ops,
                        input,
                        &nested_target_suffix,
                        input.ty,
                    );
                }
                return;
            }
            ResourceOp::Branch {
                output,
                then_ops,
                then_value,
                else_ops,
                else_value,
                ..
            } if output == value => {
                collect_return_transfers_from_value_to_suffix(
                    out,
                    params,
                    then_ops,
                    then_value,
                    target_suffix,
                    target_ty,
                );
                collect_return_transfers_from_value_to_suffix(
                    out,
                    params,
                    else_ops,
                    else_value,
                    target_suffix,
                    target_ty,
                );
                return;
            }
            ResourceOp::Match { output, arms, .. } if output == value => {
                for arm in arms {
                    collect_return_transfers_from_value_to_suffix(
                        out,
                        params,
                        &arm.ops,
                        &arm.value,
                        target_suffix,
                        target_ty,
                    );
                }
                return;
            }
            ResourceOp::DeclareLocal {
                place,
                initializer: Some(initializer),
                ..
            } if place == value => {
                collect_return_transfers_from_value_to_suffix(
                    out,
                    params,
                    prior_ops,
                    initializer,
                    target_suffix,
                    target_ty,
                );
                return;
            }
            ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. }
                if output == value =>
            {
                collect_return_transfers_from_value_to_suffix(
                    out,
                    params,
                    prior_ops,
                    source,
                    target_suffix,
                    target_ty,
                );
                return;
            }
            ResourceOp::Assign {
                target,
                value: assigned,
                ..
            } if target == value => {
                collect_return_transfers_from_value_to_suffix(
                    out,
                    params,
                    prior_ops,
                    assigned,
                    target_suffix,
                    target_ty,
                );
                return;
            }
            ResourceOp::Call { output, .. }
            | ResourceOp::IndirectCall { output, .. }
            | ResourceOp::Expr { output, .. }
                if output == value =>
            {
                return;
            }
            _ => {}
        }
    }
}

pub(super) fn collect_return_storage_markers(
    out: &mut Vec<CollectionSlotLifecycleReturnSlot>,
    markers: &[Place],
    value: &Place,
    state: CollectionSlotState,
) {
    for marker in markers {
        let Some(suffix) = place_suffix_after_prefix(marker, value) else {
            continue;
        };
        push_return_slot(
            out,
            CollectionSlotLifecycleReturnSlot {
                suffix,
                ty: marker.ty,
                state,
            },
        );
    }
}

pub(super) fn push_return_slot(
    out: &mut Vec<CollectionSlotLifecycleReturnSlot>,
    slot: CollectionSlotLifecycleReturnSlot,
) {
    if let Some(existing) = out
        .iter_mut()
        .find(|existing| existing.suffix == slot.suffix && existing.ty == slot.ty)
    {
        existing.state = merge_collection_slot_states(existing.state, slot.state);
    } else {
        out.push(slot);
    }
}

fn push_return_transfer(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    transfer: CollectionSlotLifecycleReturnTransfer,
) {
    if !out.iter().any(|existing| existing == &transfer) {
        out.push(transfer);
    }
}
