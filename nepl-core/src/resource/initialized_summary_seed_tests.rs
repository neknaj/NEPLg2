use alloc::{string::ToString, vec};

use crate::source_map::CompilerMemoryType;
use crate::types::{TypeCtx, TypeKind};

use super::initialized_summary_seed::summary_input_type_may_seed_raw_address_alias;

#[test]
fn summary_seed_excludes_string_views_and_plain_aggregates() {
    let mut types = TypeCtx::new();
    let str_ty = types.str();
    let i32_ty = types.i32();
    let plain_view = types.register_named(
        "DocumentView".to_string(),
        TypeKind::Struct {
            name: "DocumentView".to_string(),
            type_params: vec![],
            fields: vec![str_ty, i32_ty],
            field_names: vec!["source".to_string(), "offset".to_string()],
        },
    );

    assert!(!summary_input_type_may_seed_raw_address_alias(
        &types, str_ty
    ));
    assert!(!summary_input_type_may_seed_raw_address_alias(
        &types, plain_view
    ));
}

#[test]
fn summary_seed_keeps_raw_i32_and_registered_memory_carriers() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let mem_ptr = types.register_named(
        "MemPtr".to_string(),
        TypeKind::Struct {
            name: "MemPtr".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["raw".to_string()],
        },
    );
    types.mark_compiler_memory_type(mem_ptr, CompilerMemoryType::RawPointer);
    let region_token = types.register_named(
        "RegionToken".to_string(),
        TypeKind::Struct {
            name: "RegionToken".to_string(),
            type_params: vec![],
            fields: vec![i32_ty, i32_ty],
            field_names: vec!["raw".to_string(), "size".to_string()],
        },
    );
    types.mark_compiler_memory_type(region_token, CompilerMemoryType::OwnerToken);
    let carrier = types.register_named(
        "PointerCarrier".to_string(),
        TypeKind::Struct {
            name: "PointerCarrier".to_string(),
            type_params: vec![],
            fields: vec![mem_ptr],
            field_names: vec!["ptr".to_string()],
        },
    );

    assert!(summary_input_type_may_seed_raw_address_alias(
        &types, i32_ty
    ));
    assert!(summary_input_type_may_seed_raw_address_alias(
        &types, mem_ptr
    ));
    assert!(summary_input_type_may_seed_raw_address_alias(
        &types,
        region_token
    ));
    assert!(summary_input_type_may_seed_raw_address_alias(
        &types, carrier
    ));
}
