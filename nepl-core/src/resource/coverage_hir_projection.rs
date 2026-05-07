extern crate alloc;

use alloc::string::String;

use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::runtime_helpers::helper_base_name;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::coverage_hir_projection_aggregate::{
    aggregate_field_exists, aggregate_field_matches_selector,
    compiler_field_address_base_and_offset, reference_target_type,
};

pub(super) fn field_get_call_owner<'a>(
    callee: &FuncRef,
    args: &'a [HirExpr],
    field_ty: TypeId,
    types: &TypeCtx,
    string_literals: &[String],
) -> Option<&'a HirExpr> {
    let name = match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => helper_base_name(name),
        FuncRef::Trait { .. } => return None,
    };
    if name != "get" {
        return None;
    }
    let owner = args.first()?;
    aggregate_field_matches_selector(types, owner.ty, args.get(1)?, field_ty, string_literals)
        .then_some(owner)
}

pub(super) fn get_field_intrinsic_owner<'a>(
    name: &str,
    args: &'a [HirExpr],
    field_ty: TypeId,
    types: &TypeCtx,
    string_literals: &[String],
) -> Option<&'a HirExpr> {
    if helper_base_name(name) != "get_field" {
        return None;
    }
    let owner = args.first()?;
    aggregate_field_matches_selector(types, owner.ty, args.get(1)?, field_ty, string_literals)
        .then_some(owner)
}

pub(super) fn get_field_ref_intrinsic_owner<'a>(
    name: &str,
    args: &'a [HirExpr],
    ref_ty: TypeId,
    types: &TypeCtx,
    string_literals: &[String],
) -> Option<&'a HirExpr> {
    if helper_base_name(name) != "get_field_ref" {
        return None;
    }
    let owner = args.first()?;
    let owner_ty = reference_target_type(types, owner.ty)?;
    let field_ty = reference_target_type(types, ref_ty)?;
    aggregate_field_matches_selector(types, owner_ty, args.get(1)?, field_ty, string_literals)
        .then_some(owner)
}

pub(super) fn raw_load_address_expr(expr: &HirExpr) -> Option<&HirExpr> {
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. } if name == "load" => args.first(),
        HirExprKind::Call { callee, args } if callee_is_raw_load(callee) => args.first(),
        _ => None,
    }
}

pub(super) fn callee_projects_reference_address(
    callee: &FuncRef,
    args: &[HirExpr],
    types: &TypeCtx,
) -> bool {
    projects_reference_address(callee_base_name(callee), args, types)
}

pub(super) fn callee_projects_reference_field(
    callee: &FuncRef,
    args: &[HirExpr],
    ref_ty: TypeId,
    types: &TypeCtx,
) -> bool {
    projects_reference_field(callee_base_name(callee), args, ref_ty, types)
}

pub(super) fn intrinsic_projects_reference_address(
    name: &str,
    args: &[HirExpr],
    types: &TypeCtx,
) -> bool {
    projects_reference_address(Some(helper_base_name(name)), args, types)
}

pub(super) fn intrinsic_projects_reference_field(
    name: &str,
    args: &[HirExpr],
    ref_ty: TypeId,
    types: &TypeCtx,
) -> bool {
    projects_reference_field(Some(helper_base_name(name)), args, ref_ty, types)
}

pub(super) fn compiler_field_load_base_and_offset<'a>(
    name: &str,
    args: &'a [HirExpr],
    field_ty: TypeId,
    types: &TypeCtx,
) -> Option<(&'a HirExpr, usize)> {
    if name != "load" {
        return None;
    }
    let address = args.first()?;
    let (base, offset) = compiler_field_address_base_and_offset(address)?;
    aggregate_field_exists(types, base.ty, offset, field_ty).then_some((base, offset))
}

fn projects_reference_address(name: Option<&str>, args: &[HirExpr], types: &TypeCtx) -> bool {
    matches!(name, Some("add" | "sub"))
        && args.first().is_some_and(|arg| {
            matches!(
                types.get_ref(types.resolve_named_type_id(types.resolve_id(arg.ty))),
                TypeKind::Reference(_, _)
            )
        })
}

fn projects_reference_field(
    name: Option<&str>,
    args: &[HirExpr],
    ref_ty: TypeId,
    types: &TypeCtx,
) -> bool {
    if !matches!(name, Some("add")) || args.len() != 2 {
        return false;
    }
    let Some(owner) = args.first() else {
        return false;
    };
    let Some(owner_ty) = reference_target_type(types, owner.ty) else {
        return false;
    };
    let Some(field_ty) = reference_target_type(types, ref_ty) else {
        return false;
    };
    let HirExprKind::LiteralI32(offset) = args[1].kind else {
        return false;
    };
    if offset < 0 {
        return false;
    }
    aggregate_field_exists(types, owner_ty, offset as usize, field_ty)
}

fn callee_base_name(callee: &FuncRef) -> Option<&str> {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Some(helper_base_name(name)),
        FuncRef::Trait { .. } => None,
    }
}

fn callee_is_raw_load(callee: &FuncRef) -> bool {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => {
            let base = helper_base_name(name);
            base == "load" || base.starts_with("load_")
        }
        FuncRef::Trait { .. } => false,
    }
}
