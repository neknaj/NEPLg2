extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_model::CollectionSlotLifecycleReturnPath;
use super::collection_slot_summary_return_model::CollectionSlotLifecycleReturnTransfer;
use super::collection_slot_summary_return_path_call::{
    collect_return_paths_from_call_summary, collect_return_paths_from_indirect_call_summary,
};
use super::collection_slot_summary_return_path_control::return_value_is_never;
use super::collection_slot_summary_return_path_model::{push_return_path, ReturnPathBuildState};
use super::collection_slot_summary_return_path_slots::collect_return_slots_for_value;
use super::collection_slot_summary_return_path_state::return_path_states_after_ops;
use super::collection_slot_summary_return_unique::push_return_transfer;
use super::collection_slot_summary_target::summary_place_for_params;
use super::initialized::ResourceCheckEngine;
use super::model::{Place, PlaceProjection, ResourceLocal, ResourceOp};
use super::place_utils::{construct_aggregate_field_place, place_suffix_after_prefix};

pub(super) fn collect_return_paths_from_value_to_suffix(
    out: &mut Vec<CollectionSlotLifecycleReturnPath>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    start: ReturnPathBuildState,
    ops: &[ResourceOp],
    value: &Place,
    target_suffix: &[PlaceProjection],
    target_ty: crate::types::TypeId,
) {
    for path in return_path_states_after_ops(engine, params, start.clone(), ops) {
        collect_direct_return_path(out, params, path, value, target_suffix, target_ty);
    }
    collect_return_paths_from_value_producer(out, engine, params, start, ops, value, target_suffix);
}

fn collect_direct_return_path(
    out: &mut Vec<CollectionSlotLifecycleReturnPath>,
    params: &[ResourceLocal],
    path: ReturnPathBuildState,
    value: &Place,
    target_suffix: &[PlaceProjection],
    target_ty: crate::types::TypeId,
) {
    let mut return_transfers = Vec::new();
    let mut return_slots = Vec::new();
    let canonical_value = path
        .state
        .raw_aliases
        .canonicalize_owner_cell_address(value);
    if let Some(source) = summary_place_for_params(params, &canonical_value) {
        push_return_transfer(
            &mut return_transfers,
            CollectionSlotLifecycleReturnTransfer {
                source,
                target_suffix: target_suffix.to_vec(),
                target_ty,
            },
        );
    }
    collect_return_slots_for_value(&mut return_slots, &path.state, value, target_suffix);
    if !return_transfers.is_empty() || !return_slots.is_empty() {
        push_return_path(
            out,
            CollectionSlotLifecycleReturnPath {
                ops: path.ops,
                return_transfers,
                return_slots,
            },
        );
    }
}

fn collect_return_paths_from_value_producer(
    out: &mut Vec<CollectionSlotLifecycleReturnPath>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    start: ReturnPathBuildState,
    ops: &[ResourceOp],
    value: &Place,
    target_suffix: &[PlaceProjection],
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
                    collect_return_paths_from_value_to_suffix(
                        out,
                        engine,
                        params,
                        start.clone(),
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
                for branch_start in return_path_states_after_ops(engine, params, start, prior_ops) {
                    if !return_value_is_never(engine, then_value) {
                        collect_return_paths_from_value_to_suffix(
                            out,
                            engine,
                            params,
                            branch_start.clone(),
                            then_ops,
                            then_value,
                            target_suffix,
                            value.ty,
                        );
                    }
                    if !return_value_is_never(engine, else_value) {
                        collect_return_paths_from_value_to_suffix(
                            out,
                            engine,
                            params,
                            branch_start,
                            else_ops,
                            else_value,
                            target_suffix,
                            value.ty,
                        );
                    }
                }
                return;
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                ..
            } if output == value => {
                for match_start in return_path_states_after_ops(engine, params, start, prior_ops) {
                    for arm in arms {
                        if return_value_is_never(engine, &arm.value) {
                            continue;
                        }
                        let Some(arm_state) =
                            super::collection_slot_summary_match_state::collection_slot_summary_match_arm_entry_state(
                                engine,
                                &match_start.state,
                                scrutinee,
                                arm,
                            )
                        else {
                            continue;
                        };
                        collect_return_paths_from_value_to_suffix(
                            out,
                            engine,
                            params,
                            ReturnPathBuildState {
                                state: arm_state,
                                ops: match_start.ops.clone(),
                            },
                            &arm.ops,
                            &arm.value,
                            target_suffix,
                            value.ty,
                        );
                    }
                }
                return;
            }
            ResourceOp::DeclareLocal {
                place,
                initializer: Some(initializer),
                ..
            } if place == value => {
                collect_return_paths_from_value_to_suffix(
                    out,
                    engine,
                    params,
                    start,
                    prior_ops,
                    initializer,
                    target_suffix,
                    value.ty,
                );
                return;
            }
            ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. }
                if output == value =>
            {
                collect_return_paths_from_value_to_suffix(
                    out,
                    engine,
                    params,
                    start,
                    prior_ops,
                    source,
                    target_suffix,
                    value.ty,
                );
                return;
            }
            ResourceOp::Assign {
                target,
                value: assigned,
                ..
            } if target == value => {
                collect_return_paths_from_value_to_suffix(
                    out,
                    engine,
                    params,
                    start,
                    prior_ops,
                    assigned,
                    target_suffix,
                    value.ty,
                );
                return;
            }
            ResourceOp::Call {
                output,
                target,
                args,
                ..
            } if output == value => {
                for callsite in return_path_states_after_ops(engine, params, start, prior_ops) {
                    collect_return_paths_from_call_summary(
                        out,
                        engine,
                        params,
                        callsite,
                        args,
                        target,
                        target_suffix,
                    );
                }
                return;
            }
            ResourceOp::IndirectCall {
                output,
                callee,
                args,
                ..
            } if output == value => {
                for callsite in return_path_states_after_ops(engine, params, start, prior_ops) {
                    collect_return_paths_from_indirect_call_summary(
                        out,
                        engine,
                        params,
                        callsite,
                        callee,
                        args,
                        target_suffix,
                    );
                }
                return;
            }
            ResourceOp::Expr { output, .. } if output == value => return,
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
            } if place == value => return,
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
            | ResourceOp::Construct { .. }
            | ResourceOp::Branch { .. }
            | ResourceOp::Loop { .. }
            | ResourceOp::Match { .. } => {}
        }
    }
}
