use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::resource_primitives::type_is_owner_token;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{Place, PlaceProjection};
use super::owner_summary_leaf::{
    push_nested_owner_leaf_projections, AggregateProjectionKind, OwnerLeafPlace,
    OwnerLeafProjection,
};
use super::owner_summary_owner_token_leaf::owner_token_raw_i32_leaf_projections;
use super::owner_summary_owner_token_type::type_contains_owner_token;
use super::place_utils::place_with_suffix;

#[derive(Clone, Copy)]
enum RawI32OwnerLeafMode {
    PlainRawI32Allowed,
    OwnerTokenOnly,
}

pub(super) fn raw_i32_owner_leaf_places_for_summary(
    types: &TypeCtx,
    base: &Place,
) -> Vec<OwnerLeafPlace> {
    let mode = if type_contains_owner_token(types, base.ty) {
        RawI32OwnerLeafMode::OwnerTokenOnly
    } else {
        RawI32OwnerLeafMode::PlainRawI32Allowed
    };
    raw_i32_owner_leaf_projections_mapped(
        types,
        base.ty,
        &BTreeMap::new(),
        &mut BTreeSet::new(),
        mode,
    )
    .into_iter()
    .map(|leaf| OwnerLeafPlace {
        place: place_with_suffix(base, &leaf.suffix, leaf.ty),
        suffix: leaf.suffix,
    })
    .collect()
}

fn raw_i32_owner_leaf_projections_mapped(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
    mode: RawI32OwnerLeafMode,
) -> Vec<OwnerLeafProjection> {
    let mapped = mapped_type_id(types, ty, mapping);
    if type_is_owner_token(types, mapped) {
        return owner_token_raw_i32_leaf_projections(types, mapped);
    }
    if matches!(types.get_ref(types.resolve_id(mapped)), TypeKind::I32) {
        return match mode {
            RawI32OwnerLeafMode::PlainRawI32Allowed => vec![OwnerLeafProjection {
                suffix: Vec::new(),
                ty: mapped,
            }],
            RawI32OwnerLeafMode::OwnerTokenOnly => Vec::new(),
        };
    }
    if !seen.insert(mapped) {
        return Vec::new();
    }
    let out = match types.get_ref(mapped) {
        TypeKind::Struct { .. } => aggregate_raw_i32_owner_leaf_projections(
            types,
            mapped,
            AggregateProjectionKind::Struct,
            mapping,
            seen,
            mode,
        ),
        TypeKind::Tuple { .. } => aggregate_raw_i32_owner_leaf_projections(
            types,
            mapped,
            AggregateProjectionKind::Tuple,
            mapping,
            seen,
            mode,
        ),
        TypeKind::Enum { variants, .. } => {
            enum_raw_i32_owner_leaf_projections(types, variants, mapping, seen, mode)
        }
        TypeKind::Apply { base, args } => {
            apply_raw_i32_owner_leaf_projections(types, mapped, *base, args, mapping, seen, mode)
        }
        TypeKind::Var(var) => var
            .binding
            .map(|binding| {
                raw_i32_owner_leaf_projections_mapped(types, binding, mapping, seen, mode)
            })
            .unwrap_or_default(),
        TypeKind::Unit
        | TypeKind::Never
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Named(_)
        | TypeKind::Box(_)
        | TypeKind::Reference(_, _)
        | TypeKind::Function { .. }
        | TypeKind::I32 => Vec::new(),
    };
    seen.remove(&mapped);
    out
}

fn aggregate_raw_i32_owner_leaf_projections(
    types: &TypeCtx,
    ty: TypeId,
    kind: AggregateProjectionKind,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
    mode: RawI32OwnerLeafMode,
) -> Vec<OwnerLeafProjection> {
    let mut out = Vec::new();
    for (index, field) in aggregate_fields_with_offsets(types, ty)
        .into_iter()
        .enumerate()
    {
        let projection = match kind {
            AggregateProjectionKind::Struct => PlaceProjection::Field {
                index,
                offset_bytes: field.offset,
            },
            AggregateProjectionKind::Tuple => PlaceProjection::TupleField {
                index,
                offset_bytes: field.offset,
            },
        };
        let children = raw_i32_owner_leaf_projections_mapped(types, field.ty, mapping, seen, mode);
        push_nested_owner_leaf_projections(&mut out, projection, children);
    }
    out
}

fn enum_raw_i32_owner_leaf_projections(
    types: &TypeCtx,
    variants: &[crate::types::EnumVariantInfo],
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
    mode: RawI32OwnerLeafMode,
) -> Vec<OwnerLeafProjection> {
    let mut out = Vec::new();
    for variant in variants {
        let Some(payload) = variant.payload else {
            continue;
        };
        let payload_ty = mapped_type_id(types, payload, mapping);
        let projection = PlaceProjection::EnumPayload {
            variant: variant.name.clone(),
        };
        let children =
            raw_i32_owner_leaf_projections_mapped(types, payload_ty, mapping, seen, mode);
        push_nested_owner_leaf_projections(&mut out, projection, children);
    }
    out
}

fn apply_raw_i32_owner_leaf_projections(
    types: &TypeCtx,
    apply_ty: TypeId,
    base: TypeId,
    args: &[TypeId],
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
    mode: RawI32OwnerLeafMode,
) -> Vec<OwnerLeafProjection> {
    let base = types.resolve_named_type_id(base);
    match types.get_ref(base) {
        TypeKind::Struct { type_params, .. } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            aggregate_raw_i32_owner_leaf_projections(
                types,
                apply_ty,
                AggregateProjectionKind::Struct,
                &nested_mapping,
                seen,
                mode,
            )
        }
        TypeKind::Tuple { .. } => aggregate_raw_i32_owner_leaf_projections(
            types,
            apply_ty,
            AggregateProjectionKind::Tuple,
            mapping,
            seen,
            mode,
        ),
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            enum_raw_i32_owner_leaf_projections(types, variants, &nested_mapping, seen, mode)
        }
        _ => raw_i32_owner_leaf_projections_mapped(types, base, mapping, seen, mode),
    }
}
