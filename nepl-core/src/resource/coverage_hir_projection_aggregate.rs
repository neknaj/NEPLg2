extern crate alloc;

use alloc::string::String;

use crate::hir::{HirExpr, HirExprKind};
use crate::layout::aggregate_fields_with_offsets;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::type_pattern::field_type_matches_result;

pub(super) fn literal_field_name<'a>(
    string_literals: &'a [String],
    expr: &HirExpr,
) -> Option<&'a str> {
    match &expr.kind {
        HirExprKind::LiteralStr(index) => string_literals.get(*index as usize).map(String::as_str),
        _ => None,
    }
}

pub(super) fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}

pub(super) fn aggregate_field_exists(
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

pub(super) fn aggregate_field_exists_by_name(
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

pub(super) fn aggregate_field_matches_selector(
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
