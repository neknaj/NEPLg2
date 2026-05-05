extern crate alloc;

use alloc::string::String;

use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::layout::aggregate_fields_with_offsets;
use crate::runtime_helpers::helper_base_name;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::type_pattern::field_type_matches_result;

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
    let field_name = literal_field_name(string_literals, args.get(1)?)?;
    aggregate_field_exists_by_name(types, owner.ty, field_name, field_ty).then_some(owner)
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
    let field_name = literal_field_name(string_literals, args.get(1)?)?;
    aggregate_field_exists_by_name(types, owner.ty, field_name, field_ty).then_some(owner)
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

pub(super) fn intrinsic_projects_reference_address(
    name: &str,
    args: &[HirExpr],
    types: &TypeCtx,
) -> bool {
    projects_reference_address(Some(helper_base_name(name)), args, types)
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

fn literal_field_name<'a>(string_literals: &'a [String], expr: &HirExpr) -> Option<&'a str> {
    match &expr.kind {
        HirExprKind::LiteralStr(index) => string_literals.get(*index as usize).map(String::as_str),
        _ => None,
    }
}

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}

fn compiler_field_address_base_and_offset(expr: &HirExpr) -> Option<(&HirExpr, usize)> {
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && args.len() == 2 => {
            let offset = match args[1].kind {
                HirExprKind::LiteralI32(value) if value >= 0 => value as usize,
                _ => return None,
            };
            Some((&args[0], offset))
        }
        HirExprKind::Call { callee, args }
            if callee_base_name(callee).is_some_and(|name| name == "add") && args.len() == 2 =>
        {
            let offset = match args[1].kind {
                HirExprKind::LiteralI32(value) if value >= 0 => value as usize,
                _ => return None,
            };
            Some((&args[0], offset))
        }
        _ => Some((expr, 0)),
    }
}

fn aggregate_field_exists(
    types: &TypeCtx,
    owner_ty: TypeId,
    offset: usize,
    field_ty: TypeId,
) -> bool {
    if !is_aggregate_projection_owner(types, owner_ty) {
        return false;
    }
    aggregate_fields_with_offsets(types, owner_ty)
        .iter()
        .any(|field| field.offset == offset && field_type_matches_result(types, field.ty, field_ty))
}

fn aggregate_field_exists_by_name(
    types: &TypeCtx,
    owner_ty: TypeId,
    field_name: &str,
    field_ty: TypeId,
) -> bool {
    let Some(index) = aggregate_field_index(types, owner_ty, field_name) else {
        return false;
    };
    aggregate_fields_with_offsets(types, owner_ty)
        .get(index)
        .is_some_and(|field| field_type_matches_result(types, field.ty, field_ty))
}

fn aggregate_field_matches_selector(
    types: &TypeCtx,
    owner_ty: TypeId,
    selector: &HirExpr,
    field_ty: TypeId,
    string_literals: &[String],
) -> bool {
    let index = match &selector.kind {
        HirExprKind::LiteralI32(value) if *value >= 0 => Some(*value as usize),
        HirExprKind::LiteralStr(index) => string_literals
            .get(*index as usize)
            .and_then(|field_name| aggregate_field_index(types, owner_ty, field_name)),
        _ => None,
    };
    index
        .and_then(|index| {
            aggregate_fields_with_offsets(types, owner_ty)
                .get(index)
                .copied()
        })
        .is_some_and(|field| field_type_matches_result(types, field.ty, field_ty))
}

fn aggregate_field_index(types: &TypeCtx, owner_ty: TypeId, field_name: &str) -> Option<usize> {
    let resolved = types.resolve_named_type_id(types.resolve_id(owner_ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { field_names, .. } => {
            field_names.iter().position(|name| name == field_name)
        }
        TypeKind::Tuple { .. } => field_name.parse::<usize>().ok(),
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct { field_names, .. } => {
                    field_names.iter().position(|name| name == field_name)
                }
                TypeKind::Tuple { .. } => field_name.parse::<usize>().ok(),
                _ => None,
            }
        }
        _ => None,
    }
}

fn is_aggregate_projection_owner(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { .. } | TypeKind::Tuple { .. } => true,
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(
                types.get_ref(base),
                TypeKind::Struct { .. } | TypeKind::Tuple { .. }
            )
        }
        _ => false,
    }
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
