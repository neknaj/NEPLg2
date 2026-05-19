extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::cell_state::CellTable;
use super::function_alias::{construct_function_alias_fields, FunctionAliasTable};
use super::i32_call_facts::record_direct_call_i32_facts;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_alias_flow_apply::{
    apply_direct_call_raw_alias_summary, apply_indirect_call_raw_alias_summary,
    construct_raw_cell_address_alias_fields,
};
use super::initialized_str_layout::seed_str_storage_layout;
use super::model::{
    EffectOp, Place, PlaceProjection, RawMemoryOp, ResourceCallTarget, ResourceExprKind,
    ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator,
};
use super::place_utils::{
    match_bind_payload_place, place_suffix_after_prefix, projected_place_with_concrete_type,
    raw_memory_cell_place, reference_target_place, type_can_seed_raw_address_alias,
};
use super::summary_index::{FunctionSummary, SummaryIndex};
use super::summary_worklist::SummaryWorklist;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ScalarReturnSummary {
    pub(super) function: String,
    pub(super) parameters: Vec<Place>,
    pub(super) aliases: Vec<I32ScalarReturnAlias>,
    pub(super) constant: Option<i32>,
}

pub(super) type I32ScalarReturnSummaryIndex<'a> = SummaryIndex<'a, I32ScalarReturnSummary>;

impl FunctionSummary for I32ScalarReturnSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ScalarReturnAlias {
    pub(super) parameter_index: usize,
    pub(super) parameter_projection: Vec<PlaceProjection>,
    pub(super) scalar_ty: TypeId,
}

pub(super) fn compute_i32_scalar_return_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
) -> Vec<I32ScalarReturnSummary> {
    let mut worklist = SummaryWorklist::new(module);
    let mut summaries = Vec::new();
    while let Some(function_index) = worklist.pop() {
        let function = &module.functions[function_index];
        let scalar_summary_index = I32ScalarReturnSummaryIndex::new(&summaries);
        let summary = function_i32_scalar_return_summary(
            function,
            &scalar_summary_index,
            raw_alias_summaries,
            types,
        );
        if update_i32_scalar_return_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_i32_scalar_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
    }
    summaries
}

pub(super) fn apply_direct_call_i32_scalar_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    target: &ResourceCallTarget,
    args: &[Place],
    summaries: &I32ScalarReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> bool {
    let ResourceCallTarget::User { name, .. } = target else {
        return false;
    };
    let Some(summary) = summaries.get(name) else {
        return false;
    };
    apply_i32_scalar_summary(raw_aliases, output, args, summary, types)
}

fn update_i32_scalar_return_summary(
    summaries: &mut Vec<I32ScalarReturnSummary>,
    summary: I32ScalarReturnSummary,
) -> bool {
    let has_facts = !summary.aliases.is_empty() || summary.constant.is_some();
    let position = summaries
        .iter()
        .position(|existing| existing.function == summary.function);
    match (has_facts, position) {
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

fn function_i32_scalar_return_summary(
    function: &ResourceFunction,
    scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> I32ScalarReturnSummary {
    let mut guaranteed_aliases = None;
    let mut guaranteed_constant = None;
    for block in &function.blocks {
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut function_aliases = FunctionAliasTable::default();
        for param in &function.params {
            if type_can_seed_raw_address_alias(types, param.place.ty) {
                raw_aliases.mark(&param.place);
            }
            let mut cells = CellTable::default();
            seed_str_storage_layout(types, &mut cells, &mut raw_aliases, &param.place);
            if let Some(target_ty) = reference_target_type(types, param.place.ty) {
                let target = reference_target_place(&param.place, target_ty);
                if type_can_seed_raw_address_alias(types, target.ty) {
                    raw_aliases.mark(&target);
                }
                seed_str_storage_layout(types, &mut cells, &mut raw_aliases, &target);
            }
        }
        propagate_i32_scalar_ops(
            &mut raw_aliases,
            &mut function_aliases,
            &block.ops,
            scalar_summaries,
            raw_alias_summaries,
            types,
        );
        if let ResourceTerminator::Return { value, .. } = &block.terminator {
            let path_aliases = value
                .as_ref()
                .map(|value| collect_i32_scalar_return_aliases(function, &raw_aliases, value))
                .unwrap_or_default();
            merge_guaranteed_facts(&mut guaranteed_aliases, path_aliases);
            let path_constant = value
                .as_ref()
                .and_then(|value| raw_aliases.i32_value(value));
            merge_guaranteed_constant(&mut guaranteed_constant, path_constant);
        }
    }
    I32ScalarReturnSummary {
        function: function.name.clone(),
        parameters: function
            .params
            .iter()
            .map(|param| param.place.clone())
            .collect(),
        aliases: guaranteed_aliases.unwrap_or_default(),
        constant: guaranteed_constant.flatten(),
    }
}

fn propagate_i32_scalar_ops(
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
        ResourceOp::CallEffect { .. } | ResourceOp::EndScope { .. } => {}
    }
}

fn collect_i32_scalar_return_aliases(
    function: &ResourceFunction,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
) -> Vec<I32ScalarReturnAlias> {
    let mut aliases = Vec::new();
    for scalar_alias in raw_aliases.scalar_aliases_for_value(value) {
        for (parameter_index, param) in function.params.iter().enumerate() {
            let Some(parameter_projection) = place_suffix_after_prefix(&scalar_alias, &param.place)
            else {
                continue;
            };
            push_unique_i32_scalar_return_alias(
                &mut aliases,
                I32ScalarReturnAlias {
                    parameter_index,
                    parameter_projection,
                    scalar_ty: scalar_alias.ty,
                },
            );
        }
    }
    aliases
}

fn apply_i32_scalar_summary(
    raw_aliases: &mut RawCellAddressAliases,
    output: &Place,
    args: &[Place],
    summary: &I32ScalarReturnSummary,
    types: &TypeCtx,
) -> bool {
    let mut applied = false;
    if let Some(value) = summary.constant {
        raw_aliases.set_i32_value(output, value);
        applied = true;
    }
    for (alias, arg) in summary
        .aliases
        .iter()
        .filter_map(|alias| args.get(alias.parameter_index).map(|arg| (alias, arg)))
    {
        let source = projected_place_with_concrete_type(
            types,
            arg,
            &alias.parameter_projection,
            alias.scalar_ty,
        );
        raw_aliases.copy_scalar_facts_if_tracked(&source, output);
        applied = true;
    }
    applied
}

fn push_unique_i32_scalar_return_alias(
    aliases: &mut Vec<I32ScalarReturnAlias>,
    alias: I32ScalarReturnAlias,
) {
    if aliases.iter().any(|existing| existing == &alias) {
        return;
    }
    aliases.push(alias);
}

fn merge_guaranteed_facts<T: Clone + Eq>(guaranteed: &mut Option<Vec<T>>, path: Vec<T>) {
    match guaranteed {
        Some(existing) => {
            existing.retain(|fact| path.contains(fact));
        }
        None => {
            *guaranteed = Some(path);
        }
    }
}

fn merge_guaranteed_constant(guaranteed: &mut Option<Option<i32>>, path: Option<i32>) {
    match guaranteed {
        Some(existing) if *existing == path => {}
        Some(existing) => {
            *existing = None;
        }
        None => {
            *guaranteed = Some(path);
        }
    }
}

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        crate::types::TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}
