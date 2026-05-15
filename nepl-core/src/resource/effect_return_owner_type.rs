use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::layout::{extend_type_mapping, mapped_type_id};
use crate::types::{TypeCtx, TypeId, TypeKind};

pub(super) fn raw_identity_type_is_opaque_owner(types: &TypeCtx, ty: TypeId) -> bool {
    if types.resolve_named_type_id(types.resolve_id(ty)) == types.str() {
        return true;
    }
    is_region_token_type(types, ty)
}

pub(super) fn raw_identity_type_is_structural_owner_carrier(types: &TypeCtx, ty: TypeId) -> bool {
    raw_identity_struct_type_contains_opaque_owner(types, ty, &BTreeMap::new(), &mut Vec::new())
}

fn raw_identity_struct_type_contains_opaque_owner(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut Vec<TypeId>,
) -> bool {
    let resolved = mapped_type_id(types, ty, mapping);
    if raw_identity_type_is_opaque_owner(types, resolved) {
        return true;
    }
    if seen.contains(&resolved) {
        return false;
    }
    seen.push(resolved);
    let result = match types.get_ref(resolved) {
        TypeKind::Struct { fields, .. } => fields
            .iter()
            .any(|field| raw_identity_type_contains_opaque_owner(types, *field, mapping, seen)),
        TypeKind::Apply { base, args } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct {
                    type_params,
                    fields,
                    ..
                } => {
                    let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
                    fields.iter().any(|field| {
                        raw_identity_type_contains_opaque_owner(
                            types,
                            *field,
                            &nested_mapping,
                            seen,
                        )
                    })
                }
                _ => false,
            }
        }
        TypeKind::Named(_) => {
            let named = types.resolve_named_type_id(resolved);
            named != resolved
                && raw_identity_struct_type_contains_opaque_owner(types, named, mapping, seen)
        }
        _ => false,
    };
    seen.pop();
    result
}

fn raw_identity_type_contains_opaque_owner(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut Vec<TypeId>,
) -> bool {
    let resolved = mapped_type_id(types, ty, mapping);
    if raw_identity_type_is_opaque_owner(types, resolved) {
        return true;
    }
    if seen.contains(&resolved) {
        return false;
    }
    seen.push(resolved);
    let result = match types.get_ref(resolved) {
        TypeKind::Struct { fields, .. } => fields
            .iter()
            .any(|field| raw_identity_type_contains_opaque_owner(types, *field, mapping, seen)),
        TypeKind::Tuple { items } => items
            .iter()
            .any(|item| raw_identity_type_contains_opaque_owner(types, *item, mapping, seen)),
        TypeKind::Apply { base, args } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct {
                    type_params,
                    fields,
                    ..
                } => {
                    let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
                    fields.iter().any(|field| {
                        raw_identity_type_contains_opaque_owner(
                            types,
                            *field,
                            &nested_mapping,
                            seen,
                        )
                    })
                }
                TypeKind::Tuple { items } => items.iter().any(|item| {
                    raw_identity_type_contains_opaque_owner(types, *item, mapping, seen)
                }),
                _ => false,
            }
        }
        TypeKind::Reference(target, _) | TypeKind::Box(target) => {
            raw_identity_type_contains_opaque_owner(types, *target, mapping, seen)
        }
        TypeKind::Named(_) => {
            let named = types.resolve_named_type_id(resolved);
            named != resolved
                && raw_identity_type_contains_opaque_owner(types, named, mapping, seen)
        }
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Enum { .. }
        | TypeKind::Function { .. }
        | TypeKind::Var(_) => false,
    };
    seen.pop();
    result
}

fn is_region_token_type(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { name, .. } => name == "RegionToken",
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(types.get_ref(base), TypeKind::Struct { name, .. } if name == "RegionToken")
        }
        _ => false,
    }
}
