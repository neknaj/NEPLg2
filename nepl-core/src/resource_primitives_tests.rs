use super::*;
use alloc::string::ToString;
use alloc::vec;

fn register_memory_struct(types: &mut TypeCtx, name: &str, field_names: &[&str]) -> TypeId {
    let type_param = types.fresh_var(Some("T".to_string()));
    let i32_ty = types.i32();
    let fields = field_names.iter().map(|_| i32_ty).collect();
    let field_names = field_names.iter().map(|name| (*name).to_string()).collect();
    let ty = types.register_named(
        name.to_string(),
        TypeKind::Struct {
            doc: None,
            name: name.to_string(),
            type_params: vec![type_param],
            fields,
            field_names,
        },
    );
    if let Some(memory_type) = compiler_memory_type_from_constructor_name(name) {
        types.mark_compiler_memory_type(ty, memory_type);
    }
    ty
}

#[test]
fn compiler_memory_type_classifies_base_and_applied_types() {
    let mut types = TypeCtx::new();
    let mem_ptr = register_memory_struct(&mut types, RAW_POINTER_TYPE_NAME, &["raw"]);
    let region = register_memory_struct(&mut types, OWNER_TOKEN_TYPE_NAME, &["raw", "size"]);
    let u8_ty = types.u8();
    let applied_mem_ptr = types.apply(mem_ptr, vec![u8_ty]);
    let applied_region = types.apply(region, vec![u8_ty]);

    assert_eq!(
        compiler_memory_type_of_type(&types, mem_ptr),
        Some(CompilerMemoryType::RawPointer)
    );
    assert_eq!(
        compiler_memory_type_of_type(&types, applied_mem_ptr),
        Some(CompilerMemoryType::RawPointer)
    );
    assert_eq!(
        compiler_memory_type_of_type(&types, region),
        Some(CompilerMemoryType::OwnerToken)
    );
    assert_eq!(
        compiler_memory_type_of_type(&types, applied_region),
        Some(CompilerMemoryType::OwnerToken)
    );
    assert!(!type_is_raw_pointer(&types, applied_region));
    assert!(!type_is_owner_token(&types, applied_mem_ptr));
}

#[test]
fn same_name_structs_are_not_memory_types_without_proven_identity() {
    let mut types = TypeCtx::new();
    let type_param = types.fresh_var(Some("T".to_string()));
    let i32_ty = types.i32();
    let fake_mem_ptr = types.register_named(
        RAW_POINTER_TYPE_NAME.to_string(),
        TypeKind::Struct {
            doc: None,
            name: RAW_POINTER_TYPE_NAME.to_string(),
            type_params: vec![type_param],
            fields: vec![i32_ty],
            field_names: vec!["raw".to_string()],
        },
    );
    let applied_fake = types.apply(fake_mem_ptr, vec![types.u8()]);

    assert_eq!(compiler_memory_type_of_type(&types, fake_mem_ptr), None);
    assert_eq!(compiler_memory_type_of_type(&types, applied_fake), None);
    assert!(!type_is_raw_pointer(&types, fake_mem_ptr));
}

#[test]
fn memory_helper_primitive_classifies_suffixed_symbols() {
    assert_eq!(
        MemoryHelperPrimitive::from_symbol("core/mem::mem_ptr_addr__u8"),
        Some(MemoryHelperPrimitive::MemPtrAddr)
    );
    assert_eq!(
        MemoryHelperPrimitive::from_symbol("region_ptr_at__i32"),
        Some(MemoryHelperPrimitive::RegionPtrAt)
    );
    assert_eq!(MemoryHelperPrimitive::from_symbol("alloc_region"), None);
}

#[test]
fn memory_helper_primitive_separates_address_view_boundary_evidence() {
    assert!(MemoryHelperPrimitive::MemPtrAddr.is_raw_address_view_boundary_evidence());
    assert!(MemoryHelperPrimitive::MemPtrWrap.is_raw_address_view_boundary_evidence());
    assert!(MemoryHelperPrimitive::MemPtrAdd.is_raw_address_view_boundary_evidence());
    assert!(MemoryHelperPrimitive::RegionPtr.is_raw_address_view_boundary_evidence());
    assert!(MemoryHelperPrimitive::RegionPtrAt.is_raw_address_view_boundary_evidence());
    assert!(MemoryHelperPrimitive::RegionTokenRawRef.is_raw_address_view_boundary_evidence());
    assert!(MemoryHelperPrimitive::StrAddr.is_raw_address_view_boundary_evidence());
    assert!(MemoryHelperPrimitive::StrFromAddrUnchecked.is_raw_address_view_boundary_evidence());
    assert!(!MemoryHelperPrimitive::RegionNew.is_raw_address_view_boundary_evidence());
}
