use alloc::string::ToString;
use alloc::vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::effect_return_escape::raw_identity_return_projection_is_escape;
use super::effect_return_protection::raw_identity_projection_has_owner_protection;
use super::model::{Place, PlaceProjection, ResourceId};

struct OwnerCarrierTypes {
    result_owner_ty: TypeId,
    owner_ty: TypeId,
    region_ty: TypeId,
    i32_ty: TypeId,
}

fn owner_carrier_types() -> (TypeCtx, OwnerCarrierTypes) {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let item_param = types.fresh_var(Some("T".to_string()));
    let result_ok_param = types.fresh_var(Some("T".to_string()));
    let result_err_param = types.fresh_var(Some("E".to_string()));
    let region_base = types.register_named(
        "RegionToken".to_string(),
        TypeKind::Struct {
            name: "RegionToken".to_string(),
            type_params: vec![item_param],
            fields: vec![i32_ty, i32_ty],
            field_names: vec!["raw".to_string(), "size".to_string()],
        },
    );
    let region_ty = types.apply(region_base, vec![i32_ty]);
    let owner_ty = types.register_named(
        "OwnerBox".to_string(),
        TypeKind::Struct {
            name: "OwnerBox".to_string(),
            type_params: vec![],
            fields: vec![i32_ty, region_ty],
            field_names: vec!["len".to_string(), "region".to_string()],
        },
    );
    let result_base = types.register_named(
        "Result".to_string(),
        TypeKind::Enum {
            name: "Result".to_string(),
            type_params: vec![result_ok_param, result_err_param],
            variants: vec![
                crate::types::EnumVariantInfo {
                    name: "Ok".to_string(),
                    payload: Some(result_ok_param),
                },
                crate::types::EnumVariantInfo {
                    name: "Err".to_string(),
                    payload: Some(result_err_param),
                },
            ],
        },
    );
    let result_owner_ty = types.apply(result_base, vec![owner_ty, i32_ty]);
    (
        types,
        OwnerCarrierTypes {
            result_owner_ty,
            owner_ty,
            region_ty,
            i32_ty,
        },
    )
}

#[test]
fn return_escape_protects_region_token_identity_inside_result_owner_payload() {
    let (types, tys) = owner_carrier_types();
    let returned = Place::temporary(ResourceId(0), tys.result_owner_ty);
    let suffix = vec![
        PlaceProjection::EnumPayload {
            variant: "Ok".to_string(),
        },
        PlaceProjection::Field {
            index: 1,
            offset_bytes: 4,
        },
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
    ];

    assert!(!raw_identity_return_projection_is_escape(
        Some(&types),
        &returned,
        &suffix,
        tys.i32_ty,
    ));
}

#[test]
fn return_escape_rejects_public_i32_field_outside_owner_token() {
    let (types, tys) = owner_carrier_types();
    let returned = Place::temporary(ResourceId(0), tys.result_owner_ty);
    let suffix = vec![
        PlaceProjection::EnumPayload {
            variant: "Ok".to_string(),
        },
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
    ];

    assert!(raw_identity_return_projection_is_escape(
        Some(&types),
        &returned,
        &suffix,
        tys.i32_ty,
    ));
}

#[test]
fn return_escape_treats_final_owner_carrier_payload_as_protected() {
    let (types, tys) = owner_carrier_types();
    let returned = Place::temporary(ResourceId(0), tys.result_owner_ty);
    let suffix = vec![PlaceProjection::EnumPayload {
        variant: "Ok".to_string(),
    }];

    assert!(raw_identity_projection_has_owner_protection(
        &types,
        returned.ty,
        &suffix,
    ));
    assert!(!raw_identity_return_projection_is_escape(
        Some(&types),
        &returned,
        &suffix,
        tys.owner_ty,
    ));
}

#[test]
fn return_escape_keeps_region_token_itself_as_owner_provenance() {
    let (types, tys) = owner_carrier_types();
    let returned = Place::temporary(ResourceId(0), tys.region_ty);

    assert!(!raw_identity_return_projection_is_escape(
        Some(&types),
        &returned,
        &[],
        tys.region_ty,
    ));
}
