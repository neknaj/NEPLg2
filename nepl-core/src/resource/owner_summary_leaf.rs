use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, extend_type_mapping, mapped_type_id};
use crate::resource_primitives::{type_is_owner_token, type_is_raw_pointer};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::{Place, PlaceProjection};
use super::owner_summary_owner_token_leaf::owner_token_raw_i32_leaf_projections;
use super::owner_summary_variant_leaf::enum_owner_leaf_projections;
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

#[derive(Clone)]
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
    if type_is_owner_token(types, mapped) {
        return owner_token_raw_i32_leaf_projections(types, mapped);
    }
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

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;

    use crate::resource_primitives::{CompilerMemoryFieldSpec, OWNER_TOKEN_TYPE_NAME};
    use crate::source_map::CompilerMemoryType;
    use crate::types::{TypeCtx, TypeKind};

    use super::super::model::{Place, PlaceProjection};
    use super::owner_leaf_places;

    #[test]
    fn owner_leaf_places_seed_owner_token_raw_field() {
        let mut types = TypeCtx::new();
        let type_param = types.fresh_var(Some("T".to_string()));
        let i32_ty = types.i32();
        let token_ty = types.register_named(
            OWNER_TOKEN_TYPE_NAME.to_string(),
            TypeKind::Struct {
                name: OWNER_TOKEN_TYPE_NAME.to_string(),
                type_params: vec![type_param],
                fields: vec![i32_ty, i32_ty],
                field_names: vec![
                    CompilerMemoryFieldSpec::RawI32.name().to_string(),
                    CompilerMemoryFieldSpec::SizeI32.name().to_string(),
                ],
            },
        );
        types.mark_compiler_memory_type(token_ty, CompilerMemoryType::OwnerToken);
        let base = Place::local("token".to_string(), token_ty);

        let leaves = owner_leaf_places(&types, &base);

        assert_eq!(leaves.len(), 1);
        assert_eq!(leaves[0].place.ty, i32_ty);
        assert_eq!(
            leaves[0].suffix,
            vec![PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            }]
        );
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
