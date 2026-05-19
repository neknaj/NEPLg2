use alloc::{string::String, vec};

use crate::types::{EnumVariantInfo, TypeCtx, TypeKind};

use super::initialized_summary_variant_type::return_type_may_have_variant_param_summary;

#[test]
fn variant_summary_gate_accepts_enum_and_enum_apply_returns() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let enum_ty = types.register_named(
        String::from("MaybeRaw"),
        TypeKind::Enum {
            name: String::from("MaybeRaw"),
            type_params: vec![],
            variants: vec![EnumVariantInfo {
                name: String::from("Some"),
                payload: Some(i32_ty),
            }],
        },
    );
    let param = types.fresh_var(Some(String::from("T")));
    let generic_enum = types.register_named(
        String::from("BoxedChoice"),
        TypeKind::Enum {
            name: String::from("BoxedChoice"),
            type_params: vec![param],
            variants: vec![EnumVariantInfo {
                name: String::from("Boxed"),
                payload: Some(param),
            }],
        },
    );
    let enum_apply = types.apply(generic_enum, vec![i32_ty]);

    assert!(return_type_may_have_variant_param_summary(&types, enum_ty));
    assert!(return_type_may_have_variant_param_summary(
        &types, enum_apply
    ));
}

#[test]
fn variant_summary_gate_skips_concrete_non_enum_returns() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let struct_ty = types.register_named(
        String::from("StringBuilder"),
        TypeKind::Struct {
            name: String::from("StringBuilder"),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec![String::from("raw")],
        },
    );
    let param = types.fresh_var(Some(String::from("T")));
    let generic_struct = types.register_named(
        String::from("Storage"),
        TypeKind::Struct {
            name: String::from("Storage"),
            type_params: vec![param],
            fields: vec![i32_ty],
            field_names: vec![String::from("raw")],
        },
    );
    let struct_apply = types.apply(generic_struct, vec![i32_ty]);

    assert!(!return_type_may_have_variant_param_summary(
        &types,
        types.str()
    ));
    assert!(!return_type_may_have_variant_param_summary(
        &types, struct_ty
    ));
    assert!(!return_type_may_have_variant_param_summary(
        &types,
        struct_apply
    ));
}

#[test]
fn variant_summary_gate_keeps_unresolved_types_conservative() {
    let mut types = TypeCtx::new();
    let unresolved_var = types.fresh_var(Some(String::from("Unknown")));
    let unresolved_named = types.register_named(
        String::from("AliasToMissing"),
        TypeKind::Named(String::from("Missing")),
    );

    assert!(return_type_may_have_variant_param_summary(
        &types,
        unresolved_var
    ));
    assert!(return_type_may_have_variant_param_summary(
        &types,
        unresolved_named
    ));
}
