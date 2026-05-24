extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_model::CollectionSlotLifecycleReturnPath;
use super::collection_slot_summary_projection::summary_suffix_for_params;
use super::collection_slot_summary_return_model::CollectionSlotLifecycleReturnTransfer;
use super::collection_slot_summary_return_path_call::{
    collect_return_paths_from_call_summary, collect_return_paths_from_indirect_call_summary,
};
use super::collection_slot_summary_return_path_control::return_value_is_never;
use super::collection_slot_summary_return_path_model::{push_return_path, ReturnPathBuildState};
use super::collection_slot_summary_return_path_slots::collect_return_slots_for_value;
use super::collection_slot_summary_return_path_state::return_path_states_after_ops;
use super::collection_slot_summary_return_range::collect_return_ranges_for_value;
use super::collection_slot_summary_return_unique::{
    push_return_range, push_return_slot, push_return_transfer,
};
use super::collection_slot_summary_target::summary_place_for_params_with_aliases;
use super::i32_scalar_return_facts::collect_i32_scalar_return_facts_for_value_suffix;
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
    if collect_return_paths_from_value_producer(
        out,
        engine,
        params,
        start.clone(),
        ops,
        value,
        target_suffix,
    ) {
        return;
    }
    for path in return_path_states_after_ops(engine, params, start, ops) {
        collect_direct_return_path(
            out,
            engine,
            params,
            path,
            ops,
            value,
            target_suffix,
            target_ty,
        );
    }
}

fn collect_direct_return_path(
    out: &mut Vec<CollectionSlotLifecycleReturnPath>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    path: ReturnPathBuildState,
    ops: &[ResourceOp],
    value: &Place,
    target_suffix: &[PlaceProjection],
    target_ty: crate::types::TypeId,
) {
    let mut return_transfers = Vec::new();
    let mut return_slots = Vec::new();
    let mut return_ranges = Vec::new();
    let canonical_value = path
        .state
        .raw_aliases
        .canonicalize_owner_cell_address(value);
    if let Some(source) =
        summary_place_for_params_with_aliases(params, &path.state.raw_aliases, &canonical_value)
    {
        if let Some(target_suffix) = summary_suffix_for_params(params, target_suffix) {
            push_return_transfer(
                &mut return_transfers,
                CollectionSlotLifecycleReturnTransfer {
                    source,
                    target_suffix,
                    target_ty,
                },
            );
        }
    }
    collect_storage_relocate_return_transfers(
        &mut return_transfers,
        params,
        &path,
        ops,
        value,
        target_suffix,
    );
    collect_return_slots_for_value(&mut return_slots, params, &path.state, value, target_suffix);
    collect_return_ranges_for_value(
        &mut return_ranges,
        params,
        &path.state,
        value,
        target_suffix,
    );
    let i32_scalar_facts = collect_i32_scalar_return_facts_for_value_suffix(
        params,
        engine.types,
        &path.state.raw_aliases,
        value,
        target_suffix,
    );
    if !return_transfers.is_empty()
        || !return_slots.is_empty()
        || !return_ranges.is_empty()
        || !i32_scalar_facts.is_empty()
    {
        push_return_path(
            out,
            CollectionSlotLifecycleReturnPath {
                ops: path.ops,
                return_transfers,
                return_slots,
                return_ranges,
                i32_scalar_facts,
            },
        );
    }
}

fn collect_storage_relocate_return_transfers(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    params: &[ResourceLocal],
    path: &ReturnPathBuildState,
    ops: &[ResourceOp],
    value: &Place,
    target_suffix: &[PlaceProjection],
) {
    for op in ops {
        let ResourceOp::CollectionStorageRelocate {
            old_storage,
            new_storage,
            ..
        } = op
        else {
            continue;
        };
        let old_storage = path
            .state
            .raw_aliases
            .canonicalize_owner_cell_address(old_storage);
        let new_storage = path
            .state
            .raw_aliases
            .canonicalize_owner_cell_address(new_storage);
        let Some(storage_suffix) = place_suffix_after_prefix(&new_storage, value) else {
            continue;
        };
        let Some(source) =
            summary_place_for_params_with_aliases(params, &path.state.raw_aliases, &old_storage)
        else {
            continue;
        };
        let mut composed_target_suffix = target_suffix.to_vec();
        composed_target_suffix.extend(storage_suffix);
        let Some(target_suffix) = summary_suffix_for_params(params, &composed_target_suffix) else {
            continue;
        };
        push_return_transfer(
            out,
            CollectionSlotLifecycleReturnTransfer {
                source,
                target_suffix,
                target_ty: new_storage.ty,
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
) -> bool {
    for index in (0..ops.len()).rev() {
        let prior_ops = &ops[..index];
        match &ops[index] {
            ResourceOp::Construct {
                output,
                kind,
                inputs,
                ..
            } if output == value => {
                let mut construct_paths = Vec::new();
                for (input_index, input) in inputs.iter().enumerate() {
                    let field = construct_aggregate_field_place(output, kind, input_index, input);
                    let Some(field_suffix) = place_suffix_after_prefix(&field, output) else {
                        continue;
                    };
                    let mut nested_target_suffix = target_suffix.to_vec();
                    nested_target_suffix.extend(field_suffix);
                    collect_return_paths_from_value_to_suffix(
                        &mut construct_paths,
                        engine,
                        params,
                        start.clone(),
                        prior_ops,
                        input,
                        &nested_target_suffix,
                        input.ty,
                    );
                }
                push_merged_construct_return_paths(out, construct_paths);
                return true;
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
                return true;
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
                return true;
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
                return true;
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
                return true;
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
                return true;
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
                return true;
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
                return true;
            }
            ResourceOp::Expr { output, .. } if output == value => {}
            ResourceOp::Borrow { output, .. }
            | ResourceOp::FunctionValue { output, .. }
            | ResourceOp::RawMemory { output, .. }
                if output == value =>
            {
                return true;
            }
            ResourceOp::RawAddressAlias { target, .. }
            | ResourceOp::RawAddressView { target, .. }
            | ResourceOp::StorageOrigin { target, .. }
                if target == value =>
            {
                return true;
            }
            ResourceOp::DeclareLocal {
                place,
                initializer: None,
                ..
            } if place == value => return true,
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
            | ResourceOp::CollectionSlotTransformRange { .. }
            | ResourceOp::Construct { .. }
            | ResourceOp::Branch { .. }
            | ResourceOp::Loop { .. }
            | ResourceOp::Match { .. } => {}
        }
    }
    false
}

fn push_merged_construct_return_paths(
    out: &mut Vec<CollectionSlotLifecycleReturnPath>,
    paths: Vec<CollectionSlotLifecycleReturnPath>,
) {
    let mut merged_paths = Vec::new();
    for path in paths {
        if let Some(existing) = merged_paths
            .iter_mut()
            .find(|existing: &&mut CollectionSlotLifecycleReturnPath| existing.ops == path.ops)
        {
            for transfer in path.return_transfers {
                push_return_transfer(&mut existing.return_transfers, transfer);
            }
            for slot in path.return_slots {
                push_return_slot(&mut existing.return_slots, slot);
            }
            for range in path.return_ranges {
                push_return_range(&mut existing.return_ranges, range);
            }
            existing.i32_scalar_facts.extend(path.i32_scalar_facts);
        } else {
            push_return_path(&mut merged_paths, path);
        }
    }
    for path in merged_paths {
        push_return_path(out, path);
    }
}
