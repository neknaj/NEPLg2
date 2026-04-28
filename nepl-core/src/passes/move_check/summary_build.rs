use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::{FuncRef, HirBlock, HirExpr, HirExprKind, HirModule};
use crate::layout::storage_size_bytes;
use crate::types::TypeId;

use super::provenance::{
    i32_const_from_value, is_field_get_name, is_mem_ptr_add_name, is_mem_ptr_addr_name,
    is_mem_ptr_wrap_name, is_raw_address_add_name, is_region_new_name, is_region_ptr_at_name,
    is_region_ptr_name, raw_bulk_copy_size_arg_bytes, raw_byte_write_size_arg_bytes,
    raw_dealloc_place_key, raw_dealloc_size_arg_bytes, raw_memory_place_key,
    raw_store_write_size_bytes,
};
use super::raw_memory::{
    raw_memory_call_kind, raw_memory_helper_name_is_tracked, RawMemoryCallKind,
};
use super::summary::{
    add_child_raw_memory_effects, extend_unique_raw_memory_effects,
    merge_matching_raw_alias_summaries, value_alias_summary_from_raw_summary,
    FunctionRawAliasSummary, RawMemoryEffectSummary,
};
use super::{
    aggregate_field_function_placeholder_aliases, aggregate_field_placeholder_aliases,
    enum_variants_for_type, function_call_raw_alias_summary,
    function_param_enum_payload_field_function_alias_key,
    function_param_enum_payload_field_raw_alias_key,
    function_param_enum_payload_function_alias_key, function_param_enum_payload_raw_alias_key,
    function_param_field_function_alias_key, function_param_field_raw_alias_key,
    function_param_function_alias_key, function_param_raw_alias_key,
    instantiate_function_raw_alias_summary_from_value_summaries, is_function_type, is_never_type,
    match_bind_aggregate_field_function_aliases, match_bind_aggregate_field_raw_aliases,
    match_bind_function_value_aliases, match_bind_raw_addr_alias,
    raw_alias_summary_needs_call_site_specialization, singleton_function_alias,
    specialized_function_raw_alias_summary, value_alias_summary_from_value, MoveCheckContext,
};
fn raw_memory_effect_summary_from_call(
    callee: &FuncRef,
    args: &[HirExpr],
    result_ty: TypeId,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<RawMemoryEffectSummary> {
    let kind = raw_memory_call_kind(callee, args, result_ty, tctx)?;
    match kind {
        RawMemoryCallKind::Load => {
            let addr = args.get(0)?;
            Some(RawMemoryEffectSummary::Load {
                place: raw_dealloc_place_key(addr, ctx, tctx)?,
                size: storage_size_bytes(tctx, result_ty),
            })
        }
        RawMemoryCallKind::Store => {
            let addr = args.get(0)?;
            let value = args.get(1)?;
            let place = raw_dealloc_place_key(addr, ctx, tctx)?;
            if !tctx.is_copy(value.ty) {
                Some(RawMemoryEffectSummary::Store {
                    place,
                    size: storage_size_bytes(tctx, value.ty),
                })
            } else {
                Some(RawMemoryEffectSummary::ByteWrite {
                    place,
                    size: raw_store_write_size_bytes(callee, Some(value), tctx),
                })
            }
        }
        RawMemoryCallKind::Dealloc => {
            let addr = args.get(0)?;
            Some(RawMemoryEffectSummary::Dealloc {
                place: raw_dealloc_place_key(addr, ctx, tctx)?,
                size: raw_dealloc_size_arg_bytes(args.get(1), tctx),
            })
        }
        RawMemoryCallKind::Realloc => {
            let addr = args.get(0)?;
            Some(RawMemoryEffectSummary::Realloc {
                place: raw_dealloc_place_key(addr, ctx, tctx)?,
                size: raw_dealloc_size_arg_bytes(args.get(1), tctx),
            })
        }
        RawMemoryCallKind::BulkCopy => {
            let dst = args.get(0)?;
            let src = args.get(1)?;
            Some(RawMemoryEffectSummary::BulkCopy {
                dst: raw_dealloc_place_key(dst, ctx, tctx)?,
                src: raw_dealloc_place_key(src, ctx, tctx)?,
                size: raw_bulk_copy_size_arg_bytes(args, tctx),
            })
        }
        RawMemoryCallKind::ByteWrite => {
            let addr = args.get(0)?;
            Some(RawMemoryEffectSummary::ByteWrite {
                place: raw_dealloc_place_key(addr, ctx, tctx)?,
                size: raw_byte_write_size_arg_bytes(callee, args, tctx),
            })
        }
    }
}

fn raw_memory_effect_summary_from_intrinsic(
    name: &str,
    args: &[HirExpr],
    result_ty: TypeId,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<RawMemoryEffectSummary> {
    match name {
        "load" if args.len() == 1 && !tctx.is_copy(result_ty) => {
            let addr = args.get(0)?;
            Some(RawMemoryEffectSummary::Load {
                place: raw_memory_place_key(addr, ctx, tctx)?,
                size: storage_size_bytes(tctx, result_ty),
            })
        }
        "store" if args.len() >= 2 => {
            let addr = args.get(0)?;
            let value = args.get(1)?;
            let place = raw_memory_place_key(addr, ctx, tctx)?;
            if !tctx.is_copy(value.ty) {
                Some(RawMemoryEffectSummary::Store {
                    place,
                    size: storage_size_bytes(tctx, value.ty),
                })
            } else {
                Some(RawMemoryEffectSummary::ByteWrite {
                    place,
                    size: Some(storage_size_bytes(tctx, value.ty)),
                })
            }
        }
        _ => None,
    }
}

fn seed_summary_param_aliases(
    ctx: &mut MoveCheckContext,
    param: &crate::hir::HirParam,
    index: usize,
    tctx: &crate::types::TypeCtx,
) {
    ctx.declare_param(param.name.clone());
    ctx.set_raw_addr_alias(&param.name, Some(function_param_raw_alias_key(index)));
    let aggregate_aliases = aggregate_field_placeholder_aliases(tctx, param.ty, |offset| {
        function_param_field_raw_alias_key(index, offset)
    });
    ctx.set_aggregate_field_raw_aliases(&param.name, aggregate_aliases);
    let aggregate_function_aliases =
        aggregate_field_function_placeholder_aliases(tctx, param.ty, |offset| {
            function_param_field_function_alias_key(index, offset)
        });
    ctx.set_aggregate_field_function_aliases(&param.name, aggregate_function_aliases);

    let mut enum_payload_aliases = BTreeMap::new();
    let mut enum_payload_aggregate_aliases = BTreeMap::new();
    let mut enum_payload_aggregate_function_aliases = BTreeMap::new();
    let mut enum_payload_function_aliases = BTreeMap::new();
    for (variant, payload_ty) in enum_variants_for_type(tctx, param.ty) {
        if let Some(payload_ty) = payload_ty {
            enum_payload_aliases.insert(
                variant.clone(),
                function_param_enum_payload_raw_alias_key(index, variant.as_str()),
            );
            let aggregate_aliases =
                aggregate_field_placeholder_aliases(tctx, payload_ty, |offset| {
                    function_param_enum_payload_field_raw_alias_key(index, variant.as_str(), offset)
                });
            if !aggregate_aliases.is_empty() {
                enum_payload_aggregate_aliases.insert(variant.clone(), aggregate_aliases);
            }
            let aggregate_function_aliases =
                aggregate_field_function_placeholder_aliases(tctx, payload_ty, |offset| {
                    function_param_enum_payload_field_function_alias_key(
                        index,
                        variant.as_str(),
                        offset,
                    )
                });
            if !aggregate_function_aliases.is_empty() {
                enum_payload_aggregate_function_aliases
                    .insert(variant.clone(), aggregate_function_aliases);
            }
            if is_function_type(tctx, payload_ty) {
                enum_payload_function_aliases.insert(
                    variant.clone(),
                    singleton_function_alias(function_param_enum_payload_function_alias_key(
                        index,
                        variant.as_str(),
                    )),
                );
            }
        }
    }
    ctx.set_enum_payload_raw_aliases(&param.name, enum_payload_aliases);
    ctx.set_enum_payload_aggregate_field_raw_aliases(&param.name, enum_payload_aggregate_aliases);
    ctx.set_enum_payload_aggregate_field_function_aliases(
        &param.name,
        enum_payload_aggregate_function_aliases,
    );
    ctx.set_enum_payload_function_aliases(&param.name, enum_payload_function_aliases);
    if is_function_type(tctx, param.ty) {
        ctx.set_function_value_aliases(
            &param.name,
            singleton_function_alias(function_param_function_alias_key(index)),
        );
    }
}

fn base_raw_alias_summary_from_value(
    expr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> FunctionRawAliasSummary {
    let value = value_alias_summary_from_value(expr, ctx, tctx);
    FunctionRawAliasSummary {
        raw_addr_alias: value.raw_addr_alias,
        aggregate_field_raw_aliases: value.aggregate_field_raw_aliases,
        aggregate_field_function_aliases: value.aggregate_field_function_aliases,
        enum_payload_raw_aliases: value.enum_payload_raw_aliases,
        enum_payload_aggregate_field_raw_aliases: value.enum_payload_aggregate_field_raw_aliases,
        enum_payload_aggregate_field_function_aliases: value
            .enum_payload_aggregate_field_function_aliases,
        enum_payload_function_aliases: value.enum_payload_function_aliases,
        function_value_aliases: value.function_value_aliases,
        raw_memory_effects: Vec::new(),
    }
}

enum SimpleRawAliasSummaryFrame<'a> {
    Expr(&'a HirExpr),
    FinishCall {
        callee: &'a FuncRef,
        args: &'a [HirExpr],
        arg_count: usize,
    },
}

fn direct_call_summary_fast_path_supported(callee: &FuncRef) -> bool {
    let FuncRef::User(name, _, _) = callee else {
        return false;
    };
    !(name == "if"
        || name == "while"
        || is_field_get_name(name)
        || is_mem_ptr_addr_name(name)
        || is_mem_ptr_wrap_name(name)
        || is_mem_ptr_add_name(name)
        || is_raw_address_add_name(name)
        || is_region_ptr_name(name)
        || is_region_new_name(name)
        || is_region_ptr_at_name(name)
        || raw_memory_helper_name_is_tracked(name))
}

fn simple_call_tree_raw_alias_summary_iteratively(
    expr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<FunctionRawAliasSummary> {
    let mut frames = Vec::new();
    let mut summaries = Vec::new();
    frames.push(SimpleRawAliasSummaryFrame::Expr(expr));
    while let Some(frame) = frames.pop() {
        match frame {
            SimpleRawAliasSummaryFrame::Expr(expr) => match &expr.kind {
                HirExprKind::Var(_)
                | HirExprKind::FnValue(_)
                | HirExprKind::LiteralI32(_)
                | HirExprKind::LiteralF32(_)
                | HirExprKind::LiteralBool(_)
                | HirExprKind::LiteralStr(_)
                | HirExprKind::Unit
                | HirExprKind::Drop { .. } => {
                    summaries.push(base_raw_alias_summary_from_value(expr, ctx, tctx));
                }
                HirExprKind::Call { callee, args } => {
                    if raw_memory_call_kind(callee, args, expr.ty, tctx).is_some() {
                        return None;
                    }
                    if !direct_call_summary_fast_path_supported(callee) {
                        return None;
                    }
                    frames.push(SimpleRawAliasSummaryFrame::FinishCall {
                        callee,
                        args,
                        arg_count: args.len(),
                    });
                    for arg in args.iter().rev() {
                        frames.push(SimpleRawAliasSummaryFrame::Expr(arg));
                    }
                }
                HirExprKind::CallIndirect { .. }
                | HirExprKind::If { .. }
                | HirExprKind::While { .. }
                | HirExprKind::Match { .. }
                | HirExprKind::Block(_)
                | HirExprKind::Let { .. }
                | HirExprKind::Set { .. }
                | HirExprKind::Intrinsic { .. }
                | HirExprKind::EnumConstruct { .. }
                | HirExprKind::StructConstruct { .. }
                | HirExprKind::TupleConstruct { .. }
                | HirExprKind::AddrOf(_)
                | HirExprKind::Deref(_) => return None,
            },
            SimpleRawAliasSummaryFrame::FinishCall {
                callee,
                args,
                arg_count,
            } => {
                let mut child_summaries = Vec::with_capacity(arg_count);
                for _ in 0..arg_count {
                    child_summaries.push(summaries.pop()?);
                }
                child_summaries.reverse();

                let mut child_effects = Vec::new();
                let mut arg_summaries = Vec::with_capacity(arg_count);
                for child in child_summaries {
                    extend_unique_raw_memory_effects(
                        &mut child_effects,
                        child.raw_memory_effects.clone(),
                    );
                    arg_summaries.push(value_alias_summary_from_raw_summary(&child));
                }

                let mut summary = match callee {
                    FuncRef::User(name, _, _) => {
                        if let Some(callee_summary) =
                            ctx.function_raw_alias_summaries.get(name.as_str())
                        {
                            let instantiated =
                                instantiate_function_raw_alias_summary_from_value_summaries(
                                    callee_summary,
                                    &arg_summaries,
                                    ctx,
                                    tctx,
                                    ctx.function_raw_alias_summaries.len().saturating_add(1),
                                );
                            if raw_alias_summary_needs_call_site_specialization(&instantiated) {
                                specialized_function_raw_alias_summary(name, args, ctx, tctx)
                                    .unwrap_or(instantiated)
                            } else {
                                instantiated
                            }
                        } else {
                            FunctionRawAliasSummary::default()
                        }
                    }
                    _ => return None,
                };
                let mut effects = child_effects;
                extend_unique_raw_memory_effects(&mut effects, summary.raw_memory_effects);
                summary.raw_memory_effects = effects;
                summaries.push(summary);
            }
        }
    }
    if summaries.len() == 1 {
        summaries.pop()
    } else {
        None
    }
}

pub(super) fn expression_raw_alias_summary(
    expr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> FunctionRawAliasSummary {
    if let Some(summary) = simple_call_tree_raw_alias_summary_iteratively(expr, ctx, tctx) {
        return summary;
    }
    match &expr.kind {
        HirExprKind::Block(block) => block_raw_alias_summary(block, ctx, tctx),
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            let cond_summary = expression_raw_alias_summary(cond, ctx, tctx);
            let mut branch_summaries = Vec::new();
            if !is_never_type(tctx, then_branch.ty) {
                branch_summaries.push(expression_raw_alias_summary(then_branch, ctx, tctx));
            }
            if !is_never_type(tctx, else_branch.ty) {
                branch_summaries.push(expression_raw_alias_summary(else_branch, ctx, tctx));
            }
            let mut summary = merge_matching_raw_alias_summaries(branch_summaries);
            let mut effects = cond_summary.raw_memory_effects;
            extend_unique_raw_memory_effects(&mut effects, summary.raw_memory_effects);
            summary.raw_memory_effects = effects;
            summary
        }
        HirExprKind::While { cond, body } => {
            let mut summary = FunctionRawAliasSummary::default();
            let cond_summary = expression_raw_alias_summary(cond, ctx, tctx);
            let body_summary = expression_raw_alias_summary(body, ctx, tctx);
            extend_unique_raw_memory_effects(
                &mut summary.raw_memory_effects,
                cond_summary.raw_memory_effects,
            );
            extend_unique_raw_memory_effects(
                &mut summary.raw_memory_effects,
                body_summary.raw_memory_effects,
            );
            summary
        }
        HirExprKind::Match { scrutinee, arms } => {
            let scrutinee_summary = expression_raw_alias_summary(scrutinee, ctx, tctx);
            let mut branch_summaries = Vec::new();
            for arm in arms {
                if is_never_type(tctx, arm.body.ty) {
                    continue;
                }
                let mut arm_ctx = ctx.clone_for_alias_summary();
                if let Some(bind) = &arm.bind_local {
                    let raw_addr_alias = match_bind_raw_addr_alias(scrutinee, arm, &arm_ctx, tctx);
                    let aggregate_field_raw_aliases =
                        match_bind_aggregate_field_raw_aliases(scrutinee, arm, &arm_ctx, tctx);
                    let aggregate_field_function_aliases =
                        match_bind_aggregate_field_function_aliases(scrutinee, arm, &arm_ctx, tctx);
                    let function_value_aliases =
                        match_bind_function_value_aliases(scrutinee, arm, &arm_ctx, tctx);
                    arm_ctx.declare_var(bind.clone());
                    arm_ctx.set_raw_addr_alias(bind, raw_addr_alias);
                    arm_ctx.set_aggregate_field_raw_aliases(bind, aggregate_field_raw_aliases);
                    arm_ctx.set_aggregate_field_function_aliases(
                        bind,
                        aggregate_field_function_aliases,
                    );
                    arm_ctx.set_function_value_aliases(bind, function_value_aliases);
                }
                branch_summaries.push(expression_raw_alias_summary(&arm.body, &arm_ctx, tctx));
            }
            let mut summary = merge_matching_raw_alias_summaries(branch_summaries);
            let mut effects = scrutinee_summary.raw_memory_effects;
            extend_unique_raw_memory_effects(&mut effects, summary.raw_memory_effects);
            summary.raw_memory_effects = effects;
            summary
        }
        HirExprKind::Call { callee, args } => {
            let mut summary = base_raw_alias_summary_from_value(expr, ctx, tctx);
            let child_summaries = args
                .iter()
                .map(|arg| expression_raw_alias_summary(arg, ctx, tctx));
            add_child_raw_memory_effects(&mut summary, child_summaries);
            if let Some(effect) =
                raw_memory_effect_summary_from_call(callee, args, expr.ty, ctx, tctx)
            {
                extend_unique_raw_memory_effects(&mut summary.raw_memory_effects, [effect]);
            } else if let Some(call_summary) = function_call_raw_alias_summary(expr, ctx, tctx) {
                extend_unique_raw_memory_effects(
                    &mut summary.raw_memory_effects,
                    call_summary.raw_memory_effects,
                );
            }
            summary
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            let mut summary = base_raw_alias_summary_from_value(expr, ctx, tctx);
            let callee_summary = expression_raw_alias_summary(callee, ctx, tctx);
            let child_summaries = core::iter::once(callee.as_ref())
                .chain(args.iter())
                .map(|arg| expression_raw_alias_summary(arg, ctx, tctx));
            add_child_raw_memory_effects(&mut summary, child_summaries);
            if !callee_summary.function_value_aliases.is_empty() {
                let arg_summaries = args
                    .iter()
                    .map(|arg| value_alias_summary_from_value(arg, ctx, tctx))
                    .collect::<Vec<_>>();
                for callee_alias in callee_summary.function_value_aliases {
                    extend_unique_raw_memory_effects(
                        &mut summary.raw_memory_effects,
                        [RawMemoryEffectSummary::IndirectCall {
                            callee: callee_alias,
                            args: arg_summaries.clone(),
                        }],
                    );
                }
            }
            summary
        }
        HirExprKind::StructConstruct { fields, .. } => {
            let mut summary = base_raw_alias_summary_from_value(expr, ctx, tctx);
            let child_summaries = fields
                .iter()
                .map(|field| expression_raw_alias_summary(field, ctx, tctx));
            add_child_raw_memory_effects(&mut summary, child_summaries);
            summary
        }
        HirExprKind::TupleConstruct { items } => {
            let mut summary = base_raw_alias_summary_from_value(expr, ctx, tctx);
            let child_summaries = items
                .iter()
                .map(|item| expression_raw_alias_summary(item, ctx, tctx));
            add_child_raw_memory_effects(&mut summary, child_summaries);
            summary
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            let mut summary = base_raw_alias_summary_from_value(expr, ctx, tctx);
            if let Some(payload) = payload {
                let child = expression_raw_alias_summary(payload, ctx, tctx);
                extend_unique_raw_memory_effects(
                    &mut summary.raw_memory_effects,
                    child.raw_memory_effects,
                );
            }
            summary
        }
        HirExprKind::Intrinsic { name, args, .. } => {
            let mut summary = base_raw_alias_summary_from_value(expr, ctx, tctx);
            let child_summaries = args
                .iter()
                .map(|arg| expression_raw_alias_summary(arg, ctx, tctx));
            add_child_raw_memory_effects(&mut summary, child_summaries);
            if let Some(effect) =
                raw_memory_effect_summary_from_intrinsic(name, args, expr.ty, ctx, tctx)
            {
                extend_unique_raw_memory_effects(&mut summary.raw_memory_effects, [effect]);
            }
            summary
        }
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            expression_raw_alias_summary(value, ctx, tctx)
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
            let mut summary = base_raw_alias_summary_from_value(expr, ctx, tctx);
            let child = expression_raw_alias_summary(inner, ctx, tctx);
            extend_unique_raw_memory_effects(
                &mut summary.raw_memory_effects,
                child.raw_memory_effects,
            );
            summary
        }
        _ => FunctionRawAliasSummary {
            ..base_raw_alias_summary_from_value(expr, ctx, tctx)
        },
    }
}

pub(super) fn block_raw_alias_summary(
    block: &HirBlock,
    base_ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> FunctionRawAliasSummary {
    let mut ctx = base_ctx.clone_for_alias_summary();
    let mut last_summary = FunctionRawAliasSummary::default();
    let mut raw_memory_effects = Vec::new();
    for line in &block.lines {
        match &line.expr.kind {
            HirExprKind::Let { name, value, .. } => {
                let value_summary = expression_raw_alias_summary(value, &ctx, tctx);
                let i32_const_alias = i32_const_from_value(value, &ctx, tctx);
                extend_unique_raw_memory_effects(
                    &mut raw_memory_effects,
                    value_summary.raw_memory_effects.clone(),
                );
                ctx.declare_var(name.clone());
                ctx.set_raw_addr_alias(name, value_summary.raw_addr_alias);
                ctx.set_i32_const_alias(name, i32_const_alias);
                ctx.set_enum_payload_raw_aliases(name, value_summary.enum_payload_raw_aliases);
                ctx.set_aggregate_field_raw_aliases(
                    name,
                    value_summary.aggregate_field_raw_aliases,
                );
                ctx.set_aggregate_field_function_aliases(
                    name,
                    value_summary.aggregate_field_function_aliases,
                );
                ctx.set_enum_payload_aggregate_field_raw_aliases(
                    name,
                    value_summary.enum_payload_aggregate_field_raw_aliases,
                );
                ctx.set_enum_payload_aggregate_field_function_aliases(
                    name,
                    value_summary.enum_payload_aggregate_field_function_aliases,
                );
                ctx.set_enum_payload_function_aliases(
                    name,
                    value_summary.enum_payload_function_aliases,
                );
                ctx.set_function_value_aliases(name, value_summary.function_value_aliases);
                last_summary = FunctionRawAliasSummary::default();
            }
            HirExprKind::Set { name, value } => {
                let value_summary = expression_raw_alias_summary(value, &ctx, tctx);
                let i32_const_alias = i32_const_from_value(value, &ctx, tctx);
                extend_unique_raw_memory_effects(
                    &mut raw_memory_effects,
                    value_summary.raw_memory_effects.clone(),
                );
                ctx.set_raw_addr_alias(name, value_summary.raw_addr_alias);
                ctx.set_i32_const_alias(name, i32_const_alias);
                ctx.set_enum_payload_raw_aliases(name, value_summary.enum_payload_raw_aliases);
                ctx.set_aggregate_field_raw_aliases(
                    name,
                    value_summary.aggregate_field_raw_aliases,
                );
                ctx.set_aggregate_field_function_aliases(
                    name,
                    value_summary.aggregate_field_function_aliases,
                );
                ctx.set_enum_payload_aggregate_field_raw_aliases(
                    name,
                    value_summary.enum_payload_aggregate_field_raw_aliases,
                );
                ctx.set_enum_payload_aggregate_field_function_aliases(
                    name,
                    value_summary.enum_payload_aggregate_field_function_aliases,
                );
                ctx.set_enum_payload_function_aliases(
                    name,
                    value_summary.enum_payload_function_aliases,
                );
                ctx.set_function_value_aliases(name, value_summary.function_value_aliases);
                last_summary = FunctionRawAliasSummary::default();
            }
            _ => {
                let line_summary = expression_raw_alias_summary(&line.expr, &ctx, tctx);
                extend_unique_raw_memory_effects(
                    &mut raw_memory_effects,
                    line_summary.raw_memory_effects.clone(),
                );
                last_summary = line_summary;
            }
        }
    }
    last_summary.raw_memory_effects = raw_memory_effects;
    last_summary
}

fn summarize_function_raw_aliases(
    module: &HirModule,
    func: &crate::hir::HirFunction,
    tctx: &crate::types::TypeCtx,
    function_raw_alias_summaries: &BTreeMap<String, FunctionRawAliasSummary>,
) -> FunctionRawAliasSummary {
    let mut ctx = MoveCheckContext::new(module);
    ctx.function_raw_alias_summaries = function_raw_alias_summaries.clone();
    ctx.push_scope();
    for (index, param) in func.params.iter().enumerate() {
        seed_summary_param_aliases(&mut ctx, param, index, tctx);
    }
    match &func.body {
        crate::hir::HirBody::Block(block) => block_raw_alias_summary(block, &ctx, tctx),
        _ => FunctionRawAliasSummary::default(),
    }
}

pub(super) fn build_function_raw_alias_summaries(
    module: &HirModule,
    tctx: &crate::types::TypeCtx,
) -> BTreeMap<String, FunctionRawAliasSummary> {
    let mut summaries = BTreeMap::new();
    for _ in 0..=module.functions.len() {
        let mut changed = false;
        for func in &module.functions {
            let summary = summarize_function_raw_aliases(module, func, tctx, &summaries);
            if summaries.get(func.name.as_str()) != Some(&summary) {
                summaries.insert(func.name.clone(), summary);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    summaries
}
