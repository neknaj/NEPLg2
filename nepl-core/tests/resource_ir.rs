use nepl_core::ast::Effect;
use nepl_core::hir::{
    FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirLine, HirModule, HirParam,
};
use nepl_core::resource::{
    check_hir_resource_safety_shadow, check_resource_borrow_lifetimes,
    check_resource_effect_boundaries, check_resource_initialized_moves,
    check_resource_owner_obligations, compare_hir_resource_lowering, lower_hir_module_skeleton,
    AggregateKind, BorrowKind, BorrowState, CellState, EffectOp, OwnerState, Place,
    PlaceProjection, PlaceRoot, RawMemoryOp, ResourceBlock, ResourceBlockId,
    ResourceBorrowDiagnostic, ResourceBorrowOperation, ResourceCallTarget, ResourceCheckDiagnostic,
    ResourceCheckOperation, ResourceCoverageDiagnostic, ResourceCoverageKind,
    ResourceEffectBoundaryDiagnostic, ResourceFunction, ResourceId, ResourceLocal, ResourceModule,
    ResourceOp, ResourceOwnerDiagnostic, ResourceOwnerOperation, ResourceTerminator,
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
    assert!(dump.contains("effect internal_alloc"));
    assert!(dump.contains("effect unsafe_memory(store)"));
    assert!(dump.contains("effect unsafe_memory(load)"));
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
fn resource_ir_effect_check_reports_raw_alloc_return_escape() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty: TypeId(20),
            params: vec![],
            result: i32_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![HirLine {
                    expr: HirExpr {
                        ty: i32_ty,
                        kind: HirExprKind::Call {
                            callee: FuncRef::User("alloc_raw".to_string(), vec![], None),
                            args: vec![HirExpr {
                                ty: i32_ty,
                                kind: HirExprKind::LiteralI32(4),
                                span,
                            }],
                        },
                        span,
                    },
                    drop_result: false,
                }],
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
    let report = check_resource_effect_boundaries(&resource);
    assert_eq!(report.functions[0].counts.internal_allocs, 1);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function,
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_effect_check_reports_raw_alloc_escape_through_raw_slot() {
    let i32_ty = TypeId(1);
    let span = Span::dummy();
    let size = Place::temporary(ResourceId(0), i32_ty);
    let raw = Place::temporary(ResourceId(1), i32_ty);
    let slot_size = Place::temporary(ResourceId(2), i32_ty);
    let slot = Place::temporary(ResourceId(3), i32_ty);
    let loaded = Place::temporary(ResourceId(4), i32_ty);
    let module = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            params: vec![],
            result: i32_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::Expr {
                        kind: nepl_core::resource::ResourceExprKind::Literal,
                        output: size.clone(),
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: raw.clone(),
                        args: vec![size],
                        span,
                    },
                    ResourceOp::Expr {
                        kind: nepl_core::resource::ResourceExprKind::Literal,
                        output: slot_size.clone(),
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: slot.clone(),
                        args: vec![slot_size],
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Store,
                        output: Place::temporary(ResourceId(5), TypeId(0)),
                        args: vec![slot.clone(), raw],
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Load,
                        output: loaded.clone(),
                        args: vec![slot],
                        span,
                    },
                ],
                terminator: ResourceTerminator::Return {
                    value: Some(loaded),
                    span,
                },
                span,
            }],
            span,
        }],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_effect_boundaries(&module);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function,
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_effect_check_reports_raw_alloc_escape_through_parameter_slot_alias() {
    let unit_ty = TypeId(0);
    let i32_ty = TypeId(1);
    let span = Span::dummy();
    let slot = Place::local("slot".to_string(), i32_ty);
    let alias = Place::temporary(ResourceId(0), i32_ty);
    let size = Place::temporary(ResourceId(1), i32_ty);
    let raw = Place::temporary(ResourceId(2), i32_ty);
    let loaded = Place::temporary(ResourceId(3), i32_ty);
    let module = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            params: vec![ResourceLocal {
                name: "slot".to_string(),
                ty: i32_ty,
                mutable: false,
                place: slot.clone(),
            }],
            result: i32_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::Read {
                        source: slot.clone(),
                        output: alias.clone(),
                        span,
                    },
                    ResourceOp::Expr {
                        kind: nepl_core::resource::ResourceExprKind::Literal,
                        output: size.clone(),
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: raw.clone(),
                        args: vec![size],
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Store,
                        output: Place::temporary(ResourceId(4), unit_ty),
                        args: vec![alias, raw],
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Load,
                        output: loaded.clone(),
                        args: vec![slot],
                        span,
                    },
                ],
                terminator: ResourceTerminator::Return {
                    value: Some(loaded),
                    span,
                },
                span,
            }],
            span,
        }],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_effect_boundaries(&module);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function,
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_effect_check_reports_raw_alloc_escape_through_returned_slot_alias() {
    let unit_ty = TypeId(0);
    let i32_ty = TypeId(1);
    let span = Span::dummy();
    let id_param = Place::local("slot".to_string(), i32_ty);
    let main_slot = Place::local("slot".to_string(), i32_ty);
    let alias = Place::temporary(ResourceId(0), i32_ty);
    let size = Place::temporary(ResourceId(1), i32_ty);
    let raw = Place::temporary(ResourceId(2), i32_ty);
    let loaded = Place::temporary(ResourceId(3), i32_ty);
    let module = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "slot_id".to_string(),
                params: vec![ResourceLocal {
                    name: "slot".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: id_param.clone(),
                }],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![],
                    terminator: ResourceTerminator::Return {
                        value: Some(id_param),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                params: vec![ResourceLocal {
                    name: "slot".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: main_slot.clone(),
                }],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::Call {
                            output: alias.clone(),
                            target: ResourceCallTarget::User {
                                name: "slot_id".to_string(),
                                type_args: vec![],
                            },
                            args: vec![main_slot.clone()],
                            effect: EffectOp::UserCall {
                                name: "slot_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::Expr {
                            kind: nepl_core::resource::ResourceExprKind::Literal,
                            output: size.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: raw.clone(),
                            args: vec![size],
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Store,
                            output: Place::temporary(ResourceId(4), unit_ty),
                            args: vec![alias, raw],
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Load,
                            output: loaded.clone(),
                            args: vec![main_slot],
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(loaded),
                        span,
                    },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_effect_boundaries(&module);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function,
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_effect_check_reports_raw_alloc_escape_wrapped_in_struct() {
    let i32_ty = TypeId(1);
    let box_ty = TypeId(2);
    let span = Span::dummy();
    let size = Place::temporary(ResourceId(0), i32_ty);
    let raw = Place::temporary(ResourceId(1), i32_ty);
    let boxed = Place::temporary(ResourceId(2), box_ty);
    let module = ResourceModule {
        functions: vec![ResourceFunction {
            name: "make_box".to_string(),
            params: vec![],
            result: box_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::Expr {
                        kind: nepl_core::resource::ResourceExprKind::Literal,
                        output: size.clone(),
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: raw.clone(),
                        args: vec![size],
                        span,
                    },
                    ResourceOp::Construct {
                        output: boxed.clone(),
                        kind: AggregateKind::Struct {
                            name: "RawBox".to_string(),
                        },
                        inputs: vec![raw],
                        span,
                    },
                ],
                terminator: ResourceTerminator::Return {
                    value: Some(boxed),
                    span,
                },
                span,
            }],
            span,
        }],
        entry: None,
        string_literals: vec![],
    };

    let report = check_resource_effect_boundaries(&module);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function,
            ..
        } if function == "make_box"
    )));
}

#[test]
fn resource_ir_effect_check_reports_raw_alloc_escape_through_identity_call() {
    let i32_ty = TypeId(1);
    let span = Span::dummy();
    let param = Place::local("p".to_string(), i32_ty);
    let size = Place::temporary(ResourceId(0), i32_ty);
    let raw = Place::temporary(ResourceId(1), i32_ty);
    let forwarded = Place::temporary(ResourceId(2), i32_ty);
    let module = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "raw_id".to_string(),
                params: vec![nepl_core::resource::ResourceLocal {
                    name: "p".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: param.clone(),
                }],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![],
                    terminator: ResourceTerminator::Return {
                        value: Some(param),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                params: vec![],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::Expr {
                            kind: nepl_core::resource::ResourceExprKind::Literal,
                            output: size.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: raw.clone(),
                            args: vec![size],
                            span,
                        },
                        ResourceOp::Call {
                            output: forwarded.clone(),
                            target: ResourceCallTarget::User {
                                name: "raw_id".to_string(),
                                type_args: vec![],
                            },
                            args: vec![raw],
                            effect: EffectOp::UserCall {
                                name: "raw_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(forwarded),
                        span,
                    },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_effect_boundaries(&module);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function,
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_effect_check_reports_raw_alloc_escape_through_function_value_call() {
    let i32_ty = TypeId(1);
    let fn_ty = TypeId(2);
    let span = Span::dummy();
    let param = Place::local("p".to_string(), i32_ty);
    let function_value = Place::temporary(ResourceId(0), fn_ty);
    let function_local = Place::local("f".to_string(), fn_ty);
    let size = Place::temporary(ResourceId(1), i32_ty);
    let raw = Place::temporary(ResourceId(2), i32_ty);
    let forwarded = Place::temporary(ResourceId(3), i32_ty);
    let module = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "raw_id".to_string(),
                params: vec![nepl_core::resource::ResourceLocal {
                    name: "p".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: param.clone(),
                }],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![],
                    terminator: ResourceTerminator::Return {
                        value: Some(param),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                params: vec![],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::FunctionValue {
                            output: function_value.clone(),
                            name: "raw_id".to_string(),
                            effect: EffectOp::UserCall {
                                name: "raw_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::DeclareLocal {
                            place: function_local.clone(),
                            mutable: false,
                            initializer: Some(function_value),
                            span,
                        },
                        ResourceOp::Expr {
                            kind: nepl_core::resource::ResourceExprKind::Literal,
                            output: size.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: raw.clone(),
                            args: vec![size],
                            span,
                        },
                        ResourceOp::IndirectCall {
                            output: forwarded.clone(),
                            callee: function_local,
                            params: vec![i32_ty],
                            result: i32_ty,
                            args: vec![raw],
                            effect: EffectOp::Unknown {
                                reason: "test function value".to_string(),
                            },
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(forwarded),
                        span,
                    },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_effect_boundaries(&module);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function,
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_effect_check_reports_raw_alloc_escape_through_higher_order_helper() {
    let i32_ty = TypeId(1);
    let fn_ty = TypeId(2);
    let span = Span::dummy();
    let apply_p = Place::local("p".to_string(), i32_ty);
    let apply_f = Place::local("f".to_string(), fn_ty);
    let apply_result = Place::temporary(ResourceId(0), i32_ty);
    let size = Place::temporary(ResourceId(1), i32_ty);
    let raw = Place::temporary(ResourceId(2), i32_ty);
    let function_value = Place::temporary(ResourceId(3), fn_ty);
    let forwarded = Place::temporary(ResourceId(4), i32_ty);
    let module = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "apply".to_string(),
                params: vec![
                    nepl_core::resource::ResourceLocal {
                        name: "p".to_string(),
                        ty: i32_ty,
                        mutable: false,
                        place: apply_p.clone(),
                    },
                    nepl_core::resource::ResourceLocal {
                        name: "f".to_string(),
                        ty: fn_ty,
                        mutable: false,
                        place: apply_f.clone(),
                    },
                ],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![ResourceOp::IndirectCall {
                        output: apply_result.clone(),
                        callee: apply_f,
                        params: vec![i32_ty],
                        result: i32_ty,
                        args: vec![apply_p],
                        effect: EffectOp::Unknown {
                            reason: "function parameter".to_string(),
                        },
                        span,
                    }],
                    terminator: ResourceTerminator::Return {
                        value: Some(apply_result),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                params: vec![],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::Expr {
                            kind: nepl_core::resource::ResourceExprKind::Literal,
                            output: size.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: raw.clone(),
                            args: vec![size],
                            span,
                        },
                        ResourceOp::FunctionValue {
                            output: function_value.clone(),
                            name: "raw_id".to_string(),
                            effect: EffectOp::UserCall {
                                name: "raw_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::Call {
                            output: forwarded.clone(),
                            target: ResourceCallTarget::User {
                                name: "apply".to_string(),
                                type_args: vec![],
                            },
                            args: vec![raw, function_value],
                            effect: EffectOp::UserCall {
                                name: "apply".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(forwarded),
                        span,
                    },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_effect_boundaries(&module);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function,
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_effect_check_reports_unsafe_memory_in_pure_function() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty: TypeId(21),
            params: vec![],
            result: unit_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![HirLine {
                    expr: HirExpr {
                        ty: unit_ty,
                        kind: HirExprKind::Intrinsic {
                            name: "store".to_string(),
                            type_args: vec![i32_ty],
                            args: vec![
                                HirExpr {
                                    ty: i32_ty,
                                    kind: HirExprKind::LiteralI32(16),
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
                }],
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
    let report = check_resource_effect_boundaries(&resource);
    assert_eq!(report.functions[0].counts.unsafe_memory_ops, 1);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction {
            function,
            operation,
            ..
        } if function == "main" && operation == "store"
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

#[test]
fn resource_ir_owner_check_accepts_deallocated_alloc() {
    let types = TypeCtx::new();
    let span = Span::dummy();
    let module = raw_owner_module(types.unit(), types.i32(), span, 1);

    let resource = lower_hir_module_skeleton(&module);
    let report = check_resource_owner_obligations(&resource);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_owner_check_reports_leaked_alloc() {
    let types = TypeCtx::new();
    let span = Span::dummy();
    let module = raw_owner_module(types.unit(), types.i32(), span, 0);

    let resource = lower_hir_module_skeleton(&module);
    let report = check_resource_owner_obligations(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerLeaked {
            function,
            place,
            ..
        } if function == "main" && matches!(&place.root, PlaceRoot::Local(name) if name == "p")
    )));
}

#[test]
fn resource_ir_owner_check_reports_double_dealloc() {
    let types = TypeCtx::new();
    let span = Span::dummy();
    let module = raw_owner_module(types.unit(), types.i32(), span, 2);

    let resource = lower_hir_module_skeleton(&module);
    let report = check_resource_owner_obligations(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerUnavailable {
            function,
            operation: ResourceOwnerOperation::Read,
            place,
            state: OwnerState::Moved,
            ..
        } if function == "main" && matches!(&place.root, PlaceRoot::Local(name) if name == "p")
    )));
}

#[test]
fn resource_ir_owner_check_reports_helper_alloc_return_leak() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_owned = Place::temporary(ResourceId(0), i32_ty);
    let main_owned = Place::temporary(ResourceId(1), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "alloc_owner".to_string(),
                params: vec![],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: helper_owned.clone(),
                        args: vec![],
                        span,
                    }],
                    terminator: ResourceTerminator::Return {
                        value: Some(helper_owned),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                params: vec![],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![ResourceOp::Call {
                        output: main_owned.clone(),
                        target: ResourceCallTarget::User {
                            name: "alloc_owner".to_string(),
                            type_args: vec![],
                        },
                        args: vec![],
                        effect: EffectOp::UserCall {
                            name: "alloc_owner".to_string(),
                            effect: Effect::Pure,
                        },
                        span,
                    }],
                    terminator: ResourceTerminator::Return { value: None, span },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_owner_obligations(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerLeaked {
            function,
            place,
            ..
        } if function == "main" && place == &main_owned
    )));
}

#[test]
fn resource_ir_owner_check_transfers_owner_returned_by_helper() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_param = Place::local("p".to_string(), i32_ty);
    let p = Place::temporary(ResourceId(0), i32_ty);
    let returned = Place::temporary(ResourceId(1), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "owner_id".to_string(),
                params: vec![ResourceLocal {
                    name: "p".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: helper_param.clone(),
                }],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![],
                    terminator: ResourceTerminator::Return {
                        value: Some(helper_param),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                params: vec![],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: p.clone(),
                            args: vec![],
                            span,
                        },
                        ResourceOp::Call {
                            output: returned.clone(),
                            target: ResourceCallTarget::User {
                                name: "owner_id".to_string(),
                                type_args: vec![],
                            },
                            args: vec![p],
                            effect: EffectOp::UserCall {
                                name: "owner_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Dealloc,
                            output: Place::temporary(ResourceId(2), unit_ty),
                            args: vec![returned],
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return { value: None, span },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_owner_obligations(&resource);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_owner_check_reports_function_value_alloc_return_leak() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_owned = Place::temporary(ResourceId(0), i32_ty);
    let callee = Place::temporary(ResourceId(1), i32_ty);
    let main_owned = Place::temporary(ResourceId(2), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "alloc_owner".to_string(),
                params: vec![],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: helper_owned.clone(),
                        args: vec![],
                        span,
                    }],
                    terminator: ResourceTerminator::Return {
                        value: Some(helper_owned),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                params: vec![],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::FunctionValue {
                            output: callee.clone(),
                            name: "alloc_owner".to_string(),
                            effect: EffectOp::UserCall {
                                name: "alloc_owner".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::IndirectCall {
                            output: main_owned.clone(),
                            callee,
                            params: vec![],
                            result: i32_ty,
                            args: vec![],
                            effect: EffectOp::UserCall {
                                name: "alloc_owner".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return { value: None, span },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_owner_obligations(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerLeaked {
            function,
            place,
            ..
        } if function == "main" && place == &main_owned
    )));
}

#[test]
fn resource_ir_owner_check_transfers_owner_returned_by_function_value() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_param = Place::local("p".to_string(), i32_ty);
    let p = Place::temporary(ResourceId(0), i32_ty);
    let callee = Place::temporary(ResourceId(1), i32_ty);
    let returned = Place::temporary(ResourceId(2), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "owner_id".to_string(),
                params: vec![ResourceLocal {
                    name: "p".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: helper_param.clone(),
                }],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![],
                    terminator: ResourceTerminator::Return {
                        value: Some(helper_param),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                params: vec![],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: p.clone(),
                            args: vec![],
                            span,
                        },
                        ResourceOp::FunctionValue {
                            output: callee.clone(),
                            name: "owner_id".to_string(),
                            effect: EffectOp::UserCall {
                                name: "owner_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::IndirectCall {
                            output: returned.clone(),
                            callee,
                            params: vec![i32_ty],
                            result: i32_ty,
                            args: vec![p],
                            effect: EffectOp::UserCall {
                                name: "owner_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Dealloc,
                            output: Place::temporary(ResourceId(3), unit_ty),
                            args: vec![returned],
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return { value: None, span },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_owner_obligations(&resource);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_owner_check_transfers_owner_returned_by_unknown_callback() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let p = Place::temporary(ResourceId(0), i32_ty);
    let callee = Place::local("callback".to_string(), i32_ty);
    let returned = Place::temporary(ResourceId(1), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: p.clone(),
                args: vec![],
                span,
            },
            ResourceOp::IndirectCall {
                output: returned.clone(),
                callee,
                params: vec![i32_ty],
                result: i32_ty,
                args: vec![p],
                effect: EffectOp::Unknown {
                    reason: "callback parameter".to_string(),
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(2), unit_ty),
                args: vec![returned],
                span,
            },
        ],
    );

    let report = check_resource_owner_obligations(&resource);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_owner_check_moves_owner_into_constructed_aggregate() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let wrapper_ty = types.register_named(
        "Wrapper".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "Wrapper".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["ptr".to_string()],
        },
    );
    let span = Span::dummy();
    let p = Place::temporary(ResourceId(0), i32_ty);
    let wrapper = Place::temporary(ResourceId(1), wrapper_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: p.clone(),
                args: vec![],
                span,
            },
            ResourceOp::Construct {
                output: wrapper,
                kind: AggregateKind::Struct {
                    name: "Wrapper".to_string(),
                },
                inputs: vec![p.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(2), unit_ty),
                args: vec![p.clone()],
                span,
            },
        ],
    );

    let report = check_resource_owner_obligations(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerUnavailable {
            function,
            operation: ResourceOwnerOperation::Dealloc,
            place,
            state: OwnerState::Moved,
            ..
        } if function == "main" && place == &p
    )));
}

#[test]
fn resource_ir_owner_check_moves_aggregate_owner_projection() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let wrapper_ty = types.register_named(
        "Wrapper".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "Wrapper".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["ptr".to_string()],
        },
    );
    let span = Span::dummy();
    let p = Place::temporary(ResourceId(0), i32_ty);
    let wrapper = Place::temporary(ResourceId(1), wrapper_ty);
    let moved_wrapper = Place::temporary(ResourceId(2), wrapper_ty);
    let old_field = wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let moved_field = moved_wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: p.clone(),
                args: vec![],
                span,
            },
            ResourceOp::Construct {
                output: wrapper.clone(),
                kind: AggregateKind::Struct {
                    name: "Wrapper".to_string(),
                },
                inputs: vec![p],
                span,
            },
            ResourceOp::Move {
                source: wrapper,
                output: moved_wrapper,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(3), unit_ty),
                args: vec![old_field.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(4), unit_ty),
                args: vec![moved_field],
                span,
            },
        ],
    );

    let report = check_resource_owner_obligations(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerUnavailable {
            function,
            operation: ResourceOwnerOperation::Dealloc,
            place,
            state: OwnerState::Moved,
            ..
        } if function == "main" && place == &old_field
    )));
}

#[test]
fn resource_ir_owner_check_reports_aggregate_owner_return_leak_in_caller() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let wrapper_ty = types.register_named(
        "Wrapper".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "Wrapper".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["ptr".to_string()],
        },
    );
    let span = Span::dummy();
    let helper_ptr = Place::temporary(ResourceId(0), i32_ty);
    let helper_wrapper = Place::temporary(ResourceId(1), wrapper_ty);
    let main_wrapper = Place::temporary(ResourceId(2), wrapper_ty);
    let main_field = main_wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "make_wrapper".to_string(),
                params: vec![],
                result: wrapper_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: helper_ptr.clone(),
                            args: vec![],
                            span,
                        },
                        ResourceOp::Construct {
                            output: helper_wrapper.clone(),
                            kind: AggregateKind::Struct {
                                name: "Wrapper".to_string(),
                            },
                            inputs: vec![helper_ptr],
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(helper_wrapper),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                params: vec![],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![ResourceOp::Call {
                        output: main_wrapper,
                        target: ResourceCallTarget::User {
                            name: "make_wrapper".to_string(),
                            type_args: vec![],
                        },
                        args: vec![],
                        effect: EffectOp::UserCall {
                            name: "make_wrapper".to_string(),
                            effect: Effect::Pure,
                        },
                        span,
                    }],
                    terminator: ResourceTerminator::Return { value: None, span },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_owner_obligations(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerLeaked {
            function,
            place,
            ..
        } if function == "main" && place == &main_field
    )));
}

#[test]
fn resource_ir_owner_check_transfers_aggregate_owner_descendants_returned_by_helper() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let wrapper_ty = types.register_named(
        "Wrapper".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "Wrapper".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["ptr".to_string()],
        },
    );
    let span = Span::dummy();
    let helper_param = Place::local("w".to_string(), wrapper_ty);
    let p = Place::temporary(ResourceId(0), i32_ty);
    let wrapper = Place::temporary(ResourceId(1), wrapper_ty);
    let returned = Place::temporary(ResourceId(2), wrapper_ty);
    let returned_field = returned.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "id_wrapper".to_string(),
                params: vec![ResourceLocal {
                    name: "w".to_string(),
                    ty: wrapper_ty,
                    mutable: false,
                    place: helper_param.clone(),
                }],
                result: wrapper_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![],
                    terminator: ResourceTerminator::Return {
                        value: Some(helper_param),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                params: vec![],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: p.clone(),
                            args: vec![],
                            span,
                        },
                        ResourceOp::Construct {
                            output: wrapper.clone(),
                            kind: AggregateKind::Struct {
                                name: "Wrapper".to_string(),
                            },
                            inputs: vec![p],
                            span,
                        },
                        ResourceOp::Call {
                            output: returned,
                            target: ResourceCallTarget::User {
                                name: "id_wrapper".to_string(),
                                type_args: vec![],
                            },
                            args: vec![wrapper],
                            effect: EffectOp::UserCall {
                                name: "id_wrapper".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Dealloc,
                            output: Place::temporary(ResourceId(3), unit_ty),
                            args: vec![returned_field],
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return { value: None, span },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_owner_obligations(&resource);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_borrow_check_allows_shared_read_until_release() {
    let types = TypeCtx::new();
    let span = Span::dummy();
    let x = Place::local("x".to_string(), types.i32());
    let borrow = Place::temporary(ResourceId(0), types.i32());
    let value = Place::temporary(ResourceId(1), types.i32());
    let resource = manual_resource_module(
        types.unit(),
        span,
        vec![
            ResourceOp::Borrow {
                source: x.clone(),
                output: borrow.clone(),
                kind: BorrowKind::Shared,
                span,
            },
            ResourceOp::Read {
                source: x,
                output: value,
                span,
            },
            ResourceOp::Drop {
                place: borrow,
                span,
            },
        ],
    );

    let report = check_resource_borrow_lifetimes(&resource);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_borrow_check_reports_read_during_unique_borrow() {
    let types = TypeCtx::new();
    let span = Span::dummy();
    let x = Place::local("x".to_string(), types.i32());
    let resource = manual_resource_module(
        types.unit(),
        span,
        vec![
            ResourceOp::Borrow {
                source: x.clone(),
                output: Place::temporary(ResourceId(0), types.i32()),
                kind: BorrowKind::Unique,
                span,
            },
            ResourceOp::Read {
                source: x,
                output: Place::temporary(ResourceId(1), types.i32()),
                span,
            },
        ],
    );

    let report = check_resource_borrow_lifetimes(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceBorrowDiagnostic::BorrowConflict {
            function,
            operation: ResourceBorrowOperation::Read,
            active: BorrowState::Unique { .. },
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_borrow_check_reports_unique_conflict_with_shared_borrow() {
    let types = TypeCtx::new();
    let span = Span::dummy();
    let x = Place::local("x".to_string(), types.i32());
    let resource = manual_resource_module(
        types.unit(),
        span,
        vec![
            ResourceOp::Borrow {
                source: x.clone(),
                output: Place::temporary(ResourceId(0), types.i32()),
                kind: BorrowKind::Shared,
                span,
            },
            ResourceOp::Borrow {
                source: x,
                output: Place::temporary(ResourceId(1), types.i32()),
                kind: BorrowKind::Unique,
                span,
            },
        ],
    );

    let report = check_resource_borrow_lifetimes(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceBorrowDiagnostic::BorrowConflict {
            function,
            operation: ResourceBorrowOperation::UniqueBorrow,
            active: BorrowState::Shared { count: 1 },
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_borrow_check_releases_shared_before_unique_borrow() {
    let types = TypeCtx::new();
    let span = Span::dummy();
    let x = Place::local("x".to_string(), types.i32());
    let shared = Place::temporary(ResourceId(0), types.i32());
    let resource = manual_resource_module(
        types.unit(),
        span,
        vec![
            ResourceOp::Borrow {
                source: x.clone(),
                output: shared.clone(),
                kind: BorrowKind::Shared,
                span,
            },
            ResourceOp::Drop {
                place: shared,
                span,
            },
            ResourceOp::Borrow {
                source: x,
                output: Place::temporary(ResourceId(1), types.i32()),
                kind: BorrowKind::Unique,
                span,
            },
        ],
    );

    let report = check_resource_borrow_lifetimes(&resource);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_borrow_check_reports_returned_borrow_token() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let x = Place::local("x".to_string(), i32_ty);
    let shared = Place::temporary(ResourceId(0), i32_ty);
    let resource = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            params: vec![],
            result: i32_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![ResourceOp::Borrow {
                    source: x,
                    output: shared.clone(),
                    kind: BorrowKind::Shared,
                    span,
                }],
                terminator: ResourceTerminator::Return {
                    value: Some(shared),
                    span,
                },
                span,
            }],
            span,
        }],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_borrow_lifetimes(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceBorrowDiagnostic::BorrowConflict {
            function,
            operation: ResourceBorrowOperation::ReturnValue,
            active: BorrowState::Shared { count: 1 },
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_borrow_check_reports_borrow_token_returned_by_helper() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_param = Place::local("token".to_string(), i32_ty);
    let x = Place::local("x".to_string(), i32_ty);
    let shared = Place::temporary(ResourceId(0), i32_ty);
    let returned = Place::temporary(ResourceId(1), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "borrow_id".to_string(),
                params: vec![ResourceLocal {
                    name: "token".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: helper_param.clone(),
                }],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![],
                    terminator: ResourceTerminator::Return {
                        value: Some(helper_param),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                params: vec![],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::Borrow {
                            source: x,
                            output: shared.clone(),
                            kind: BorrowKind::Shared,
                            span,
                        },
                        ResourceOp::Call {
                            output: returned.clone(),
                            target: ResourceCallTarget::User {
                                name: "borrow_id".to_string(),
                                type_args: vec![],
                            },
                            args: vec![shared],
                            effect: EffectOp::UserCall {
                                name: "borrow_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(returned),
                        span,
                    },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_borrow_lifetimes(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceBorrowDiagnostic::BorrowConflict {
            function,
            operation: ResourceBorrowOperation::ReturnValue,
            active: BorrowState::Shared { .. },
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_borrow_check_reports_borrow_token_returned_by_function_value() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_param = Place::local("token".to_string(), i32_ty);
    let x = Place::local("x".to_string(), i32_ty);
    let shared = Place::temporary(ResourceId(0), i32_ty);
    let callee = Place::temporary(ResourceId(1), i32_ty);
    let returned = Place::temporary(ResourceId(2), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "borrow_id".to_string(),
                params: vec![ResourceLocal {
                    name: "token".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: helper_param.clone(),
                }],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![],
                    terminator: ResourceTerminator::Return {
                        value: Some(helper_param),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                params: vec![],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::Borrow {
                            source: x,
                            output: shared.clone(),
                            kind: BorrowKind::Shared,
                            span,
                        },
                        ResourceOp::FunctionValue {
                            output: callee.clone(),
                            name: "borrow_id".to_string(),
                            effect: EffectOp::UserCall {
                                name: "borrow_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::IndirectCall {
                            output: returned.clone(),
                            callee,
                            params: vec![i32_ty],
                            result: i32_ty,
                            args: vec![shared],
                            effect: EffectOp::UserCall {
                                name: "borrow_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(returned),
                        span,
                    },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_borrow_lifetimes(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceBorrowDiagnostic::BorrowConflict {
            function,
            operation: ResourceBorrowOperation::ReturnValue,
            active: BorrowState::Shared { .. },
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_borrow_check_reports_borrow_token_returned_by_unknown_callback() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let x = Place::local("x".to_string(), i32_ty);
    let shared = Place::temporary(ResourceId(0), i32_ty);
    let callee = Place::local("callback".to_string(), i32_ty);
    let returned = Place::temporary(ResourceId(1), i32_ty);
    let resource = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            params: vec![],
            result: i32_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::Borrow {
                        source: x,
                        output: shared.clone(),
                        kind: BorrowKind::Shared,
                        span,
                    },
                    ResourceOp::IndirectCall {
                        output: returned.clone(),
                        callee,
                        params: vec![i32_ty],
                        result: i32_ty,
                        args: vec![shared],
                        effect: EffectOp::Unknown {
                            reason: "callback parameter".to_string(),
                        },
                        span,
                    },
                ],
                terminator: ResourceTerminator::Return {
                    value: Some(returned),
                    span,
                },
                span,
            }],
            span,
        }],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_borrow_lifetimes(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceBorrowDiagnostic::BorrowConflict {
            function,
            operation: ResourceBorrowOperation::ReturnValue,
            active: BorrowState::Shared { .. },
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_borrow_check_does_not_return_unknown_callback_token_with_mismatched_result() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let x = Place::local("x".to_string(), i32_ty);
    let shared = Place::temporary(ResourceId(0), i32_ty);
    let callee = Place::local("callback".to_string(), bool_ty);
    let returned = Place::temporary(ResourceId(1), bool_ty);
    let resource = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            params: vec![],
            result: bool_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::Borrow {
                        source: x,
                        output: shared.clone(),
                        kind: BorrowKind::Shared,
                        span,
                    },
                    ResourceOp::IndirectCall {
                        output: returned.clone(),
                        callee,
                        params: vec![i32_ty],
                        result: bool_ty,
                        args: vec![shared],
                        effect: EffectOp::Unknown {
                            reason: "callback parameter".to_string(),
                        },
                        span,
                    },
                ],
                terminator: ResourceTerminator::Return {
                    value: Some(returned),
                    span,
                },
                span,
            }],
            span,
        }],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_borrow_lifetimes(&resource);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_borrow_check_allows_return_after_borrow_release() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let x = Place::local("x".to_string(), i32_ty);
    let shared = Place::temporary(ResourceId(0), i32_ty);
    let value = Place::temporary(ResourceId(1), i32_ty);
    let resource = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            params: vec![],
            result: i32_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::Borrow {
                        source: x.clone(),
                        output: shared.clone(),
                        kind: BorrowKind::Shared,
                        span,
                    },
                    ResourceOp::Drop {
                        place: shared,
                        span,
                    },
                    ResourceOp::Read {
                        source: x,
                        output: value.clone(),
                        span,
                    },
                ],
                terminator: ResourceTerminator::Return {
                    value: Some(value),
                    span,
                },
                span,
            }],
            span,
        }],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_borrow_lifetimes(&resource);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_shadow_report_combines_lowering_and_resource_checks() {
    let (types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let module = non_copy_read_module(unit_ty, owned_ty, span, false);

    let report = check_hir_resource_safety_shadow(&module, &types);

    assert!(!report.has_lowering_diagnostics());
    assert!(report.has_resource_diagnostics());
    assert_eq!(report.resource_diagnostic_count(), 1);
    assert!(report
        .initialized_moves
        .diagnostics
        .iter()
        .any(|diagnostic| matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable {
                    function,
                    operation: ResourceCheckOperation::Read,
                    state: CellState::Moved,
                    ..
                } if function == "main"
        )));
    assert_eq!(report.owner_obligations.diagnostics, vec![]);
    assert_eq!(report.borrow_lifetimes.diagnostics, vec![]);
    assert_eq!(report.effect_boundaries.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_merge_reports_moved_on_one_branch() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    types.register_copy_impl_target(types.bool());
    let unit_ty = types.unit();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let x = Place::local("x".to_string(), owned_ty);
    let init = Place::temporary(ResourceId(0), owned_ty);
    let cond = Place::temporary(ResourceId(1), bool_ty);
    let then_value = Place::temporary(ResourceId(2), unit_ty);
    let else_value = Place::temporary(ResourceId(3), unit_ty);
    let branch_output = Place::temporary(ResourceId(4), unit_ty);
    let moved = Place::temporary(ResourceId(5), owned_ty);
    let after = Place::temporary(ResourceId(6), owned_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Construct {
                output: init.clone(),
                kind: AggregateKind::Struct {
                    name: "Owned".to_string(),
                },
                inputs: vec![],
                span,
            },
            ResourceOp::DeclareLocal {
                place: x.clone(),
                mutable: false,
                initializer: Some(init),
                span,
            },
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: cond.clone(),
                ty: bool_ty,
                span,
            },
            ResourceOp::Branch {
                output: branch_output,
                condition: cond,
                then_ops: vec![
                    ResourceOp::Read {
                        source: x.clone(),
                        output: moved,
                        span,
                    },
                    ResourceOp::Expr {
                        kind: nepl_core::resource::ResourceExprKind::Literal,
                        output: then_value.clone(),
                        ty: unit_ty,
                        span,
                    },
                ],
                then_value,
                else_ops: vec![ResourceOp::Expr {
                    kind: nepl_core::resource::ResourceExprKind::Literal,
                    output: else_value.clone(),
                    ty: unit_ty,
                    span,
                }],
                else_value,
                span,
            },
            ResourceOp::Read {
                source: x,
                output: after,
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceCheckDiagnostic::CellUnavailable {
            function,
            operation: ResourceCheckOperation::Read,
            state: CellState::MaybeMoved,
            ..
        } if function == "main"
    )));
    assert_eq!(report.deferred.branch_merges, 0);
}

#[test]
fn resource_ir_cell_merge_reports_moved_after_loop_body() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    types.register_copy_impl_target(types.bool());
    let unit_ty = types.unit();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let x = Place::local("x".to_string(), owned_ty);
    let init = Place::temporary(ResourceId(0), owned_ty);
    let cond = Place::temporary(ResourceId(1), bool_ty);
    let moved = Place::temporary(ResourceId(2), owned_ty);
    let after = Place::temporary(ResourceId(3), owned_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Construct {
                output: init.clone(),
                kind: AggregateKind::Struct {
                    name: "Owned".to_string(),
                },
                inputs: vec![],
                span,
            },
            ResourceOp::DeclareLocal {
                place: x.clone(),
                mutable: false,
                initializer: Some(init),
                span,
            },
            ResourceOp::Loop {
                condition_ops: vec![ResourceOp::Expr {
                    kind: nepl_core::resource::ResourceExprKind::Literal,
                    output: cond.clone(),
                    ty: bool_ty,
                    span,
                }],
                condition: cond,
                body_ops: vec![ResourceOp::Read {
                    source: x.clone(),
                    output: moved,
                    span,
                }],
                span,
            },
            ResourceOp::Read {
                source: x,
                output: after,
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceCheckDiagnostic::CellUnavailable {
            function,
            operation: ResourceCheckOperation::Read,
            state: CellState::MaybeMoved,
            ..
        } if function == "main"
    )));
    assert_eq!(report.deferred.loop_merges, 0);
}

#[test]
fn resource_ir_owner_merge_rejects_dealloc_after_conditional_dealloc() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let p = Place::local("p".to_string(), i32_ty);
    let cond = Place::temporary(ResourceId(0), bool_ty);
    let branch_output = Place::temporary(ResourceId(1), unit_ty);
    let then_value = Place::temporary(ResourceId(2), unit_ty);
    let else_value = Place::temporary(ResourceId(3), unit_ty);
    let size = Place::temporary(ResourceId(4), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: p.clone(),
                args: vec![],
                span,
            },
            ResourceOp::Branch {
                output: branch_output,
                condition: cond,
                then_ops: vec![ResourceOp::RawMemory {
                    operation: RawMemoryOp::Dealloc,
                    output: Place::temporary(ResourceId(5), unit_ty),
                    args: vec![p.clone(), size.clone()],
                    span,
                }],
                then_value,
                else_ops: vec![],
                else_value,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(6), unit_ty),
                args: vec![p, size],
                span,
            },
        ],
    );

    let report = check_resource_owner_obligations(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerUnavailable {
            function,
            operation: ResourceOwnerOperation::Dealloc,
            state: OwnerState::MaybeFreed,
            ..
        } if function == "main"
    )));
    assert_eq!(report.deferred.branch_merges, 0);
}

#[test]
fn resource_ir_borrow_merge_rejects_mutation_after_branch_borrow() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let x = Place::local("x".to_string(), i32_ty);
    let cond = Place::temporary(ResourceId(0), bool_ty);
    let branch_output = Place::temporary(ResourceId(1), unit_ty);
    let then_value = Place::temporary(ResourceId(2), unit_ty);
    let else_value = Place::temporary(ResourceId(3), unit_ty);
    let shared = Place::temporary(ResourceId(4), i32_ty);
    let value = Place::temporary(ResourceId(5), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Branch {
                output: branch_output,
                condition: cond,
                then_ops: vec![ResourceOp::Borrow {
                    source: x.clone(),
                    output: shared,
                    kind: BorrowKind::Shared,
                    span,
                }],
                then_value,
                else_ops: vec![],
                else_value,
                span,
            },
            ResourceOp::Assign {
                target: x,
                value,
                span,
            },
        ],
    );

    let report = check_resource_borrow_lifetimes(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceBorrowDiagnostic::BorrowConflict {
            function,
            operation: ResourceBorrowOperation::Assign,
            active: BorrowState::Shared { count: 1 },
            ..
        } if function == "main"
    )));
    assert_eq!(report.deferred.branch_merges, 0);
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

fn manual_resource_module(unit_ty: TypeId, span: Span, ops: Vec<ResourceOp>) -> ResourceModule {
    ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            params: vec![],
            result: unit_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops,
                terminator: ResourceTerminator::Return { value: None, span },
                span,
            }],
            span,
        }],
        entry: Some("main".to_string()),
        string_literals: vec![],
    }
}

fn raw_owner_module(
    unit_ty: TypeId,
    i32_ty: TypeId,
    span: Span,
    dealloc_count: usize,
) -> HirModule {
    let mut lines = vec![HirLine {
        expr: HirExpr {
            ty: unit_ty,
            kind: HirExprKind::Let {
                name: "p".to_string(),
                mutable: false,
                value: Box::new(HirExpr {
                    ty: i32_ty,
                    kind: HirExprKind::Call {
                        callee: FuncRef::User("alloc_raw".to_string(), vec![], None),
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
    }];
    for _ in 0..dealloc_count {
        lines.push(HirLine {
            expr: HirExpr {
                ty: unit_ty,
                kind: HirExprKind::Call {
                    callee: FuncRef::User("dealloc_raw".to_string(), vec![], None),
                    args: vec![
                        HirExpr {
                            ty: i32_ty,
                            kind: HirExprKind::Var("p".to_string()),
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
            },
            drop_result: true,
        });
    }

    HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty: TypeId(10),
            params: vec![],
            result: unit_ty,
            effect: Effect::Impure,
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
