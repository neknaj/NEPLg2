use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use super::lower_hir_module;
use crate::ast::Effect;
use crate::hir::{
    HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirLine, HirModule, HirParam,
};
use crate::resource::model::{PlaceProjection, ResourceOp};
use crate::resource_primitives::CollectionSlotLifecyclePrimitive;
use crate::source_map::CompilerMemoryType;
use crate::span::{FileId, Span};
use crate::types::{TypeCtx, TypeId, TypeKind};

fn test_span() -> Span {
    Span::new(FileId(0), 0, 1)
}

fn function_type(types: &mut TypeCtx, params: Vec<TypeId>, result: TypeId) -> TypeId {
    types.function(vec![], params, result, Effect::Pure)
}

fn owner_token_type(types: &mut TypeCtx) -> TypeId {
    let i32_ty = types.i32();
    let ty = types.register_named(
        "RegionToken".to_string(),
        TypeKind::Struct {
            name: "RegionToken".to_string(),
            type_params: vec![],
            fields: vec![i32_ty, i32_ty],
            field_names: vec!["raw".to_string(), "size".to_string()],
        },
    );
    types.mark_compiler_memory_type(ty, CompilerMemoryType::OwnerToken);
    ty
}

fn var(name: &str, ty: TypeId) -> HirExpr {
    HirExpr {
        ty,
        kind: HirExprKind::Var(name.to_string()),
        span: test_span(),
    }
}

fn main_with_expr(types: &mut TypeCtx, params: Vec<HirParam>, expr: HirExpr) -> HirModule {
    let param_types = params.iter().map(|param| param.ty).collect();
    let unit_ty = types.unit();
    HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            origin_name: "main".to_string(),
            func_ty: function_type(types, param_types, unit_ty),
            params,
            result: unit_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![HirLine {
                    expr,
                    drop_result: true,
                }],
                ty: unit_ty,
                span: test_span(),
            }),
            span: test_span(),
        }],
        entry: Some("main".to_string()),
        externs: vec![],
        string_literals: vec![],
        traits: vec![],
        impls: vec![],
    }
}

#[test]
fn collection_slot_lifecycle_intrinsic_lowers_storage_relocate() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let owner_ty = owner_token_type(&mut types);
    let expr = HirExpr {
        ty: unit_ty,
        kind: HirExprKind::Intrinsic {
            name: CollectionSlotLifecyclePrimitive::StorageRelocate
                .intrinsic_name()
                .to_string(),
            type_args: vec![],
            args: vec![var("old_storage", owner_ty), var("new_storage", owner_ty)],
        },
        span: test_span(),
    };
    let module = main_with_expr(
        &mut types,
        vec![
            HirParam {
                name: "old_storage".to_string(),
                ty: owner_ty,
                mutable: false,
            },
            HirParam {
                name: "new_storage".to_string(),
                ty: owner_ty,
                mutable: false,
            },
        ],
        expr,
    );

    let resource = lower_hir_module(&module, &types);

    let relocate = resource.functions[0].blocks[0]
        .ops
        .iter()
        .find_map(|op| match op {
            ResourceOp::CollectionStorageRelocate {
                old_storage,
                new_storage,
                ..
            } => Some((old_storage, new_storage)),
            _ => None,
        })
        .expect("storage relocate op should be produced");
    assert_eq!(relocate.0.ty, types.i32());
    assert_eq!(relocate.1.ty, types.i32());
    assert_ne!(relocate.0, relocate.1);
    assert_eq!(
        relocate.0.projections.as_slice(),
        &[PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        }]
    );
    assert_eq!(
        relocate.1.projections.as_slice(),
        &[PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        }]
    );
}
