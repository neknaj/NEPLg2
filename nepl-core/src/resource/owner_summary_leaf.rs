use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::resource_primitives::type_is_raw_pointer;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{Place, PlaceProjection, ResourceFunction};
use super::owner_summary_i32_leaf::raw_i32_owner_leaf_places;
use super::owner_summary_owner_token_type::type_contains_owner_token;
use super::owner_summary_raw_consumption::{
    function_consumes_raw_owner_from, function_returns_raw_owner_from,
};
use super::owner_summary_variant_leaf::enum_owner_leaf_projections;
use super::place_utils::place_with_suffix;
use super::summary::OwnerReturnSummaryIndex;

pub(super) struct OwnerLeafPlace {
    pub(super) place: Place,
    pub(super) suffix: Vec<PlaceProjection>,
}

pub(super) fn owner_leaf_places(types: &TypeCtx, base: &Place) -> Vec<OwnerLeafPlace> {
    owner_leaf_projections(types, base.ty)
        .into_iter()
        .map(|leaf| OwnerLeafPlace {
            place: place_with_suffix(base, &leaf.suffix, leaf.ty),
            suffix: leaf.suffix,
        })
        .collect()
}

pub(super) fn owner_seed_leaf_places(
    types: &TypeCtx,
    function: &ResourceFunction,
    summaries: &OwnerReturnSummaryIndex<'_>,
    _parameter_index: usize,
    base: &Place,
) -> Vec<OwnerLeafPlace> {
    let mut leaves = owner_leaf_places(types, base);
    for leaf in raw_i32_owner_leaf_places(types, base) {
        if raw_i32_leaf_is_copy_metadata(types, base, &leaf) {
            continue;
        }
        let consumes_raw_owner = function_consumes_raw_owner_from(function, &leaf.place, summaries);
        let returns_aggregate_raw_owner = !leaf.suffix.is_empty()
            && function_returns_raw_owner_from(function, &leaf.place, summaries);
        if (consumes_raw_owner || returns_aggregate_raw_owner)
            && !leaves
                .iter()
                .any(|existing| existing.place == leaf.place && existing.suffix == leaf.suffix)
        {
            leaves.push(leaf);
        }
    }
    leaves
}

fn raw_i32_leaf_is_copy_metadata(types: &TypeCtx, base: &Place, leaf: &OwnerLeafPlace) -> bool {
    !leaf.suffix.is_empty()
        && types.is_copy(base.ty)
        && !type_is_raw_pointer(types, base.ty)
        && !type_contains_owner_token(types, base.ty)
}

pub(super) struct OwnerLeafProjection {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
}

fn owner_leaf_projections(types: &TypeCtx, ty: TypeId) -> Vec<OwnerLeafProjection> {
    owner_leaf_projections_mapped(types, ty, &BTreeMap::new(), &mut BTreeSet::new())
}

pub(super) fn owner_leaf_projections_mapped(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
) -> Vec<OwnerLeafProjection> {
    let mapped = mapped_type_id(types, ty, mapping);
    if !seen.insert(mapped) {
        return vec![OwnerLeafProjection {
            suffix: Vec::new(),
            ty: mapped,
        }];
    }
    let out = match types.get_ref(mapped) {
        TypeKind::Unit | TypeKind::Never | TypeKind::Reference(_, _) => Vec::new(),
        TypeKind::Struct { .. } if type_is_raw_pointer(types, mapped) => {
            mem_ptr_owner_leaf_projections(types, mapped, AggregateProjectionKind::Struct)
        }
        TypeKind::Struct { .. } => aggregate_owner_leaf_projections(
            types,
            mapped,
            AggregateProjectionKind::Struct,
            mapping,
            seen,
        ),
        TypeKind::Tuple { .. } => aggregate_owner_leaf_projections(
            types,
            mapped,
            AggregateProjectionKind::Tuple,
            mapping,
            seen,
        ),
        TypeKind::Enum { variants, .. } => {
            enum_owner_leaf_projections(types, variants, mapping, seen)
        }
        TypeKind::Apply { base, args } => {
            apply_owner_leaf_projections(types, mapped, *base, args, mapping, seen)
        }
        TypeKind::Var(var) => var
            .binding
            .map(|binding| owner_leaf_projections_mapped(types, binding, mapping, seen))
            .unwrap_or_else(|| {
                vec![OwnerLeafProjection {
                    suffix: Vec::new(),
                    ty: mapped,
                }]
            }),
        TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Function { .. } => Vec::new(),
        TypeKind::Str | TypeKind::Named(_) | TypeKind::Box(_) => {
            vec![OwnerLeafProjection {
                suffix: Vec::new(),
                ty: mapped,
            }]
        }
    };
    seen.remove(&mapped);
    out
}

#[derive(Clone, Copy)]
pub(super) enum AggregateProjectionKind {
    Struct,
    Tuple,
}

fn aggregate_owner_leaf_projections(
    types: &TypeCtx,
    ty: TypeId,
    kind: AggregateProjectionKind,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
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
        let children = owner_leaf_projections_mapped(types, field.ty, mapping, seen);
        push_nested_owner_leaf_projections(&mut out, projection, children);
    }
    out
}

fn mem_ptr_owner_leaf_projections(
    types: &TypeCtx,
    ty: TypeId,
    kind: AggregateProjectionKind,
) -> Vec<OwnerLeafProjection> {
    aggregate_fields_with_offsets(types, ty)
        .into_iter()
        .next()
        .map(|field| {
            let projection = match kind {
                AggregateProjectionKind::Struct => PlaceProjection::Field {
                    index: 0,
                    offset_bytes: field.offset,
                },
                AggregateProjectionKind::Tuple => PlaceProjection::TupleField {
                    index: 0,
                    offset_bytes: field.offset,
                },
            };
            vec![OwnerLeafProjection {
                suffix: vec![projection],
                ty: field.ty,
            }]
        })
        .unwrap_or_default()
}

fn apply_owner_leaf_projections(
    types: &TypeCtx,
    apply_ty: TypeId,
    base: TypeId,
    args: &[TypeId],
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
) -> Vec<OwnerLeafProjection> {
    let base = types.resolve_named_type_id(base);
    match types.get_ref(base) {
        TypeKind::Struct { .. } if type_is_raw_pointer(types, apply_ty) => {
            mem_ptr_owner_leaf_projections(types, apply_ty, AggregateProjectionKind::Struct)
        }
        TypeKind::Struct { type_params, .. } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            aggregate_owner_leaf_projections(
                types,
                apply_ty,
                AggregateProjectionKind::Struct,
                &nested_mapping,
                seen,
            )
        }
        TypeKind::Tuple { .. } => aggregate_owner_leaf_projections(
            types,
            apply_ty,
            AggregateProjectionKind::Tuple,
            mapping,
            seen,
        ),
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            enum_owner_leaf_projections(types, variants, &nested_mapping, seen)
        }
        _ => vec![OwnerLeafProjection {
            suffix: Vec::new(),
            ty: mapped_type_id(types, base, mapping),
        }],
    }
}

pub(super) fn push_nested_owner_leaf_projections(
    out: &mut Vec<OwnerLeafProjection>,
    projection: PlaceProjection,
    children: Vec<OwnerLeafProjection>,
) {
    for mut child in children {
        child.suffix.insert(0, projection.clone());
        out.push(child);
    }
}
