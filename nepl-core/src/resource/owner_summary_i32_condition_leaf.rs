use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{Place, PlaceProjection};
use super::owner_summary_leaf::{
    push_nested_owner_leaf_projections, AggregateProjectionKind, OwnerLeafPlace,
    OwnerLeafProjection,
};
use super::place_utils::place_with_suffix;

#[derive(Default)]
pub(super) struct I32LeafProjectionCache {
    entries: Vec<(TypeId, Vec<OwnerLeafProjection>)>,
}

impl I32LeafProjectionCache {
    pub(super) fn leaf_places_for_conditions(
        &mut self,
        types: &TypeCtx,
        base: &Place,
    ) -> Vec<OwnerLeafPlace> {
        self.projections(types, base.ty)
            .into_iter()
            .map(|leaf| OwnerLeafPlace {
                place: place_with_suffix(base, &leaf.suffix, leaf.ty),
                suffix: leaf.suffix,
            })
            .collect()
    }

    pub(super) fn leaf_places_for_conditions_with_projection_filter<F>(
        &mut self,
        types: &TypeCtx,
        base: &Place,
        target_suffix: &[PlaceProjection],
        projection_is_possible: &F,
    ) -> Vec<OwnerLeafPlace>
    where
        F: Fn(&[PlaceProjection]) -> bool,
    {
        let mut projections = Vec::new();
        let mut leaf_suffix = Vec::new();
        let mut return_projection = target_suffix.to_vec();
        push_i32_leaf_projections_mapped_filtered(
            types,
            base.ty,
            &BTreeMap::new(),
            &mut BTreeSet::new(),
            &mut leaf_suffix,
            &mut return_projection,
            projection_is_possible,
            &mut projections,
        );
        projections
            .into_iter()
            .map(|leaf| OwnerLeafPlace {
                place: place_with_suffix(base, &leaf.suffix, leaf.ty),
                suffix: leaf.suffix,
            })
            .collect()
    }

    fn projections(&mut self, types: &TypeCtx, ty: TypeId) -> Vec<OwnerLeafProjection> {
        if let Some((_, projections)) = self.entries.iter().find(|(entry_ty, _)| *entry_ty == ty) {
            return projections.clone();
        }
        let projections =
            i32_leaf_projections_mapped(types, ty, &BTreeMap::new(), &mut BTreeSet::new());
        self.entries.push((ty, projections.clone()));
        projections
    }
}

pub(super) fn i32_leaf_places_for_conditions(types: &TypeCtx, base: &Place) -> Vec<OwnerLeafPlace> {
    I32LeafProjectionCache::default().leaf_places_for_conditions(types, base)
}

fn i32_leaf_projections_mapped(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
) -> Vec<OwnerLeafProjection> {
    let mapped = mapped_type_id(types, ty, mapping);
    if matches!(types.get_ref(types.resolve_id(mapped)), TypeKind::I32) {
        return vec![OwnerLeafProjection {
            suffix: Vec::new(),
            ty: mapped,
        }];
    }
    if !seen.insert(mapped) {
        return Vec::new();
    }
    let out = match types.get_ref(mapped) {
        TypeKind::Struct { .. } => aggregate_i32_leaf_projections(
            types,
            mapped,
            AggregateProjectionKind::Struct,
            mapping,
            seen,
        ),
        TypeKind::Tuple { .. } => aggregate_i32_leaf_projections(
            types,
            mapped,
            AggregateProjectionKind::Tuple,
            mapping,
            seen,
        ),
        TypeKind::Enum { variants, .. } => {
            enum_i32_leaf_projections(types, variants, mapping, seen)
        }
        TypeKind::Apply { base, args } => {
            apply_i32_leaf_projections(types, mapped, *base, args, mapping, seen)
        }
        TypeKind::Var(var) => var
            .binding
            .map(|binding| i32_leaf_projections_mapped(types, binding, mapping, seen))
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

fn aggregate_i32_leaf_projections(
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
        let children = i32_leaf_projections_mapped(types, field.ty, mapping, seen);
        push_nested_owner_leaf_projections(&mut out, projection, children);
    }
    out
}

fn enum_i32_leaf_projections(
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
        let children = i32_leaf_projections_mapped(types, payload_ty, mapping, seen);
        push_nested_owner_leaf_projections(&mut out, projection, children);
    }
    out
}

fn apply_i32_leaf_projections(
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
            aggregate_i32_leaf_projections(
                types,
                apply_ty,
                AggregateProjectionKind::Struct,
                &nested_mapping,
                seen,
            )
        }
        TypeKind::Tuple { .. } => aggregate_i32_leaf_projections(
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
            enum_i32_leaf_projections(types, variants, &nested_mapping, seen)
        }
        _ => i32_leaf_projections_mapped(types, base, mapping, seen),
    }
}

fn push_i32_leaf_projections_mapped_filtered<F>(
    types: &TypeCtx,
    ty: TypeId,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
    leaf_suffix: &mut Vec<PlaceProjection>,
    return_projection: &mut Vec<PlaceProjection>,
    projection_is_possible: &F,
    out: &mut Vec<OwnerLeafProjection>,
) where
    F: Fn(&[PlaceProjection]) -> bool,
{
    let mapped = mapped_type_id(types, ty, mapping);
    if matches!(types.get_ref(types.resolve_id(mapped)), TypeKind::I32) {
        out.push(OwnerLeafProjection {
            suffix: leaf_suffix.clone(),
            ty: mapped,
        });
        return;
    }
    if !seen.insert(mapped) {
        return;
    }
    match types.get_ref(mapped) {
        TypeKind::Struct { .. } => push_aggregate_i32_leaf_projections_filtered(
            types,
            mapped,
            AggregateProjectionKind::Struct,
            mapping,
            seen,
            leaf_suffix,
            return_projection,
            projection_is_possible,
            out,
        ),
        TypeKind::Tuple { .. } => push_aggregate_i32_leaf_projections_filtered(
            types,
            mapped,
            AggregateProjectionKind::Tuple,
            mapping,
            seen,
            leaf_suffix,
            return_projection,
            projection_is_possible,
            out,
        ),
        TypeKind::Enum { variants, .. } => push_enum_i32_leaf_projections_filtered(
            types,
            variants,
            mapping,
            seen,
            leaf_suffix,
            return_projection,
            projection_is_possible,
            out,
        ),
        TypeKind::Apply { base, args } => push_apply_i32_leaf_projections_filtered(
            types,
            mapped,
            *base,
            args,
            mapping,
            seen,
            leaf_suffix,
            return_projection,
            projection_is_possible,
            out,
        ),
        TypeKind::Var(var) => {
            if let Some(binding) = var.binding {
                push_i32_leaf_projections_mapped_filtered(
                    types,
                    binding,
                    mapping,
                    seen,
                    leaf_suffix,
                    return_projection,
                    projection_is_possible,
                    out,
                );
            }
        }
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
        | TypeKind::I32 => {}
    }
    seen.remove(&mapped);
}

fn push_aggregate_i32_leaf_projections_filtered<F>(
    types: &TypeCtx,
    ty: TypeId,
    kind: AggregateProjectionKind,
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
    leaf_suffix: &mut Vec<PlaceProjection>,
    return_projection: &mut Vec<PlaceProjection>,
    projection_is_possible: &F,
    out: &mut Vec<OwnerLeafProjection>,
) where
    F: Fn(&[PlaceProjection]) -> bool,
{
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
        leaf_suffix.push(projection.clone());
        return_projection.push(projection);
        if projection_is_possible(return_projection) {
            push_i32_leaf_projections_mapped_filtered(
                types,
                field.ty,
                mapping,
                seen,
                leaf_suffix,
                return_projection,
                projection_is_possible,
                out,
            );
        }
        return_projection.pop();
        leaf_suffix.pop();
    }
}

fn push_enum_i32_leaf_projections_filtered<F>(
    types: &TypeCtx,
    variants: &[crate::types::EnumVariantInfo],
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
    leaf_suffix: &mut Vec<PlaceProjection>,
    return_projection: &mut Vec<PlaceProjection>,
    projection_is_possible: &F,
    out: &mut Vec<OwnerLeafProjection>,
) where
    F: Fn(&[PlaceProjection]) -> bool,
{
    for variant in variants {
        let Some(payload) = variant.payload else {
            continue;
        };
        let payload_ty = mapped_type_id(types, payload, mapping);
        let projection = PlaceProjection::EnumPayload {
            variant: variant.name.clone(),
        };
        leaf_suffix.push(projection.clone());
        return_projection.push(projection);
        if projection_is_possible(return_projection) {
            push_i32_leaf_projections_mapped_filtered(
                types,
                payload_ty,
                mapping,
                seen,
                leaf_suffix,
                return_projection,
                projection_is_possible,
                out,
            );
        }
        return_projection.pop();
        leaf_suffix.pop();
    }
}

fn push_apply_i32_leaf_projections_filtered<F>(
    types: &TypeCtx,
    apply_ty: TypeId,
    base: TypeId,
    args: &[TypeId],
    mapping: &BTreeMap<TypeId, TypeId>,
    seen: &mut BTreeSet<TypeId>,
    leaf_suffix: &mut Vec<PlaceProjection>,
    return_projection: &mut Vec<PlaceProjection>,
    projection_is_possible: &F,
    out: &mut Vec<OwnerLeafProjection>,
) where
    F: Fn(&[PlaceProjection]) -> bool,
{
    let base = types.resolve_named_type_id(base);
    match types.get_ref(base) {
        TypeKind::Struct { type_params, .. } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            push_aggregate_i32_leaf_projections_filtered(
                types,
                apply_ty,
                AggregateProjectionKind::Struct,
                &nested_mapping,
                seen,
                leaf_suffix,
                return_projection,
                projection_is_possible,
                out,
            );
        }
        TypeKind::Tuple { .. } => push_aggregate_i32_leaf_projections_filtered(
            types,
            apply_ty,
            AggregateProjectionKind::Tuple,
            mapping,
            seen,
            leaf_suffix,
            return_projection,
            projection_is_possible,
            out,
        ),
        TypeKind::Enum {
            type_params,
            variants,
            ..
        } => {
            let nested_mapping = extend_type_mapping(types, mapping, type_params, args);
            push_enum_i32_leaf_projections_filtered(
                types,
                variants,
                &nested_mapping,
                seen,
                leaf_suffix,
                return_projection,
                projection_is_possible,
                out,
            );
        }
        _ => push_i32_leaf_projections_mapped_filtered(
            types,
            base,
            mapping,
            seen,
            leaf_suffix,
            return_projection,
            projection_is_possible,
            out,
        ),
    }
}
