use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::resource_primitives::{type_is_raw_pointer, type_preserves_raw_address_identity};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{
    AggregateKind, Place, PlaceProjection, PlaceRoot, RawMemoryOp, ResourceMatchArm,
    ResourceMatchPattern,
};
use super::variant_name::{
    match_pattern_variant_name, normalize_variant_name, variant_names_match,
};

pub(super) fn should_track(place: &Place) -> bool {
    !matches!(place.root, PlaceRoot::Unknown)
}

pub(super) fn raw_memory_cell_place(address: &Place, ty: TypeId) -> Place {
    address.clone().with_projection(PlaceProjection::Deref, ty)
}

pub(super) fn raw_memory_unknown_offset_cell_place(address: &Place, ty: TypeId) -> Place {
    let address = address.clone().with_projection(
        PlaceProjection::StorageOffset(super::model::ResourceOffset::Unknown),
        ty,
    );
    raw_memory_cell_place(&address, ty)
}

pub(super) fn reference_target_place(reference: &Place, target_ty: TypeId) -> Place {
    reference
        .clone()
        .with_projection(PlaceProjection::Deref, target_ty)
}

pub(super) fn type_preserves_raw_address_alias(types: &TypeCtx, ty: TypeId) -> bool {
    type_preserves_raw_address_identity(types, ty)
}

pub(super) fn type_can_seed_raw_address_alias(types: &TypeCtx, ty: TypeId) -> bool {
    type_can_seed_raw_address_alias_mapped(types, ty, &BTreeMap::new(), &mut Vec::new())
}

fn type_can_seed_raw_address_alias_mapped(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut Vec<TypeId>,
) -> bool {
    let resolved = mapped_type_id(types, ty, mapping);
    if seen.contains(&resolved) {
        return false;
    }
    seen.push(resolved);
    let result = match types.get_ref(resolved) {
        TypeKind::Struct { fields, .. } => {
            type_preserves_raw_address_identity(types, resolved)
                || fields.iter().any(|field| {
                    type_can_seed_raw_address_alias_mapped(types, *field, mapping, seen)
                })
        }
        TypeKind::Enum { variants, .. } => variants.iter().any(|variant| {
            variant.payload.is_some_and(|payload| {
                type_can_seed_raw_address_alias_mapped(types, payload, mapping, seen)
            })
        }),
        TypeKind::Tuple { items } => items
            .iter()
            .any(|item| type_can_seed_raw_address_alias_mapped(types, *item, mapping, seen)),
        TypeKind::Apply { base, args } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct {
                    type_params,
                    fields,
                    ..
                } => {
                    if type_preserves_raw_address_identity(types, resolved) {
                        true
                    } else {
                        let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
                        fields.iter().any(|field| {
                            type_can_seed_raw_address_alias_mapped(
                                types,
                                *field,
                                &nested_mapping,
                                seen,
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
                            type_can_seed_raw_address_alias_mapped(
                                types,
                                payload,
                                &nested_mapping,
                                seen,
                            )
                        })
                    })
                }
                TypeKind::Tuple { items } => items.iter().any(|item| {
                    type_can_seed_raw_address_alias_mapped(types, *item, mapping, seen)
                }),
                _ => false,
            }
        }
        TypeKind::Reference(target, _) | TypeKind::Box(target) => {
            type_can_seed_raw_address_alias_mapped(types, *target, mapping, seen)
        }
        TypeKind::Named(_) => {
            let named = types.resolve_named_type_id(resolved);
            named != resolved && type_can_seed_raw_address_alias_mapped(types, named, mapping, seen)
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

pub(super) fn structural_i32_projection_preserves_raw_address(
    types: &TypeCtx,
    source: &Place,
    target: &Place,
) -> bool {
    types.resolve_id(source.ty) == types.i32()
        && types.resolve_id(target.ty) == types.i32()
        && source.projections.iter().any(|projection| {
            matches!(
                projection,
                PlaceProjection::Field { .. }
                    | PlaceProjection::TupleField { .. }
                    | PlaceProjection::EnumPayload { .. }
            )
        })
}

pub(super) fn call_uses_checked_mem_ptr_wrapper(types: &TypeCtx, args: &[Place]) -> bool {
    args.first()
        .map(|arg| type_is_raw_pointer(types, arg.ty))
        .unwrap_or(false)
}

pub(super) fn checked_mem_ptr_wrapper_arg_indices(
    types: &TypeCtx,
    operation: RawMemoryOp,
    args: &[Place],
) -> Vec<usize> {
    let indices: &[usize] = match operation {
        RawMemoryOp::Load | RawMemoryOp::Store | RawMemoryOp::FillBytes | RawMemoryOp::Fill => &[0],
        RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => &[0, 1],
        RawMemoryOp::Alloc
        | RawMemoryOp::Dealloc
        | RawMemoryOp::Realloc
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow => &[],
    };
    indices
        .iter()
        .copied()
        .filter(|index| {
            args.get(*index)
                .is_some_and(|arg| type_is_raw_pointer(types, arg.ty))
        })
        .collect()
}

pub(super) fn construct_aggregate_field_place(
    output: &Place,
    kind: &AggregateKind,
    index: usize,
    input: &Place,
) -> Place {
    let mut place = output.clone();
    match kind {
        AggregateKind::Struct { field_offsets, .. } => {
            place.projections.push(PlaceProjection::Field {
                index,
                offset_bytes: field_offsets[index],
            });
        }
        AggregateKind::Tuple { field_offsets } => {
            place.projections.push(PlaceProjection::TupleField {
                index,
                offset_bytes: field_offsets[index],
            });
        }
        AggregateKind::Enum { variant, .. } => {
            place.projections.push(PlaceProjection::EnumPayload {
                variant: normalize_variant_name(variant),
            });
            if index > 0 {
                place.projections.push(PlaceProjection::TupleField {
                    index,
                    offset_bytes: 0,
                });
            }
        }
    }
    place.ty = input.ty;
    place
}

pub(super) fn replace_place_prefix(
    place: &Place,
    prefix: &Place,
    replacement: &Place,
) -> Option<Place> {
    place_suffix_after_prefix(place, prefix).map(|suffix| {
        let ty = if suffix.is_empty() {
            replacement.ty
        } else {
            place.ty
        };
        place_with_suffix(replacement, &suffix, ty)
    })
}

pub(super) fn places_overlap(left: &Place, right: &Place) -> bool {
    place_suffix_after_prefix(left, right).is_some()
        || place_suffix_after_prefix(right, left).is_some()
}

pub(super) fn raw_address_view_candidate_bases(place: &Place) -> Vec<Place> {
    let mut out = Vec::new();
    push_unique_place(&mut out, place);
    for index in 0..place.projections.len() {
        let mut prefix = place.clone();
        prefix.projections.truncate(index);
        push_unique_place(&mut out, &prefix);
    }
    out
}

pub(super) fn place_suffix_after_prefix(
    place: &Place,
    prefix: &Place,
) -> Option<Vec<PlaceProjection>> {
    if place.root != prefix.root || place.projections.len() < prefix.projections.len() {
        return None;
    }
    if place.projections[..prefix.projections.len()] != prefix.projections[..] {
        return None;
    }
    Some(place.projections[prefix.projections.len()..].to_vec())
}

pub(super) fn place_with_suffix(base: &Place, suffix: &[PlaceProjection], ty: TypeId) -> Place {
    let mut out = base.clone();
    out.projections.extend_from_slice(suffix);
    out.ty = ty;
    out
}

pub(super) fn place_with_checked_suffix(
    types: Option<&TypeCtx>,
    base: &Place,
    suffix: &[PlaceProjection],
    fallback_ty: TypeId,
) -> Option<Place> {
    let Some(types) = types else {
        return Some(place_with_suffix(base, suffix, fallback_ty));
    };
    let mut out = base.clone();
    let mut current_ty = base.ty;
    for projection in suffix {
        current_ty = projection_result_type(types, current_ty, projection)?;
        out.projections.push(projection.clone());
        out.ty = current_ty;
    }
    Some(out)
}

pub(super) fn projected_place_with_concrete_type(
    types: &TypeCtx,
    base: &Place,
    suffix: &[PlaceProjection],
    fallback_ty: TypeId,
) -> Place {
    let mut out = base.clone();
    let mut current_ty = base.ty;
    for projection in suffix {
        current_ty = projection_result_type(types, current_ty, projection).unwrap_or(fallback_ty);
        out.projections.push(projection.clone());
        out.ty = current_ty;
    }
    if suffix.is_empty() {
        out.ty = base.ty;
    }
    out
}

pub(super) fn projection_result_type(
    types: &TypeCtx,
    base_ty: TypeId,
    projection: &PlaceProjection,
) -> Option<TypeId> {
    match projection {
        PlaceProjection::Field { index, .. } | PlaceProjection::TupleField { index, .. } => {
            aggregate_fields_with_offsets(types, base_ty)
                .get(*index)
                .map(|field| normalize_projection_type(types, field.ty))
        }
        PlaceProjection::EnumPayload { variant } => enum_payload_type(types, base_ty, variant),
        PlaceProjection::Deref => reference_target_type(types, base_ty),
        PlaceProjection::StorageOffset(_) => Some(base_ty),
    }
}

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(normalize_projection_type(types, *target)),
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
            .find(|variant| variant_names_match(&variant.name, variant_name))
            .and_then(|variant| variant.payload)
            .map(|payload| normalize_projection_type(types, payload)),
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
                .find(|variant| variant_names_match(&variant.name, variant_name))
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

fn normalize_projection_type(types: &TypeCtx, ty: TypeId) -> TypeId {
    types.resolve_named_type_id(types.resolve_id(ty))
}

pub(super) fn match_bind_payload_place(
    scrutinee: &Place,
    arm: &ResourceMatchArm,
    bind_local: &Place,
) -> Option<Place> {
    if arm.bind_is_borrow {
        return None;
    }
    let variant = match_arm_variant_payload_name(arm)?;
    Some(
        scrutinee
            .clone()
            .with_projection(PlaceProjection::EnumPayload { variant }, bind_local.ty),
    )
}

pub(super) fn match_arm_variant_payload_name(arm: &ResourceMatchArm) -> Option<String> {
    let ResourceMatchPattern::Variant(_) = &arm.pattern else {
        return None;
    };
    match_pattern_variant_name(&arm.pattern)
}

pub(super) fn push_unique_place(places: &mut Vec<Place>, place: &Place) {
    if !places.iter().any(|existing| existing == place) {
        places.push(place.clone());
    }
}

pub(super) fn push_unique_usize(values: &mut Vec<usize>, value: usize) {
    if !values.contains(&value) {
        values.push(value);
    }
}
