use alloc::string::ToString;
use alloc::vec;

use crate::types::{TypeCtx, TypeKind};

use super::raw_pointer_type::type_can_carry_raw_pointer_alias_summary;

#[test]
fn summary_carrier_includes_plain_i32_raw_address_slots() {
    let types = TypeCtx::new();

    assert!(type_can_carry_raw_pointer_alias_summary(
        &types,
        types.i32()
    ));
}

#[test]
fn summary_carrier_recurses_through_aggregates() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let wrapped_i32 = types.register_named(
        "RawAddressBox".to_string(),
        TypeKind::Struct {
            name: "RawAddressBox".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["raw".to_string()],
        },
    );

    assert!(type_can_carry_raw_pointer_alias_summary(
        &types,
        wrapped_i32
    ));
}
