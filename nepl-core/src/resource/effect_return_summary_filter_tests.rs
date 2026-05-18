use alloc::string::ToString;
use alloc::vec;

use crate::resource::model::{Place, ResourceId};
use crate::source_map::CompilerMemoryType;
use crate::types::{EnumVariantInfo, TypeId, TypeKind};

use super::*;

fn returned_place(ty: TypeId) -> Place {
    Place::temporary(ResourceId(0), ty)
}

#[test]
fn summary_filter_skips_owner_protected_string_returns() {
    let types = TypeCtx::new();
    let returned = returned_place(types.str());

    assert!(!raw_identity_return_projection_requires_summary(
        Some(&types),
        &returned,
        &[],
        types.str(),
    ));
}

#[test]
fn summary_filter_keeps_public_raw_identity_returns() {
    let mut types = TypeCtx::new();
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

    assert!(raw_identity_return_projection_requires_summary(
        Some(&types),
        &returned_place(i32_ty),
        &[],
        i32_ty,
    ));
    assert!(raw_identity_return_projection_requires_summary(
        Some(&types),
        &returned_place(mem_ptr_ty),
        &[],
        mem_ptr_ty,
    ));
}

#[test]
fn summary_filter_respects_reference_target_identity() {
    let mut types = TypeCtx::new();
    let str_ref_ty = types.reference(types.str(), false);
    let i32_ref_ty = types.reference(types.i32(), false);

    assert!(!raw_identity_return_projection_requires_summary(
        Some(&types),
        &returned_place(str_ref_ty),
        &[],
        str_ref_ty,
    ));
    assert!(raw_identity_return_projection_requires_summary(
        Some(&types),
        &returned_place(i32_ref_ty),
        &[],
        i32_ref_ty,
    ));
}

#[test]
fn summary_filter_keeps_direct_owner_token_internal_provenance() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let region_token_ty = types.register_named(
        "RegionToken".to_string(),
        TypeKind::Struct {
            name: "RegionToken".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["raw".to_string()],
        },
    );

    types.mark_compiler_memory_type(region_token_ty, CompilerMemoryType::OwnerToken);

    assert!(raw_identity_return_projection_requires_summary(
        Some(&types),
        &returned_place(region_token_ty),
        &[],
        region_token_ty,
    ));
}

#[test]
fn summary_filter_keeps_owner_token_payload_internal_provenance() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let str_ty = types.str();
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
    let result_ty = types.register_named(
        "RegionResult".to_string(),
        TypeKind::Enum {
            name: "RegionResult".to_string(),
            type_params: vec![],
            variants: vec![
                EnumVariantInfo {
                    name: "Ok".to_string(),
                    payload: Some(region_token_ty),
                },
                EnumVariantInfo {
                    name: "Err".to_string(),
                    payload: Some(str_ty),
                },
            ],
        },
    );

    assert!(raw_identity_return_projection_requires_summary(
        Some(&types),
        &returned_place(result_ty),
        &[],
        result_ty,
    ));
}

#[test]
fn summary_filter_hides_owner_token_inside_aggregate() {
    let mut types = TypeCtx::new();
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
    let builder_ty = types.register_named(
        "Builder".to_string(),
        TypeKind::Struct {
            name: "Builder".to_string(),
            type_params: vec![],
            fields: vec![region_token_ty, i32_ty, i32_ty],
            field_names: vec!["region".to_string(), "len".to_string(), "cap".to_string()],
        },
    );

    assert!(!raw_identity_return_projection_requires_summary(
        Some(&types),
        &returned_place(builder_ty),
        &[],
        builder_ty,
    ));
}
