use nepl_core::layout::{aggregate_fields_with_offsets, storage_align_bytes, storage_size_bytes};
use nepl_core::types::{TypeCtx, TypeKind};

#[test]
fn bool_field_offsets_use_backend_scalar_width() {
    let mut types = TypeCtx::new();
    let bool_ty = types.bool();
    let i32_ty = types.i32();
    let pair_ty = types.register_named(
        "BoolThenI32".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "BoolThenI32".to_string(),
            type_params: vec![],
            fields: vec![bool_ty, i32_ty],
            field_names: vec!["flag".to_string(), "value".to_string()],
        },
    );

    let fields = aggregate_fields_with_offsets(&types, pair_ty);

    assert_eq!(storage_size_bytes(&types, bool_ty), 4);
    assert_eq!(fields[0].offset, 0);
    assert_eq!(fields[1].offset, 4);
    assert_eq!(storage_size_bytes(&types, pair_ty), 8);
}

#[test]
fn generic_struct_layout_substitutes_field_sizes_once() {
    let mut types = TypeCtx::new();
    let i64_ty = types.register_named("i64".to_string(), TypeKind::Named("i64".to_string()));
    let bool_ty = types.bool();
    let type_param = types.fresh_var(Some("T".to_string()));
    let generic_ty = types.register_named(
        "GenericPair".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "GenericPair".to_string(),
            type_params: vec![type_param],
            fields: vec![type_param, bool_ty],
            field_names: vec!["head".to_string(), "tail".to_string()],
        },
    );
    let applied_ty = types.apply(generic_ty, vec![i64_ty]);

    let fields = aggregate_fields_with_offsets(&types, applied_ty);

    assert_eq!(fields[0].ty, i64_ty);
    assert_eq!(fields[0].offset, 0);
    assert_eq!(fields[1].offset, 8);
    assert_eq!(storage_size_bytes(&types, applied_ty), 12);
    assert_eq!(storage_align_bytes(&types, applied_ty), 8);
}

#[test]
fn compiler_passes_do_not_reintroduce_local_storage_layout_helpers() {
    let sources = [
        include_str!("../src/typecheck.rs"),
        include_str!("../src/passes/drop_insertion.rs"),
        include_str!("../src/codegen_wasm.rs"),
        include_str!("../src/codegen_llvm.rs"),
    ];

    for source in sources {
        assert!(!source.contains("fn type_storage_size_bytes"));
        assert!(!source.contains("fn type_storage_align_bytes"));
        assert!(!source.contains("fn mapped_storage_type_id"));
        assert!(!source.contains("fn aggregate_fields_with_offsets"));
    }
}

#[test]
fn drop_insertion_uses_resource_drop_requirement_for_drop_classification() {
    let source = include_str!("../src/passes/drop_insertion.rs");

    assert!(source.contains("ResourceDropRequirement"));
    assert!(source.contains("resource_drop_requirement_for_type"));
    assert!(!source.contains("fn structural_drop_fields"));
    assert!(!source.contains("fn type_needs_structural_drop"));
    assert!(!source.contains("fn structural_enum_field_drop_lines"));
}
