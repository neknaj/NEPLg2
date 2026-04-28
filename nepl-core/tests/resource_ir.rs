use nepl_core::ast::Effect;
use nepl_core::hir::{
    HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirLine, HirModule, HirParam,
};
use nepl_core::resource::{lower_hir_module_skeleton, AggregateKind, PlaceRoot, ResourceOp};
use nepl_core::span::Span;
use nepl_core::types::TypeId;

#[test]
fn resource_ir_lowering_skeleton_tracks_locals_and_dump() {
    let unit_ty = TypeId(0);
    let i32_ty = TypeId(1);
    let span = Span::dummy();
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty: TypeId(2),
            params: vec![HirParam {
                name: "arg".to_string(),
                ty: i32_ty,
                mutable: false,
            }],
            result: i32_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![
                    HirLine {
                        expr: HirExpr {
                            ty: unit_ty,
                            kind: HirExprKind::Let {
                                name: "x".to_string(),
                                mutable: true,
                                value: Box::new(HirExpr {
                                    ty: i32_ty,
                                    kind: HirExprKind::LiteralI32(1),
                                    span,
                                }),
                            },
                            span,
                        },
                        drop_result: true,
                    },
                    HirLine {
                        expr: HirExpr {
                            ty: i32_ty,
                            kind: HirExprKind::Var("x".to_string()),
                            span,
                        },
                        drop_result: false,
                    },
                ],
                ty: i32_ty,
                span,
            }),
            span,
        }],
        entry: Some("main".to_string()),
        externs: vec![],
        string_literals: vec![],
        traits: vec![],
        impls: vec![],
    };

    let resource = lower_hir_module_skeleton(&module);
    let function = &resource.functions[0];
    assert_eq!(function.params.len(), 1);
    assert_eq!(function.blocks.len(), 1);
    assert!(function.blocks[0].ops.iter().any(|op| matches!(
        op,
        ResourceOp::DeclareLocal { place, mutable: true, .. }
            if matches!(&place.root, PlaceRoot::Local(name) if name == "x")
    )));
    assert!(function.blocks[0].ops.iter().any(|op| matches!(
        op,
        ResourceOp::Read { source, output, .. }
            if matches!(&source.root, PlaceRoot::Local(name) if name == "x")
                && matches!(&output.root, PlaceRoot::Temporary(_))
    )));

    assert_eq!(
        resource.dump_text(),
        concat!(
            "resource_module entry=main\n",
            "fn main effect=Pure result=t1 span=0:0-0\n",
            "  param arg mut=false ty=t1 place=%arg:t1\n",
            "  block b0:\n",
            "    expr Block out=tmp0:t1 ty=t1 span=0:0-0\n",
            "    expr Literal out=tmp1:t1 ty=t1 span=0:0-0\n",
            "    declare %x:t1 mut=true init=tmp1:t1 span=0:0-0\n",
            "    expr Let out=tmp2:t0 ty=t0 span=0:0-0\n",
            "    read %x:t1 -> tmp3:t1 span=0:0-0\n",
            "    expr LocalRead out=tmp3:t1 ty=t1 span=0:0-0\n",
            "    terminator return tmp3:t1 span=0:0-0\n"
        )
    );
}

#[test]
fn resource_ir_lowering_preserves_aggregate_and_branch_structure() {
    let unit_ty = TypeId(0);
    let i32_ty = TypeId(1);
    let bool_ty = TypeId(2);
    let tuple_ty = TypeId(3);
    let span = Span::dummy();
    let tuple_expr = HirExpr {
        ty: tuple_ty,
        kind: HirExprKind::TupleConstruct {
            items: vec![
                HirExpr {
                    ty: i32_ty,
                    kind: HirExprKind::Var("arg".to_string()),
                    span,
                },
                HirExpr {
                    ty: i32_ty,
                    kind: HirExprKind::LiteralI32(2),
                    span,
                },
            ],
        },
        span,
    };
    let branch_expr = HirExpr {
        ty: tuple_ty,
        kind: HirExprKind::If {
            cond: Box::new(HirExpr {
                ty: bool_ty,
                kind: HirExprKind::LiteralBool(true),
                span,
            }),
            then_branch: Box::new(HirExpr {
                ty: tuple_ty,
                kind: HirExprKind::Var("pair".to_string()),
                span,
            }),
            else_branch: Box::new(HirExpr {
                ty: tuple_ty,
                kind: HirExprKind::TupleConstruct {
                    items: vec![
                        HirExpr {
                            ty: i32_ty,
                            kind: HirExprKind::LiteralI32(3),
                            span,
                        },
                        HirExpr {
                            ty: i32_ty,
                            kind: HirExprKind::LiteralI32(4),
                            span,
                        },
                    ],
                },
                span,
            }),
        },
        span,
    };
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty: TypeId(4),
            params: vec![HirParam {
                name: "arg".to_string(),
                ty: i32_ty,
                mutable: false,
            }],
            result: unit_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![
                    HirLine {
                        expr: HirExpr {
                            ty: unit_ty,
                            kind: HirExprKind::Let {
                                name: "pair".to_string(),
                                mutable: false,
                                value: Box::new(tuple_expr),
                            },
                            span,
                        },
                        drop_result: true,
                    },
                    HirLine {
                        expr: branch_expr,
                        drop_result: true,
                    },
                ],
                ty: unit_ty,
                span,
            }),
            span,
        }],
        entry: Some("main".to_string()),
        externs: vec![],
        string_literals: vec![],
        traits: vec![],
        impls: vec![],
    };

    let resource = lower_hir_module_skeleton(&module);
    let ops = &resource.functions[0].blocks[0].ops;
    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::Construct {
            kind: AggregateKind::Tuple,
            inputs,
            ..
        } if inputs.len() == 2
    )));
    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::Branch {
            then_ops,
            else_ops,
            ..
        } if then_ops.iter().any(|nested| matches!(nested, ResourceOp::Read { .. }))
            && else_ops.iter().any(|nested| matches!(nested, ResourceOp::Construct {
                kind: AggregateKind::Tuple,
                ..
            }))
    )));

    let dump = resource.dump_text();
    assert!(dump.contains("construct tuple"));
    assert!(dump.contains("branch tmp"));
    assert!(dump.contains("then value="));
    assert!(dump.contains("else value="));
}

#[test]
fn resource_ir_lowering_uses_declared_local_type_for_drop() {
    let unit_ty = TypeId(0);
    let i32_ty = TypeId(1);
    let span = Span::dummy();
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty: TypeId(2),
            params: vec![],
            result: unit_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![
                    HirLine {
                        expr: HirExpr {
                            ty: unit_ty,
                            kind: HirExprKind::Let {
                                name: "x".to_string(),
                                mutable: false,
                                value: Box::new(HirExpr {
                                    ty: i32_ty,
                                    kind: HirExprKind::LiteralI32(1),
                                    span,
                                }),
                            },
                            span,
                        },
                        drop_result: true,
                    },
                    HirLine {
                        expr: HirExpr {
                            ty: unit_ty,
                            kind: HirExprKind::Drop {
                                name: "x".to_string(),
                            },
                            span,
                        },
                        drop_result: true,
                    },
                ],
                ty: unit_ty,
                span,
            }),
            span,
        }],
        entry: Some("main".to_string()),
        externs: vec![],
        string_literals: vec![],
        traits: vec![],
        impls: vec![],
    };

    let dump = lower_hir_module_skeleton(&module).dump_text();
    assert!(dump.contains("drop %x:t1 span=0:0-0"));
    assert!(!dump.contains("drop %x:t0 span=0:0-0"));
}
