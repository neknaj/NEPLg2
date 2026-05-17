use alloc::string::ToString;
use alloc::vec;

use crate::resource_primitives::{CompilerMemoryFieldSpec, OWNER_TOKEN_TYPE_NAME};
use crate::source_map::CompilerMemoryType;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::super::model::PlaceProjection;
use super::owner_token_raw_i32_leaf_projections;

fn owner_token_struct(types: &mut TypeCtx, mark: bool) -> TypeId {
    let type_param = types.fresh_var(Some("T".to_string()));
    let i32_ty = types.i32();
    let ty = types.register_named(
        OWNER_TOKEN_TYPE_NAME.to_string(),
        TypeKind::Struct {
            doc: None,
            name: OWNER_TOKEN_TYPE_NAME.to_string(),
            type_params: vec![type_param],
            fields: vec![i32_ty, i32_ty],
            field_names: vec![
                CompilerMemoryFieldSpec::RawI32.name().to_string(),
                CompilerMemoryFieldSpec::SizeI32.name().to_string(),
            ],
        },
    );
    if mark {
        types.mark_compiler_memory_type(ty, CompilerMemoryType::OwnerToken);
    }
    ty
}

#[test]
fn owner_token_raw_i32_leaf_uses_compiler_memory_field_spec() {
    let mut types = TypeCtx::new();
    let token = owner_token_struct(&mut types, true);

    let leaves = owner_token_raw_i32_leaf_projections(&types, token);

    assert_eq!(leaves.len(), 1);
    assert_eq!(leaves[0].ty, types.i32());
    assert_eq!(
        leaves[0].suffix,
        vec![PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        }]
    );
}

#[test]
fn unproven_same_shape_struct_is_not_owner_token_leaf() {
    let mut types = TypeCtx::new();
    let token = owner_token_struct(&mut types, false);

    assert!(owner_token_raw_i32_leaf_projections(&types, token).is_empty());
}
