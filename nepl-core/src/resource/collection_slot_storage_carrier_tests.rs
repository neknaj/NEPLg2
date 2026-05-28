use alloc::string::ToString;
use alloc::vec;

use crate::source_map::CompilerMemoryType;
use crate::types::{TypeId, TypeKind};

use super::*;

fn register_empty_struct(types: &mut TypeCtx, name: &str) -> TypeId {
    types.register_named(
        name.to_string(),
        TypeKind::Struct {
            name: name.to_string(),
            type_params: vec![],
            fields: vec![],
            field_names: vec![],
        },
    )
}

fn register_region_token(types: &mut TypeCtx) -> TypeId {
    let raw_ty = types.i32();
    let value_ty = types.fresh_var(Some("T".to_string()));
    let region_token_ty = types.register_named(
        "RegionToken".to_string(),
        TypeKind::Struct {
            name: "RegionToken".to_string(),
            type_params: vec![value_ty],
            fields: vec![raw_ty, raw_ty],
            field_names: vec!["raw".to_string(), "size".to_string()],
        },
    );
    types.mark_compiler_memory_type(region_token_ty, CompilerMemoryType::OwnerToken);
    region_token_ty
}

#[test]
fn plain_non_copy_aggregate_does_not_force_collection_slot_storage() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    let storage_ty = register_empty_struct(&mut types, "PlainNonCopy");

    assert!(!type_can_carry_collection_slot_storage(&types, storage_ty));
}

#[test]
fn owner_token_with_copy_payload_does_not_carry_collection_slots() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.u8());
    let region_token = register_region_token(&mut types);
    let byte_region = types.apply(region_token, vec![types.u8()]);

    assert!(!type_can_carry_collection_slot_storage(&types, byte_region));
}

#[test]
fn owner_token_with_non_copy_payload_carries_collection_slots() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    let payload_ty = register_empty_struct(&mut types, "OwnedPayload");
    let region_token = register_region_token(&mut types);
    let owned_region = types.apply(region_token, vec![payload_ty]);

    assert!(type_can_carry_collection_slot_storage(&types, owned_region));
}

#[test]
fn copy_scalar_and_raw_pointer_do_not_carry_collection_slot_storage() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let mem_ptr_ty = register_empty_struct(&mut types, "MemPtr");
    types.mark_compiler_memory_type(mem_ptr_ty, CompilerMemoryType::RawPointer);

    assert!(!type_can_carry_collection_slot_storage(&types, types.i32()));
    assert!(!type_can_carry_collection_slot_storage(&types, mem_ptr_ty));
}
