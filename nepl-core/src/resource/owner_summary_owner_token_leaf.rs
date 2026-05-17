use alloc::vec;
use alloc::vec::Vec;

use crate::layout::{aggregate_fields_with_offsets, FieldLayout};
use crate::resource_primitives::{
    compiler_memory_type_field_index, type_is_owner_token, CompilerMemoryFieldSpec,
};
use crate::source_map::CompilerMemoryType;
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
    if !type_is_owner_token(types, ty) {
        return None;
    }
    let raw_index = compiler_memory_type_field_index(
        CompilerMemoryType::OwnerToken,
        CompilerMemoryFieldSpec::RawI32,
    )?;
    let expected_field_name = CompilerMemoryFieldSpec::RawI32.name();
    let field_names = owner_token_field_names(types, ty)?;
    field_names
        .get(raw_index)
        .is_some_and(|field_name| field_name.as_str() == expected_field_name)
        .then_some(raw_index)
}

fn owner_token_field_names(types: &TypeCtx, ty: TypeId) -> Option<&[alloc::string::String]> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { field_names, .. } => Some(field_names),
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct { field_names, .. } => Some(field_names),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
#[path = "owner_summary_owner_token_leaf_tests.rs"]
mod tests;
