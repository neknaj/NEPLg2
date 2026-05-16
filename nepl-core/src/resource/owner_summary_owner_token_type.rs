use alloc::collections::{BTreeMap, BTreeSet};

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::resource_primitives::type_is_owner_token;
use crate::types::{TypeCtx, TypeId, TypeKind};

pub(super) fn type_contains_owner_token(types: &TypeCtx, ty: TypeId) -> bool {
    type_contains_owner_token_mapped(types, ty, &BTreeMap::new(), &mut BTreeSet::new())
}

fn type_contains_owner_token_mapped(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
) -> bool {
    let mapped = mapped_type_id(types, ty, mapping);
    if type_is_owner_token(types, mapped) {
        return true;
    }
    if !seen.insert(mapped) {
        return false;
    }
    let contains = match types.get_ref(mapped) {
        TypeKind::Struct { .. } | TypeKind::Tuple { .. } => {
            aggregate_fields_with_offsets(types, mapped)
                .into_iter()
                .any(|field| type_contains_owner_token_mapped(types, field.ty, mapping, seen))
        }
        TypeKind::Enum { variants, .. } => variants.iter().any(|variant| {
            variant.payload.is_some_and(|payload| {
                type_contains_owner_token_mapped(types, payload, mapping, seen)
            })
        }),
        TypeKind::Apply { base, args } => {
            apply_contains_owner_token(types, *base, args, mapping, seen)
        }
        TypeKind::Var(var) => var
            .binding
            .is_some_and(|binding| type_contains_owner_token_mapped(types, binding, mapping, seen)),
        TypeKind::Box(inner) | TypeKind::Reference(inner, _) => {
            type_contains_owner_token_mapped(types, *inner, mapping, seen)
        }
        TypeKind::Unit
        | TypeKind::Never
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Named(_)
        | TypeKind::Function { .. } => false,
    };
    seen.remove(&mapped);
    contains
}

fn apply_contains_owner_token(
    types: &TypeCtx,
    base: TypeId,
    args: &[TypeId],
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
) -> bool {
    let base = types.resolve_named_type_id(base);
    match types.get_ref(base) {
        TypeKind::Struct { type_params, .. } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            aggregate_fields_with_offsets(types, base)
                .into_iter()
                .any(|field| {
                    type_contains_owner_token_mapped(types, field.ty, &nested_mapping, seen)
                })
        }
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            variants.iter().any(|variant| {
                variant.payload.is_some_and(|payload| {
                    type_contains_owner_token_mapped(types, payload, &nested_mapping, seen)
                })
            })
        }
        TypeKind::Tuple { .. } => aggregate_fields_with_offsets(types, base)
            .into_iter()
            .any(|field| type_contains_owner_token_mapped(types, field.ty, mapping, seen)),
        _ => type_contains_owner_token_mapped(types, base, mapping, seen),
    }
}
