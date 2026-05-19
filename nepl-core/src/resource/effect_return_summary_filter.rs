extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use crate::layout::{extend_type_mapping, mapped_type_id};
use crate::resource_primitives::{type_is_owner_token, type_is_raw_pointer};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::effect_return_owner_type::raw_identity_type_is_structural_owner_carrier;
use super::model::{Place, PlaceProjection};
use super::place_utils::projection_result_type;

pub(super) fn raw_identity_return_projection_requires_summary(
    types: Option<&TypeCtx>,
    returned: &Place,
    suffix: &[PlaceProjection],
    projection_ty: TypeId,
) -> bool {
    let Some(types) = types else {
        return true;
    };
    if raw_identity_projection_has_summary_owner_carrier_protection(types, returned.ty, suffix) {
        return false;
    }
    raw_identity_type_can_propagate_public_escape(
        types,
        projection_ty,
        &BTreeMap::new(),
        &mut Vec::new(),
    )
}

fn raw_identity_projection_has_summary_owner_carrier_protection(
    types: &TypeCtx,
    root_ty: TypeId,
    suffix: &[PlaceProjection],
) -> bool {
    let mut current_ty = types.resolve_named_type_id(types.resolve_id(root_ty));
    if raw_identity_type_blocks_structural_summary(types, current_ty) {
        return true;
    }
    for projection in suffix {
        if raw_identity_type_blocks_structural_summary(types, current_ty) {
            return true;
        }
        current_ty = projection_result_type(types, current_ty, projection).unwrap_or(current_ty);
        if raw_identity_type_blocks_structural_summary(types, current_ty) {
            return true;
        }
    }
    false
}

fn raw_identity_type_blocks_structural_summary(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    resolved == types.str()
        || (!type_is_owner_token(types, resolved)
            && !raw_identity_type_is_enum_like(types, resolved)
            && raw_identity_type_is_structural_owner_carrier(types, resolved))
}

fn raw_identity_type_is_enum_like(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Enum { .. } => true,
        TypeKind::Apply { base, .. } => {
            matches!(
                types.get_ref(types.resolve_named_type_id(*base)),
                TypeKind::Enum { .. }
            )
        }
        _ => false,
    }
}

fn raw_identity_type_can_propagate_public_escape(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut Vec<TypeId>,
) -> bool {
    let resolved = mapped_type_id(types, ty, mapping);
    if type_is_raw_pointer(types, resolved) {
        return true;
    }
    if type_is_owner_token(types, resolved) {
        return true;
    }
    if raw_identity_type_blocks_structural_summary(types, resolved) {
        return false;
    }
    if seen.contains(&resolved) {
        return false;
    }
    seen.push(resolved);
    let result = match types.get_ref(resolved) {
        TypeKind::I32 => true,
        TypeKind::Struct { fields, .. } => fields.iter().any(|field| {
            raw_identity_type_can_propagate_public_escape(types, *field, mapping, seen)
        }),
        TypeKind::Enum { variants, .. } => variants.iter().any(|variant| {
            variant.payload.is_some_and(|payload| {
                raw_identity_type_can_propagate_public_escape(types, payload, mapping, seen)
            })
        }),
        TypeKind::Tuple { items } => items
            .iter()
            .any(|item| raw_identity_type_can_propagate_public_escape(types, *item, mapping, seen)),
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
                        raw_identity_type_can_propagate_public_escape(
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
                            raw_identity_type_can_propagate_public_escape(
                                types,
                                payload,
                                &nested_mapping,
                                seen,
                            )
                        })
                    })
                }
                TypeKind::Tuple { items } => items.iter().any(|item| {
                    raw_identity_type_can_propagate_public_escape(types, *item, mapping, seen)
                }),
                _ => false,
            }
        }
        TypeKind::Reference(target, _) | TypeKind::Box(target) => {
            raw_identity_type_can_propagate_public_escape(types, *target, mapping, seen)
        }
        TypeKind::Named(_) => {
            let named = types.resolve_named_type_id(resolved);
            named != resolved
                && raw_identity_type_can_propagate_public_escape(types, named, mapping, seen)
        }
        TypeKind::Var(_) => true,
        TypeKind::Unit
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Function { .. } => false,
    };
    seen.pop();
    result
}

#[cfg(test)]
#[path = "effect_return_summary_filter_tests.rs"]
mod tests;
