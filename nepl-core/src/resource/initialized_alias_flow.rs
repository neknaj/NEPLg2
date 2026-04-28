extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    AggregateKind, EffectOp, Place, RawMemoryOp, ResourceCallTarget, ResourceExprKind,
    ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::place_utils::construct_aggregate_field_place;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellAddressReturnSummary {
    function: String,
    parameter_indices: Vec<usize>,
}

pub(super) fn expr_kind_preserves_raw_alias(kind: ResourceExprKind) -> bool {
    matches!(
        kind,
        ResourceExprKind::LocalRead
            | ResourceExprKind::Call
            | ResourceExprKind::IndirectCall
            | ResourceExprKind::Branch
            | ResourceExprKind::Match
            | ResourceExprKind::Construct
    )
}

pub(super) fn compute_raw_cell_address_return_summaries(
    module: &ResourceModule,
) -> Vec<RawCellAddressReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let mut parameter_indices = Vec::new();
            for (index, param) in function.params.iter().enumerate() {
                if function_returns_raw_cell_address_alias(function, &param.place, &summaries) {
                    parameter_indices.push(index);
                }
            }
            if !parameter_indices.is_empty() {
                next.push(RawCellAddressReturnSummary {
                    function: function.name.clone(),
                    parameter_indices,
                });
            }
        }
        if next == summaries {
            return summaries;
        }
        summaries = next;
    }
    summaries
}

fn function_returns_raw_cell_address_alias(
    function: &ResourceFunction,
    parameter: &Place,
    summaries: &[RawCellAddressReturnSummary],
) -> bool {
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut function_aliases = FunctionAliasTable::default();
    raw_aliases.mark(parameter);
    for block in &function.blocks {
        propagate_raw_address_alias_ops(
            &mut raw_aliases,
            &mut function_aliases,
            &block.ops,
            summaries,
        );
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            if raw_aliases.aliases(value, parameter) {
                return true;
            }
        }
    }
    false
}

fn propagate_raw_address_alias_ops(
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    ops: &[ResourceOp],
    summaries: &[RawCellAddressReturnSummary],
) {
    for op in ops {
        propagate_raw_address_alias_op(raw_aliases, function_aliases, op, summaries);
    }
}

fn propagate_raw_address_alias_op(
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    op: &ResourceOp,
    summaries: &[RawCellAddressReturnSummary],
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
            | RawMemoryOp::Fill
            | RawMemoryOp::Other { .. } => {}
        },
        ResourceOp::RawAddressAlias { source, target, .. } => {
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
            );
            propagate_raw_address_alias_ops(
                &mut else_aliases,
                &mut else_function_aliases,
                else_ops,
                summaries,
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
            );
            let mut body_aliases = condition_aliases.clone();
            let mut body_function_aliases = condition_function_aliases.clone();
            propagate_raw_address_alias_ops(
                &mut body_aliases,
                &mut body_function_aliases,
                body_ops,
                summaries,
            );
            *raw_aliases = RawCellAddressAliases::merge_paths(&[condition_aliases, body_aliases]);
            *function_aliases = FunctionAliasTable::merge_paths(&[
                condition_function_aliases,
                body_function_aliases,
            ]);
        }
        ResourceOp::Match { output, arms, .. } => {
            let mut alias_paths = Vec::new();
            let mut function_alias_paths = Vec::new();
            for arm in arms {
                let mut arm_aliases = raw_aliases.clone();
                let mut arm_function_aliases = function_aliases.clone();
                if let Some(bind_local) = &arm.bind_local {
                    arm_aliases.clear(bind_local);
                }
                propagate_raw_address_alias_ops(
                    &mut arm_aliases,
                    &mut arm_function_aliases,
                    &arm.ops,
                    summaries,
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
            if !expr_kind_preserves_raw_alias(*kind) {
                raw_aliases.clear(output);
            }
        }
        ResourceOp::Borrow { output, .. } => raw_aliases.clear(output),
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
    apply_raw_alias_summary(raw_aliases, output, args, summary)
}

pub(super) fn apply_indirect_call_raw_alias_summary(
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    output: &Place,
    callee: &Place,
    args: &[Place],
    summaries: &[RawCellAddressReturnSummary],
) -> bool {
    let functions = function_aliases.functions(callee);
    let mut applied = false;
    for function in functions {
        if let Some(summary) = summaries
            .iter()
            .find(|summary| summary.function == function.as_str())
        {
            applied |= apply_raw_alias_summary(raw_aliases, output, args, summary);
        }
    }
    applied
}

fn apply_raw_alias_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    args: &[Place],
    summary: &RawCellAddressReturnSummary,
) -> bool {
    let mut applied = false;
    for arg in summary
        .parameter_indices
        .iter()
        .filter_map(|index| args.get(*index))
    {
        raw_aliases.copy_alias_or_seed(arg, output);
        applied = true;
    }
    applied
}
