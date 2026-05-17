use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::layout::{extend_type_mapping, mapped_type_id};
use crate::resource_primitives::type_is_raw_pointer;
use crate::types::{TypeCtx, TypeId, TypeKind};

pub(super) fn type_can_seed_non_owning_raw_pointer_alias(types: &TypeCtx, ty: TypeId) -> bool {
    type_can_seed_non_owning_raw_pointer_alias_mapped(types, ty, &BTreeMap::new(), &mut Vec::new())
}

fn type_can_seed_non_owning_raw_pointer_alias_mapped(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut Vec<TypeId>,
) -> bool {
    let resolved = mapped_type_id(types, ty, mapping);
    if type_is_raw_pointer(types, resolved) {
        return true;
    }
    if seen.contains(&resolved) {
        return false;
    }
    seen.push(resolved);
    let result = match types.get_ref(resolved) {
        TypeKind::Struct { fields, .. } => fields.iter().any(|field| {
            type_can_seed_non_owning_raw_pointer_alias_mapped(types, *field, mapping, seen)
        }),
        TypeKind::Enum { variants, .. } => variants.iter().any(|variant| {
            variant.payload.is_some_and(|payload| {
                type_can_seed_non_owning_raw_pointer_alias_mapped(types, payload, mapping, seen)
            })
        }),
        TypeKind::Tuple { items } => items.iter().any(|item| {
            type_can_seed_non_owning_raw_pointer_alias_mapped(types, *item, mapping, seen)
        }),
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
                        type_can_seed_non_owning_raw_pointer_alias_mapped(
                            types,
                            *field,
                            &nested_mapping,
                            seen,
                        )
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
                            type_can_seed_non_owning_raw_pointer_alias_mapped(
                                types,
                                payload,
                                &nested_mapping,
                                seen,
                            )
                        })
                    })
                }
                TypeKind::Tuple { items } => items.iter().any(|item| {
                    type_can_seed_non_owning_raw_pointer_alias_mapped(types, *item, mapping, seen)
                }),
                _ => false,
            }
        }
        TypeKind::Reference(target, _) | TypeKind::Box(target) => {
            type_can_seed_non_owning_raw_pointer_alias_mapped(types, *target, mapping, seen)
        }
        TypeKind::Named(_) => {
            let named = types.resolve_named_type_id(resolved);
            named != resolved
                && type_can_seed_non_owning_raw_pointer_alias_mapped(types, named, mapping, seen)
        }
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Function { .. }
        | TypeKind::Var(_) => false,
    };
    seen.pop();
    result
}
