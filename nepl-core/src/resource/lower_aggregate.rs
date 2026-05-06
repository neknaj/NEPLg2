extern crate alloc;

use alloc::vec::Vec;

use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::runtime_helpers::helper_base_name;
use crate::types::TypeId;

use super::lower::{
    func_ref_base_name, lower_expr_skeleton, place_from_expr_skeleton, LoweringContext,
    LoweringEnvironment,
};
use super::lower_aggregate_projection::{
    aggregate_field_projection, aggregate_field_projection_by_name,
    compiler_field_address_base_and_offset, literal_field_name, non_negative_i32_literal,
    reference_target_type,
};
use super::lower_raw_address_place::is_named_struct_type;
use super::lower_raw_memory::{raw_memory_op_from_callee, raw_memory_op_from_intrinsic};
use super::model::{Place, PlaceProjection, RawMemoryOp, ResourceOp};
use super::place_utils::raw_memory_cell_place;

pub(super) fn lower_compiler_field_load_source(
    name: &str,
    args: &[HirExpr],
    field_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    if !matches!(raw_memory_op_from_intrinsic(name), Some(RawMemoryOp::Load)) {
        return None;
    }
    let address = args.first()?;
    let (base_expr, offset_bytes) = compiler_field_address_base_and_offset(address)?;
    let projection = aggregate_field_projection(env.types, base_expr.ty, offset_bytes, field_ty)?;
    if let Some(source) =
        lower_raw_aggregate_field_source(base_expr, projection.clone(), field_ty, ops, ctx, env)
    {
        return Some(source);
    }
    let mut base = place_from_expr_skeleton(base_expr, ctx);
    if matches!(&base.root, super::model::PlaceRoot::Unknown) {
        base = lower_expr_skeleton(base_expr, ops, ctx, env);
    }
    Some(base.with_projection(projection, field_ty))
}

pub(super) fn lower_field_get_call_source(
    callee: &FuncRef,
    args: &[HirExpr],
    field_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    if func_ref_base_name(callee)? != "get" {
        return None;
    }
    let owner = args.first()?;
    let field_name = literal_field_name(env, args.get(1)?)?;
    let projection = aggregate_field_projection_by_name(env.types, owner.ty, field_name, field_ty)?;
    if let Some(source) =
        lower_raw_aggregate_field_source(owner, projection.clone(), field_ty, ops, ctx, env)
    {
        return Some(source);
    }
    let mut base = place_from_expr_skeleton(owner, ctx);
    if matches!(&base.root, super::model::PlaceRoot::Unknown) {
        base = lower_expr_skeleton(owner, ops, ctx, env);
    }
    Some(base.with_projection(projection, field_ty))
}

pub(super) fn lower_field_get_ref_call_source(
    callee: &FuncRef,
    args: &[HirExpr],
    ref_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    if func_ref_base_name(callee)? != "get_ref" {
        return None;
    }
    field_get_ref_source(args, ref_ty, ops, ctx, env)
}

pub(super) fn lower_get_field_intrinsic_source(
    name: &str,
    args: &[HirExpr],
    field_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    if helper_base_name(name) != "get_field" {
        return None;
    }
    let owner = args.first()?;
    let projection =
        if let Some(field_name) = args.get(1).and_then(|arg| literal_field_name(env, arg)) {
            aggregate_field_projection_by_name(env.types, owner.ty, field_name, field_ty)?
        } else if is_named_struct_type(env.types, owner.ty, "RegionToken")
            && is_named_struct_type(env.types, field_ty, "MemPtr")
        {
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            }
        } else {
            return None;
        };
    if let Some(source) =
        lower_raw_aggregate_field_source(owner, projection.clone(), field_ty, ops, ctx, env)
    {
        return Some(source);
    }
    let mut base = place_from_expr_skeleton(owner, ctx);
    if matches!(&base.root, super::model::PlaceRoot::Unknown) {
        base = lower_expr_skeleton(owner, ops, ctx, env);
    }
    Some(base.with_projection(projection, field_ty))
}

pub(super) fn lower_get_field_ref_intrinsic_source(
    name: &str,
    args: &[HirExpr],
    ref_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    if helper_base_name(name) != "get_field_ref" {
        return None;
    }
    field_get_ref_source(args, ref_ty, ops, ctx, env)
}

pub(super) fn lower_reference_address_projection_source(
    name: &str,
    args: &[HirExpr],
    ref_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    if helper_base_name(name) != "add" || args.len() != 2 {
        return None;
    }
    let owner = args.first()?;
    let owner_ty = reference_target_type(env.types, owner.ty)?;
    let field_ty = reference_target_type(env.types, ref_ty)?;
    let offset_bytes = non_negative_i32_literal(args.get(1)?)?;
    let projection = aggregate_field_projection(env.types, owner_ty, offset_bytes, field_ty)?;
    let base = reference_owner_target_place(owner, owner_ty, ops, ctx, env);
    Some(base.with_projection(projection, field_ty))
}

fn field_get_ref_source(
    args: &[HirExpr],
    ref_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    let owner = args.first()?;
    let owner_ref_target = reference_target_type(env.types, owner.ty);
    let owner_ty = owner_ref_target.unwrap_or(owner.ty);
    let field_ty = reference_target_type(env.types, ref_ty)?;
    let projection =
        if let Some(field_name) = args.get(1).and_then(|arg| literal_field_name(env, arg)) {
            aggregate_field_projection_by_name(env.types, owner_ty, field_name, field_ty)?
        } else if is_named_struct_type(env.types, owner_ty, "RegionToken")
            && is_named_struct_type(env.types, field_ty, "MemPtr")
        {
            PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            }
        } else {
            return None;
        };
    let base = if owner_ref_target.is_some() {
        reference_owner_target_place(owner, owner_ty, ops, ctx, env)
    } else {
        let mut base = place_from_expr_skeleton(owner, ctx);
        if matches!(&base.root, super::model::PlaceRoot::Unknown) {
            base = lower_expr_skeleton(owner, ops, ctx, env);
        }
        base
    };
    Some(base.with_projection(projection, field_ty))
}

fn reference_owner_target_place(
    owner: &HirExpr,
    target_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Place {
    if let HirExprKind::AddrOf(inner) = &owner.kind {
        let mut source = place_from_expr_skeleton(inner, ctx);
        if matches!(&source.root, super::model::PlaceRoot::Unknown) {
            source = lower_expr_skeleton(inner, ops, ctx, env);
        }
        return source;
    }
    let mut base = place_from_expr_skeleton(owner, ctx);
    if matches!(&base.root, super::model::PlaceRoot::Unknown) {
        base = lower_expr_skeleton(owner, ops, ctx, env);
    }
    base.with_projection(PlaceProjection::Deref, target_ty)
}

fn lower_raw_aggregate_field_source(
    base_expr: &HirExpr,
    projection: PlaceProjection,
    field_ty: TypeId,
    ops: &mut Vec<ResourceOp>,
    ctx: &mut LoweringContext,
    env: &LoweringEnvironment,
) -> Option<Place> {
    let address = raw_load_address_expr(base_expr)?;
    let mut address_place = place_from_expr_skeleton(address, ctx);
    if matches!(&address_place.root, super::model::PlaceRoot::Unknown) {
        address_place = lower_expr_skeleton(address, ops, ctx, env);
    }
    Some(raw_memory_cell_place(&address_place, base_expr.ty).with_projection(projection, field_ty))
}

fn raw_load_address_expr(expr: &HirExpr) -> Option<&HirExpr> {
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. }
            if matches!(raw_memory_op_from_intrinsic(name), Some(RawMemoryOp::Load)) =>
        {
            args.first()
        }
        HirExprKind::Call { callee, args }
            if matches!(raw_memory_op_from_callee(callee), Some(RawMemoryOp::Load)) =>
        {
            args.first()
        }
        _ => None,
    }
}
