use alloc::string::ToString;
use alloc::vec;

use crate::source_map::CompilerMemoryType;
use crate::types::{EnumVariantInfo, TypeCtx, TypeId, TypeKind};

use super::collection_slot_owner_carrier::type_carries_collection_slot_owner;

fn register_region_token(types: &mut TypeCtx) -> TypeId {
    let i32_ty = types.i32();
    let region_token_ty = types.register_named(
        "RegionToken".to_string(),
        TypeKind::Struct {
            name: "RegionToken".to_string(),
            type_params: vec![],
            fields: vec![i32_ty, i32_ty],
            field_names: vec!["raw".to_string(), "size".to_string()],
        },
    );
    types.mark_compiler_memory_type(region_token_ty, CompilerMemoryType::OwnerToken);
    region_token_ty
}

fn register_mem_ptr(types: &mut TypeCtx) -> TypeId {
    let i32_ty = types.i32();
    let mem_ptr_ty = types.register_named(
        "MemPtr".to_string(),
        TypeKind::Struct {
            name: "MemPtr".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["raw".to_string()],
        },
    );
    types.mark_compiler_memory_type(mem_ptr_ty, CompilerMemoryType::RawPointer);
    mem_ptr_ty
}

#[test]
fn mem_ptr_is_non_owning_for_collection_slot_transfer() {
    let mut types = TypeCtx::new();
    let mem_ptr_ty = register_mem_ptr(&mut types);

    assert!(!type_carries_collection_slot_owner(&types, mem_ptr_ty));
}

#[test]
fn reference_to_region_token_is_non_owning_for_collection_slot_transfer() {
    let mut types = TypeCtx::new();
    let region_token_ty = register_region_token(&mut types);
    let shared_region_ref_ty = types.reference(region_token_ty, false);
    let mutable_region_ref_ty = types.reference(region_token_ty, true);

    assert!(!type_carries_collection_slot_owner(
        &types,
        shared_region_ref_ty
    ));
    assert!(!type_carries_collection_slot_owner(
        &types,
        mutable_region_ref_ty
    ));
}

#[test]
fn structural_storage_carrier_owns_collection_slots() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let region_token_ty = register_region_token(&mut types);
    let storage_ty = types.register_named(
        "VecStorage".to_string(),
        TypeKind::Enum {
            name: "VecStorage".to_string(),
            type_params: vec![],
            variants: vec![
                EnumVariantInfo {
                    name: "Empty".to_string(),
                    payload: None,
                },
                EnumVariantInfo {
                    name: "Owned".to_string(),
                    payload: Some(region_token_ty),
                },
            ],
        },
    );
    let vec_ty = types.register_named(
        "Vec".to_string(),
        TypeKind::Struct {
            name: "Vec".to_string(),
            type_params: vec![],
            fields: vec![i32_ty, i32_ty, storage_ty],
            field_names: vec!["len".to_string(), "cap".to_string(), "storage".to_string()],
        },
    );

    assert!(type_carries_collection_slot_owner(&types, region_token_ty));
    assert!(type_carries_collection_slot_owner(&types, storage_ty));
    assert!(type_carries_collection_slot_owner(&types, vec_ty));
}
