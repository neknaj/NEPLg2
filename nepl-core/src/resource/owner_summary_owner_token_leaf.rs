use alloc::vec;
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, FieldLayout};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::model::PlaceProjection;
use super::owner_summary_leaf::OwnerLeafProjection;

pub(super) fn owner_token_raw_i32_leaf_projections(
    types: &TypeCtx,
    ty: TypeId,
) -> Vec<OwnerLeafProjection> {
    owner_token_raw_field(types, ty)
        .filter(|(_, field)| matches!(types.get_ref(types.resolve_id(field.ty)), TypeKind::I32))
        .map(|(index, field)| {
            vec![OwnerLeafProjection {
                suffix: vec![PlaceProjection::Field {
                    index,
                    offset_bytes: field.offset,
                }],
                ty: field.ty,
            }]
        })
        .unwrap_or_default()
}

fn owner_token_raw_field(types: &TypeCtx, ty: TypeId) -> Option<(usize, FieldLayout)> {
    let raw_index = owner_token_raw_field_index(types, ty)?;
    aggregate_fields_with_offsets(types, ty)
        .get(raw_index)
        .copied()
        .map(|field| (raw_index, field))
}

fn owner_token_raw_field_index(types: &TypeCtx, ty: TypeId) -> Option<usize> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    let field_names = match types.get_ref(resolved) {
        TypeKind::Struct { field_names, .. } => field_names,
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct { field_names, .. } => field_names,
                _ => return None,
            }
        }
        _ => return None,
    };
    field_names
        .iter()
        .position(|field_name| field_name == "raw")
}
