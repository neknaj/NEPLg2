use nepl_core::ast::Effect;
use nepl_core::hir::{
    FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirLine, HirModule, HirParam,
};
use nepl_core::resource::{
    check_resource_initialized_moves, compare_hir_resource_lowering, lower_hir_module_skeleton,
    AggregateKind, CellState, EffectOp, PlaceRoot, RawMemoryOp, ResourceBlock, ResourceBlockId,
    ResourceCallTarget, ResourceCheckDiagnostic, ResourceCheckOperation,
    ResourceCoverageDiagnostic, ResourceCoverageKind, ResourceFunction, ResourceId, ResourceModule,
    ResourceOp, ResourceTerminator,
};
use nepl_core::span::Span;
use nepl_core::types::{TypeCtx, TypeId, TypeKind};

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
    let coverage = compare_hir_resource_lowering(&module, &resource);
    assert_eq!(coverage.diagnostics, vec![]);

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

#[test]
fn resource_ir_lowering_preserves_raw_memory_operations() {
    let unit_ty = TypeId(0);
    let i32_ty = TypeId(1);
    let span = Span::dummy();
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty: TypeId(2),
            params: vec![],
            result: i32_ty,
            effect: Effect::Impure,
            body: HirBody::Block(HirBlock {
                lines: vec![
                    HirLine {
                        expr: HirExpr {
                            ty: unit_ty,
                            kind: HirExprKind::Let {
                                name: "p".to_string(),
                                mutable: false,
                                value: Box::new(HirExpr {
                                    ty: i32_ty,
                                    kind: HirExprKind::Call {
                                        callee: FuncRef::User(
                                            "alloc_raw".to_string(),
                                            vec![],
                                            None,
                                        ),
                                        args: vec![HirExpr {
                                            ty: i32_ty,
                                            kind: HirExprKind::LiteralI32(4),
                                            span,
                                        }],
                                    },
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
                            kind: HirExprKind::Intrinsic {
                                name: "store".to_string(),
                                type_args: vec![i32_ty],
                                args: vec![
                                    HirExpr {
                                        ty: i32_ty,
                                        kind: HirExprKind::Var("p".to_string()),
                                        span,
                                    },
                                    HirExpr {
                                        ty: i32_ty,
                                        kind: HirExprKind::LiteralI32(7),
                                        span,
                                    },
                                ],
                            },
                            span,
                        },
                        drop_result: true,
                    },
                    HirLine {
                        expr: HirExpr {
                            ty: i32_ty,
                            kind: HirExprKind::Intrinsic {
                                name: "load".to_string(),
                                type_args: vec![i32_ty],
                                args: vec![HirExpr {
                                    ty: i32_ty,
                                    kind: HirExprKind::Var("p".to_string()),
                                    span,
                                }],
                            },
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
    let coverage = compare_hir_resource_lowering(&module, &resource);
    assert_eq!(coverage.diagnostics, vec![]);

    let ops = &resource.functions[0].blocks[0].ops;
    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Alloc,
            args,
            ..
        } if args.len() == 1
    )));
    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Store,
            args,
            ..
        } if args.len() == 2
    )));
    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            args,
            ..
        } if args.len() == 1
    )));

    let dump = resource.dump_text();
    assert!(dump.contains("effect unsafe_memory(alloc_raw)"));
    assert!(dump.contains("raw_memory alloc"));
    assert!(dump.contains("raw_memory store"));
    assert!(dump.contains("raw_memory load"));

    let mut broken = resource.clone();
    broken.functions[0].blocks[0]
        .ops
        .retain(|op| !matches!(op, ResourceOp::RawMemory { .. }));
    let broken_coverage = compare_hir_resource_lowering(&module, &broken);
    assert!(broken_coverage
        .diagnostics
        .iter()
        .any(|diagnostic| matches!(
            diagnostic,
            ResourceCoverageDiagnostic::CountMismatch {
                function,
                kind: ResourceCoverageKind::RawMemory,
                hir: 3,
                resource: 0,
            } if function == "main"
        )));
}

#[test]
fn resource_ir_lowering_preserves_call_targets_and_callback_places() {
    let unit_ty = TypeId(0);
    let i32_ty = TypeId(1);
    let fn_ty = TypeId(2);
    let span = Span::dummy();
    let main = HirFunction {
        doc: None,
        name: "main".to_string(),
        func_ty: TypeId(3),
        params: vec![HirParam {
            name: "arg".to_string(),
            ty: i32_ty,
            mutable: false,
        }],
        result: i32_ty,
        effect: Effect::Impure,
        body: HirBody::Block(HirBlock {
            lines: vec![
                HirLine {
                    expr: HirExpr {
                        ty: unit_ty,
                        kind: HirExprKind::Let {
                            name: "f".to_string(),
                            mutable: false,
                            value: Box::new(HirExpr {
                                ty: fn_ty,
                                kind: HirExprKind::FnValue("callee".to_string()),
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
                        kind: HirExprKind::Call {
                            callee: FuncRef::User("callee".to_string(), vec![], None),
                            args: vec![HirExpr {
                                ty: i32_ty,
                                kind: HirExprKind::Var("arg".to_string()),
                                span,
                            }],
                        },
                        span,
                    },
                    drop_result: true,
                },
                HirLine {
                    expr: HirExpr {
                        ty: i32_ty,
                        kind: HirExprKind::CallIndirect {
                            callee: Box::new(HirExpr {
                                ty: fn_ty,
                                kind: HirExprKind::Var("f".to_string()),
                                span,
                            }),
                            params: vec![i32_ty],
                            result: i32_ty,
                            args: vec![HirExpr {
                                ty: i32_ty,
                                kind: HirExprKind::Var("arg".to_string()),
                                span,
                            }],
                        },
                        span,
                    },
                    drop_result: false,
                },
            ],
            ty: i32_ty,
            span,
        }),
        span,
    };
    let callee = HirFunction {
        doc: None,
        name: "callee".to_string(),
        func_ty: TypeId(4),
        params: vec![HirParam {
            name: "value".to_string(),
            ty: i32_ty,
            mutable: false,
        }],
        result: i32_ty,
        effect: Effect::Impure,
        body: HirBody::Block(HirBlock {
            lines: vec![HirLine {
                expr: HirExpr {
                    ty: i32_ty,
                    kind: HirExprKind::LiteralI32(0),
                    span,
                },
                drop_result: false,
            }],
            ty: i32_ty,
            span,
        }),
        span,
    };
    let module = HirModule {
        functions: vec![main, callee],
        entry: Some("main".to_string()),
        externs: vec![],
        string_literals: vec![],
        traits: vec![],
        impls: vec![],
    };

    let resource = lower_hir_module_skeleton(&module);
    let coverage = compare_hir_resource_lowering(&module, &resource);
    assert_eq!(coverage.diagnostics, vec![]);

    let ops = &resource.functions[0].blocks[0].ops;
    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::FunctionValue {
            name,
            effect: EffectOp::UserCall {
                effect: Effect::Impure,
                ..
            },
            ..
        } if name == "callee"
    )));
    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::Call {
            target: ResourceCallTarget::User { name, .. },
            args,
            effect: EffectOp::UserCall {
                effect: Effect::Impure,
                ..
            },
            ..
        } if name == "callee" && args.len() == 1
    )));
    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::IndirectCall {
            params,
            result,
            args,
            ..
        } if params == &vec![i32_ty] && result == &i32_ty && args.len() == 1
    )));

    let dump = resource.dump_text();
    assert!(dump.contains("function_value callee"));
    assert!(dump.contains("call user(callee<>)"));
    assert!(dump.contains("effect=call(callee,Impure)"));
    assert!(dump.contains("indirect_call"));
}

#[test]
fn resource_ir_check_allows_repeated_copy_reads() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty: TypeId(8),
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
                            ty: i32_ty,
                            kind: HirExprKind::Var("x".to_string()),
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
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_check_reports_non_copy_use_after_move() {
    let (types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let module = non_copy_read_module(unit_ty, owned_ty, span, false);

    let resource = lower_hir_module_skeleton(&module);
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceCheckDiagnostic::CellUnavailable {
            function,
            operation: ResourceCheckOperation::Read,
            place,
            state: CellState::Moved,
            ..
        } if function == "main" && matches!(&place.root, PlaceRoot::Local(name) if name == "x")
    )));
}

#[test]
fn resource_ir_check_reports_read_after_drop() {
    let (types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let module = non_copy_read_module(unit_ty, owned_ty, span, true);

    let resource = lower_hir_module_skeleton(&module);
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceCheckDiagnostic::CellUnavailable {
            function,
            operation: ResourceCheckOperation::Read,
            place,
            state: CellState::Dropped,
            ..
        } if function == "main" && matches!(&place.root, PlaceRoot::Local(name) if name == "x")
    )));
}

#[test]
fn resource_ir_check_reports_uninitialized_read() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let x = nepl_core::resource::Place::local("x".to_string(), i32_ty);
    let resource = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            params: vec![],
            result: unit_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::DeclareLocal {
                        place: x.clone(),
                        mutable: false,
                        initializer: None,
                        span,
                    },
                    ResourceOp::Read {
                        source: x,
                        output: nepl_core::resource::Place::temporary(ResourceId(0), i32_ty),
                        span,
                    },
                ],
                terminator: ResourceTerminator::Return { value: None, span },
                span,
            }],
            span,
        }],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_initialized_moves(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceCheckDiagnostic::CellUnavailable {
            function,
            operation: ResourceCheckOperation::Read,
            state: CellState::Uninit,
            ..
        } if function == "main"
    )));
}

fn types_with_non_copy_owned() -> (TypeCtx, TypeId) {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    let owned_ty = types.register_named(
        "Owned".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "Owned".to_string(),
            type_params: vec![],
            fields: vec![],
            field_names: vec![],
        },
    );
    (types, owned_ty)
}

fn non_copy_read_module(
    unit_ty: TypeId,
    owned_ty: TypeId,
    span: Span,
    drop_before_second_read: bool,
) -> HirModule {
    let mut lines = vec![HirLine {
        expr: HirExpr {
            ty: unit_ty,
            kind: HirExprKind::Let {
                name: "x".to_string(),
                mutable: false,
                value: Box::new(HirExpr {
                    ty: owned_ty,
                    kind: HirExprKind::StructConstruct {
                        name: "Owned".to_string(),
                        type_args: vec![],
                        fields: vec![],
                    },
                    span,
                }),
            },
            span,
        },
        drop_result: true,
    }];
    if !drop_before_second_read {
        lines.push(HirLine {
            expr: HirExpr {
                ty: owned_ty,
                kind: HirExprKind::Var("x".to_string()),
                span,
            },
            drop_result: true,
        });
    }
    if drop_before_second_read {
        lines.push(HirLine {
            expr: HirExpr {
                ty: unit_ty,
                kind: HirExprKind::Drop {
                    name: "x".to_string(),
                },
                span,
            },
            drop_result: true,
        });
    }
    lines.push(HirLine {
        expr: HirExpr {
            ty: owned_ty,
            kind: HirExprKind::Var("x".to_string()),
            span,
        },
        drop_result: true,
    });

    HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty: TypeId(9),
            params: vec![],
            result: unit_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines,
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
    }
}
