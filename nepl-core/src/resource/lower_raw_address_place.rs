use crate::hir::{HirExpr, HirExprKind};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::lower::LoweringEnvironment;
use super::model::{Place, PlaceProjection};
use super::place_utils::mem_ptr_raw_field_place;

pub(super) fn raw_address_place_from_actual_argument(
    expr: &HirExpr,
    place: &Place,
    env: &LoweringEnvironment,
) -> Place {
    if is_named_struct_type(env.types, place.ty, "MemPtr") {
        mem_ptr_raw_field_place(place, env.types.i32())
    } else if is_named_struct_type(env.types, place.ty, "RegionToken") {
        region_token_raw_field_place(place, env.types.i32())
    } else if let Some(target_ty) = reference_target_type(env.types, place.ty) {
        let target = borrowed_source_place(expr, place, target_ty);
        if is_named_struct_type(env.types, target_ty, "MemPtr") {
            mem_ptr_raw_field_place(&target, env.types.i32())
        } else if is_named_struct_type(env.types, target_ty, "RegionToken") {
            region_token_raw_field_place(&target, env.types.i32())
        } else {
            place.clone()
        }
    } else {
        place.clone()
    }
}

pub(super) fn region_token_place_from_actual_arg(
    expr: &HirExpr,
    place: &Place,
    env: &LoweringEnvironment,
) -> Option<Place> {
    if is_named_struct_type(env.types, place.ty, "RegionToken") {
        return Some(place.clone());
    }
    let target_ty = reference_target_type(env.types, place.ty)?;
    if !is_named_struct_type(env.types, target_ty, "RegionToken") {
        return None;
    }
    Some(borrowed_source_place(expr, place, target_ty))
}

pub(super) fn raw_address_alias_target(output: &Place, env: &LoweringEnvironment) -> Place {
    if is_named_struct_type(env.types, output.ty, "MemPtr") {
        mem_ptr_raw_field_place(output, env.types.i32())
    } else if is_named_struct_type(env.types, output.ty, "RegionToken") {
        region_token_raw_field_place(output, env.types.i32())
    } else {
        output.clone()
    }
}

pub(super) fn is_named_struct_type(types: &TypeCtx, ty: TypeId, expected: &str) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { name, .. } => name == expected,
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(types.get_ref(base), TypeKind::Struct { name, .. } if name == expected)
        }
        _ => false,
    }
}

pub(super) fn region_token_raw_field_place(token: &Place, raw_ty: TypeId) -> Place {
    mem_ptr_raw_field_place(&region_token_ptr_field_place(token, token.ty), raw_ty)
}

fn borrowed_source_place(expr: &HirExpr, reference_place: &Place, target_ty: TypeId) -> Place {
    if let HirExprKind::AddrOf(inner) = &expr.kind {
        if let Some(place) = simple_borrowed_expr_place(inner) {
            return place;
        }
    }
    reference_place
        .clone()
        .with_projection(PlaceProjection::Deref, target_ty)
}

fn simple_borrowed_expr_place(expr: &HirExpr) -> Option<Place> {
    match &expr.kind {
        HirExprKind::Var(name) => Some(Place::local(name.clone(), expr.ty)),
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

fn region_token_ptr_field_place(token: &Place, ptr_ty: TypeId) -> Place {
    token.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        ptr_ty,
    )
}
