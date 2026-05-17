use alloc::string::ToString;
use alloc::vec;

use crate::resource_primitives::{
    compiler_memory_type_field_index, compiler_memory_type_field_offset_bytes,
    CompilerMemoryFieldSpec, OWNER_TOKEN_TYPE_NAME, RAW_POINTER_TYPE_NAME,
};
use crate::source_map::CompilerMemoryType;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::super::model::{Place, PlaceProjection};
use super::{
    compiler_memory_field_place, mem_ptr_raw_field_place, region_token_raw_field_place,
    region_token_size_field_for_raw_owner,
};

fn memory_struct(
    types: &mut TypeCtx,
    name: &str,
    field_specs: &[CompilerMemoryFieldSpec],
    memory_type: Option<CompilerMemoryType>,
) -> TypeId {
    let type_param = types.fresh_var(Some("T".to_string()));
    let i32_ty = types.i32();
    let ty = types.register_named(
        name.to_string(),
        TypeKind::Struct {
            doc: None,
            name: name.to_string(),
            type_params: vec![type_param],
            fields: field_specs.iter().map(|_| i32_ty).collect(),
            field_names: field_specs
                .iter()
                .map(|field| field.name().to_string())
                .collect(),
        },
    );
    if let Some(memory_type) = memory_type {
        types.mark_compiler_memory_type(ty, memory_type);
    }
    ty
}

#[test]
fn compiler_memory_field_place_requires_registered_memory_identity() {
    let mut types = TypeCtx::new();
    let mem_ptr = memory_struct(
        &mut types,
        RAW_POINTER_TYPE_NAME,
        &[CompilerMemoryFieldSpec::RawI32],
        Some(CompilerMemoryType::RawPointer),
    );
    let fake = memory_struct(
        &mut types,
        "FakeMemPtr",
        &[CompilerMemoryFieldSpec::RawI32],
        None,
    );

    let raw = mem_ptr_raw_field_place(
        &types,
        &Place::local("ptr".to_string(), mem_ptr),
        types.i32(),
    );
    assert_eq!(
        raw.projections,
        vec![PlaceProjection::Field {
            index: compiler_memory_type_field_index(
                CompilerMemoryType::RawPointer,
                CompilerMemoryFieldSpec::RawI32,
            )
            .unwrap(),
            offset_bytes: compiler_memory_type_field_offset_bytes(
                CompilerMemoryType::RawPointer,
                CompilerMemoryFieldSpec::RawI32,
            )
            .unwrap(),
        }]
    );
    assert!(compiler_memory_field_place(
        &types,
        &Place::local("fake".to_string(), fake),
        CompilerMemoryType::RawPointer,
        CompilerMemoryFieldSpec::RawI32,
        types.i32(),
    )
    .is_none());
}

#[test]
fn region_token_size_sibling_uses_shared_field_spec() {
    let mut types = TypeCtx::new();
    let token = memory_struct(
        &mut types,
        OWNER_TOKEN_TYPE_NAME,
        &[
            CompilerMemoryFieldSpec::RawI32,
            CompilerMemoryFieldSpec::SizeI32,
        ],
        Some(CompilerMemoryType::OwnerToken),
    );
    let token = Place::local("token".to_string(), token);
    let raw = region_token_raw_field_place(&types, &token, types.i32());

    let size = region_token_size_field_for_raw_owner(&raw).expect("RegionToken size sibling");

    assert_eq!(
        size.projections,
        vec![PlaceProjection::Field {
            index: compiler_memory_type_field_index(
                CompilerMemoryType::OwnerToken,
                CompilerMemoryFieldSpec::SizeI32,
            )
            .unwrap(),
            offset_bytes: compiler_memory_type_field_offset_bytes(
                CompilerMemoryType::OwnerToken,
                CompilerMemoryFieldSpec::SizeI32,
            )
            .unwrap(),
        }]
    );
}
