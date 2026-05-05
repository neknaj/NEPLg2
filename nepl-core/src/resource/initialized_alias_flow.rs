extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias::{ProjectedRawCellAddressAlias, RawCellAddressAliases};
use super::initialized_alias_type::{
    projected_place_with_concrete_type, type_preserves_raw_address_alias,
};
use super::initialized_projection_domain::{
    normalize_storage_offsets, widen_projection, MAX_EXACT_PROJECTION_FACTS_PER_SHAPE,
};
use super::model::{
    AggregateKind, EffectOp, Place, PlaceProjection, RawMemoryOp, ResourceCallTarget,
    ResourceExprKind, ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::place_utils::{construct_aggregate_field_place, match_bind_payload_place};
use super::raw_address_seed::should_seed_raw_address_parameter;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct RawCellAddressReturnSummary {
    function: String,
    aliases: Vec<RawCellAddressReturnAlias>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct RawCellAddressReturnAlias {
    parameter_index: usize,
    parameter_projection: Vec<PlaceProjection>,
    parameter_ty: TypeId,
    return_projection: Vec<PlaceProjection>,
    return_ty: TypeId,
}

pub(super) fn expr_kind_preserves_raw_alias(kind: ResourceExprKind) -> bool {
    matches!(
        kind,
        ResourceExprKind::LocalRead
            | ResourceExprKind::Call
            | ResourceExprKind::IndirectCall
            | ResourceExprKind::Intrinsic
            | ResourceExprKind::Branch
            | ResourceExprKind::Match
            | ResourceExprKind::Construct
    )
}

pub(super) fn compute_raw_cell_address_return_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
) -> Vec<RawCellAddressReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let mut aliases = Vec::new();
            for (index, param) in function.params.iter().enumerate() {
                if !should_seed_raw_address_parameter(function, &param.place, types) {
                    continue;
                }
                aliases.extend(function_raw_cell_address_return_aliases(
                    function,
                    index,
                    &param.place,
                    &summaries,
                    types,
                ));
            }
            if !aliases.is_empty() {
                aliases.sort();
                next.push(RawCellAddressReturnSummary {
                    function: function.name.clone(),
                    aliases,
                });
            }
        }
        next.sort();
        if next == summaries {
            return summaries;
        }
        summaries = next;
    }
    summaries
}

fn function_raw_cell_address_return_aliases(
    function: &ResourceFunction,
    parameter_index: usize,
    parameter: &Place,
    summaries: &[RawCellAddressReturnSummary],
    types: &TypeCtx,
) -> Vec<RawCellAddressReturnAlias> {
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut function_aliases = FunctionAliasTable::default();
    raw_aliases.mark(parameter);
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
    summaries: &[RawCellAddressReturnSummary],
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
    summaries: &[RawCellAddressReturnSummary],
    types: &TypeCtx,
) {
    match op {
        ResourceOp::DeclareLocal {
            place, initializer, ..
        } => {
            if let Some(initializer) = initializer {
                raw_aliases.copy_alias_or_seed(initializer, place);
                function_aliases.copy_alias(initializer, place);
            } else {
                raw_aliases.clear(place);
            }
        }
        ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
            raw_aliases.copy_alias_or_seed(source, output);
            function_aliases.copy_alias(source, output);
        }
        ResourceOp::Assign { target, value, .. } => {
            raw_aliases.copy_alias_or_seed(value, target);
            function_aliases.copy_alias(value, target);
        }
        ResourceOp::RawMemory {
            operation, output, ..
        } => match operation {
            RawMemoryOp::Alloc | RawMemoryOp::Realloc => raw_aliases.mark(output),
            RawMemoryOp::Load
            | RawMemoryOp::Store
            | RawMemoryOp::Dealloc
            | RawMemoryOp::BulkCopy
            | RawMemoryOp::BulkMove
            | RawMemoryOp::MemorySize
            | RawMemoryOp::MemoryGrow
            | RawMemoryOp::Fill { .. }
            | RawMemoryOp::Other { .. } => {}
        },
        ResourceOp::RawAddressAlias { source, target, .. }
        | ResourceOp::RawAddressView { source, target, .. } => {
            raw_aliases.copy_alias_or_seed(source, target);
        }
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
                EffectOp::InternalAlloc | EffectOp::UnsafeMemory { .. }
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
            then_aliases.copy_alias_or_seed(then_value, output);
            else_aliases.copy_alias_or_seed(else_value, output);
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
                        arm_aliases.copy_alias_or_seed(&source, bind_local);
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
                arm_aliases.copy_alias_or_seed(&arm.value, output);
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
            if matches!(kind, ResourceExprKind::Borrow) {
                raw_aliases.clear_exact(output);
            } else if !expr_kind_preserves_raw_alias(*kind)
                && !(matches!(kind, ResourceExprKind::Deref)
                    && type_preserves_raw_address_alias(types, output.ty))
            {
                raw_aliases.clear(output);
            }
        }
        ResourceOp::Borrow { source, output, .. } => {
            let deref_output = output
                .clone()
                .with_projection(PlaceProjection::Deref, source.ty);
            raw_aliases.copy_alias_or_seed(source, &deref_output);
        }
        ResourceOp::Drop { place, .. } => raw_aliases.clear(place),
        ResourceOp::CallEffect { .. } => {}
    }
}

pub(super) fn construct_raw_cell_address_alias_fields(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    kind: &AggregateKind,
    inputs: &[Place],
) {
    for (index, input) in inputs.iter().enumerate() {
        let field = construct_aggregate_field_place(output, kind, index, input);
        raw_aliases.copy_alias_or_seed(input, &field);
    }
}

pub(super) fn apply_direct_call_raw_alias_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    target: &ResourceCallTarget,
    args: &[Place],
    summaries: &[RawCellAddressReturnSummary],
    types: &TypeCtx,
) -> bool {
    let ResourceCallTarget::User { name, .. } = target else {
        return false;
    };
    let Some(summary) = summaries
        .iter()
        .find(|summary| summary.function == name.as_str())
    else {
        return false;
    };
    apply_raw_alias_summary(raw_aliases, output, args, summary, types)
}

pub(super) fn apply_indirect_call_raw_alias_summary(
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    output: &Place,
    callee: &Place,
    args: &[Place],
    summaries: &[RawCellAddressReturnSummary],
    types: &TypeCtx,
) -> bool {
    let functions = function_aliases.functions(callee);
    let mut applied = false;
    for function in functions {
        if let Some(summary) = summaries
            .iter()
            .find(|summary| summary.function == function.as_str())
        {
            applied |= apply_raw_alias_summary(raw_aliases, output, args, summary, types);
        }
    }
    applied
}

fn apply_raw_alias_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    args: &[Place],
    summary: &RawCellAddressReturnSummary,
    types: &TypeCtx,
) -> bool {
    let mut applied = false;
    for (alias, arg) in summary
        .aliases
        .iter()
        .filter_map(|alias| args.get(alias.parameter_index).map(|arg| (alias, arg)))
    {
        let source = projected_place_with_concrete_type(
            types,
            arg,
            &alias.parameter_projection,
            alias.parameter_ty,
        );
        let return_fallback_ty = if alias.return_ty == alias.parameter_ty {
            source.ty
        } else {
            alias.return_ty
        };
        let target = projected_place_with_concrete_type(
            types,
            output,
            &alias.return_projection,
            return_fallback_ty,
        );
        raw_aliases.copy_alias_or_seed(&source, &target);
        applied = true;
    }
    applied
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

fn push_unique_return_alias(
    aliases: &mut Vec<RawCellAddressReturnAlias>,
    alias: RawCellAddressReturnAlias,
) {
    let alias = normalize_return_alias(alias);
    if aliases.iter().any(|existing| existing == &alias) {
        return;
    }
    let compatible_count = aliases
        .iter()
        .filter(|existing| widen_return_alias(existing, &alias).is_some())
        .count();
    if compatible_count >= MAX_EXACT_PROJECTION_FACTS_PER_SHAPE {
        if let Some(widened) = aliases
            .iter()
            .find_map(|existing| widen_return_alias(existing, &alias))
        {
            if !aliases.iter().any(|existing| existing == &widened) {
                aliases.push(widened);
            }
            return;
        }
    }
    aliases.push(alias);
    aliases.sort();
}

fn normalize_return_alias(mut alias: RawCellAddressReturnAlias) -> RawCellAddressReturnAlias {
    alias.parameter_projection = normalize_storage_offsets(alias.parameter_projection);
    alias.return_projection = normalize_storage_offsets(alias.return_projection);
    alias
}

fn widen_return_alias(
    existing: &RawCellAddressReturnAlias,
    incoming: &RawCellAddressReturnAlias,
) -> Option<RawCellAddressReturnAlias> {
    if existing.parameter_index != incoming.parameter_index
        || existing.parameter_ty != incoming.parameter_ty
        || existing.return_ty != incoming.return_ty
    {
        return None;
    }
    let parameter_projection = widen_projection(
        &existing.parameter_projection,
        &incoming.parameter_projection,
    )?;
    let return_projection =
        widen_projection(&existing.return_projection, &incoming.return_projection)?;
    Some(RawCellAddressReturnAlias {
        parameter_index: existing.parameter_index,
        parameter_projection,
        parameter_ty: existing.parameter_ty,
        return_projection,
        return_ty: existing.return_ty,
    })
}

#[cfg(test)]
mod tests {
    use super::super::model::ResourceOffset;
    use super::*;
    use crate::types::TypeId;
    use alloc::vec;

    #[test]
    fn return_alias_offsets_form_finite_dynamic_widening_domain() {
        let ty = TypeId(1);
        let mut aliases = Vec::new();

        for offset in 0..(MAX_EXACT_PROJECTION_FACTS_PER_SHAPE + 8) {
            push_unique_return_alias(
                &mut aliases,
                RawCellAddressReturnAlias {
                    parameter_index: 0,
                    parameter_projection: Vec::new(),
                    parameter_ty: ty,
                    return_projection: vec![PlaceProjection::StorageOffset(ResourceOffset::Exact(
                        offset,
                    ))],
                    return_ty: ty,
                },
            );
        }

        assert_eq!(aliases.len(), MAX_EXACT_PROJECTION_FACTS_PER_SHAPE + 1);
        assert!(aliases.iter().any(|alias| alias.return_projection
            == vec![PlaceProjection::StorageOffset(ResourceOffset::Dynamic)]));
    }
}
