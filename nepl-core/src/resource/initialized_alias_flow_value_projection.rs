use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias_flow::{
    expr_kind_preserves_raw_alias, expr_output_preserves_raw_alias, push_unique_return_alias,
    RawCellAddressReturnAlias, RawCellAddressReturnSummary, RawCellAddressReturnSummaryIndex,
};
use super::model::{
    EffectOp, Place, PlaceProjection, ResourceCallTarget, ResourceFunction, ResourceOp,
    ResourceTerminator,
};
use super::place_utils::{
    construct_aggregate_field_place, match_bind_payload_place, place_suffix_after_prefix,
    projected_place_with_concrete_type, replace_place_prefix,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValueProjectionAlias {
    place: Place,
    parameter_index: usize,
    parameter_projection: Vec<PlaceProjection>,
    parameter_ty: TypeId,
}

pub(super) fn function_value_projection_return_aliases(
    function: &ResourceFunction,
    summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> Vec<RawCellAddressReturnAlias> {
    if !function_allows_value_projection_summary(function) {
        return Vec::new();
    }
    let mut value_aliases = function
        .params
        .iter()
        .enumerate()
        .filter(|(_, param)| type_can_seed_value_projection_summary(types, param.place.ty))
        .map(|(index, param)| ValueProjectionAlias {
            place: param.place.clone(),
            parameter_index: index,
            parameter_projection: Vec::new(),
            parameter_ty: param.place.ty,
        })
        .collect::<Vec<_>>();
    let mut function_aliases = FunctionAliasTable::default();
    for block in &function.blocks {
        propagate_value_projection_ops(
            &mut value_aliases,
            &mut function_aliases,
            &block.ops,
            summaries,
            types,
        );
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            return value_projection_return_aliases(&value_aliases, value);
        }
    }
    Vec::new()
}

fn function_allows_value_projection_summary(function: &ResourceFunction) -> bool {
    function_has_simple_value_projection_body(function)
}

fn type_can_seed_value_projection_summary(types: &TypeCtx, ty: TypeId) -> bool {
    match types.get_ref(types.resolve_id(ty)) {
        TypeKind::Struct { .. }
        | TypeKind::Enum { .. }
        | TypeKind::Tuple { .. }
        | TypeKind::Apply { .. }
        | TypeKind::Reference(_, _)
        | TypeKind::Box(_) => true,
        TypeKind::Named(_) => {
            let resolved = types.resolve_named_type_id(ty);
            resolved != ty && type_can_seed_value_projection_summary(types, resolved)
        }
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Function { .. }
        | TypeKind::Var(_) => false,
    }
}

fn function_has_simple_value_projection_body(function: &ResourceFunction) -> bool {
    let [block] = function.blocks.as_slice() else {
        return false;
    };
    block.ops.iter().all(value_projection_op_is_simple)
}

fn value_projection_op_is_simple(op: &ResourceOp) -> bool {
    matches!(
        op,
        ResourceOp::DeclareLocal { .. }
            | ResourceOp::Read { .. }
            | ResourceOp::Move { .. }
            | ResourceOp::Assign { .. }
            | ResourceOp::Construct { .. }
            | ResourceOp::FunctionValue { .. }
            | ResourceOp::Call { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::Expr { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::EndScope { .. }
    )
}

fn propagate_value_projection_ops(
    value_aliases: &mut Vec<ValueProjectionAlias>,
    function_aliases: &mut FunctionAliasTable,
    ops: &[ResourceOp],
    summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) {
    for op in ops {
        propagate_value_projection_op(value_aliases, function_aliases, op, summaries, types);
    }
}

fn propagate_value_projection_op(
    value_aliases: &mut Vec<ValueProjectionAlias>,
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
                copy_value_projection_aliases(value_aliases, initializer, place);
                function_aliases.copy_alias(initializer, place);
            } else {
                clear_value_projection_aliases(value_aliases, place);
            }
        }
        ResourceOp::Read { source, output, .. } | ResourceOp::Move { source, output, .. } => {
            copy_value_projection_aliases(value_aliases, source, output);
            function_aliases.copy_alias(source, output);
        }
        ResourceOp::Assign { target, value, .. } => {
            copy_value_projection_aliases(value_aliases, value, target);
            function_aliases.copy_alias(value, target);
        }
        ResourceOp::Construct {
            output,
            kind,
            inputs,
            ..
        } => {
            clear_value_projection_aliases(value_aliases, output);
            for (index, input) in inputs.iter().enumerate() {
                let field = construct_aggregate_field_place(output, kind, index, input);
                copy_value_projection_aliases(value_aliases, input, &field);
            }
            construct_function_alias_fields(function_aliases, output, kind, inputs);
        }
        ResourceOp::FunctionValue { output, name, .. } => {
            clear_value_projection_aliases(value_aliases, output);
            function_aliases.set_alias(output, name.clone());
        }
        ResourceOp::Call {
            output,
            target,
            args,
            effect,
            ..
        } => {
            if matches!(
                effect,
                EffectOp::InternalAlloc { .. } | EffectOp::UnsafeMemory { .. }
            ) || !apply_direct_call_value_projection_summary(
                value_aliases,
                output,
                target,
                args,
                summaries,
                types,
            ) {
                clear_value_projection_aliases(value_aliases, output);
            }
        }
        ResourceOp::IndirectCall {
            output,
            callee,
            args,
            ..
        } => {
            if !apply_indirect_call_value_projection_summary(
                value_aliases,
                function_aliases,
                output,
                callee,
                args,
                summaries,
                types,
            ) {
                clear_value_projection_aliases(value_aliases, output);
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
            let mut then_aliases = value_aliases.clone();
            let mut else_aliases = value_aliases.clone();
            let mut then_function_aliases = function_aliases.clone();
            let mut else_function_aliases = function_aliases.clone();
            propagate_value_projection_ops(
                &mut then_aliases,
                &mut then_function_aliases,
                then_ops,
                summaries,
                types,
            );
            propagate_value_projection_ops(
                &mut else_aliases,
                &mut else_function_aliases,
                else_ops,
                summaries,
                types,
            );
            copy_value_projection_aliases(&mut then_aliases, then_value, output);
            copy_value_projection_aliases(&mut else_aliases, else_value, output);
            then_function_aliases.copy_alias(then_value, output);
            else_function_aliases.copy_alias(else_value, output);
            *value_aliases = merge_value_projection_alias_paths(&[then_aliases, else_aliases]);
            *function_aliases =
                FunctionAliasTable::merge_paths(&[then_function_aliases, else_function_aliases]);
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            let mut condition_aliases = value_aliases.clone();
            let mut condition_function_aliases = function_aliases.clone();
            propagate_value_projection_ops(
                &mut condition_aliases,
                &mut condition_function_aliases,
                condition_ops,
                summaries,
                types,
            );
            let mut body_aliases = condition_aliases.clone();
            let mut body_function_aliases = condition_function_aliases.clone();
            propagate_value_projection_ops(
                &mut body_aliases,
                &mut body_function_aliases,
                body_ops,
                summaries,
                types,
            );
            *value_aliases = merge_value_projection_alias_paths(&[condition_aliases, body_aliases]);
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
                let mut arm_aliases = value_aliases.clone();
                let mut arm_function_aliases = function_aliases.clone();
                if let Some(bind_local) = &arm.bind_local {
                    if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
                        copy_value_projection_aliases(&mut arm_aliases, &source, bind_local);
                        arm_function_aliases.copy_alias(&source, bind_local);
                    } else {
                        clear_value_projection_aliases(&mut arm_aliases, bind_local);
                    }
                }
                propagate_value_projection_ops(
                    &mut arm_aliases,
                    &mut arm_function_aliases,
                    &arm.ops,
                    summaries,
                    types,
                );
                copy_value_projection_aliases(&mut arm_aliases, &arm.value, output);
                arm_function_aliases.copy_alias(&arm.value, output);
                alias_paths.push(arm_aliases);
                function_alias_paths.push(arm_function_aliases);
            }
            if !alias_paths.is_empty() {
                *value_aliases = merge_value_projection_alias_paths(&alias_paths);
                *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
            }
        }
        ResourceOp::Expr { output, kind, .. } => {
            if !expr_kind_preserves_raw_alias(*kind)
                && !expr_output_preserves_raw_alias(types, *kind, output)
            {
                clear_value_projection_aliases(value_aliases, output);
            }
        }
        ResourceOp::RawMemory { output, .. }
        | ResourceOp::RawAddressAlias { target: output, .. }
        | ResourceOp::RawAddressView { target: output, .. }
        | ResourceOp::Borrow { output, .. } => {
            clear_value_projection_aliases(value_aliases, output)
        }
        ResourceOp::StorageOrigin { .. } => {}
        ResourceOp::Drop { place, .. } => clear_value_projection_aliases(value_aliases, place),
        ResourceOp::CallEffect { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. } => {}
    }
}

fn copy_value_projection_aliases(
    aliases: &mut Vec<ValueProjectionAlias>,
    source: &Place,
    target: &Place,
) {
    if source == target {
        return;
    }
    let copies = aliases
        .iter()
        .filter_map(|alias| {
            replace_place_prefix(&alias.place, source, target).map(|place| ValueProjectionAlias {
                place,
                parameter_index: alias.parameter_index,
                parameter_projection: alias.parameter_projection.clone(),
                parameter_ty: alias.parameter_ty,
            })
        })
        .collect::<Vec<_>>();
    clear_value_projection_aliases(aliases, target);
    for copy in copies {
        push_unique_value_projection_alias(aliases, copy);
    }
}

fn clear_value_projection_aliases(aliases: &mut Vec<ValueProjectionAlias>, place: &Place) {
    aliases.retain(|alias| place_suffix_after_prefix(&alias.place, place).is_none());
}

fn merge_value_projection_alias_paths(
    paths: &[Vec<ValueProjectionAlias>],
) -> Vec<ValueProjectionAlias> {
    let Some((first, rest)) = paths.split_first() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for alias in first {
        if rest
            .iter()
            .all(|path| path.iter().any(|existing| existing == alias))
        {
            push_unique_value_projection_alias(&mut out, alias.clone());
        }
    }
    out
}

fn value_projection_return_aliases(
    aliases: &[ValueProjectionAlias],
    value: &Place,
) -> Vec<RawCellAddressReturnAlias> {
    let mut out = Vec::new();
    for alias in aliases {
        let Some(return_projection) = place_suffix_after_prefix(&alias.place, value) else {
            continue;
        };
        push_unique_return_alias(
            &mut out,
            RawCellAddressReturnAlias {
                parameter_index: alias.parameter_index,
                parameter_projection: alias.parameter_projection.clone(),
                parameter_ty: alias.parameter_ty,
                return_projection,
                return_ty: alias.place.ty,
            },
        );
    }
    out
}

fn push_unique_value_projection_alias(
    aliases: &mut Vec<ValueProjectionAlias>,
    alias: ValueProjectionAlias,
) {
    if !aliases.iter().any(|existing| existing == &alias) {
        aliases.push(alias);
    }
}

fn apply_direct_call_value_projection_summary(
    value_aliases: &mut Vec<ValueProjectionAlias>,
    output: &Place,
    target: &ResourceCallTarget,
    args: &[Place],
    summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> bool {
    let ResourceCallTarget::User { name, .. } = target else {
        return false;
    };
    let Some(summary) = summaries.get(name) else {
        return false;
    };
    apply_value_projection_summary(value_aliases, output, args, summary, types)
}

fn apply_indirect_call_value_projection_summary(
    value_aliases: &mut Vec<ValueProjectionAlias>,
    function_aliases: &FunctionAliasTable,
    output: &Place,
    callee: &Place,
    args: &[Place],
    summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> bool {
    let functions = function_aliases.functions(callee);
    let mut applied = false;
    for function in functions {
        if let Some(summary) = summaries.get(function) {
            applied |= apply_value_projection_summary(value_aliases, output, args, summary, types);
        }
    }
    applied
}

fn apply_value_projection_summary(
    value_aliases: &mut Vec<ValueProjectionAlias>,
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
        copy_value_projection_aliases(value_aliases, &source, &target);
        applied = true;
    }
    applied
}
