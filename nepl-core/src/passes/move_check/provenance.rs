use alloc::string::{String, ToString};

use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::layout::{aggregate_fields_with_offsets, storage_size_bytes};
use crate::types::{TypeId, TypeKind};

use super::raw_place::{
    combine_raw_memory_offsets, format_raw_memory_place_key_parts, parse_raw_memory_place_key,
};
use super::state::FieldMovePath;
use super::{
    aggregate_field_index_layout_from_selector, aggregate_field_layout_from_selector,
    aggregate_field_raw_aliases_from_value, field_get_projection, function_call_raw_alias_summary,
    MoveCheckContext,
};

pub(super) struct RawAggregateFieldProjection<'a> {
    pub(super) addr: &'a HirExpr,
    pub(super) field_ty: TypeId,
    pub(super) place: String,
    pub(super) size: usize,
}

pub(super) fn field_move_path_from_addr(
    addr: &HirExpr,
    field_ty: TypeId,
    tctx: &crate::types::TypeCtx,
) -> Option<FieldMovePath> {
    let (owner, owner_ty, offset) = base_owner(addr)?;
    let matches = aggregate_fields_with_offsets(tctx, owner_ty)
        .into_iter()
        .enumerate()
        .filter_map(|(index, field)| {
            (field.offset == offset && tctx.same_type(field.ty, field_ty)).then_some(index)
        })
        .collect::<alloc::vec::Vec<_>>();
    if !matches.is_empty() {
        Some(FieldMovePath {
            owner: owner.to_string(),
            field_index: (matches.len() == 1).then_some(matches[0]),
            offset,
            field_ty,
        })
    } else {
        None
    }
}

pub(super) fn field_reference_path_from_addr(
    addr: &HirExpr,
    tctx: &crate::types::TypeCtx,
) -> Option<FieldMovePath> {
    let field_ty = match tctx.get_ref(tctx.resolve_id(addr.ty)) {
        TypeKind::Reference(inner, _) => *inner,
        _ => return None,
    };
    field_move_path_from_addr(addr, field_ty, tctx)
}

pub(super) fn field_move_path_from_selector(
    owner_expr: &HirExpr,
    selector: &HirExpr,
    result_ty: TypeId,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<FieldMovePath> {
    let (owner, owner_ty, base_offset) = base_owner(owner_expr)?;
    let (field_index, field_offset, field_ty) =
        aggregate_field_index_layout_from_selector(owner_ty, selector, ctx, tctx)?;
    let field_ty = if tctx.same_type(field_ty, result_ty) {
        field_ty
    } else {
        result_ty
    };
    Some(FieldMovePath {
        owner: owner.to_string(),
        field_index: Some(field_index),
        offset: base_offset + field_offset,
        field_ty,
    })
}

fn base_owner(expr: &HirExpr) -> Option<(&str, TypeId, usize)> {
    match &expr.kind {
        HirExprKind::Var(name) => Some((name.as_str(), expr.ty, 0)),
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && args.len() >= 2 => {
            let (owner, owner_ty, base_offset) = base_owner(&args[0])?;
            let offset = match &args[1].kind {
                HirExprKind::LiteralI32(value) if *value >= 0 => *value as usize,
                _ => return None,
            };
            Some((owner, owner_ty, base_offset + offset))
        }
        _ => None,
    }
}

fn raw_aggregate_load_addr<'a>(
    expr: &'a HirExpr,
    tctx: &crate::types::TypeCtx,
) -> Option<&'a HirExpr> {
    if aggregate_fields_with_offsets(tctx, expr.ty).is_empty() {
        return None;
    }
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. }
            if name == "load" && args.len() == 1 && tctx.same_type(args[0].ty, tctx.i32()) =>
        {
            Some(&args[0])
        }
        HirExprKind::Call { callee, args }
            if args.len() == 1 && tctx.same_type(args[0].ty, tctx.i32()) =>
        {
            let name = func_ref_name(callee)?;
            if name == "load" || name.starts_with("load_") {
                Some(&args[0])
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(super) fn raw_aggregate_field_projection_from_get_field<'a>(
    expr: &'a HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<RawAggregateFieldProjection<'a>> {
    let HirExprKind::Intrinsic { name, args, .. } = &expr.kind else {
        return None;
    };
    if name != "get_field" || args.len() != 2 {
        return None;
    }
    raw_aggregate_field_projection_from_owner_selector(&args[0], &args[1], expr.ty, ctx, tctx)
}

pub(super) fn raw_aggregate_field_projection_from_get_call<'a>(
    callee: &FuncRef,
    args: &'a [HirExpr],
    result_ty: TypeId,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<RawAggregateFieldProjection<'a>> {
    let name = func_ref_name(callee)?;
    if !is_field_get_name(name) || args.len() < 2 {
        return None;
    }
    raw_aggregate_field_projection_from_owner_selector(&args[0], &args[1], result_ty, ctx, tctx)
}

fn raw_aggregate_field_projection_from_owner_selector<'a>(
    owner: &'a HirExpr,
    selector: &HirExpr,
    result_ty: TypeId,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<RawAggregateFieldProjection<'a>> {
    let addr = raw_aggregate_load_addr(owner, tctx)?;
    let (field_offset, field_ty) =
        aggregate_field_layout_from_selector(owner.ty, selector, ctx, tctx)?;
    if !tctx.same_type(field_ty, result_ty) {
        return None;
    }
    let base_key = raw_memory_place_key(addr, ctx, tctx)?;
    let (base, base_offset) = parse_raw_memory_place_key(base_key.as_str());
    let field_offset = i64::try_from(field_offset).ok()?;
    let place = format_raw_memory_place_key_parts(
        base.as_str(),
        combine_raw_memory_offsets(base_offset, Some(field_offset)),
    );
    Some(RawAggregateFieldProjection {
        addr,
        field_ty,
        place,
        size: storage_size_bytes(tctx, field_ty),
    })
}

pub(super) fn i32_const_from_value(
    expr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<i64> {
    match &expr.kind {
        HirExprKind::LiteralI32(value) => Some(i64::from(*value)),
        HirExprKind::Var(name) => ctx.i32_const_alias(name),
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => {
            if name == "size_of" && type_args.len() == 1 {
                return i64::try_from(storage_size_bytes(tctx, type_args[0])).ok();
            }
            i32_const_from_named_call(name, args, ctx, tctx)
        }
        HirExprKind::Call { callee, args } => {
            let name = func_ref_name(callee)?;
            i32_const_from_named_call(name, args, ctx, tctx)
                .or_else(|| i32_const_from_size_of_call(name, ctx, tctx))
        }
        _ => None,
    }
}

fn i32_const_from_named_call(
    name: &str,
    args: &[HirExpr],
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<i64> {
    if args.len() != 2 {
        return None;
    }
    let left = i32_const_from_value(&args[0], ctx, tctx)?;
    let right = i32_const_from_value(&args[1], ctx, tctx)?;
    if is_raw_address_add_name(name) {
        left.checked_add(right)
    } else if is_i32_sub_name(name) {
        left.checked_sub(right)
    } else if is_i32_mul_name(name) {
        left.checked_mul(right)
    } else {
        None
    }
}

fn i32_const_from_size_of_call(
    name: &str,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<i64> {
    if !name.starts_with("size_of__") {
        return None;
    }
    let func = ctx.function_defs.get(name)?;
    match &func.body {
        crate::hir::HirBody::Block(block) if block.lines.len() == 1 => {
            let HirExprKind::Intrinsic {
                name, type_args, ..
            } = &block.lines[0].expr.kind
            else {
                return None;
            };
            if name == "size_of" && type_args.len() == 1 {
                i64::try_from(storage_size_bytes(tctx, type_args[0])).ok()
            } else {
                None
            }
        }
        _ => None,
    }
}

fn negated_i32_const_from_value(
    expr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<i64> {
    i32_const_from_value(expr, ctx, tctx).map(|value| -value)
}

pub(super) fn raw_memory_place_key(
    addr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    fn inner(
        expr: &HirExpr,
        ctx: &MoveCheckContext,
        tctx: &crate::types::TypeCtx,
    ) -> Option<(String, Option<i64>)> {
        match &expr.kind {
            HirExprKind::Var(name) => ctx
                .raw_addr_alias(name)
                .map(parse_raw_memory_place_key)
                .or_else(|| Some((name.clone(), Some(0)))),
            HirExprKind::LiteralI32(value) => Some((String::from("$abs"), Some(i64::from(*value)))),
            HirExprKind::Intrinsic { name, args, .. }
                if (name == "add" || name == "sub") && args.len() >= 2 =>
            {
                let (base, base_offset) = inner(&args[0], ctx, tctx)?;
                let offset = if name == "add" {
                    i32_const_from_value(&args[1], ctx, tctx)
                } else {
                    negated_i32_const_from_value(&args[1], ctx, tctx)
                };
                Some((base, combine_raw_memory_offsets(base_offset, offset)))
            }
            HirExprKind::Call { callee, args }
                if args.len() >= 2 && tctx.same_type(expr.ty, tctx.i32()) =>
            {
                let name = func_ref_name(callee)?;
                if is_raw_address_add_name(name) {
                    let (base, base_offset) = inner(&args[0], ctx, tctx)?;
                    let offset = i32_const_from_value(&args[1], ctx, tctx);
                    Some((base, combine_raw_memory_offsets(base_offset, offset)))
                } else if is_i32_sub_name(name) {
                    let (base, base_offset) = inner(&args[0], ctx, tctx)?;
                    let offset = negated_i32_const_from_value(&args[1], ctx, tctx);
                    Some((base, combine_raw_memory_offsets(base_offset, offset)))
                } else {
                    function_call_raw_alias_summary(expr, ctx, tctx)
                        .and_then(|summary| summary.raw_addr_alias)
                        .map(|key| parse_raw_memory_place_key(key.as_str()))
                }
            }
            HirExprKind::Call { callee, args } if args.len() == 1 => {
                let name = func_ref_name(callee)?;
                if !is_mem_ptr_addr_name(name) {
                    return None;
                }
                raw_memory_place_key_from_mem_ptr(&args[0], ctx, tctx)
                    .map(|key| parse_raw_memory_place_key(key.as_str()))
            }
            _ => None,
        }
    }

    let (base, offset) = inner(addr, ctx, tctx)?;
    Some(format_raw_memory_place_key_parts(base.as_str(), offset))
}

pub(super) fn raw_addr_alias_from_value(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    if let Some(key) = raw_memory_place_key_from_mem_ptr(value, ctx, tctx) {
        return Some(key);
    }
    if let Some(key) = raw_memory_place_key_from_region_token(value, ctx, tctx) {
        return Some(key);
    }
    if let Some(key) = raw_alias_from_aggregate_field_load(value, ctx, tctx) {
        return Some(key);
    }
    if let Some(summary) = function_call_raw_alias_summary(value, ctx, tctx) {
        if let Some(alias) = summary.raw_addr_alias {
            return Some(alias);
        }
    }
    if tctx.same_type(value.ty, tctx.i32()) {
        raw_memory_place_key(value, ctx, tctx)
    } else {
        None
    }
}

pub(super) fn func_ref_name(callee: &FuncRef) -> Option<&str> {
    match callee {
        FuncRef::User(name, _, _) | FuncRef::Builtin(name) => Some(name.as_str()),
        FuncRef::Trait { .. } => None,
    }
}

pub(super) fn is_mem_ptr_addr_name(name: &str) -> bool {
    name == "mem_ptr_addr" || name.starts_with("mem_ptr_addr_")
}

pub(super) fn is_mem_ptr_wrap_name(name: &str) -> bool {
    name == "mem_ptr_wrap" || name.starts_with("mem_ptr_wrap_")
}

pub(super) fn is_mem_ptr_add_name(name: &str) -> bool {
    name == "mem_ptr_add" || name.starts_with("mem_ptr_add_")
}

pub(super) fn is_raw_address_add_name(name: &str) -> bool {
    name == "add" || name.starts_with("add__i32_i32__i32__")
}

fn is_i32_sub_name(name: &str) -> bool {
    name == "sub" || name.starts_with("sub__i32_i32__i32__")
}

fn is_i32_mul_name(name: &str) -> bool {
    name == "mul" || name.starts_with("mul__i32_i32__i32__")
}

pub(super) fn is_region_ptr_name(name: &str) -> bool {
    name == "region_ptr" || name.starts_with("region_ptr_")
}

pub(super) fn is_region_new_name(name: &str) -> bool {
    name == "region_new" || name.starts_with("region_new_")
}

pub(super) fn is_region_ptr_at_name(name: &str) -> bool {
    name == "region_ptr_at" || name.starts_with("region_ptr_at_")
}

pub(super) fn is_field_get_name(name: &str) -> bool {
    name == "get" || name.starts_with("get__")
}

pub(super) fn is_mem_ptr_type(tctx: &crate::types::TypeCtx, ty: TypeId) -> bool {
    match tctx.get_ref(tctx.resolve_id(ty)) {
        TypeKind::Struct { name, .. } if name == "MemPtr" => true,
        TypeKind::Apply { base, .. } => match tctx.get_ref(tctx.resolve_id(*base)) {
            TypeKind::Struct { name, .. } => name == "MemPtr",
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn is_region_token_type(tctx: &crate::types::TypeCtx, ty: TypeId) -> bool {
    match tctx.get_ref(tctx.resolve_id(ty)) {
        TypeKind::Struct { name, .. } if name == "RegionToken" => true,
        TypeKind::Apply { base, .. } => match tctx.get_ref(tctx.resolve_id(*base)) {
            TypeKind::Struct { name, .. } => name == "RegionToken",
            _ => false,
        },
        _ => false,
    }
}

pub(super) fn aggregate_field_raw_alias_at(
    owner: &HirExpr,
    offset: usize,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    if let HirExprKind::Var(name) = &owner.kind {
        return ctx
            .aggregate_field_raw_alias(name, offset)
            .map(ToString::to_string);
    }
    aggregate_field_raw_aliases_from_value(owner, ctx, tctx)
        .get(&offset)
        .cloned()
}

fn raw_alias_from_aggregate_field_load(
    value: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    let HirExprKind::Intrinsic { name, args, .. } = &value.kind else {
        return None;
    };
    if name != "load" || args.len() != 1 {
        return None;
    }
    let path = field_move_path_from_addr(&args[0], value.ty, tctx)?;
    ctx.aggregate_field_raw_alias(path.owner.as_str(), path.offset)
        .map(ToString::to_string)
}

pub(super) fn raw_memory_place_key_from_mem_ptr(
    expr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    match &expr.kind {
        HirExprKind::Var(name) if is_mem_ptr_type(tctx, expr.ty) => ctx
            .raw_addr_alias(name)
            .map(ToString::to_string)
            .or_else(|| Some(alloc::format!("$memptr:{}", name))),
        HirExprKind::Call { callee, args } if args.len() == 1 => {
            let name = func_ref_name(callee)?;
            if is_mem_ptr_wrap_name(name) {
                raw_memory_place_key(&args[0], ctx, tctx)
            } else if is_region_ptr_name(name) {
                raw_memory_place_key_from_region_token(&args[0], ctx, tctx)
            } else {
                function_call_raw_alias_summary(expr, ctx, tctx)
                    .and_then(|summary| summary.raw_addr_alias)
            }
        }
        HirExprKind::Call { callee, args } if args.len() >= 2 && is_mem_ptr_type(tctx, expr.ty) => {
            let name = func_ref_name(callee)?;
            if is_mem_ptr_add_name(name) {
                let key = raw_memory_place_key_from_mem_ptr(&args[0], ctx, tctx)?;
                let offset = i32_const_from_value(&args[1], ctx, tctx);
                let (base, base_offset) = parse_raw_memory_place_key(key.as_str());
                Some(format_raw_memory_place_key_parts(
                    base.as_str(),
                    combine_raw_memory_offsets(base_offset, offset),
                ))
            } else if is_field_get_name(name) {
                if let Some((owner, offset, _)) = field_get_projection(expr, ctx, tctx) {
                    if let Some(alias) = aggregate_field_raw_alias_at(owner, offset, ctx, tctx) {
                        return Some(alias);
                    }
                }
                if is_region_token_type(tctx, args[0].ty) {
                    raw_memory_place_key_from_region_token(&args[0], ctx, tctx)
                } else {
                    None
                }
            } else {
                function_call_raw_alias_summary(expr, ctx, tctx)
                    .and_then(|summary| summary.raw_addr_alias)
            }
        }
        HirExprKind::StructConstruct { name, fields, .. }
            if name == "MemPtr" && fields.len() == 1 =>
        {
            raw_memory_place_key(&fields[0], ctx, tctx)
        }
        HirExprKind::Intrinsic { name, args, .. }
            if name == "load" && args.len() == 1 && is_mem_ptr_type(tctx, expr.ty) =>
        {
            raw_alias_from_aggregate_field_load(expr, ctx, tctx)
        }
        HirExprKind::Call { .. } => function_call_raw_alias_summary(expr, ctx, tctx)
            .and_then(|summary| summary.raw_addr_alias),
        _ => None,
    }
}

pub(super) fn raw_memory_place_key_from_region_token(
    expr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    match &expr.kind {
        HirExprKind::Var(name) if is_region_token_type(tctx, expr.ty) => ctx
            .raw_addr_alias(name)
            .map(ToString::to_string)
            .or_else(|| Some(alloc::format!("$region:{}", name))),
        HirExprKind::Call { callee, args } if args.len() >= 2 => {
            let name = func_ref_name(callee)?;
            if is_region_new_name(name) {
                return raw_memory_place_key_from_mem_ptr(&args[0], ctx, tctx);
            }
            if is_field_get_name(name) && is_region_token_type(tctx, expr.ty) {
                let (owner, offset, _) = field_get_projection(expr, ctx, tctx)?;
                return aggregate_field_raw_alias_at(owner, offset, ctx, tctx);
            }
            function_call_raw_alias_summary(expr, ctx, tctx)
                .and_then(|summary| summary.raw_addr_alias)
        }
        HirExprKind::StructConstruct { name, fields, .. }
            if name == "RegionToken" && !fields.is_empty() =>
        {
            raw_memory_place_key_from_mem_ptr(&fields[0], ctx, tctx)
        }
        HirExprKind::Intrinsic { name, args, .. }
            if name == "load" && args.len() == 1 && is_region_token_type(tctx, expr.ty) =>
        {
            raw_alias_from_aggregate_field_load(expr, ctx, tctx)
        }
        HirExprKind::Call { .. } => function_call_raw_alias_summary(expr, ctx, tctx)
            .and_then(|summary| summary.raw_addr_alias),
        _ => None,
    }
}
