extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::i32_call_facts::record_direct_call_i32_facts;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_alias_flow_apply::{
    apply_direct_call_raw_alias_summary, apply_indirect_call_raw_alias_summary,
    construct_raw_cell_address_alias_fields,
};
use super::initialized_scalar_flow::{
    apply_direct_call_i32_scalar_summary, I32ScalarReturnSummaryIndex,
};
use super::model::{EffectOp, RawMemoryOp, ResourceExprKind, ResourceOp};
use super::place_utils::{match_bind_payload_place, raw_memory_cell_place, reference_target_place};

pub(super) fn propagate_i32_scalar_ops(
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    ops: &[ResourceOp],
    scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) {
    for op in ops {
        propagate_i32_scalar_op(
            raw_aliases,
            function_aliases,
            op,
            scalar_summaries,
            raw_alias_summaries,
            types,
        );
    }
}

fn propagate_i32_scalar_op(
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    op: &ResourceOp,
    scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
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
            operation,
            output,
            args,
            ..
        } => match operation {
            RawMemoryOp::Alloc | RawMemoryOp::Realloc => raw_aliases.mark(output),
            RawMemoryOp::Load => {
                if let Some(address) = args.first() {
                    let address = raw_aliases.canonicalize(address);
                    let cell = raw_memory_cell_place(&address, output.ty);
                    raw_aliases.copy_scalar_facts_if_tracked(&cell, output);
                } else {
                    raw_aliases.clear(output);
                }
            }
            RawMemoryOp::LoadU8 => raw_aliases.clear(output),
            RawMemoryOp::Store
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
                raw_aliases.record_raw_address_view_origin(source, target);
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
            raw_aliases.clear(output);
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
            ) {
                return;
            }
            let raw_applied = apply_direct_call_raw_alias_summary(
                raw_aliases,
                output,
                target,
                args,
                raw_alias_summaries,
                types,
            );
            let scalar_applied = apply_direct_call_i32_scalar_summary(
                raw_aliases,
                output,
                target,
                args,
                scalar_summaries,
                types,
            );
            if !raw_applied && !scalar_applied {
                raw_aliases.clear(output);
            }
            record_direct_call_i32_facts(raw_aliases, target, output, args);
        }
        ResourceOp::IndirectCall {
            output,
            callee,
            args,
            ..
        } => {
            let raw_applied = apply_indirect_call_raw_alias_summary(
                raw_aliases,
                function_aliases,
                output,
                callee,
                args,
                raw_alias_summaries,
                types,
            );
            if !raw_applied {
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
            propagate_i32_scalar_ops(
                &mut then_aliases,
                &mut then_function_aliases,
                then_ops,
                scalar_summaries,
                raw_alias_summaries,
                types,
            );
            propagate_i32_scalar_ops(
                &mut else_aliases,
                &mut else_function_aliases,
                else_ops,
                scalar_summaries,
                raw_alias_summaries,
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
            propagate_i32_scalar_ops(
                &mut condition_aliases,
                &mut condition_function_aliases,
                condition_ops,
                scalar_summaries,
                raw_alias_summaries,
                types,
            );
            let mut body_aliases = condition_aliases.clone();
            let mut body_function_aliases = condition_function_aliases.clone();
            propagate_i32_scalar_ops(
                &mut body_aliases,
                &mut body_function_aliases,
                body_ops,
                scalar_summaries,
                raw_alias_summaries,
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
                propagate_i32_scalar_ops(
                    &mut arm_aliases,
                    &mut arm_function_aliases,
                    &arm.ops,
                    scalar_summaries,
                    raw_alias_summaries,
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
        ResourceOp::Expr { output, kind, .. } => match kind {
            ResourceExprKind::LiteralI32(value) => {
                raw_aliases.clear(output);
                raw_aliases.set_i32_value(output, *value);
            }
            ResourceExprKind::LayoutSizeOf(ty) => {
                raw_aliases.clear(output);
                raw_aliases.set_i32_type_size(output, *ty);
            }
            ResourceExprKind::LocalRead
            | ResourceExprKind::Call
            | ResourceExprKind::IndirectCall
            | ResourceExprKind::Intrinsic
            | ResourceExprKind::Borrow
            | ResourceExprKind::Branch
            | ResourceExprKind::Match
            | ResourceExprKind::Construct => {}
            ResourceExprKind::Literal
            | ResourceExprKind::FunctionValue
            | ResourceExprKind::Loop
            | ResourceExprKind::Block
            | ResourceExprKind::Let
            | ResourceExprKind::Set
            | ResourceExprKind::Deref
            | ResourceExprKind::Drop => raw_aliases.clear(output),
        },
        ResourceOp::Borrow { source, output, .. } => {
            raw_aliases.mark(output);
            let target = reference_target_place(output, source.ty);
            raw_aliases.copy_alias_if_tracked(source, &target);
        }
        ResourceOp::Drop { place, .. } => raw_aliases.clear(place),
        ResourceOp::CallEffect { .. }
        | ResourceOp::EndScope { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. } => {}
    }
}
