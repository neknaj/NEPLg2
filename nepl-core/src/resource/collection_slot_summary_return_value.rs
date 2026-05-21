extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_return_call::{
    collect_return_transfers_from_call_summary, collect_return_transfers_from_indirect_call_summary,
};
use super::collection_slot_summary_return_model::CollectionSlotLifecycleReturnTransfer;
use super::collection_slot_summary_return_path_control::return_value_is_never;
use super::collection_slot_summary_return_state::collection_slot_summary_state_after_ops;
use super::collection_slot_summary_return_unique::push_return_transfer;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, ResourceOp};
use super::place_utils::construct_aggregate_field_place;

pub(super) fn collect_return_transfers_from_value_to_suffix(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[super::model::ResourceLocal],
    state_at_start: &CollectionSlotSummaryBuildState,
    ops: &[ResourceOp],
    value: &Place,
    target_suffix: &[super::model::PlaceProjection],
    target_ty: crate::types::TypeId,
) {
    let state_at_value = collection_slot_summary_state_after_ops(engine, state_at_start, ops);
    let canonical_value = state_at_value
        .raw_aliases
        .canonicalize_owner_cell_address(value);
    if let Some(source) =
        super::collection_slot_summary_target::summary_place_for_params(params, &canonical_value)
    {
        push_return_transfer(
            out,
            CollectionSlotLifecycleReturnTransfer {
                source,
                target_suffix: target_suffix.to_vec(),
                target_ty,
            },
        );
    }
    collect_return_transfers_from_value_producer(
        out,
        engine,
        params,
        state_at_start,
        ops,
        value,
        target_suffix,
        target_ty,
    );
}

fn collect_return_transfers_from_value_producer(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[super::model::ResourceLocal],
    state_at_start: &CollectionSlotSummaryBuildState,
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
                    let Some(field_suffix) =
                        super::place_utils::place_suffix_after_prefix(&field, output)
                    else {
                        continue;
                    };
                    let mut nested_target_suffix = target_suffix.to_vec();
                    nested_target_suffix.extend(field_suffix);
                    collect_return_transfers_from_value_to_suffix(
                        out,
                        engine,
                        params,
                        state_at_start,
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
                let branch_state =
                    collection_slot_summary_state_after_ops(engine, state_at_start, prior_ops);
                if !return_value_is_never(engine, then_value) {
                    collect_return_transfers_from_value_to_suffix(
                        out,
                        engine,
                        params,
                        &branch_state,
                        then_ops,
                        then_value,
                        target_suffix,
                        target_ty,
                    );
                }
                if !return_value_is_never(engine, else_value) {
                    collect_return_transfers_from_value_to_suffix(
                        out,
                        engine,
                        params,
                        &branch_state,
                        else_ops,
                        else_value,
                        target_suffix,
                        target_ty,
                    );
                }
                return;
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                ..
            } if output == value => {
                let match_state =
                    collection_slot_summary_state_after_ops(engine, state_at_start, prior_ops);
                for arm in arms {
                    if return_value_is_never(engine, &arm.value) {
                        continue;
                    }
                    let Some(arm_state) =
                        super::collection_slot_summary_match_state::collection_slot_summary_match_arm_entry_state(
                            engine,
                            &match_state,
                            scrutinee,
                            arm,
                        )
                    else {
                        continue;
                    };
                    collect_return_transfers_from_value_to_suffix(
                        out,
                        engine,
                        params,
                        &arm_state,
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
                    engine,
                    params,
                    state_at_start,
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
                    engine,
                    params,
                    state_at_start,
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
                    engine,
                    params,
                    state_at_start,
                    prior_ops,
                    assigned,
                    target_suffix,
                    target_ty,
                );
                return;
            }
            ResourceOp::Call {
                output,
                target,
                args,
                ..
            } if output == value => {
                let callsite_state =
                    collection_slot_summary_state_after_ops(engine, state_at_start, prior_ops);
                collect_return_transfers_from_call_summary(
                    out,
                    engine,
                    params,
                    &callsite_state.raw_aliases,
                    args,
                    target,
                    target_suffix,
                );
                return;
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                ..
            } if output == value => {
                let callsite_state =
                    collection_slot_summary_state_after_ops(engine, state_at_start, prior_ops);
                collect_return_transfers_from_indirect_call_summary(
                    out,
                    engine,
                    params,
                    &callsite_state.raw_aliases,
                    &callsite_state.function_aliases,
                    callee,
                    args,
                    target_suffix,
                );
                return;
            }
            ResourceOp::Expr { output, .. } if output == value => {
                return;
            }
            ResourceOp::Borrow { output, .. }
            | ResourceOp::FunctionValue { output, .. }
            | ResourceOp::RawMemory { output, .. }
                if output == value =>
            {
                return;
            }
            ResourceOp::RawAddressAlias { target, .. }
            | ResourceOp::RawAddressView { target, .. }
            | ResourceOp::StorageOrigin { target, .. }
                if target == value =>
            {
                return;
            }
            ResourceOp::DeclareLocal {
                place,
                initializer: None,
                ..
            } if place == value => {
                return;
            }
            ResourceOp::Expr { .. }
            | ResourceOp::DeclareLocal { .. }
            | ResourceOp::Read { .. }
            | ResourceOp::Assign { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Move { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::EndScope { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::FunctionValue { .. }
            | ResourceOp::Call { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::RawMemory { .. }
            | ResourceOp::RawAddressAlias { .. }
            | ResourceOp::RawAddressView { .. }
            | ResourceOp::StorageOrigin { .. }
            | ResourceOp::CollectionSlotLifecycle { .. }
            | ResourceOp::CollectionStorageRelocate { .. }
            | ResourceOp::CollectionSlotDropTraversal { .. }
            | ResourceOp::Construct { .. }
            | ResourceOp::Branch { .. }
            | ResourceOp::Loop { .. }
            | ResourceOp::Match { .. } => {}
        }
    }
}
