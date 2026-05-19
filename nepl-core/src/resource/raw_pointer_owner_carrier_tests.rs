use alloc::string::ToString;
use alloc::vec;

use crate::source_map::CompilerMemoryType;
use crate::types::{EnumVariantInfo, TypeCtx, TypeId, TypeKind};

use super::raw_pointer_type::type_can_carry_raw_pointer_alias_summary;

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

#[test]
fn summary_carrier_excludes_owner_token_carriers() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let region_token_ty = register_region_token(&mut types);
    let builder_ty = types.register_named(
        "Builder".to_string(),
        TypeKind::Struct {
            name: "Builder".to_string(),
            type_params: vec![],
            fields: vec![region_token_ty, i32_ty, i32_ty],
            field_names: vec!["region".to_string(), "len".to_string(), "cap".to_string()],
        },
    );

    assert!(!type_can_carry_raw_pointer_alias_summary(
        &types,
        region_token_ty
    ));
    assert!(!type_can_carry_raw_pointer_alias_summary(
        &types, builder_ty
    ));
}

#[test]
fn summary_carrier_excludes_enum_backed_owner_storage_carriers() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let region_token_ty = register_region_token(&mut types);
    let storage_ty = types.register_named(
        "ByteBuilderStorage".to_string(),
        TypeKind::Enum {
            name: "ByteBuilderStorage".to_string(),
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
    let builder_ty = types.register_named(
        "ByteBuilder".to_string(),
        TypeKind::Struct {
            name: "ByteBuilder".to_string(),
            type_params: vec![],
            fields: vec![storage_ty, i32_ty, i32_ty],
            field_names: vec!["storage".to_string(), "len".to_string(), "cap".to_string()],
        },
    );

    assert!(!type_can_carry_raw_pointer_alias_summary(
        &types, storage_ty
    ));
    assert!(!type_can_carry_raw_pointer_alias_summary(
        &types, builder_ty
    ));
}
