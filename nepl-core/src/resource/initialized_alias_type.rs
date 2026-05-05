extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{Place, PlaceProjection};

pub(super) fn projected_place_with_concrete_type(
    types: &TypeCtx,
    base: &Place,
    projection: &[PlaceProjection],
    fallback_ty: TypeId,
) -> Place {
    let mut out = base.clone();
    let mut current_ty = base.ty;
    for item in projection {
        current_ty = projection_result_type(types, current_ty, item).unwrap_or(fallback_ty);
        out.projections.push(item.clone());
        out.ty = current_ty;
    }
    if projection.is_empty() {
        out.ty = base.ty;
    }
    out
}

pub(super) fn type_preserves_raw_address_alias(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { name, .. } => name == "MemPtr" || name == "RegionToken",
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(
                types.get_ref(base),
                TypeKind::Struct { name, .. } if name == "MemPtr" || name == "RegionToken"
            )
        }
        _ => false,
    }
}

pub(super) fn type_can_seed_raw_address_alias(types: &TypeCtx, ty: TypeId) -> bool {
    let mut visiting = BTreeSet::new();
    type_can_seed_raw_address_alias_inner(types, ty, &BTreeMap::new(), &mut visiting)
}

fn type_can_seed_raw_address_alias_inner(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    visiting: &mut BTreeSet<TypeId>,
) -> bool {
    let resolved = mapped_type_id(types, ty, mapping);
    if !visiting.insert(resolved) {
        return false;
    }
    let out = match types.get_ref(resolved) {
        TypeKind::I32 | TypeKind::Str => true,
        TypeKind::Struct { name, fields, .. } => {
            name == "MemPtr"
                || name == "RegionToken"
                || fields.iter().any(|field| {
                    type_can_seed_raw_address_alias_inner(types, *field, mapping, visiting)
                })
        }
        TypeKind::Tuple { items } => items
            .iter()
            .any(|item| type_can_seed_raw_address_alias_inner(types, *item, mapping, visiting)),
        TypeKind::Enum { variants, .. } => variants.iter().any(|variant| {
            variant.payload.is_some_and(|payload| {
                type_can_seed_raw_address_alias_inner(types, payload, mapping, visiting)
            })
        }),
        TypeKind::Apply { base, args } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct {
                    name,
                    type_params,
                    fields,
                    ..
                } => {
                    name == "MemPtr" || name == "RegionToken" || {
                        let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
                        fields.iter().any(|field| {
                            type_can_seed_raw_address_alias_inner(
                                types,
                                *field,
                                &nested_mapping,
                                visiting,
                            )
                        })
                    }
                }
                TypeKind::Enum {
                    type_params,
                    variants,
                    ..
                } => {
                    let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
                    variants.iter().any(|variant| {
                        variant.payload.is_some_and(|payload| {
                            type_can_seed_raw_address_alias_inner(
                                types,
                                payload,
                                &nested_mapping,
                                visiting,
                            )
                        })
                    })
                }
                TypeKind::Tuple { items } => items.iter().any(|item| {
                    type_can_seed_raw_address_alias_inner(types, *item, mapping, visiting)
                }),
                _ => false,
            }
        }
        TypeKind::Reference(inner, _) | TypeKind::Box(inner) => {
            type_can_seed_raw_address_alias_inner(types, *inner, mapping, visiting)
        }
        TypeKind::Var(var) => var.binding.is_none_or(|binding| {
            type_can_seed_raw_address_alias_inner(types, binding, mapping, visiting)
        }),
        TypeKind::Named(_) => {
            let named = types.resolve_named_type_id(resolved);
            named != resolved
                && type_can_seed_raw_address_alias_inner(types, named, mapping, visiting)
        }
        _ => false,
    };
    visiting.remove(&resolved);
    out
}

fn projection_result_type(
    types: &TypeCtx,
    base_ty: TypeId,
    projection: &PlaceProjection,
) -> Option<TypeId> {
    match projection {
        PlaceProjection::Field { index, .. } | PlaceProjection::TupleField { index, .. } => {
            aggregate_fields_with_offsets(types, base_ty)
                .get(*index)
                .map(|field| field.ty)
        }
        PlaceProjection::EnumPayload { variant } => enum_payload_type(types, base_ty, variant),
        PlaceProjection::Deref => reference_inner_type(types, base_ty),
        PlaceProjection::StorageOffset(_) => None,
    }
}

fn reference_inner_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_id(ty);
    match types.get_ref(resolved) {
        TypeKind::Reference(inner, _) => Some(*inner),
        _ => None,
    }
}

fn enum_payload_type(types: &TypeCtx, enum_ty: TypeId, variant_name: &str) -> Option<TypeId> {
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
            let mapping = type_arg_mapping(types, type_params, args);
            variants
                .iter()
                .find(|variant| variant_name_matches(&variant.name, variant_name))
                .and_then(|variant| variant.payload)
                .map(|payload| mapped_existing_type_id(types, payload, &mapping))
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

fn mapped_existing_type_id(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
) -> TypeId {
    let resolved = types.resolve_id(ty);
    mapping.get(&resolved).copied().unwrap_or(resolved)
}

fn type_arg_mapping(
    types: &TypeCtx,
    type_params: &[TypeId],
    args: &[TypeId],
) -> BTreeMap<TypeId, TypeId> {
    type_params
        .iter()
        .copied()
        .zip(args.iter().copied())
        .map(|(param, arg)| (types.resolve_id(param), types.resolve_id(arg)))
        .collect()
}
