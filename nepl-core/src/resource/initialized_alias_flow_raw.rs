use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias::{ProjectedRawCellAddressAlias, RawCellAddressAliases};
use super::initialized_alias_flow::{
    expr_kind_preserves_raw_alias, expr_output_preserves_raw_alias, push_unique_return_alias,
    RawCellAddressReturnAlias, RawCellAddressReturnSummaryIndex,
};
use super::initialized_alias_flow_apply::{
    apply_direct_call_raw_alias_summary, apply_indirect_call_raw_alias_summary,
    construct_raw_cell_address_alias_fields,
};
use super::model::{
    EffectOp, Place, RawMemoryOp, ResourceFunction, ResourceOp, ResourceTerminator,
};
use super::place_utils::{
    match_bind_payload_place, reference_target_place, type_can_seed_raw_address_alias,
};

pub(super) fn function_raw_cell_address_return_aliases(
    function: &ResourceFunction,
    parameter_index: usize,
    parameter: &Place,
    summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> Vec<RawCellAddressReturnAlias> {
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut function_aliases = FunctionAliasTable::default();
    if type_can_seed_raw_address_alias(types, parameter.ty) {
        raw_aliases.mark(parameter);
    }
    let mut aliases = Vec::new();
    for block in &function.blocks {
        propagate_raw_address_alias_ops(
            &mut raw_aliases,
            &mut function_aliases,
            &block.ops,
            summaries,
            types,
        );
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            for alias in raw_aliases.projected_aliases_between(value, parameter) {
                push_unique_return_alias(
                    &mut aliases,
                    return_alias_from_projected(parameter_index, alias),
                );
            }
        }
    }
    aliases
}

fn propagate_raw_address_alias_ops(
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    ops: &[ResourceOp],
    summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) {
    for op in ops {
        propagate_raw_address_alias_op(raw_aliases, function_aliases, op, summaries, types);
    }
}

fn propagate_raw_address_alias_op(
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    op: &ResourceOp,
    summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) {
    match op {
        ResourceOp::DeclareLocal {
            place, initializer, ..
        } => {
            if let Some(initializer) = initializer {
                raw_aliases.copy_alias_if_tracked(initializer, place);
                function_aliases.copy_alias(initializer, place);
            } else {
                raw_aliases.clear(place);
            }
        }
        ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
            raw_aliases.copy_alias_if_tracked(source, output);
            function_aliases.copy_alias(source, output);
        }
        ResourceOp::Assign { target, value, .. } => {
            raw_aliases.copy_alias_if_tracked(value, target);
            function_aliases.copy_alias(value, target);
        }
        ResourceOp::RawMemory {
            operation, output, ..
        } => match operation {
            RawMemoryOp::Alloc | RawMemoryOp::Realloc => raw_aliases.mark(output),
            RawMemoryOp::Load
            | RawMemoryOp::LoadU8
            | RawMemoryOp::Store
            | RawMemoryOp::StoreU8
            | RawMemoryOp::Dealloc
            | RawMemoryOp::BulkCopy
            | RawMemoryOp::BulkMove
            | RawMemoryOp::MemorySize
            | RawMemoryOp::MemoryGrow
            | RawMemoryOp::FillBytes
            | RawMemoryOp::Fill => {}
        },
        ResourceOp::RawAddressAlias { source, target, .. } => {
            raw_aliases.copy_explicit_raw_address_alias(source, target);
        }
        ResourceOp::RawAddressView { source, target, .. } => {
            if raw_aliases.raw_address_view_source_is_known(source) {
                raw_aliases.copy_explicit_raw_address_alias(source, target);
            } else {
                raw_aliases.clear(target);
            }
        }
        ResourceOp::StorageOrigin { .. } => {}
        ResourceOp::Construct {
            output,
            kind,
            inputs,
            ..
        } => {
            raw_aliases.clear(output);
            construct_raw_cell_address_alias_fields(raw_aliases, output, kind, inputs);
            construct_function_alias_fields(function_aliases, output, kind, inputs);
        }
        ResourceOp::FunctionValue { output, name, .. } => {
            function_aliases.set_alias(output, name.clone());
        }
        ResourceOp::Call {
            output,
            target,
            args,
            effect,
            ..
        } => {
            if !matches!(
                effect,
                EffectOp::InternalAlloc { .. } | EffectOp::UnsafeMemory { .. }
            ) && !apply_direct_call_raw_alias_summary(
                raw_aliases,
                output,
                target,
                args,
                summaries,
                types,
            ) {
                raw_aliases.clear(output);
            }
        }
        ResourceOp::IndirectCall {
            output,
            callee,
            args,
            ..
        } => {
            if !apply_indirect_call_raw_alias_summary(
                raw_aliases,
                function_aliases,
                output,
                callee,
                args,
                summaries,
                types,
            ) {
                raw_aliases.clear(output);
            }
        }
        ResourceOp::Branch {
            output,
            then_ops,
            then_value,
            else_ops,
            else_value,
            ..
        } => {
            let mut then_aliases = raw_aliases.clone();
            let mut else_aliases = raw_aliases.clone();
            let mut then_function_aliases = function_aliases.clone();
            let mut else_function_aliases = function_aliases.clone();
            propagate_raw_address_alias_ops(
                &mut then_aliases,
                &mut then_function_aliases,
                then_ops,
                summaries,
                types,
            );
            propagate_raw_address_alias_ops(
                &mut else_aliases,
                &mut else_function_aliases,
                else_ops,
                summaries,
                types,
            );
            then_aliases.copy_alias_if_tracked(then_value, output);
            else_aliases.copy_alias_if_tracked(else_value, output);
            then_function_aliases.copy_alias(then_value, output);
            else_function_aliases.copy_alias(else_value, output);
            *raw_aliases = RawCellAddressAliases::merge_paths(&[then_aliases, else_aliases]);
            *function_aliases =
                FunctionAliasTable::merge_paths(&[then_function_aliases, else_function_aliases]);
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            let mut condition_aliases = raw_aliases.clone();
            let mut condition_function_aliases = function_aliases.clone();
            propagate_raw_address_alias_ops(
                &mut condition_aliases,
                &mut condition_function_aliases,
                condition_ops,
                summaries,
                types,
            );
            let mut body_aliases = condition_aliases.clone();
            let mut body_function_aliases = condition_function_aliases.clone();
            propagate_raw_address_alias_ops(
                &mut body_aliases,
                &mut body_function_aliases,
                body_ops,
                summaries,
                types,
            );
            *raw_aliases = RawCellAddressAliases::merge_paths(&[condition_aliases, body_aliases]);
            *function_aliases = FunctionAliasTable::merge_paths(&[
                condition_function_aliases,
                body_function_aliases,
            ]);
        }
        ResourceOp::Match {
            output,
            scrutinee,
            arms,
            ..
        } => {
            let mut alias_paths = Vec::new();
            let mut function_alias_paths = Vec::new();
            for arm in arms {
                let mut arm_aliases = raw_aliases.clone();
                let mut arm_function_aliases = function_aliases.clone();
                if let Some(bind_local) = &arm.bind_local {
                    if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
                        arm_aliases.copy_alias_if_tracked(&source, bind_local);
                        arm_function_aliases.copy_alias(&source, bind_local);
                    } else {
                        arm_aliases.clear(bind_local);
                    }
                }
                propagate_raw_address_alias_ops(
                    &mut arm_aliases,
                    &mut arm_function_aliases,
                    &arm.ops,
                    summaries,
                    types,
                );
                arm_aliases.copy_alias_if_tracked(&arm.value, output);
                arm_function_aliases.copy_alias(&arm.value, output);
                alias_paths.push(arm_aliases);
                function_alias_paths.push(arm_function_aliases);
            }
            if !alias_paths.is_empty() {
                *raw_aliases = RawCellAddressAliases::merge_paths(&alias_paths);
                *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
            }
        }
        ResourceOp::Expr { output, kind, .. } => {
            if !expr_kind_preserves_raw_alias(*kind)
                && !expr_output_preserves_raw_alias(types, *kind, output)
            {
                raw_aliases.clear(output);
            }
        }
        ResourceOp::Borrow { source, output, .. } => {
            raw_aliases.mark(output);
            let target = reference_target_place(output, source.ty);
            raw_aliases.copy_alias_if_tracked(source, &target);
        }
        ResourceOp::Drop { place, .. } => raw_aliases.clear(place),
        ResourceOp::CallEffect { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. } => {}
    }
}

fn return_alias_from_projected(
    parameter_index: usize,
    alias: ProjectedRawCellAddressAlias,
) -> RawCellAddressReturnAlias {
    RawCellAddressReturnAlias {
        parameter_index,
        parameter_projection: alias.right_projection,
        parameter_ty: alias.right_ty,
        return_projection: alias.left_projection,
        return_ty: alias.left_ty,
    }
}
