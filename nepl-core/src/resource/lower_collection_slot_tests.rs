use alloc::boxed::Box;
use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use super::lower_hir_module;
use crate::ast::Effect;
use crate::hir::{
    HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirLine, HirModule, HirParam,
};
use crate::resource::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotReplacement,
};
use crate::resource::model::{PlaceProjection, ResourceOffset, ResourceOp};
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

fn raw_pointer_type(types: &mut TypeCtx) -> TypeId {
    let i32_ty = types.i32();
    let ty = types.register_named(
        "MemPtr".to_string(),
        TypeKind::Struct {
            name: "MemPtr".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["raw".to_string()],
        },
    );
    types.mark_compiler_memory_type(ty, CompilerMemoryType::RawPointer);
    ty
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

fn literal_i32(ty: TypeId, value: i32) -> HirExpr {
    HirExpr {
        ty,
        kind: HirExprKind::LiteralI32(value),
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
fn collection_slot_lifecycle_intrinsic_lowers_slot_event_from_pointer_and_offset() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let payload_ty = types.register_named(
        "Payload".to_string(),
        TypeKind::Struct {
            name: "Payload".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["value".to_string()],
        },
    );
    let raw_ptr_ty = raw_pointer_type(&mut types);
    let expr = HirExpr {
        ty: unit_ty,
        kind: HirExprKind::Intrinsic {
            name: CollectionSlotLifecyclePrimitive::InitializeEmpty
                .intrinsic_name()
                .to_string(),
            type_args: vec![payload_ty],
            args: vec![var("ptr", raw_ptr_ty), literal_i32(i32_ty, 8)],
        },
        span: test_span(),
    };
    let module = main_with_expr(
        &mut types,
        vec![HirParam {
            name: "ptr".to_string(),
            ty: raw_ptr_ty,
            mutable: false,
        }],
        expr,
    );

    let resource = lower_hir_module(&module, &types);
    let ops = &resource.functions[0].blocks[0].ops;
    let lifecycle = ops
        .iter()
        .find_map(|op| match op {
            ResourceOp::CollectionSlotLifecycle { target, event, .. } => Some((target, event)),
            _ => None,
        })
        .expect("collection slot intrinsic must produce ResourceOp");

    assert_eq!(
        *lifecycle.1,
        CollectionSlotLifecycleEvent::InitializeEmpty {
            value_ty: payload_ty
        }
    );
    assert!(lifecycle.0.projections.iter().any(|projection| {
        matches!(
            projection,
            PlaceProjection::StorageOffset(ResourceOffset::Known(8))
        )
    }));
    assert!(lifecycle
        .0
        .projections
        .iter()
        .any(|projection| matches!(projection, PlaceProjection::Deref)));
}

#[test]
fn collection_slot_lifecycle_intrinsic_lowers_storage_dealloc_from_owner_token() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let owner_ty = owner_token_type(&mut types);
    let ref_owner_ty = types.reference(owner_ty, false);
    let expr = HirExpr {
        ty: unit_ty,
        kind: HirExprKind::Intrinsic {
            name: CollectionSlotLifecyclePrimitive::StorageDealloc
                .intrinsic_name()
                .to_string(),
            type_args: vec![],
            args: vec![HirExpr {
                ty: ref_owner_ty,
                kind: HirExprKind::AddrOf(Box::new(var("region", owner_ty))),
                span: test_span(),
            }],
        },
        span: test_span(),
    };
    let module = main_with_expr(
        &mut types,
        vec![HirParam {
            name: "region".to_string(),
            ty: owner_ty,
            mutable: false,
        }],
        expr,
    );

    let resource = lower_hir_module(&module, &types);
    let ops = &resource.functions[0].blocks[0].ops;

    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::CollectionSlotLifecycle {
            event: CollectionSlotLifecycleEvent::StorageDealloc,
            ..
        }
    )));
}

#[test]
fn collection_slot_lifecycle_intrinsic_lowers_replace_owner_policy() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let u8_ty = types.u8();
    let raw_ptr_ty = raw_pointer_type(&mut types);
    let expr = HirExpr {
        ty: unit_ty,
        kind: HirExprKind::Intrinsic {
            name: CollectionSlotLifecyclePrimitive::ReplaceDropOld
                .intrinsic_name()
                .to_string(),
            type_args: vec![i32_ty, u8_ty],
            args: vec![var("ptr", raw_ptr_ty), literal_i32(i32_ty, 0)],
        },
        span: test_span(),
    };
    let module = main_with_expr(
        &mut types,
        vec![HirParam {
            name: "ptr".to_string(),
            ty: raw_ptr_ty,
            mutable: false,
        }],
        expr,
    );

    let resource = lower_hir_module(&module, &types);
    let lifecycle = resource.functions[0].blocks[0]
        .ops
        .iter()
        .find_map(|op| match op {
            ResourceOp::CollectionSlotLifecycle { target, event, .. } => Some((target, event)),
            _ => None,
        })
        .expect("replace intrinsic must produce ResourceOp");

    assert_eq!(lifecycle.0.ty, i32_ty);
    assert_eq!(
        *lifecycle.1,
        CollectionSlotLifecycleEvent::ReplaceInitialized {
            old_ty: i32_ty,
            new_ty: u8_ty,
            old_owner: CollectionSlotReplacement::DropOldOwner,
        }
    );
}
