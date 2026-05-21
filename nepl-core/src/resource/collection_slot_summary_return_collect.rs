extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_state_merge::merge_collection_slot_states;
use super::collection_slot_summary_model::CollectionSlotLifecycleFunctionSummary;
use super::collection_slot_summary_return_model::{
    CollectionSlotLifecycleReturnSlot, CollectionSlotLifecycleReturnTransfer,
};
use super::collection_slot_summary_target::{instantiate_summary_target, summary_place_for_params};
use super::function_alias::{
    function_aliases_after_ops, function_aliases_for_match_arm, FunctionAliasTable,
};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceCallTarget, ResourceLocal, ResourceOp};
use super::place_utils::{construct_aggregate_field_place, place_suffix_after_prefix};

pub(super) fn collect_return_transfers_from_ops(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    function_aliases_at_start: &FunctionAliasTable,
    ops: &[ResourceOp],
    value: &Place,
) {
    collect_return_transfers_from_value_to_suffix(
        out,
        engine,
        params,
        raw_aliases,
        function_aliases_at_start,
        ops,
        value,
        &[],
        value.ty,
    );
}

fn collect_return_transfers_from_value_to_suffix(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    function_aliases_at_start: &FunctionAliasTable,
    ops: &[ResourceOp],
    value: &Place,
    target_suffix: &[super::model::PlaceProjection],
    target_ty: crate::types::TypeId,
) {
    let canonical_value = raw_aliases.canonicalize_owner_cell_address(value);
    if let Some(source) = summary_place_for_params(params, &canonical_value) {
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
        raw_aliases,
        function_aliases_at_start,
        ops,
        value,
        target_suffix,
        target_ty,
    );
}

fn collect_return_transfers_from_value_producer(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    function_aliases_at_start: &FunctionAliasTable,
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
                        engine,
                        params,
                        raw_aliases,
                        function_aliases_at_start,
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
                let branch_function_aliases =
                    function_aliases_after_ops(function_aliases_at_start, prior_ops);
                collect_return_transfers_from_value_to_suffix(
                    out,
                    engine,
                    params,
                    raw_aliases,
                    &branch_function_aliases,
                    then_ops,
                    then_value,
                    target_suffix,
                    target_ty,
                );
                collect_return_transfers_from_value_to_suffix(
                    out,
                    engine,
                    params,
                    raw_aliases,
                    &branch_function_aliases,
                    else_ops,
                    else_value,
                    target_suffix,
                    target_ty,
                );
                return;
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                ..
            } if output == value => {
                let match_function_aliases =
                    function_aliases_after_ops(function_aliases_at_start, prior_ops);
                for arm in arms {
                    let arm_function_aliases =
                        function_aliases_for_match_arm(&match_function_aliases, scrutinee, arm);
                    collect_return_transfers_from_value_to_suffix(
                        out,
                        engine,
                        params,
                        raw_aliases,
                        &arm_function_aliases,
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
                    raw_aliases,
                    function_aliases_at_start,
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
                    raw_aliases,
                    function_aliases_at_start,
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
                    raw_aliases,
                    function_aliases_at_start,
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
                collect_return_transfers_from_call_summary(
                    out,
                    engine,
                    params,
                    raw_aliases,
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
                let callsite_function_aliases =
                    function_aliases_after_ops(function_aliases_at_start, prior_ops);
                collect_return_transfers_from_indirect_call_summary(
                    out,
                    engine,
                    params,
                    raw_aliases,
                    &callsite_function_aliases,
                    callee,
                    args,
                    target_suffix,
                );
                return;
            }
            ResourceOp::Expr { output, .. } if output == value => {
                return;
            }
            _ => {}
        }
    }
}

fn collect_return_transfers_from_call_summary(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    target: &ResourceCallTarget,
    target_suffix: &[super::model::PlaceProjection],
) {
    let ResourceCallTarget::User { name, .. } = target else {
        return;
    };
    if let Some(summary) = engine.collection_slot_summaries.get(name) {
        collect_return_transfers_from_summary(
            out,
            engine,
            params,
            raw_aliases,
            args,
            summary,
            target_suffix,
        );
    }
}

fn collect_return_transfers_from_indirect_call_summary(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    callee: &Place,
    args: &[Place],
    target_suffix: &[super::model::PlaceProjection],
) {
    for function in function_aliases.functions(callee) {
        if let Some(summary) = engine.collection_slot_summaries.get(function) {
            collect_return_transfers_from_summary(
                out,
                engine,
                params,
                raw_aliases,
                args,
                summary,
                target_suffix,
            );
        }
    }
}

fn collect_return_transfers_from_summary(
    out: &mut Vec<CollectionSlotLifecycleReturnTransfer>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    summary: &CollectionSlotLifecycleFunctionSummary,
    target_suffix: &[super::model::PlaceProjection],
) {
    for transfer in &summary.return_transfers {
        let Some(source) = instantiate_summary_target(engine, args, &transfer.source) else {
            continue;
        };
        let source = raw_aliases.canonicalize_owner_cell_address(&source);
        let Some(source) = summary_place_for_params(params, &source) else {
            continue;
        };
        let mut composed_target_suffix = target_suffix.to_vec();
        composed_target_suffix.extend_from_slice(&transfer.target_suffix);
        push_return_transfer(
            out,
            CollectionSlotLifecycleReturnTransfer {
                source,
                target_suffix: composed_target_suffix,
                target_ty: transfer.target_ty,
            },
        );
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
