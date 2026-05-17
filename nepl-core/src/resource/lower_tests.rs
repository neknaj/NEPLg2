use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::hir::{
    FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirLine, HirModule, HirParam,
};
use crate::resource::model::{PlaceProjection, ResourceCallTarget, ResourceOp};
use crate::source_map::CompilerMemoryType;
use crate::span::{FileId, Span};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::lower_hir_module;

fn test_span() -> Span {
    Span::new(FileId(0), 0, 1)
}

fn literal_i32(ty: TypeId, value: i32) -> HirExpr {
    HirExpr {
        ty,
        kind: HirExprKind::LiteralI32(value),
        span: test_span(),
    }
}

fn literal_str(ty: TypeId, index: u32) -> HirExpr {
    HirExpr {
        ty,
        kind: HirExprKind::LiteralStr(index),
        span: test_span(),
    }
}

fn function_type(types: &mut TypeCtx, params: Vec<TypeId>, result: TypeId) -> TypeId {
    types.function(vec![], params, result, Effect::Pure)
}

#[test]
fn ordinary_get_direct_call_is_not_field_projection() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let str_ty = types.str();
    let pair_ty = types.register_named(
        "Pair".to_string(),
        TypeKind::Struct {
            name: "Pair".to_string(),
            type_params: vec![],
            fields: vec![i32_ty, i32_ty],
            field_names: vec!["left".to_string(), "right".to_string()],
        },
    );

    let get_fn = HirFunction {
        doc: None,
        name: "get".to_string(),
        origin_name: "get".to_string(),
        func_ty: function_type(&mut types, vec![pair_ty, str_ty], i32_ty),
        params: vec![
            HirParam {
                name: "pair".to_string(),
                ty: pair_ty,
                mutable: false,
            },
            HirParam {
                name: "field".to_string(),
                ty: str_ty,
                mutable: false,
            },
        ],
        result: i32_ty,
        effect: Effect::Pure,
        body: HirBody::Block(HirBlock {
            lines: vec![HirLine {
                expr: literal_i32(i32_ty, 7),
                drop_result: false,
            }],
            ty: i32_ty,
            span: test_span(),
        }),
        span: test_span(),
    };

    let pair_expr = HirExpr {
        ty: pair_ty,
        kind: HirExprKind::StructConstruct {
            name: "Pair".to_string(),
            type_args: vec![],
            fields: vec![literal_i32(i32_ty, 1), literal_i32(i32_ty, 2)],
        },
        span: test_span(),
    };
    let selector_expr = HirExpr {
        ty: str_ty,
        kind: HirExprKind::LiteralStr(0),
        span: test_span(),
    };
    let call_expr = HirExpr {
        ty: i32_ty,
        kind: HirExprKind::Call {
            callee: FuncRef::User("get".to_string(), vec![], None),
            args: vec![pair_expr, selector_expr],
        },
        span: test_span(),
    };
    let main_fn = HirFunction {
        doc: None,
        name: "main".to_string(),
        origin_name: "main".to_string(),
        func_ty: function_type(&mut types, vec![], i32_ty),
        params: vec![],
        result: i32_ty,
        effect: Effect::Pure,
        body: HirBody::Block(HirBlock {
            lines: vec![HirLine {
                expr: call_expr,
                drop_result: false,
            }],
            ty: i32_ty,
            span: test_span(),
        }),
        span: test_span(),
    };
    let module = HirModule {
        functions: vec![get_fn, main_fn],
        entry: Some("main".to_string()),
        externs: vec![],
        string_literals: vec!["left".to_string()],
        traits: vec![],
        impls: vec![],
    };

    let resource = lower_hir_module(&module, &types);
    let main = resource
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function should be lowered");
    let ops = &main.blocks[0].ops;

    assert!(
        ops.iter().any(|op| matches!(
            op,
            ResourceOp::Call {
                target: ResourceCallTarget::User { name, .. },
                ..
            } if name == "get"
        )),
        "ordinary direct call named get must remain a call, not a synthetic field read"
    );
    assert!(
        !ops.iter().any(|op| matches!(
            op,
            ResourceOp::Read { source, .. }
                if source.projections.iter().any(|projection| matches!(
                    projection,
                    PlaceProjection::Field { .. } | PlaceProjection::TupleField { .. }
                ))
        )),
        "ordinary direct call named get must not create a field projection read"
    );
}

#[test]
fn transparent_raw_address_return_ignores_ordinary_get_call() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let str_ty = types.str();
    let raw_ptr_ty = types.register_named(
        "MemPtr".to_string(),
        TypeKind::Struct {
            name: "MemPtr".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["raw".to_string()],
        },
    );
    types.mark_compiler_memory_type(raw_ptr_ty, CompilerMemoryType::RawPointer);

    let get_fn = HirFunction {
        doc: None,
        name: "get".to_string(),
        origin_name: "get".to_string(),
        func_ty: function_type(&mut types, vec![raw_ptr_ty, str_ty], i32_ty),
        params: vec![
            HirParam {
                name: "ptr".to_string(),
                ty: raw_ptr_ty,
                mutable: false,
            },
            HirParam {
                name: "field".to_string(),
                ty: str_ty,
                mutable: false,
            },
        ],
        result: i32_ty,
        effect: Effect::Pure,
        body: HirBody::Block(HirBlock {
            lines: vec![HirLine {
                expr: literal_i32(i32_ty, 99),
                drop_result: false,
            }],
            ty: i32_ty,
            span: test_span(),
        }),
        span: test_span(),
    };

    let project_raw_fn = HirFunction {
        doc: None,
        name: "project_raw".to_string(),
        origin_name: "project_raw".to_string(),
        func_ty: function_type(&mut types, vec![raw_ptr_ty], i32_ty),
        params: vec![HirParam {
            name: "ptr".to_string(),
            ty: raw_ptr_ty,
            mutable: false,
        }],
        result: i32_ty,
        effect: Effect::Pure,
        body: HirBody::Block(HirBlock {
            lines: vec![HirLine {
                expr: HirExpr {
                    ty: i32_ty,
                    kind: HirExprKind::Call {
                        callee: FuncRef::User("get".to_string(), vec![], None),
                        args: vec![
                            HirExpr {
                                ty: raw_ptr_ty,
                                kind: HirExprKind::Var("ptr".to_string()),
                                span: test_span(),
                            },
                            literal_str(str_ty, 0),
                        ],
                    },
                    span: test_span(),
                },
                drop_result: false,
            }],
            ty: i32_ty,
            span: test_span(),
        }),
        span: test_span(),
    };

    let main_fn = HirFunction {
        doc: None,
        name: "main".to_string(),
        origin_name: "main".to_string(),
        func_ty: function_type(&mut types, vec![raw_ptr_ty], i32_ty),
        params: vec![HirParam {
            name: "ptr".to_string(),
            ty: raw_ptr_ty,
            mutable: false,
        }],
        result: i32_ty,
        effect: Effect::Pure,
        body: HirBody::Block(HirBlock {
            lines: vec![HirLine {
                expr: HirExpr {
                    ty: i32_ty,
                    kind: HirExprKind::Call {
                        callee: FuncRef::User("project_raw".to_string(), vec![], None),
                        args: vec![HirExpr {
                            ty: raw_ptr_ty,
                            kind: HirExprKind::Var("ptr".to_string()),
                            span: test_span(),
                        }],
                    },
                    span: test_span(),
                },
                drop_result: false,
            }],
            ty: i32_ty,
            span: test_span(),
        }),
        span: test_span(),
    };
    let module = HirModule {
        functions: vec![get_fn, project_raw_fn, main_fn],
        entry: Some("main".to_string()),
        externs: vec![],
        string_literals: vec!["raw".to_string()],
        traits: vec![],
        impls: vec![],
    };

    let resource = lower_hir_module(&module, &types);
    let main = resource
        .functions
        .iter()
        .find(|function| function.name == "main")
        .expect("main function should be lowered");
    let ops = &main.blocks[0].ops;

    assert!(
        ops.iter().any(|op| matches!(
            op,
            ResourceOp::Call {
                target: ResourceCallTarget::User { name, .. },
                ..
            } if name == "project_raw"
        )),
        "transparent return analysis must keep the ordinary wrapper call visible"
    );
    assert!(
        !ops.iter().any(|op| matches!(
            op,
            ResourceOp::RawAddressAlias { .. } | ResourceOp::RawAddressView { .. }
        )),
        "ordinary get in a helper return must not create raw-address projection proof"
    );
}
