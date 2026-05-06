extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias::{ProjectedRawCellAddressAlias, RawCellAddressAliases};
use super::model::{
    AggregateKind, EffectOp, Place, PlaceProjection, RawMemoryOp, ResourceCallTarget,
    ResourceExprKind, ResourceFunction, ResourceModule, ResourceOffset, ResourceOp,
    ResourceTerminator,
};
use super::place_utils::{
    construct_aggregate_field_place, match_bind_payload_place, place_suffix_after_prefix,
    projected_place_with_concrete_type, reference_target_place, replace_place_prefix,
    type_can_seed_raw_address_alias, type_preserves_raw_address_alias,
};
use super::summary_worklist::SummaryWorklist;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellAddressReturnSummary {
    function: String,
    parameters: Vec<Place>,
    aliases: Vec<RawCellAddressReturnAlias>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawCellAddressReturnAlias {
    parameter_index: usize,
    parameter_projection: Vec<PlaceProjection>,
    parameter_ty: TypeId,
    return_projection: Vec<PlaceProjection>,
    return_ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValueProjectionAlias {
    place: Place,
    parameter_index: usize,
    parameter_projection: Vec<PlaceProjection>,
    parameter_ty: TypeId,
}

pub(super) fn expr_kind_preserves_raw_alias(kind: ResourceExprKind) -> bool {
    matches!(
        kind,
        ResourceExprKind::LocalRead
            | ResourceExprKind::Call
            | ResourceExprKind::IndirectCall
            | ResourceExprKind::Intrinsic
            | ResourceExprKind::Borrow
            | ResourceExprKind::Branch
            | ResourceExprKind::Match
            | ResourceExprKind::Construct
    )
}

pub(super) fn compute_raw_cell_address_return_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
) -> Vec<RawCellAddressReturnSummary> {
    let mut worklist = SummaryWorklist::new(module);
    let mut summaries = Vec::new();
    while let Some(function_index) = worklist.pop() {
        let function = &module.functions[function_index];
        let summary = function_raw_cell_address_return_summary(function, &summaries, types);
        if update_raw_cell_address_return_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_raw_alias_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
    }
    summaries
}

fn update_raw_cell_address_return_summary(
    summaries: &mut Vec<RawCellAddressReturnSummary>,
    summary: RawCellAddressReturnSummary,
) -> bool {
    let has_aliases = !summary.aliases.is_empty();
    let position = summaries
        .iter()
        .position(|existing| existing.function == summary.function);
    match (has_aliases, position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
        }
        (true, None) => {
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
            true
        }
        (false, None) => false,
    }
}

fn function_raw_cell_address_return_summary(
    function: &ResourceFunction,
    summaries: &[RawCellAddressReturnSummary],
    types: &TypeCtx,
) -> RawCellAddressReturnSummary {
    let mut aliases = function_value_projection_return_aliases(function, summaries, types);
    for (index, param) in function.params.iter().enumerate() {
        for alias in function_raw_cell_address_return_aliases(
            function,
            index,
            &param.place,
            summaries,
            types,
        ) {
            push_unique_return_alias(&mut aliases, alias);
        }
    }
    RawCellAddressReturnSummary {
        function: function.name.clone(),
        parameters: function
            .params
            .iter()
            .map(|param| param.place.clone())
            .collect(),
        aliases,
    }
}

fn function_value_projection_return_aliases(
    function: &ResourceFunction,
    summaries: &[RawCellAddressReturnSummary],
    types: &TypeCtx,
) -> Vec<RawCellAddressReturnAlias> {
    if !function_allows_value_projection_summary(function, types) {
        return Vec::new();
    }
    let mut value_aliases = function
        .params
        .iter()
        .enumerate()
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

fn function_allows_value_projection_summary(function: &ResourceFunction, types: &TypeCtx) -> bool {
    function_has_simple_value_projection_body(function)
        && type_is_result_enum(types, function.result)
}

fn function_has_simple_value_projection_body(function: &ResourceFunction) -> bool {
    let [block] = function.blocks.as_slice() else {
        return false;
    };
    block.ops.iter().all(value_projection_op_is_simple)
}

fn type_is_result_enum(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Enum { name, .. } => name == "Result",
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(types.get_ref(base), TypeKind::Enum { name, .. } if name == "Result")
        }
        _ => false,
    }
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

fn function_raw_cell_address_return_aliases(
    function: &ResourceFunction,
    parameter_index: usize,
    parameter: &Place,
    summaries: &[RawCellAddressReturnSummary],
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

fn propagate_value_projection_ops(
    value_aliases: &mut Vec<ValueProjectionAlias>,
    function_aliases: &mut FunctionAliasTable,
    ops: &[ResourceOp],
    summaries: &[RawCellAddressReturnSummary],
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
    summaries: &[RawCellAddressReturnSummary],
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
        ResourceOp::Drop { place, .. } => clear_value_projection_aliases(value_aliases, place),
        ResourceOp::CallEffect { .. } | ResourceOp::EndScope { .. } => {}
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
            | RawMemoryOp::Store
            | RawMemoryOp::Dealloc
            | RawMemoryOp::BulkCopy
            | RawMemoryOp::BulkMove
            | RawMemoryOp::MemorySize
            | RawMemoryOp::MemoryGrow
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
        ResourceOp::CallEffect { .. } | ResourceOp::EndScope { .. } => {}
    }
}

fn expr_output_preserves_raw_alias(
    types: &TypeCtx,
    kind: ResourceExprKind,
    output: &Place,
) -> bool {
    matches!(kind, ResourceExprKind::Deref) && type_preserves_raw_address_alias(types, output.ty)
}

pub(super) fn construct_raw_cell_address_alias_fields(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    kind: &AggregateKind,
    inputs: &[Place],
) {
    for (index, input) in inputs.iter().enumerate() {
        let field = construct_aggregate_field_place(output, kind, index, input);
        raw_aliases.copy_alias_if_tracked(input, &field);
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

fn apply_direct_call_value_projection_summary(
    value_aliases: &mut Vec<ValueProjectionAlias>,
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
    apply_value_projection_summary(value_aliases, output, args, summary, types)
}

fn apply_indirect_call_value_projection_summary(
    value_aliases: &mut Vec<ValueProjectionAlias>,
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
        let parameter_projection = substitute_summary_projection_offsets(
            raw_aliases,
            &alias.parameter_projection,
            summary,
            args,
        );
        let source = projected_place_with_concrete_type(
            types,
            arg,
            &parameter_projection,
            alias.parameter_ty,
        );
        let return_fallback_ty = if alias.return_ty == alias.parameter_ty {
            source.ty
        } else {
            alias.return_ty
        };
        let return_projection = substitute_summary_projection_offsets(
            raw_aliases,
            &alias.return_projection,
            summary,
            args,
        );
        let target = projected_place_with_concrete_type(
            types,
            output,
            &return_projection,
            return_fallback_ty,
        );
        raw_aliases.copy_alias_if_tracked(&source, &target);
        applied = true;
    }
    applied
}

fn substitute_summary_projection_offsets(
    raw_aliases: &RawCellAddressAliases,
    projections: &[PlaceProjection],
    summary: &RawCellAddressReturnSummary,
    args: &[Place],
) -> Vec<PlaceProjection> {
    projections
        .iter()
        .map(|projection| match projection {
            PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place }) => {
                let actual = substitute_summary_place(place, summary, args);
                if let Some(actual) = actual {
                    if let Some(value) = raw_aliases.i32_value(&actual) {
                        return PlaceProjection::StorageOffset(resource_offset_from_i32(value));
                    }
                    return PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
                        place: Box::new(actual),
                    });
                }
                projection.clone()
            }
            _ => projection.clone(),
        })
        .collect()
}

fn substitute_summary_place(
    place: &Place,
    summary: &RawCellAddressReturnSummary,
    args: &[Place],
) -> Option<Place> {
    for (index, parameter) in summary.parameters.iter().enumerate() {
        let Some(arg) = args.get(index) else {
            continue;
        };
        if let Some(replaced) = replace_place_prefix(place, parameter, arg) {
            return Some(replaced);
        }
    }
    None
}

fn resource_offset_from_i32(value: i32) -> ResourceOffset {
    usize::try_from(value)
        .map(ResourceOffset::Known)
        .unwrap_or(ResourceOffset::Unknown)
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
    if !aliases.iter().any(|existing| existing == &alias) {
        aliases.push(alias);
    }
}

#[cfg(test)]
#[path = "initialized_alias_flow_tests.rs"]
mod tests;
