extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_model::CollectionSlotLifecycleReturnPath;
use super::collection_slot_summary_projection::summary_suffix_for_params;
use super::collection_slot_summary_return_model::CollectionSlotLifecycleReturnTransfer;
use super::collection_slot_summary_return_path_call::{
    collect_return_paths_from_call_summary, collect_return_paths_from_indirect_call_summary,
};
use super::collection_slot_summary_return_path_control::{
    return_value_branch_arm_start, return_value_is_never,
};
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
use super::model::{AggregateKind, Place, PlaceProjection, ResourceLocal, ResourceOp};
use super::place_utils::{construct_aggregate_field_place, place_suffix_after_prefix};
use super::variant_name::normalize_variant_name;

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
                return_variant: None,
                preconditions: path.preconditions,
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
                collect_construct_output_return_ranges(
                    &mut construct_paths,
                    engine,
                    params,
                    start.clone(),
                    &ops[..=index],
                    output,
                    target_suffix,
                );
                if target_suffix.is_empty() {
                    if let AggregateKind::Enum { variant, .. } = kind {
                        let variant = normalize_variant_name(variant);
                        for path in &mut construct_paths {
                            path.return_variant = Some(variant.clone());
                        }
                    }
                }
                push_merged_construct_return_paths(out, construct_paths);
                return true;
            }
            ResourceOp::Branch {
                output,
                condition_fact,
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
                            return_value_branch_arm_start(
                                engine,
                                params,
                                branch_start.clone(),
                                condition_fact.as_ref().map(|fact| (fact, true)),
                            ),
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
                            return_value_branch_arm_start(
                                engine,
                                params,
                                branch_start,
                                condition_fact.as_ref().map(|fact| (fact, false)),
                            ),
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
                                preconditions: match_start.preconditions.clone(),
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

fn collect_construct_output_return_ranges(
    out: &mut Vec<CollectionSlotLifecycleReturnPath>,
    engine: &ResourceCheckEngine<'_>,
    params: &[ResourceLocal],
    start: ReturnPathBuildState,
    ops_through_construct: &[ResourceOp],
    output: &Place,
    target_suffix: &[PlaceProjection],
) {
    for path in return_path_states_after_ops(engine, params, start, ops_through_construct) {
        let mut return_ranges = Vec::new();
        collect_return_ranges_for_value(
            &mut return_ranges,
            params,
            &path.state,
            output,
            target_suffix,
        );
        let i32_scalar_facts = collect_i32_scalar_return_facts_for_value_suffix(
            params,
            engine.types,
            &path.state.raw_aliases,
            output,
            target_suffix,
        );
        if return_ranges.is_empty() && i32_scalar_facts.is_empty() {
            continue;
        }
        push_return_path(
            out,
            CollectionSlotLifecycleReturnPath {
                return_variant: None,
                preconditions: path.preconditions,
                ops: path.ops,
                return_transfers: Vec::new(),
                return_slots: Vec::new(),
                return_ranges,
                i32_scalar_facts,
            },
        );
    }
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
            .filter(|existing| existing.preconditions == path.preconditions)
            .filter(|existing| existing.return_variant == path.return_variant)
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

#[cfg(test)]
mod tests {
    use alloc::vec;
    use alloc::vec::Vec;

    use crate::types::TypeId;

    use super::super::collection_slot_summary_model::CollectionSlotLifecycleReturnPath;
    use super::super::collection_slot_summary_return_model::{
        CollectionSlotLifecyclePathPrecondition, CollectionSlotLifecyclePathPreconditionOperand,
    };
    use super::super::i32_scalar_return_facts::{I32ScalarReturnConstant, I32ScalarReturnFacts};
    use super::super::model::I32ValueCondition;
    use super::*;

    /// construct の field ごとの return path は同じ path 条件に属する場合だけ合流する。
    ///
    /// 分岐由来の path を `ops` だけで合流すると、相互排他的な precondition の scalar fact
    /// が同じ return path に混ざり、caller 側で条件不明のまま同時に適用されてしまう。
    #[test]
    fn construct_return_path_merge_keeps_distinct_preconditions_separate() {
        let scalar_ty = TypeId(1);
        let mut out = Vec::new();

        push_merged_construct_return_paths(
            &mut out,
            vec![
                scalar_path_with_condition(scalar_ty, I32ValueCondition::EqZero, 0),
                scalar_path_with_condition(scalar_ty, I32ValueCondition::NeZero, 1),
            ],
        );

        assert_eq!(
            out.len(),
            2,
            "相互排他的な return path の scalar fact を同じ construct path に合流してはならない"
        );
    }

    fn scalar_path_with_condition(
        scalar_ty: TypeId,
        condition: I32ValueCondition,
        value: i32,
    ) -> CollectionSlotLifecycleReturnPath {
        CollectionSlotLifecycleReturnPath {
            return_variant: None,
            preconditions: vec![CollectionSlotLifecyclePathPrecondition::I32Condition {
                operand: CollectionSlotLifecyclePathPreconditionOperand::KnownI32 {
                    value: 0,
                    ty: scalar_ty,
                },
                condition,
            }],
            ops: Vec::new(),
            return_transfers: Vec::new(),
            return_slots: Vec::new(),
            return_ranges: Vec::new(),
            i32_scalar_facts: I32ScalarReturnFacts {
                constants: vec![I32ScalarReturnConstant {
                    return_projection: Vec::new(),
                    scalar_ty,
                    value,
                }],
                ..I32ScalarReturnFacts::default()
            },
        }
    }
}
