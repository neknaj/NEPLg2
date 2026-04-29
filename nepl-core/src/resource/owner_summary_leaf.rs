use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{Place, PlaceProjection};
use super::place_utils::place_with_suffix;

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

struct OwnerLeafProjection {
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
}

fn owner_leaf_projections(types: &TypeCtx, ty: TypeId) -> Vec<OwnerLeafProjection> {
    owner_leaf_projections_mapped(types, ty, &BTreeMap::new(), &mut BTreeSet::new())
}

fn owner_leaf_projections_mapped(
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
        | TypeKind::Str
        | TypeKind::Named(_)
        | TypeKind::Function { .. }
        | TypeKind::Box(_) => vec![OwnerLeafProjection {
            suffix: Vec::new(),
            ty: mapped,
        }],
    };
    seen.remove(&mapped);
    out
}

#[derive(Clone, Copy)]
enum AggregateProjectionKind {
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
        push_nested_owner_leaf_projections(
            &mut out,
            projection,
            owner_leaf_projections_mapped(types, field.ty, mapping, seen),
        );
    }
    out
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

fn enum_owner_leaf_projections(
    types: &TypeCtx,
    variants: &[crate::types::EnumVariantInfo],
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
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
        push_nested_owner_leaf_projections(
            &mut out,
            projection,
            owner_leaf_projections_mapped(types, payload_ty, mapping, seen),
        );
    }
    out
}

fn push_nested_owner_leaf_projections(
    out: &mut Vec<OwnerLeafProjection>,
    projection: PlaceProjection,
    children: Vec<OwnerLeafProjection>,
) {
    for mut child in children {
        child.suffix.insert(0, projection.clone());
        out.push(child);
    }
}
