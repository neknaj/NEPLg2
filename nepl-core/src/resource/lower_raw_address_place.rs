use alloc::collections::BTreeMap;
use alloc::string::String;

use crate::hir::{FuncRef, HirExpr, HirExprKind, HirFunction};
use crate::layout::{extend_type_mapping, mapped_type_id};
use crate::runtime_helpers::helper_base_name;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::lower::LoweringEnvironment;
use super::model::{Place, PlaceProjection};

pub(super) fn function_param_index(function: &HirFunction, name: &str) -> Option<usize> {
    function.params.iter().position(|param| param.name == name)
}

pub(super) fn raw_address_place_from_argument(place: &Place, env: &LoweringEnvironment) -> Place {
    let value = value_place_from_argument(place, env);
    if is_named_struct_type(env.types, value.ty, "MemPtr") {
        mem_ptr_raw_field_place(&value, env.types.i32())
    } else {
        value
    }
}

pub(super) fn raw_address_alias_target(output: &Place, env: &LoweringEnvironment) -> Place {
    let value = value_place_from_argument(output, env);
    if is_named_struct_type(env.types, value.ty, "MemPtr") {
        mem_ptr_raw_field_place(&value, env.types.i32())
    } else if is_named_struct_type(env.types, value.ty, "RegionToken") {
        region_token_raw_field_place(&value, env.types.i32())
    } else {
        output.clone()
    }
}

pub(super) fn value_place_from_argument(place: &Place, env: &LoweringEnvironment) -> Place {
    match reference_inner_type(env.types, place.ty) {
        Some(inner) => place.clone().with_projection(PlaceProjection::Deref, inner),
        None => place.clone(),
    }
}

fn reference_inner_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_id(ty);
    match types.get_ref(resolved) {
        TypeKind::Reference(inner, _) => Some(*inner),
        _ => None,
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

pub(super) fn mem_ptr_raw_field_place(ptr: &Place, raw_ty: TypeId) -> Place {
    ptr.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        raw_ty,
    )
}

pub(super) fn result_ok_mem_ptr_raw_field_place(
    result: &Place,
    payload_ty: TypeId,
    raw_ty: TypeId,
) -> Place {
    let payload = result.clone().with_projection(
        PlaceProjection::EnumPayload {
            variant: String::from("Ok"),
        },
        payload_ty,
    );
    mem_ptr_raw_field_place(&payload, raw_ty)
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

pub(super) fn region_token_raw_field_place(token: &Place, raw_ty: TypeId) -> Place {
    mem_ptr_raw_field_place(&region_token_ptr_field_place(token, token.ty), raw_ty)
}

pub(super) fn non_negative_i32_literal_bytes(expr: &HirExpr) -> Option<usize> {
    match &expr.kind {
        HirExprKind::LiteralI32(value) if *value >= 0 => Some(*value as usize),
        _ => None,
    }
}

pub(super) fn enum_payload_type(
    types: &TypeCtx,
    enum_ty: TypeId,
    variant_name: &str,
) -> Option<TypeId> {
    let resolved = types.resolve_id(enum_ty);
    match types.get_ref(resolved) {
        TypeKind::Enum { variants, .. } => variants
            .iter()
            .find(|variant| variant_name_matches(&variant.name, variant_name))
            .and_then(|variant| variant.payload),
        TypeKind::Apply { base, args } => {
            let base = types.resolve_named_type_id(*base);
            let TypeKind::Enum {
                type_params,
                variants,
                ..
            } = types.get_ref(base)
            else {
                return None;
            };
            let mapping = extend_type_mapping(types, &BTreeMap::new(), type_params, args);
            variants
                .iter()
                .find(|variant| variant_name_matches(&variant.name, variant_name))
                .and_then(|variant| variant.payload)
                .map(|payload| mapped_type_id(types, payload, &mapping))
        }
        TypeKind::Named(_) => {
            let named = types.resolve_named_type_id(resolved);
            if named == resolved {
                None
            } else {
                enum_payload_type(types, named, variant_name)
            }
        }
        _ => {
            let named = types.resolve_named_type_id(resolved);
            if named == resolved {
                None
            } else {
                enum_payload_type(types, named, variant_name)
            }
        }
    }
}

fn variant_name_matches(defined: &str, projected: &str) -> bool {
    defined == projected || projected.rsplit("::").next() == Some(defined)
}

pub(super) fn func_ref_base_name(callee: &FuncRef) -> Option<&str> {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Some(helper_base_name(name)),
        FuncRef::Trait { .. } => None,
    }
}

pub(super) fn literal_field_name<'a>(
    env: &'a LoweringEnvironment,
    expr: &HirExpr,
) -> Option<&'a str> {
    match &expr.kind {
        HirExprKind::LiteralStr(index) => {
            env.string_literals.get(*index as usize).map(String::as_str)
        }
        _ => None,
    }
}
