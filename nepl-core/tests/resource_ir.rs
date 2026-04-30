use nepl_core::ast::Effect;
use nepl_core::hir::{
    FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirLine, HirModule, HirParam,
};
use nepl_core::loader::Loader;
use nepl_core::resource::{
    check_hir_resource_safety_shadow, check_resource_borrow_lifetimes,
    check_resource_effect_boundaries, check_resource_initialized_moves,
    check_resource_owner_obligations, compare_hir_resource_lowering, lower_hir_module,
    lower_hir_module_skeleton, AggregateKind, BorrowKind, BorrowState, CellState, EffectOp,
    OwnerState, Place, PlaceProjection, PlaceRoot, RawMemoryOp, ResourceBlock, ResourceBlockId,
    ResourceBorrowDiagnostic, ResourceBorrowOperation, ResourceCallTarget, ResourceCheckDiagnostic,
    ResourceCheckOperation, ResourceCoverageDiagnostic, ResourceCoverageKind,
    ResourceEffectBoundaryDiagnostic, ResourceExprKind, ResourceFunction, ResourceId,
    ResourceLocal, ResourceModule, ResourceOffset, ResourceOp, ResourceOwnerDiagnostic,
    ResourceOwnerOperation, ResourceTerminator,
};
use nepl_core::span::{FileId, Span};
use nepl_core::types::{TypeCtx, TypeId, TypeKind};
use nepl_core::{BuildProfile, CompileTarget};
use std::path::PathBuf;

fn stdlib_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}

fn typecheck_resource_source(source: &str) -> (HirModule, TypeCtx) {
    typecheck_resource_source_with_target(source, CompileTarget::Wasm)
}

fn typecheck_resource_source_with_target(
    source: &str,
    target: CompileTarget,
) -> (HirModule, TypeCtx) {
    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline(PathBuf::from("resource_ir_test.nepl"), source.to_string())
        .expect("load source with stdlib");
    let checked = nepl_core::typecheck::typecheck(
        &loaded.module,
        target,
        BuildProfile::Debug,
        Some(&loaded.source_map),
    );
    assert!(
        checked.diagnostics.is_empty(),
        "typecheck diagnostics: {:#?}",
        checked.diagnostics
    );
    (checked.module.expect("typechecked module"), checked.types)
}

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
            kind: AggregateKind::Tuple { .. },
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
                kind: AggregateKind::Tuple { .. },
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
                ..
            } if function == "main"
        )));
}

#[test]
fn resource_ir_lowering_coverage_guards_borrow_and_deref_places() {
    let ref_ty = TypeId(0);
    let i32_ty = TypeId(1);
    let span = Span::new(FileId(7), 11, 17);
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty: TypeId(2),
            params: vec![HirParam {
                name: "p".to_string(),
                ty: ref_ty,
                mutable: false,
            }],
            result: i32_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![
                    HirLine {
                        expr: HirExpr {
                            ty: TypeId(3),
                            kind: HirExprKind::AddrOf(Box::new(HirExpr {
                                ty: i32_ty,
                                kind: HirExprKind::Deref(Box::new(HirExpr {
                                    ty: ref_ty,
                                    kind: HirExprKind::Var("p".to_string()),
                                    span,
                                })),
                                span,
                            })),
                            span,
                        },
                        drop_result: true,
                    },
                    HirLine {
                        expr: HirExpr {
                            ty: i32_ty,
                            kind: HirExprKind::Deref(Box::new(HirExpr {
                                ty: ref_ty,
                                kind: HirExprKind::Var("p".to_string()),
                                span,
                            })),
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
        ResourceOp::Borrow { source, .. }
            if source.projections == vec![PlaceProjection::Deref]
    )));
    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::Read { source, .. }
            if source.projections == vec![PlaceProjection::Deref]
    )));
    assert!(!resource.dump_text().contains("unknown:t"));

    let mut broken = resource.clone();
    if let Some(ResourceOp::Borrow { source, .. }) = broken.functions[0].blocks[0]
        .ops
        .iter_mut()
        .find(|op| matches!(op, ResourceOp::Borrow { .. }))
    {
        *source = Place::unknown(i32_ty);
    }
    let broken_coverage = compare_hir_resource_lowering(&module, &broken);
    assert!(broken_coverage
        .diagnostics
        .iter()
        .any(|diagnostic| matches!(
            diagnostic,
            ResourceCoverageDiagnostic::UnknownPlace {
                function,
                operation,
                span: diagnostic_span,
                ..
            } if function == "main" && operation == "borrow.source" && *diagnostic_span == span
        )));
    assert!(broken_coverage
        .diagnostics
        .iter()
        .any(|diagnostic| matches!(
            diagnostic,
            ResourceCoverageDiagnostic::CountMismatch {
                function,
                kind: ResourceCoverageKind::UnknownPlace,
                hir: 0,
                resource: 1,
                span: diagnostic_span,
                ..
            } if function == "main" && *diagnostic_span == span
        )));
}

#[test]
fn resource_ir_lowering_returns_concrete_unit_place_for_while() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let bool_ty = types.bool();
    let span = Span::new(FileId(8), 20, 32);
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty: TypeId(3),
            params: vec![],
            result: unit_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![HirLine {
                    expr: HirExpr {
                        ty: unit_ty,
                        kind: HirExprKind::While {
                            cond: Box::new(HirExpr {
                                ty: bool_ty,
                                kind: HirExprKind::LiteralBool(false),
                                span,
                            }),
                            body: Box::new(HirExpr {
                                ty: unit_ty,
                                kind: HirExprKind::Block(HirBlock {
                                    lines: vec![],
                                    ty: unit_ty,
                                    span,
                                }),
                                span,
                            }),
                        },
                        span,
                    },
                    drop_result: false,
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
    let coverage = compare_hir_resource_lowering(&module, &resource);
    assert_eq!(coverage.diagnostics, vec![]);
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);

    let ops = &resource.functions[0].blocks[0].ops;
    assert!(ops.iter().any(|op| matches!(op, ResourceOp::Loop { .. })));
    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::Expr {
            kind: ResourceExprKind::Loop,
            output,
            ..
        } if !matches!(output.root, PlaceRoot::Unknown)
    )));
    assert!(matches!(
        &resource.functions[0].blocks[0].terminator,
        ResourceTerminator::Return {
            value: Some(place),
            ..
        } if !matches!(place.root, PlaceRoot::Unknown)
    ));
    assert!(!resource.dump_text().contains("unknown:t"));
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

fn raw_identity_payload_escape_after_destructive_overwrite(overwrite: RawMemoryOp) -> bool {
    let unit_ty = TypeId(0);
    let i32_ty = TypeId(1);
    let span = Span::dummy();
    let identity_size = Place::temporary(ResourceId(0), i32_ty);
    let identity = Place::temporary(ResourceId(1), i32_ty);
    let slot_size = Place::temporary(ResourceId(2), i32_ty);
    let slot = Place::temporary(ResourceId(3), i32_ty);
    let clean_size = Place::temporary(ResourceId(4), i32_ty);
    let clean_source = Place::temporary(ResourceId(5), i32_ty);
    let len = Place::temporary(ResourceId(6), i32_ty);
    let fill_value = Place::temporary(ResourceId(7), i32_ty);
    let store_unit = Place::temporary(ResourceId(8), unit_ty);
    let overwrite_unit = Place::temporary(ResourceId(9), unit_ty);
    let loaded = Place::temporary(ResourceId(10), i32_ty);
    let overwrite_args = match overwrite {
        RawMemoryOp::Fill => vec![slot.clone(), len.clone(), fill_value.clone()],
        RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => {
            vec![slot.clone(), clean_source.clone(), len.clone()]
        }
        _ => panic!("unsupported destructive overwrite operation"),
    };
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
                        kind: ResourceExprKind::Literal,
                        output: identity_size.clone(),
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: identity.clone(),
                        args: vec![identity_size],
                        span,
                    },
                    ResourceOp::Expr {
                        kind: ResourceExprKind::Literal,
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
                    ResourceOp::Expr {
                        kind: ResourceExprKind::Literal,
                        output: clean_size.clone(),
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: clean_source,
                        args: vec![clean_size],
                        span,
                    },
                    ResourceOp::Expr {
                        kind: ResourceExprKind::Literal,
                        output: len,
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::Expr {
                        kind: ResourceExprKind::Literal,
                        output: fill_value,
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Store,
                        output: store_unit,
                        args: vec![slot.clone(), identity],
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: overwrite,
                        output: overwrite_unit,
                        args: overwrite_args,
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
    report.diagnostics.iter().any(|diagnostic| {
        matches!(
            diagnostic,
            ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc { .. }
        )
    })
}

#[test]
fn resource_ir_effect_check_clears_raw_identity_payload_on_fill() {
    assert!(!raw_identity_payload_escape_after_destructive_overwrite(
        RawMemoryOp::Fill
    ));
}

#[test]
fn resource_ir_effect_check_clears_raw_identity_payload_on_bulk_copy() {
    assert!(!raw_identity_payload_escape_after_destructive_overwrite(
        RawMemoryOp::BulkCopy
    ));
}

#[test]
fn resource_ir_effect_check_clears_raw_identity_payload_on_bulk_move() {
    assert!(!raw_identity_payload_escape_after_destructive_overwrite(
        RawMemoryOp::BulkMove
    ));
}

#[test]
fn resource_ir_effect_check_preserves_raw_slot_pointer_alias_stored_in_aggregate_field() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let wrapper_ty = types.register_named(
        "PtrBox".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "PtrBox".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["ptr".to_string()],
        },
    );
    let span = Span::dummy();
    let slot_size = Place::temporary(ResourceId(0), i32_ty);
    let slot = Place::temporary(ResourceId(1), i32_ty);
    let wrapper = Place::temporary(ResourceId(2), wrapper_ty);
    let field = wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let field_ptr = Place::temporary(ResourceId(3), i32_ty);
    let raw_size = Place::temporary(ResourceId(4), i32_ty);
    let raw = Place::temporary(ResourceId(5), i32_ty);
    let loaded = Place::temporary(ResourceId(6), i32_ty);
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
                    ResourceOp::Construct {
                        output: wrapper,
                        kind: AggregateKind::Struct {
                            name: "PtrBox".to_string(),
                            field_offsets: vec![0],
                        },
                        inputs: vec![slot.clone()],
                        span,
                    },
                    ResourceOp::Read {
                        source: field,
                        output: field_ptr.clone(),
                        span,
                    },
                    ResourceOp::Expr {
                        kind: nepl_core::resource::ResourceExprKind::Literal,
                        output: raw_size.clone(),
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: raw.clone(),
                        args: vec![raw_size],
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Store,
                        output: Place::temporary(ResourceId(7), unit_ty),
                        args: vec![field_ptr, raw],
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
fn resource_ir_effect_check_preserves_raw_slot_pointer_alias_fields_across_aggregate_copy() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let wrapper_ty = types.register_named(
        "PtrBox".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "PtrBox".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["ptr".to_string()],
        },
    );
    let span = Span::dummy();
    let slot_size = Place::temporary(ResourceId(0), i32_ty);
    let slot = Place::temporary(ResourceId(1), i32_ty);
    let wrapper = Place::temporary(ResourceId(2), wrapper_ty);
    let copied_wrapper = Place::temporary(ResourceId(3), wrapper_ty);
    let copied_field = copied_wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let copied_field_ptr = Place::temporary(ResourceId(4), i32_ty);
    let raw_size = Place::temporary(ResourceId(5), i32_ty);
    let raw = Place::temporary(ResourceId(6), i32_ty);
    let loaded = Place::temporary(ResourceId(7), i32_ty);
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
                    ResourceOp::Construct {
                        output: wrapper.clone(),
                        kind: AggregateKind::Struct {
                            name: "PtrBox".to_string(),
                            field_offsets: vec![0],
                        },
                        inputs: vec![slot.clone()],
                        span,
                    },
                    ResourceOp::Read {
                        source: wrapper,
                        output: copied_wrapper,
                        span,
                    },
                    ResourceOp::Read {
                        source: copied_field,
                        output: copied_field_ptr.clone(),
                        span,
                    },
                    ResourceOp::Expr {
                        kind: nepl_core::resource::ResourceExprKind::Literal,
                        output: raw_size.clone(),
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: raw.clone(),
                        args: vec![raw_size],
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Store,
                        output: Place::temporary(ResourceId(8), unit_ty),
                        args: vec![copied_field_ptr, raw],
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
fn resource_ir_effect_check_preserves_raw_slot_identity_after_pointer_reassignment() {
    let unit_ty = TypeId(0);
    let i32_ty = TypeId(1);
    let span = Span::dummy();
    let size_a = Place::temporary(ResourceId(0), i32_ty);
    let ptr_a = Place::temporary(ResourceId(1), i32_ty);
    let size_b = Place::temporary(ResourceId(2), i32_ty);
    let ptr_b = Place::temporary(ResourceId(3), i32_ty);
    let identity_value = Place::temporary(ResourceId(4), i32_ty);
    let p = Place::local("p".to_string(), i32_ty);
    let non_identity = Place::temporary(ResourceId(5), i32_ty);
    let loaded = Place::temporary(ResourceId(6), i32_ty);
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
                        output: size_a.clone(),
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: ptr_a.clone(),
                        args: vec![size_a],
                        span,
                    },
                    ResourceOp::Expr {
                        kind: nepl_core::resource::ResourceExprKind::Literal,
                        output: size_b.clone(),
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: ptr_b.clone(),
                        args: vec![size_b],
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: identity_value.clone(),
                        args: vec![],
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Store,
                        output: Place::temporary(ResourceId(7), unit_ty),
                        args: vec![ptr_a.clone(), identity_value],
                        span,
                    },
                    ResourceOp::DeclareLocal {
                        place: p.clone(),
                        mutable: true,
                        initializer: Some(ptr_a.clone()),
                        span,
                    },
                    ResourceOp::Assign {
                        target: p.clone(),
                        value: ptr_b,
                        span,
                    },
                    ResourceOp::Expr {
                        kind: nepl_core::resource::ResourceExprKind::Literal,
                        output: non_identity.clone(),
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Store,
                        output: Place::temporary(ResourceId(8), unit_ty),
                        args: vec![p, non_identity],
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Load,
                        output: loaded.clone(),
                        args: vec![ptr_a],
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
                            field_offsets: vec![0],
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
fn resource_ir_effect_check_reports_raw_alloc_escape_read_from_constructed_aggregate_field() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let box_ty = types.register_named(
        "RawBox".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "RawBox".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["ptr".to_string()],
        },
    );
    let span = Span::dummy();
    let size = Place::temporary(ResourceId(0), i32_ty);
    let raw = Place::temporary(ResourceId(1), i32_ty);
    let boxed = Place::temporary(ResourceId(2), box_ty);
    let field = boxed.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let extracted = Place::temporary(ResourceId(3), i32_ty);
    let module = ResourceModule {
        functions: vec![ResourceFunction {
            name: "read_box_field".to_string(),
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
                    ResourceOp::Construct {
                        output: boxed,
                        kind: AggregateKind::Struct {
                            name: "RawBox".to_string(),
                            field_offsets: vec![0],
                        },
                        inputs: vec![raw],
                        span,
                    },
                    ResourceOp::Read {
                        source: field,
                        output: extracted.clone(),
                        span,
                    },
                ],
                terminator: ResourceTerminator::Return {
                    value: Some(extracted),
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
        } if function == "read_box_field"
    )));
}

#[test]
fn resource_ir_effect_check_preserves_raw_identity_fields_across_aggregate_copy() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let box_ty = types.register_named(
        "RawBox".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "RawBox".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["ptr".to_string()],
        },
    );
    let span = Span::dummy();
    let size = Place::temporary(ResourceId(0), i32_ty);
    let raw = Place::temporary(ResourceId(1), i32_ty);
    let boxed = Place::temporary(ResourceId(2), box_ty);
    let copied_box = Place::temporary(ResourceId(3), box_ty);
    let copied_field = copied_box.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let extracted = Place::temporary(ResourceId(4), i32_ty);
    let module = ResourceModule {
        functions: vec![ResourceFunction {
            name: "read_copied_box_field".to_string(),
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
                    ResourceOp::Construct {
                        output: boxed.clone(),
                        kind: AggregateKind::Struct {
                            name: "RawBox".to_string(),
                            field_offsets: vec![0],
                        },
                        inputs: vec![raw],
                        span,
                    },
                    ResourceOp::Read {
                        source: boxed,
                        output: copied_box,
                        span,
                    },
                    ResourceOp::Read {
                        source: copied_field,
                        output: extracted.clone(),
                        span,
                    },
                ],
                terminator: ResourceTerminator::Return {
                    value: Some(extracted),
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
        } if function == "read_copied_box_field"
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
fn resource_ir_effect_check_uses_known_function_alias_stored_in_aggregate_field() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let fn_ty = types.function(vec![], vec![i32_ty], i32_ty, Effect::Pure);
    let wrapper_ty = types.register_named(
        "CallbackBox".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "CallbackBox".to_string(),
            type_params: vec![],
            fields: vec![fn_ty],
            field_names: vec!["callback".to_string()],
        },
    );
    let span = Span::dummy();
    let helper_param = Place::local("p".to_string(), i32_ty);
    let helper_zero = Place::temporary(ResourceId(0), i32_ty);
    let function_value = Place::temporary(ResourceId(1), fn_ty);
    let wrapper = Place::temporary(ResourceId(2), wrapper_ty);
    let callee = wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        fn_ty,
    );
    let size = Place::temporary(ResourceId(3), i32_ty);
    let raw = Place::temporary(ResourceId(4), i32_ty);
    let forwarded = Place::temporary(ResourceId(5), i32_ty);
    let module = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "return_zero".to_string(),
                params: vec![ResourceLocal {
                    name: "p".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: helper_param,
                }],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![ResourceOp::Expr {
                        kind: nepl_core::resource::ResourceExprKind::Literal,
                        output: helper_zero.clone(),
                        ty: i32_ty,
                        span,
                    }],
                    terminator: ResourceTerminator::Return {
                        value: Some(helper_zero),
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
                            name: "return_zero".to_string(),
                            effect: EffectOp::UserCall {
                                name: "return_zero".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::Construct {
                            output: wrapper,
                            kind: AggregateKind::Struct {
                                name: "CallbackBox".to_string(),
                                field_offsets: vec![0],
                            },
                            inputs: vec![function_value],
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
                            callee,
                            params: vec![i32_ty],
                            result: i32_ty,
                            args: vec![raw],
                            effect: EffectOp::Unknown {
                                reason: "field-stored callback".to_string(),
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
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_effect_check_clears_stale_function_alias_on_assignment() {
    let i32_ty = TypeId(1);
    let fn_ty = TypeId(2);
    let span = Span::dummy();
    let known_result = Place::temporary(ResourceId(0), i32_ty);
    let known_function = Place::temporary(ResourceId(1), fn_ty);
    let function_local = Place::local("f".to_string(), fn_ty);
    let unknown_function = Place::temporary(ResourceId(2), fn_ty);
    let size = Place::temporary(ResourceId(3), i32_ty);
    let raw = Place::temporary(ResourceId(4), i32_ty);
    let forwarded = Place::temporary(ResourceId(5), i32_ty);
    let module = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "safe_zero".to_string(),
                params: vec![nepl_core::resource::ResourceLocal {
                    name: "p".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: Place::local("p".to_string(), i32_ty),
                }],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![ResourceOp::Expr {
                        kind: nepl_core::resource::ResourceExprKind::Literal,
                        output: known_result.clone(),
                        ty: i32_ty,
                        span,
                    }],
                    terminator: ResourceTerminator::Return {
                        value: Some(known_result),
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
                            output: known_function.clone(),
                            name: "safe_zero".to_string(),
                            effect: EffectOp::UserCall {
                                name: "safe_zero".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::DeclareLocal {
                            place: function_local.clone(),
                            mutable: true,
                            initializer: Some(known_function),
                            span,
                        },
                        ResourceOp::Assign {
                            target: function_local.clone(),
                            value: unknown_function,
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
                                reason: "assigned unknown callback".to_string(),
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
fn resource_ir_lowering_treats_compiler_field_load_as_field_read() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let pair_ty = types.register_named(
        "Pair".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "Pair".to_string(),
            type_params: vec![],
            fields: vec![i32_ty, i32_ty],
            field_names: vec!["left".to_string(), "right".to_string()],
        },
    );
    let func_ty = types.function(vec![], vec![pair_ty], i32_ty, Effect::Pure);
    let span = Span::dummy();
    let pair_param = HirParam {
        name: "p".to_string(),
        ty: pair_ty,
        mutable: false,
    };
    let pair_var = || HirExpr {
        ty: pair_ty,
        kind: HirExprKind::Var("p".to_string()),
        span,
    };
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty,
            params: vec![pair_param],
            result: i32_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![
                    HirLine {
                        expr: HirExpr {
                            ty: i32_ty,
                            kind: HirExprKind::Intrinsic {
                                name: "load".to_string(),
                                type_args: vec![i32_ty],
                                args: vec![pair_var()],
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
                                    kind: HirExprKind::Intrinsic {
                                        name: "add".to_string(),
                                        type_args: vec![i32_ty],
                                        args: vec![
                                            pair_var(),
                                            HirExpr {
                                                ty: i32_ty,
                                                kind: HirExprKind::LiteralI32(4),
                                                span,
                                            },
                                        ],
                                    },
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

    let resource = lower_hir_module(&module, &types);
    let ops = &resource.functions[0].blocks[0].ops;

    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::Read { source, .. }
            if matches!(&source.root, PlaceRoot::Local(name) if name == "p")
                && source.projections == [PlaceProjection::Field { index: 0, offset_bytes: 0 }]
    )));
    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::Read { source, .. }
            if matches!(&source.root, PlaceRoot::Local(name) if name == "p")
                && source.projections == [PlaceProjection::Field { index: 1, offset_bytes: 4 }]
    )));
    assert!(!ops.iter().any(|op| matches!(
        op,
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            ..
        }
    )));
}

#[test]
fn resource_ir_lowering_projects_raw_aggregate_field_without_whole_load() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let holder_ty = types.register_named(
        "Holder".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "Holder".to_string(),
            type_params: vec![],
            fields: vec![i32_ty, i32_ty],
            field_names: vec!["count".to_string(), "raw".to_string()],
        },
    );
    let func_ty = types.function(vec![], vec![i32_ty], i32_ty, Effect::Pure);
    let span = Span::dummy();
    let raw_param = HirParam {
        name: "p".to_string(),
        ty: i32_ty,
        mutable: false,
    };
    let raw_var = || HirExpr {
        ty: i32_ty,
        kind: HirExprKind::Var("p".to_string()),
        span,
    };
    let raw_holder_load = HirExpr {
        ty: holder_ty,
        kind: HirExprKind::Intrinsic {
            name: "load".to_string(),
            type_args: vec![holder_ty],
            args: vec![raw_var()],
        },
        span,
    };
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            func_ty,
            params: vec![raw_param],
            result: i32_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![HirLine {
                    expr: HirExpr {
                        ty: i32_ty,
                        kind: HirExprKind::Intrinsic {
                            name: "get_field".to_string(),
                            type_args: vec![],
                            args: vec![
                                raw_holder_load,
                                HirExpr {
                                    ty: types.str(),
                                    kind: HirExprKind::LiteralStr(0),
                                    span,
                                },
                            ],
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
        string_literals: vec!["count".to_string()],
        traits: vec![],
        impls: vec![],
    };

    let resource = lower_hir_module(&module, &types);
    let ops = &resource.functions[0].blocks[0].ops;

    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::Read { source, .. }
            if matches!(&source.root, PlaceRoot::Local(name) if name == "p")
                && source.projections == [
                    PlaceProjection::Deref,
                    PlaceProjection::Field { index: 0, offset_bytes: 0 },
                ]
    )));
    assert!(!ops.iter().any(|op| matches!(
        op,
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            output,
            ..
        } if output.ty == holder_ty
    )));
}

#[test]
fn resource_ir_typechecked_get_preserves_raw_aggregate_field_projection() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/math" as *
#import "core/traits/copy" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    ptr <MemPtr<u8>>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> 16
    store<Holder> p Holder 7 mem_ptr_wrap<u8> 64 LocalToken @token_id
    let ptr <MemPtr<u8>> get load<Holder> p "ptr"
    let raw <i32> mem_ptr_addr ptr
    let h <Holder> load<Holder> p
    add raw sub 14 64
"#;
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let main = resource
        .functions
        .iter()
        .find(|function| function.name == "main" || function.name.starts_with("main__"))
        .expect("main resource function");
    let ops = &main.blocks[0].ops;

    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::Read { source, .. }
            if matches!(&source.root, PlaceRoot::Local(name) if name == "p")
                && source.projections == [
                    PlaceProjection::Deref,
                    PlaceProjection::Field { index: 1, offset_bytes: 4 },
                ]
    )));
    let holder_raw_loads = ops
        .iter()
        .filter(|op| {
            matches!(
                op,
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Load,
                    output,
                    ..
                } if types.type_to_string(output.ty).starts_with("Holder")
            )
        })
        .count();
    assert_eq!(holder_raw_loads, 1);

    let report = check_resource_initialized_moves(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "main" || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.is_empty(),
        "raw aggregate field projection must not produce main CellState diagnostics: {:#?}",
        main_diagnostics
    );
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

    let resource = lower_hir_module(&module, &types);
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

    let resource = lower_hir_module(&module, &types);
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
fn resource_ir_cell_check_reports_raw_load_before_store() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let ptr = Place::temporary(ResourceId(0), i32_ty);
    let offset_ptr = ptr.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset { bytes: Some(4) }),
        i32_ty,
    );
    let loaded = Place::temporary(ResourceId(1), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: ptr.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded,
                args: vec![offset_ptr],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceCheckDiagnostic::CellUnavailable {
            function,
            operation: ResourceCheckOperation::RawMemoryLoadCell,
            place,
            state: CellState::Uninit,
            ..
        } if function == "main"
            && place.ty == i32_ty
            && place.projections == vec![
                PlaceProjection::StorageOffset(ResourceOffset { bytes: Some(4) }),
                PlaceProjection::Deref,
            ]
    )));
    assert!(!report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceCheckDiagnostic::CellUnavailable {
            operation: ResourceCheckOperation::RawMemoryLoadAddress,
            ..
        }
    )));
}

#[test]
fn resource_ir_cell_check_allows_external_parameter_raw_load() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let i32_ty = types.i32();
    let span = Span::dummy();
    let ptr = Place::local(String::from("p"), i32_ty);
    let loaded = Place::temporary(ResourceId(0), i32_ty);
    let resource = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            params: vec![ResourceLocal {
                name: "p".to_string(),
                ty: i32_ty,
                mutable: false,
                place: ptr.clone(),
            }],
            result: i32_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![ResourceOp::RawMemory {
                    operation: RawMemoryOp::Load,
                    output: loaded,
                    args: vec![ptr],
                    span,
                }],
                terminator: ResourceTerminator::Return { value: None, span },
                span,
            }],
            span,
        }],
        entry: Some("main".to_string()),
        string_literals: vec![],
    };

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_check_allows_external_aggregate_mem_ptr_field_raw_load() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let i32_ty = types.i32();
    let mem_ptr_ty = types.register_named(
        "MemPtr".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "MemPtr".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["raw".to_string()],
        },
    );
    let vec_ty = types.register_named(
        "VecLike".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "VecLike".to_string(),
            type_params: vec![],
            fields: vec![i32_ty, i32_ty, mem_ptr_ty],
            field_names: vec!["len".to_string(), "cap".to_string(), "data".to_string()],
        },
    );
    let vec_ref_ty = types.reference(vec_ty, false);
    let data_ref_ty = types.reference(mem_ptr_ty, false);
    let span = Span::dummy();
    let vec_param = Place::local("v".to_string(), vec_ty);
    let vec_ref = Place::temporary(ResourceId(0), vec_ref_ty);
    let data_ref_address = vec_ref.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset { bytes: Some(8) }),
        i32_ty,
    );
    let data_ref = Place::temporary(ResourceId(1), data_ref_ty);
    let data_from_ref = Place::temporary(ResourceId(2), mem_ptr_ty);
    let data_local = Place::local("v_data".to_string(), mem_ptr_ty);
    let data_value = Place::temporary(ResourceId(3), mem_ptr_ty);
    let data_raw = data_value.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let raw_addr = Place::temporary(ResourceId(4), i32_ty);
    let element_addr = raw_addr.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset { bytes: None }),
        i32_ty,
    );
    let loaded = Place::temporary(ResourceId(5), i32_ty);
    let resource = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            params: vec![ResourceLocal {
                name: "v".to_string(),
                ty: vec_ty,
                mutable: false,
                place: vec_param.clone(),
            }],
            result: i32_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::Borrow {
                        source: vec_param.clone(),
                        output: vec_ref,
                        kind: BorrowKind::Shared,
                        span,
                    },
                    ResourceOp::RawAddressAlias {
                        source: data_ref_address,
                        target: data_ref.clone(),
                        span,
                    },
                    ResourceOp::Read {
                        source: data_ref.with_projection(PlaceProjection::Deref, mem_ptr_ty),
                        output: data_from_ref.clone(),
                        span,
                    },
                    ResourceOp::Expr {
                        kind: ResourceExprKind::Deref,
                        output: data_from_ref.clone(),
                        ty: mem_ptr_ty,
                        span,
                    },
                    ResourceOp::DeclareLocal {
                        place: data_local.clone(),
                        mutable: false,
                        initializer: Some(data_from_ref),
                        span,
                    },
                    ResourceOp::Read {
                        source: data_local,
                        output: data_value,
                        span,
                    },
                    ResourceOp::RawAddressAlias {
                        source: data_raw,
                        target: raw_addr.clone(),
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Load,
                        output: loaded,
                        args: vec![element_addr],
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
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_check_tracks_external_non_copy_raw_load_after_first_move() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let token_ty = types.register_named(
        "ExternalToken".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "ExternalToken".to_string(),
            type_params: vec![],
            fields: vec![types.i32()],
            field_names: vec!["raw".to_string()],
        },
    );
    let i32_ty = types.i32();
    let span = Span::dummy();
    let ptr = Place::local(String::from("p"), i32_ty);
    let first = Place::temporary(ResourceId(0), token_ty);
    let second = Place::temporary(ResourceId(1), token_ty);
    let resource = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            params: vec![ResourceLocal {
                name: "p".to_string(),
                ty: i32_ty,
                mutable: false,
                place: ptr.clone(),
            }],
            result: types.unit(),
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Load,
                        output: first,
                        args: vec![ptr.clone()],
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Load,
                        output: second,
                        args: vec![ptr],
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
            operation: ResourceCheckOperation::RawMemoryLoadCell,
            state: CellState::Moved,
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_cell_check_canonicalizes_raw_address_local_reads() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let ptr_value = Place::temporary(ResourceId(0), i32_ty);
    let ptr_local = Place::local(String::from("p"), i32_ty);
    let first_address = Place::temporary(ResourceId(1), i32_ty);
    let value = Place::temporary(ResourceId(2), i32_ty);
    let store_out = Place::temporary(ResourceId(3), unit_ty);
    let second_address = Place::temporary(ResourceId(4), i32_ty);
    let loaded = Place::temporary(ResourceId(5), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: ptr_value.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::DeclareLocal {
                place: ptr_local.clone(),
                mutable: false,
                initializer: Some(ptr_value),
                span,
            },
            ResourceOp::Read {
                source: ptr_local.clone(),
                output: first_address.clone(),
                span,
            },
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: value.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_out,
                args: vec![first_address, value],
                span,
            },
            ResourceOp::Read {
                source: ptr_local,
                output: second_address.clone(),
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded,
                args: vec![second_address],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_check_preserves_raw_address_returned_by_helper() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_param = Place::local("p".to_string(), i32_ty);
    let ptr = Place::temporary(ResourceId(0), i32_ty);
    let value = Place::temporary(ResourceId(1), i32_ty);
    let returned = Place::temporary(ResourceId(2), i32_ty);
    let store_out = Place::temporary(ResourceId(3), unit_ty);
    let loaded = Place::temporary(ResourceId(4), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "slot_id".to_string(),
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
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Literal,
                            output: ptr.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Literal,
                            output: value.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::Call {
                            output: returned.clone(),
                            target: ResourceCallTarget::User {
                                name: "slot_id".to_string(),
                                type_args: vec![],
                            },
                            args: vec![ptr.clone()],
                            effect: EffectOp::UserCall {
                                name: "slot_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Call,
                            output: returned.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Store,
                            output: store_out,
                            args: vec![returned, value],
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Load,
                            output: loaded,
                            args: vec![ptr],
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

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_check_preserves_initialized_raw_cells_returned_by_helper() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_alloc = Place::temporary(ResourceId(0), i32_ty);
    let value = Place::temporary(ResourceId(1), i32_ty);
    let store_out = Place::temporary(ResourceId(2), unit_ty);
    let returned = Place::temporary(ResourceId(3), i32_ty);
    let loaded = Place::temporary(ResourceId(4), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "make_slot".to_string(),
                params: vec![],
                result: i32_ty,
                effect: Effect::Impure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: helper_alloc.clone(),
                            args: vec![],
                            span,
                        },
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Literal,
                            output: value.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Store,
                            output: store_out,
                            args: vec![helper_alloc.clone(), value],
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(helper_alloc),
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
                effect: Effect::Impure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::Call {
                            output: returned.clone(),
                            target: ResourceCallTarget::User {
                                name: "make_slot".to_string(),
                                type_args: vec![],
                            },
                            args: vec![],
                            effect: EffectOp::UserCall {
                                name: "make_slot".to_string(),
                                effect: Effect::Impure,
                            },
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Load,
                            output: loaded,
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

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_check_preserves_initialized_raw_cells_returned_by_branch_helper() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.bool());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let condition = Place::temporary(ResourceId(0), bool_ty);
    let then_alloc = Place::temporary(ResourceId(1), i32_ty);
    let then_value = Place::temporary(ResourceId(2), i32_ty);
    let then_store = Place::temporary(ResourceId(3), unit_ty);
    let else_alloc = Place::temporary(ResourceId(4), i32_ty);
    let else_value = Place::temporary(ResourceId(5), i32_ty);
    let else_store = Place::temporary(ResourceId(6), unit_ty);
    let branch_output = Place::temporary(ResourceId(7), i32_ty);
    let call_output = Place::temporary(ResourceId(8), i32_ty);
    let local = Place::local("p".to_string(), i32_ty);
    let loaded = Place::temporary(ResourceId(9), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "make_slot".to_string(),
                params: vec![],
                result: i32_ty,
                effect: Effect::Impure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Literal,
                            output: condition.clone(),
                            ty: bool_ty,
                            span,
                        },
                        ResourceOp::Branch {
                            output: branch_output.clone(),
                            condition,
                            condition_fact: None,
                            then_ops: vec![
                                ResourceOp::RawMemory {
                                    operation: RawMemoryOp::Alloc,
                                    output: then_alloc.clone(),
                                    args: vec![],
                                    span,
                                },
                                ResourceOp::Expr {
                                    kind: ResourceExprKind::Literal,
                                    output: then_value.clone(),
                                    ty: i32_ty,
                                    span,
                                },
                                ResourceOp::RawMemory {
                                    operation: RawMemoryOp::Store,
                                    output: then_store,
                                    args: vec![then_alloc.clone(), then_value],
                                    span,
                                },
                            ],
                            then_value: then_alloc,
                            else_ops: vec![
                                ResourceOp::RawMemory {
                                    operation: RawMemoryOp::Alloc,
                                    output: else_alloc.clone(),
                                    args: vec![],
                                    span,
                                },
                                ResourceOp::Expr {
                                    kind: ResourceExprKind::Literal,
                                    output: else_value.clone(),
                                    ty: i32_ty,
                                    span,
                                },
                                ResourceOp::RawMemory {
                                    operation: RawMemoryOp::Store,
                                    output: else_store,
                                    args: vec![else_alloc.clone(), else_value],
                                    span,
                                },
                            ],
                            else_value: else_alloc,
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(branch_output),
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
                effect: Effect::Impure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::Call {
                            output: call_output.clone(),
                            target: ResourceCallTarget::User {
                                name: "make_slot".to_string(),
                                type_args: vec![],
                            },
                            args: vec![],
                            effect: EffectOp::UserCall {
                                name: "make_slot".to_string(),
                                effect: Effect::Impure,
                            },
                            span,
                        },
                        ResourceOp::DeclareLocal {
                            place: local.clone(),
                            mutable: false,
                            initializer: Some(call_output),
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Load,
                            output: loaded,
                            args: vec![local],
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
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "returned branch raw cells should remain initialized: {:#?}\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_rekeys_raw_cells_after_loading_raw_address_cell() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let buf = Place::temporary(ResourceId(0), i32_ty);
    let fill_len = Place::temporary(ResourceId(1), i32_ty);
    let fill_value = Place::temporary(ResourceId(2), i32_ty);
    let fill_out = Place::temporary(ResourceId(3), unit_ty);
    let header = Place::temporary(ResourceId(4), i32_ty);
    let store_out = Place::temporary(ResourceId(5), unit_ty);
    let loaded_buf = Place::temporary(ResourceId(6), i32_ty);
    let loaded_cell_address = loaded_buf.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset { bytes: Some(2) }),
        i32_ty,
    );
    let loaded = Place::temporary(ResourceId(7), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: buf.clone(),
                args: vec![],
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: fill_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: fill_value.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Fill,
                output: fill_out,
                args: vec![buf.clone(), fill_len, fill_value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: header.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_out,
                args: vec![header.clone(), buf],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded_buf,
                args: vec![header],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded,
                args: vec![loaded_cell_address],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_check_summarizes_initialized_cells_behind_returned_header_pointer() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_buf = Place::temporary(ResourceId(0), i32_ty);
    let fill_len = Place::temporary(ResourceId(1), i32_ty);
    let fill_value = Place::temporary(ResourceId(2), i32_ty);
    let fill_out = Place::temporary(ResourceId(3), unit_ty);
    let helper_header = Place::temporary(ResourceId(4), i32_ty);
    let store_buf = Place::temporary(ResourceId(5), unit_ty);
    let returned_header = Place::temporary(ResourceId(6), i32_ty);
    let header_local = Place::local("sc".to_string(), i32_ty);
    let loaded_buf = Place::temporary(ResourceId(7), i32_ty);
    let buf_local = Place::local("buf".to_string(), i32_ty);
    let loaded_cell_address = buf_local.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset { bytes: Some(1) }),
        i32_ty,
    );
    let loaded = Place::temporary(ResourceId(8), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "make_header".to_string(),
                params: vec![],
                result: i32_ty,
                effect: Effect::Impure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: helper_buf.clone(),
                            args: vec![],
                            span,
                        },
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Literal,
                            output: fill_len.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Literal,
                            output: fill_value.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Fill,
                            output: fill_out,
                            args: vec![helper_buf.clone(), fill_len, fill_value],
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: helper_header.clone(),
                            args: vec![],
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Store,
                            output: store_buf,
                            args: vec![helper_header.clone(), helper_buf],
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(helper_header),
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
                effect: Effect::Impure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::Call {
                            output: returned_header.clone(),
                            target: ResourceCallTarget::User {
                                name: "make_header".to_string(),
                                type_args: vec![],
                            },
                            args: vec![],
                            effect: EffectOp::UserCall {
                                name: "make_header".to_string(),
                                effect: Effect::Impure,
                            },
                            span,
                        },
                        ResourceOp::DeclareLocal {
                            place: header_local.clone(),
                            mutable: false,
                            initializer: Some(returned_header),
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Load,
                            output: loaded_buf.clone(),
                            args: vec![header_local],
                            span,
                        },
                        ResourceOp::DeclareLocal {
                            place: buf_local.clone(),
                            mutable: false,
                            initializer: Some(loaded_buf),
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Load,
                            output: loaded,
                            args: vec![loaded_cell_address],
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

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_check_preserves_raw_address_returned_by_function_value() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let fn_ty = types.function(vec![], vec![i32_ty], i32_ty, Effect::Pure);
    let span = Span::dummy();
    let helper_param = Place::local("p".to_string(), i32_ty);
    let ptr = Place::temporary(ResourceId(0), i32_ty);
    let value = Place::temporary(ResourceId(1), i32_ty);
    let callee = Place::temporary(ResourceId(2), fn_ty);
    let returned = Place::temporary(ResourceId(3), i32_ty);
    let store_out = Place::temporary(ResourceId(4), unit_ty);
    let loaded = Place::temporary(ResourceId(5), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "slot_id".to_string(),
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
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Literal,
                            output: ptr.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Literal,
                            output: value.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::FunctionValue {
                            output: callee.clone(),
                            name: "slot_id".to_string(),
                            effect: EffectOp::UserCall {
                                name: "slot_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::IndirectCall {
                            output: returned.clone(),
                            callee,
                            params: vec![i32_ty],
                            result: i32_ty,
                            args: vec![ptr.clone()],
                            effect: EffectOp::UserCall {
                                name: "slot_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::Expr {
                            kind: ResourceExprKind::IndirectCall,
                            output: returned.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Store,
                            output: store_out,
                            args: vec![returned, value],
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Load,
                            output: loaded,
                            args: vec![ptr],
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

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_check_preserves_raw_address_stored_in_aggregate_field() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let wrapper_ty = types.register_named(
        "PtrBox".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "PtrBox".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["ptr".to_string()],
        },
    );
    let span = Span::dummy();
    let ptr = Place::temporary(ResourceId(0), i32_ty);
    let wrapper = Place::temporary(ResourceId(1), wrapper_ty);
    let field = wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let field_ptr = Place::temporary(ResourceId(2), i32_ty);
    let value = Place::temporary(ResourceId(3), i32_ty);
    let store_out = Place::temporary(ResourceId(4), unit_ty);
    let loaded = Place::temporary(ResourceId(5), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: ptr.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Construct {
                output: wrapper.clone(),
                kind: AggregateKind::Struct {
                    name: "PtrBox".to_string(),
                    field_offsets: vec![0],
                },
                inputs: vec![ptr.clone()],
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Construct,
                output: wrapper,
                ty: wrapper_ty,
                span,
            },
            ResourceOp::Read {
                source: field,
                output: field_ptr.clone(),
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: value.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_out,
                args: vec![field_ptr, value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded,
                args: vec![ptr],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_check_raw_memory_call_does_not_consume_store_value_twice() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let ptr = Place::temporary(ResourceId(0), i32_ty);
    let value = Place::temporary(ResourceId(1), owned_ty);
    let store_out = Place::temporary(ResourceId(2), unit_ty);
    let loaded = Place::temporary(ResourceId(3), owned_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: ptr.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Construct {
                output: value.clone(),
                kind: AggregateKind::Struct {
                    name: "Owned".to_string(),
                    field_offsets: vec![0],
                },
                inputs: vec![],
                span,
            },
            ResourceOp::Call {
                output: store_out.clone(),
                target: ResourceCallTarget::User {
                    name: "store".to_string(),
                    type_args: vec![owned_ty],
                },
                args: vec![ptr.clone(), value.clone()],
                effect: EffectOp::UnsafeMemory {
                    operation: "store".to_string(),
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_out,
                args: vec![ptr.clone(), value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded,
                args: vec![ptr],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_check_moves_non_copy_raw_load_cell() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let ptr = Place::temporary(ResourceId(0), i32_ty);
    let value = Place::temporary(ResourceId(1), owned_ty);
    let store_out = Place::temporary(ResourceId(2), unit_ty);
    let first = Place::temporary(ResourceId(3), owned_ty);
    let second = Place::temporary(ResourceId(4), owned_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: ptr.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: value.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_out,
                args: vec![ptr.clone(), value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: first,
                args: vec![ptr.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: second,
                args: vec![ptr],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceCheckDiagnostic::CellUnavailable {
            function,
            operation: ResourceCheckOperation::RawMemoryLoadCell,
            place,
            state: CellState::Moved,
            ..
        } if function == "main"
            && place.ty == owned_ty
            && place.projections == vec![PlaceProjection::Deref]
    )));
}

#[test]
fn resource_ir_cell_check_reports_store_over_live_raw_cell() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let ptr = Place::temporary(ResourceId(0), i32_ty);
    let first_value = Place::temporary(ResourceId(1), owned_ty);
    let first_store = Place::temporary(ResourceId(2), unit_ty);
    let second_value = Place::temporary(ResourceId(3), owned_ty);
    let second_store = Place::temporary(ResourceId(4), unit_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: ptr.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: first_value.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: first_store,
                args: vec![ptr.clone(), first_value],
                span,
            },
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: second_value.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: second_store,
                args: vec![ptr.clone(), second_value],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceCheckDiagnostic::CellUnavailable {
            function,
            operation: ResourceCheckOperation::RawMemoryStoreCell,
            place,
            state: CellState::Initialized(ty),
            ..
        } if function == "main"
            && *ty == owned_ty
            && place.ty == owned_ty
            && place.projections == vec![PlaceProjection::Deref]
    )));
}

#[test]
fn resource_ir_cell_check_allows_dealloc_after_non_copy_raw_load() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let ptr = Place::temporary(ResourceId(0), i32_ty);
    let value = Place::temporary(ResourceId(1), owned_ty);
    let store_out = Place::temporary(ResourceId(2), unit_ty);
    let loaded = Place::temporary(ResourceId(3), owned_ty);
    let dealloc_out = Place::temporary(ResourceId(4), unit_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: ptr.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: value.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_out,
                args: vec![ptr.clone(), value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded,
                args: vec![ptr.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: dealloc_out,
                args: vec![ptr],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_check_reports_destructive_raw_storage_ops_over_live_cell() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let ptr = Place::temporary(ResourceId(0), i32_ty);
    let value = Place::temporary(ResourceId(1), owned_ty);
    let store_out = Place::temporary(ResourceId(2), unit_ty);
    let dealloc_out = Place::temporary(ResourceId(3), unit_ty);
    let realloc_out = Place::temporary(ResourceId(4), i32_ty);
    let fill_out = Place::temporary(ResourceId(5), unit_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: ptr.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: value.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_out,
                args: vec![ptr.clone(), value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: dealloc_out,
                args: vec![ptr.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Realloc,
                output: realloc_out,
                args: vec![ptr.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Fill,
                output: fill_out,
                args: vec![ptr.clone()],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    for operation in [
        ResourceCheckOperation::RawMemoryDeallocCell,
        ResourceCheckOperation::RawMemoryReallocCell,
        ResourceCheckOperation::RawMemoryFillCell,
    ] {
        assert!(report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                function,
                operation: actual,
                place,
                state: CellState::Initialized(ty),
                ..
            } if function == "main"
                && *actual == operation
                && *ty == owned_ty
                && place.ty == owned_ty
                && place.projections == vec![PlaceProjection::Deref]
        )));
    }
}

#[test]
fn resource_ir_cell_check_reports_bulk_copy_of_live_non_copy_raw_cells() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let dst = Place::temporary(ResourceId(0), i32_ty);
    let src = Place::temporary(ResourceId(1), i32_ty);
    let dst_value = Place::temporary(ResourceId(2), owned_ty);
    let src_value = Place::temporary(ResourceId(3), owned_ty);
    let dst_store = Place::temporary(ResourceId(4), unit_ty);
    let src_store = Place::temporary(ResourceId(5), unit_ty);
    let copy_out = Place::temporary(ResourceId(6), unit_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: dst.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: src.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: dst_value.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::Expr {
                kind: nepl_core::resource::ResourceExprKind::Literal,
                output: src_value.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: dst_store,
                args: vec![dst.clone(), dst_value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: src_store,
                args: vec![src.clone(), src_value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::BulkCopy,
                output: copy_out,
                args: vec![dst.clone(), src.clone()],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    for operation in [
        ResourceCheckOperation::RawMemoryBulkDestinationCell,
        ResourceCheckOperation::RawMemoryBulkSourceCell,
    ] {
        assert!(report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                function,
                operation: actual,
                state: CellState::Initialized(ty),
                ..
            } if function == "main" && *actual == operation && *ty == owned_ty
        )));
    }
}

#[test]
fn resource_ir_cell_check_allows_field_read_from_constructed_aggregate() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let wrapper_ty = types.register_named(
        "Wrapper".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "Wrapper".to_string(),
            type_params: vec![],
            fields: vec![owned_ty],
            field_names: vec!["owned".to_string()],
        },
    );
    let span = Span::dummy();
    let owned = Place::temporary(ResourceId(0), owned_ty);
    let wrapper = Place::temporary(ResourceId(1), wrapper_ty);
    let field = wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        owned_ty,
    );
    let moved_field = Place::temporary(ResourceId(2), owned_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Construct {
                output: owned.clone(),
                kind: AggregateKind::Struct {
                    name: "Owned".to_string(),
                    field_offsets: vec![0],
                },
                inputs: vec![],
                span,
            },
            ResourceOp::Construct {
                output: wrapper,
                kind: AggregateKind::Struct {
                    name: "Wrapper".to_string(),
                    field_offsets: vec![0],
                },
                inputs: vec![owned],
                span,
            },
            ResourceOp::Read {
                source: field,
                output: moved_field,
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_cell_check_reports_return_after_field_move() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    let wrapper_ty = types.register_named(
        "Wrapper".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "Wrapper".to_string(),
            type_params: vec![],
            fields: vec![owned_ty],
            field_names: vec!["owned".to_string()],
        },
    );
    let span = Span::dummy();
    let owned = Place::temporary(ResourceId(0), owned_ty);
    let wrapper = Place::temporary(ResourceId(1), wrapper_ty);
    let field = wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        owned_ty,
    );
    let moved_field = Place::temporary(ResourceId(2), owned_ty);
    let resource = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            params: vec![],
            result: wrapper_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::Construct {
                        output: owned.clone(),
                        kind: AggregateKind::Struct {
                            name: "Owned".to_string(),
                            field_offsets: vec![0],
                        },
                        inputs: vec![],
                        span,
                    },
                    ResourceOp::Construct {
                        output: wrapper.clone(),
                        kind: AggregateKind::Struct {
                            name: "Wrapper".to_string(),
                            field_offsets: vec![0],
                        },
                        inputs: vec![owned],
                        span,
                    },
                    ResourceOp::Move {
                        source: field,
                        output: moved_field,
                        span,
                    },
                ],
                terminator: ResourceTerminator::Return {
                    value: Some(wrapper.clone()),
                    span,
                },
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
            operation: ResourceCheckOperation::ReturnValue,
            place,
            state: CellState::Moved,
            ..
        } if function == "main" && place == &wrapper
    )));
}

#[test]
fn resource_ir_cell_check_reports_field_read_after_aggregate_move() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let wrapper_ty = types.register_named(
        "Wrapper".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "Wrapper".to_string(),
            type_params: vec![],
            fields: vec![owned_ty],
            field_names: vec!["owned".to_string()],
        },
    );
    let span = Span::dummy();
    let owned = Place::temporary(ResourceId(0), owned_ty);
    let wrapper = Place::temporary(ResourceId(1), wrapper_ty);
    let moved_wrapper = Place::temporary(ResourceId(2), wrapper_ty);
    let field = wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        owned_ty,
    );
    let field_read = Place::temporary(ResourceId(3), owned_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Construct {
                output: owned.clone(),
                kind: AggregateKind::Struct {
                    name: "Owned".to_string(),
                    field_offsets: vec![0],
                },
                inputs: vec![],
                span,
            },
            ResourceOp::Construct {
                output: wrapper.clone(),
                kind: AggregateKind::Struct {
                    name: "Wrapper".to_string(),
                    field_offsets: vec![0],
                },
                inputs: vec![owned],
                span,
            },
            ResourceOp::Move {
                source: wrapper,
                output: moved_wrapper,
                span,
            },
            ResourceOp::Read {
                source: field.clone(),
                output: field_read,
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
            place,
            state: CellState::Moved,
            ..
        } if function == "main" && place == &field
    )));
}

#[test]
fn resource_ir_owner_check_accepts_deallocated_alloc() {
    let types = TypeCtx::new();
    let span = Span::dummy();
    let module = raw_owner_module(types.unit(), types.i32(), span, 1);

    let resource = lower_hir_module_skeleton(&module);
    let report = check_resource_owner_obligations(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_owner_check_allows_raw_pointer_read_before_dealloc() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let p = Place::local("p".to_string(), i32_ty);
    let load_address = Place::temporary(ResourceId(0), i32_ty);
    let loaded = Place::temporary(ResourceId(1), i32_ty);
    let dealloc_address = Place::temporary(ResourceId(2), i32_ty);
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
            ResourceOp::Read {
                source: p.clone(),
                output: load_address.clone(),
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded,
                args: vec![load_address],
                span,
            },
            ResourceOp::Read {
                source: p,
                output: dealloc_address.clone(),
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(3), unit_ty),
                args: vec![dealloc_address],
                span,
            },
        ],
    );

    let report = check_resource_owner_obligations(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_owner_check_allows_unmanaged_fixed_address_dealloc_without_owner() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let unmanaged = Place::local("raw_address".to_string(), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![ResourceOp::RawMemory {
            operation: RawMemoryOp::Dealloc,
            output: Place::temporary(ResourceId(0), unit_ty),
            args: vec![unmanaged],
            span,
        }],
    );

    let report = check_resource_owner_obligations(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_owner_check_reports_stale_owned_alias_dealloc_after_free() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let p = Place::local("p".to_string(), i32_ty);
    let alias = Place::temporary(ResourceId(0), i32_ty);
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
            ResourceOp::Read {
                source: p.clone(),
                output: alias.clone(),
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(1), unit_ty),
                args: vec![p],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(2), unit_ty),
                args: vec![alias.clone()],
                span,
            },
        ],
    );

    let report = check_resource_owner_obligations(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerUnavailable {
            function,
            operation: ResourceOwnerOperation::Dealloc,
            place,
            state: OwnerState::NoFreeObligation,
            ..
        } if function == "main" && place == &alias
    )));
}

#[test]
fn resource_ir_owner_check_reports_assign_over_live_owner_leak() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let old_ptr = Place::temporary(ResourceId(0), i32_ty);
    let p = Place::local("p".to_string(), i32_ty);
    let new_ptr = Place::temporary(ResourceId(1), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: old_ptr.clone(),
                args: vec![],
                span,
            },
            ResourceOp::DeclareLocal {
                place: p.clone(),
                mutable: true,
                initializer: Some(old_ptr),
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: new_ptr.clone(),
                args: vec![],
                span,
            },
            ResourceOp::Assign {
                target: p.clone(),
                value: new_ptr,
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

    let report = check_resource_owner_obligations(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerLeaked {
            function,
            place,
            ..
        } if function == "main" && place == &p
    )));
}

#[test]
fn resource_ir_owner_check_reports_assign_over_aggregate_field_owner_leak() {
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
    let old_ptr = Place::temporary(ResourceId(0), i32_ty);
    let wrapper = Place::local("wrapper".to_string(), wrapper_ty);
    let wrapper_field = wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let new_ptr = Place::temporary(ResourceId(1), i32_ty);
    let replacement = Place::temporary(ResourceId(2), wrapper_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: old_ptr.clone(),
                args: vec![],
                span,
            },
            ResourceOp::Construct {
                output: wrapper.clone(),
                kind: AggregateKind::Struct {
                    name: "Wrapper".to_string(),
                    field_offsets: vec![0],
                },
                inputs: vec![old_ptr],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: new_ptr.clone(),
                args: vec![],
                span,
            },
            ResourceOp::Construct {
                output: replacement.clone(),
                kind: AggregateKind::Struct {
                    name: "Wrapper".to_string(),
                    field_offsets: vec![0],
                },
                inputs: vec![new_ptr],
                span,
            },
            ResourceOp::Assign {
                target: wrapper,
                value: replacement,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(3), unit_ty),
                args: vec![wrapper_field.clone()],
                span,
            },
        ],
    );

    let report = check_resource_owner_obligations(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerLeaked {
            function,
            place,
            ..
        } if function == "main" && place == &wrapper_field
    )));
}

#[test]
fn resource_ir_owner_check_reports_leaked_alloc() {
    let types = TypeCtx::new();
    let span = Span::dummy();
    let module = raw_owner_module(types.unit(), types.i32(), span, 0);

    let resource = lower_hir_module_skeleton(&module);
    let report = check_resource_owner_obligations(&resource, &types);
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
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerUnavailable {
            function,
            operation: ResourceOwnerOperation::Dealloc,
            place,
            state: OwnerState::Freed,
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

    let report = check_resource_owner_obligations(&resource, &types);
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

    let report = check_resource_owner_obligations(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_owner_summary_does_not_treat_bool_parameters_as_owners() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let helper_param = Place::local("flag".to_string(), bool_ty);
    let flag = Place::temporary(ResourceId(0), bool_ty);
    let returned = Place::temporary(ResourceId(1), bool_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "bool_id".to_string(),
                params: vec![ResourceLocal {
                    name: "flag".to_string(),
                    ty: bool_ty,
                    mutable: false,
                    place: helper_param.clone(),
                }],
                result: bool_ty,
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
                        ResourceOp::Expr {
                            output: flag.clone(),
                            kind: ResourceExprKind::Literal,
                            ty: bool_ty,
                            span,
                        },
                        ResourceOp::Call {
                            output: returned,
                            target: ResourceCallTarget::User {
                                name: "bool_id".to_string(),
                                type_args: vec![],
                            },
                            args: vec![flag],
                            effect: EffectOp::UserCall {
                                name: "bool_id".to_string(),
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

    let report = check_resource_owner_obligations(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_owner_check_does_not_freshen_recursive_copy_summary() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let helper_param = Place::local("p".to_string(), i32_ty);
    let condition = Place::temporary(ResourceId(0), bool_ty);
    let branch_output = Place::temporary(ResourceId(1), i32_ty);
    let recursive_output = Place::temporary(ResourceId(2), i32_ty);
    let main_arg = Place::temporary(ResourceId(3), i32_ty);
    let main_returned = Place::temporary(ResourceId(4), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "copy_loop".to_string(),
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
                    ops: vec![ResourceOp::Branch {
                        output: branch_output.clone(),
                        condition,
                        condition_fact: None,
                        then_ops: vec![],
                        then_value: helper_param.clone(),
                        else_ops: vec![ResourceOp::Call {
                            output: recursive_output.clone(),
                            target: ResourceCallTarget::User {
                                name: "copy_loop".to_string(),
                                type_args: vec![],
                            },
                            args: vec![helper_param],
                            effect: EffectOp::UserCall {
                                name: "copy_loop".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        }],
                        else_value: recursive_output,
                        span,
                    }],
                    terminator: ResourceTerminator::Return {
                        value: Some(branch_output),
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
                        output: main_returned,
                        target: ResourceCallTarget::User {
                            name: "copy_loop".to_string(),
                            type_args: vec![],
                        },
                        args: vec![main_arg],
                        effect: EffectOp::UserCall {
                            name: "copy_loop".to_string(),
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

    let report = check_resource_owner_obligations(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_owner_check_consumes_non_returned_owner_call_argument() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_param = Place::local("p".to_string(), i32_ty);
    let p = Place::temporary(ResourceId(0), i32_ty);
    let call_result = Place::temporary(ResourceId(1), unit_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "consume_owner".to_string(),
                params: vec![ResourceLocal {
                    name: "p".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: helper_param.clone(),
                }],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![ResourceOp::RawMemory {
                        operation: RawMemoryOp::Dealloc,
                        output: Place::temporary(ResourceId(10), unit_ty),
                        args: vec![helper_param.clone()],
                        span,
                    }],
                    terminator: ResourceTerminator::Return { value: None, span },
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
                            output: call_result,
                            target: ResourceCallTarget::User {
                                name: "consume_owner".to_string(),
                                type_args: vec![],
                            },
                            args: vec![p],
                            effect: EffectOp::UserCall {
                                name: "consume_owner".to_string(),
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

    let report = check_resource_owner_obligations(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_owner_check_lets_direct_raw_memory_op_consume_argument() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_param = Place::local("p".to_string(), i32_ty);
    let p = Place::temporary(ResourceId(0), i32_ty);
    let call_result = Place::temporary(ResourceId(1), unit_ty);
    let raw_result = Place::temporary(ResourceId(2), unit_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "dealloc_raw".to_string(),
                params: vec![ResourceLocal {
                    name: "p".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: helper_param.clone(),
                }],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![ResourceOp::RawMemory {
                        operation: RawMemoryOp::Dealloc,
                        output: Place::temporary(ResourceId(10), unit_ty),
                        args: vec![helper_param],
                        span,
                    }],
                    terminator: ResourceTerminator::Return { value: None, span },
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
                            output: call_result,
                            target: ResourceCallTarget::User {
                                name: "dealloc_raw".to_string(),
                                type_args: vec![],
                            },
                            args: vec![p.clone()],
                            effect: EffectOp::UnsafeMemory {
                                operation: "dealloc_raw".to_string(),
                            },
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Dealloc,
                            output: raw_result,
                            args: vec![p],
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

    let report = check_resource_owner_obligations(&resource, &types);
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

    let report = check_resource_owner_obligations(&resource, &types);
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
fn resource_ir_owner_check_reports_function_value_stored_in_aggregate_field_alloc_return_leak() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let fn_ty = types.function(vec![], vec![], i32_ty, Effect::Pure);
    let wrapper_ty = types.register_named(
        "CallbackBox".to_string(),
        TypeKind::Struct {
            doc: None,
            name: "CallbackBox".to_string(),
            type_params: vec![],
            fields: vec![fn_ty],
            field_names: vec!["make".to_string()],
        },
    );
    let span = Span::dummy();
    let helper_owned = Place::temporary(ResourceId(0), i32_ty);
    let function_value = Place::temporary(ResourceId(1), fn_ty);
    let wrapper = Place::temporary(ResourceId(2), wrapper_ty);
    let callee = wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        fn_ty,
    );
    let returned = Place::temporary(ResourceId(3), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "make_owner".to_string(),
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
                            output: function_value.clone(),
                            name: "make_owner".to_string(),
                            effect: EffectOp::UserCall {
                                name: "make_owner".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::Construct {
                            output: wrapper,
                            kind: AggregateKind::Struct {
                                name: "CallbackBox".to_string(),
                                field_offsets: vec![0],
                            },
                            inputs: vec![function_value],
                            span,
                        },
                        ResourceOp::IndirectCall {
                            output: returned.clone(),
                            callee,
                            params: vec![],
                            result: i32_ty,
                            args: vec![],
                            effect: EffectOp::UserCall {
                                name: "make_owner".to_string(),
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

    let report = check_resource_owner_obligations(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerLeaked {
            function,
            place,
            ..
        } if function == "main" && place == &returned
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

    let report = check_resource_owner_obligations(&resource, &types);
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

    let report = check_resource_owner_obligations(&resource, &types);
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
                    field_offsets: vec![0],
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

    let report = check_resource_owner_obligations(&resource, &types);
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
                    field_offsets: vec![0],
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

    let report = check_resource_owner_obligations(&resource, &types);
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
                                field_offsets: vec![0],
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

    let report = check_resource_owner_obligations(&resource, &types);
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
                                field_offsets: vec![0],
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

    let report = check_resource_owner_obligations(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_owner_check_reinitializes_self_update_aggregate_return() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *

struct Boxed:
    ptr <i32>

fn id_box <(Boxed)->Boxed> (box):
    box

fn main <()*>()> ():
    let mut box Boxed alloc_raw 4
    set box id_box box
    dealloc_raw field::get box "ptr" 4
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("id_box__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "self-update aggregate assignment must transfer returned owner projections back into the target: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_reinitializes_self_update_fresh_projection_return() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *

struct Boxed:
    ptr <i32>

fn replace_box <(Boxed)*>Boxed> (box):
    dealloc_raw field::get box "ptr" 4
    Boxed alloc_raw 4

fn main <()*>()> ():
    let mut box Boxed alloc_raw 4
    set box replace_box box
    dealloc_raw field::get box "ptr" 4
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("replace_box__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "self-update assignment must accept a consumed old projection replaced by a fresh returned projection: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_reinitializes_self_update_report_projection_returns() {
    let source = r#"
#entry main
#indent 4
#target std

#import "alloc/string" as *
#import "std/test" as *

fn main <()* >i32> ():
    let mut report test_report_new "probe"
    set report test_report_push report assert "initial" true
    let text <str> concat "prefix-" "suffix"
    set report test_report_push report assert_str_eq "concat after allocation" "prefix-suffix" text
    set report test_report_push report assert "after concat" true
    let shown test_report_print_stdout report
    test_report_exit_code shown
"#;

    let (module, types) = typecheck_resource_source_with_target(source, CompileTarget::Wasi);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("test_report_push__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "std/test report self-update must move returned projection owners back into the assigned local: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_reinitializes_self_update_multi_str_projection_return() {
    let source = r#"
#entry main
#indent 4
#target core
#import "alloc/string" as *
#import "core/field" as field

struct Triple:
    first <str>
    second <str>
    third <str>

fn consume_str <(str)->()> (s):
    len s
    ()

fn push_triple <(Triple)->Triple> (t):
    let first0 <str> field::get t "first"
    let second0 <str> field::get t "second"
    let third0 <str> field::get t "third"
    let second1 <str> concat second0 "x"
    let third1 <str> concat third0 "y"
    Triple first0 second1 third1

fn main <()*>()> ():
    let mut t Triple "a" "b" "c"
    set t push_triple t
    consume_str field::get t "first"
    consume_str field::get t "second"
    consume_str field::get t "third"
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("push_triple__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "self-update assignment must transfer all returned str projections: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_reinitializes_self_update_str_concat_projection() {
    let source = r#"
#entry main
#indent 4
#target core
#import "alloc/string" as *
#import "core/field" as field

struct Holder:
    text <str>

fn consume_str <(str)->()> (s):
    len s
    ()

fn holder_push <(Holder)->Holder> (h):
    let text0 <str> field::get h "text"
    let text1 <str> concat text0 "x"
    Holder text1

fn main <()*>()> ():
    let mut h Holder "a"
    set h holder_push h
    consume_str field::get h "text"
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("holder_push__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "self-update assignment must transfer str concat projection returns: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_refines_zero_alloc_result_branch() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/result" as *

fn alloc_result <()*>Result<i32, str>> ():
    let p <i32> alloc_raw 4
    if:
        eq p 0
        then:
            err<i32, str> "oom"
        else:
            ok<i32, str> p

fn main <()*>()> ():
    match alloc_result:
        Result::Ok p:
            dealloc_raw p 4
        Result::Err _e:
            ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("alloc_result__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "zero alloc failure branch must not leak the nonzero owner: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_refines_realloc_result_branches() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *

fn checked_realloc_load <()->i32> ():
    let p <i32> alloc_raw 4
    store_i32 p 123
    let grown <i32> realloc_raw p 4 8
    if:
        lt 0 grown
        then:
            let v <i32> load_i32 grown
            dealloc_raw grown 8
            v
        else:
            let v <i32> load_i32 p
            dealloc_raw p 4
            v

fn main <()->i32> ():
    checked_realloc_load
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("checked_realloc_load__")
                || function.starts_with("main__")
                || function == "main"
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "realloc success must move the owner to the new address and failure must keep the old owner: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/result" as *

struct Boxed:
    ptr <i32>

fn make_box <()*>Result<Boxed, str>> ():
    let p <i32> alloc_raw 4
    if:
        eq p 0
        then:
            err<Boxed, str> "oom"
        else:
            ok<Boxed, str> Boxed p

fn unwrap_box <(Result<Boxed, str>)*>Boxed> (r):
    match r:
        Result::Ok box:
            box
        Result::Err _e:
            #intrinsic "unreachable" <> ()

fn main <()*>()> ():
    let box <Boxed> unwrap_box make_box;
    dealloc_raw field::get box "ptr" 4
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("make_box__")
                || function.starts_with("unwrap_box__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "Result::Ok field owner must move through match bind and call return summary: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/result" as *

struct Boxed:
    ptr <i32>

struct OwnedErr:
    ptr <i32>

fn make_result <(bool)*>Result<Boxed, OwnedErr>> (ok_flag):
    if:
        cond:
            ok_flag
        then:
            let p <i32> alloc_raw 4
            if:
                cond:
                    eq p 0
                then:
                    #intrinsic "unreachable" <> ()
                else:
                    ok<Boxed, OwnedErr> Boxed p
        else:
            let e <i32> alloc_raw 4
            if:
                cond:
                    eq e 0
                then:
                    #intrinsic "unreachable" <> ()
                else:
                    err<Boxed, OwnedErr> OwnedErr e

fn unwrap_box <(Result<Boxed, OwnedErr>)*>Boxed> (r):
    match r:
        Result::Ok box:
            box
        Result::Err _e:
            #intrinsic "unreachable" <> ()

fn main <()*>()> ():
    let box <Boxed> unwrap_box make_result true;
    dealloc_raw field::get box "ptr" 4
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("make_result__")
                || function.starts_with("unwrap_box__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "unwrap-style non-returning Err arm must consume the owned Err payload at call boundary: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_preserves_alloc_ptr_raw_owner_return() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/result" as *

fn alloc_addr <()*>Result<i32, str>> ():
    match alloc_ptr<u8> 8:
        Result::Err _e:
            err<i32, str> "oom"
        Result::Ok node_ptr:
            ok<i32, str> mem_ptr_addr node_ptr

fn main <()*>()> ():
    match alloc_addr:
        Result::Ok p:
            dealloc_raw p 8
        Result::Err _e:
            ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("alloc_addr__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "alloc_ptr MemPtr owner must transfer through mem_ptr_addr into Result::Ok raw address: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_transfers_raw_owner_through_str_from_addr() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/result" as *

fn finish_region <(RegionToken<u8>)->str> (region):
    let base <MemPtr<u8>> get region "ptr"
    #intrinsic "str_from_addr_unchecked" <> (mem_ptr_addr base)

fn main <()* >str> ():
    match alloc_region_bytes<u8> 4:
        Result::Ok region:
            finish_region region
        Result::Err e:
            e
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("finish_region__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "str_from_addr_unchecked must transfer the raw allocation owner into returned str: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_accepts_concat_result_output_region_transfer() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/result" as *

fn main <()* >str> ():
    match concat_result "a" "b":
        Result::Ok s:
            s
        Result::Err e:
            e
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("concat_result__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "concat_result must move the output RegionToken owner into the returned Result::Ok str: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_accepts_string_from_mem_unchecked_result_transfer() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/result" as *

fn main <()* >str> ():
    let src <str> "abc"
    match string_from_mem_unchecked_result string_data_ptr src len src:
        Result::Ok copied:
            copied
        Result::Err e:
            e
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("string_from_mem_unchecked_result__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "string_from_mem_unchecked_result must move the output RegionToken owner into the returned Result::Ok str: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_keeps_bytebuf_owner_after_raw_address_view() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/diag/error" as *
#import "alloc/io" as *
#import "alloc/string" as *
#import "core/mem" as *
#import "core/result" as *

fn make_nonempty <()* >Result<ByteBuf, StdErrorKind>> ():
    match alloc_ptr<u8> 3:
        Result::Ok out:
            let out_raw <i32> mem_ptr_addr out
            let data <MemPtr<u8>> string_data_ptr "abc"
            let data_raw <i32> mem_ptr_addr data
            mem_copy out_raw data_raw 3
            Result<ByteBuf, StdErrorKind>::Ok io_bytebuf_from_owned_ptr out 3
        Result::Err _e:
            Result<ByteBuf, StdErrorKind>::Err StdErrorKind::OutOfMemory

fn main <()* >()> ():
    match make_nonempty ():
        Result::Ok bytes:
            io_bytebuf_free bytes
        Result::Err _e:
            ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("make_nonempty__")
                || function.starts_with("io_bytebuf_free__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "binding mem_ptr_addr output for a copy must not move the allocation owner before constructing ByteBuf: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_reports_leaked_conditional_owner_return() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *

fn maybe_alloc <(bool)->i32> (flag):
    if flag:
        alloc_raw 4
    else:
        0

fn main <()->()> ():
    let _p <i32> maybe_alloc true
    ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        !report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::DeclareInitializer,
                ..
            }
        )),
        "moving a conditional owner into a local must preserve the maybe obligation instead of reporting it as unavailable: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. }
                if function.starts_with("main__")
        )),
        "a conditional owner returned from a callee must remain an obligation in the caller: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_moves_stored_tail_owner_under_new_raw_node() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/result" as *

struct List:
    ptr <i32>

fn cons <(List)*>Result<List, str>> (tail):
    let tail_ptr <i32> field::get tail "ptr"
    let node <i32> alloc_raw 8
    if:
        lt 0 node
        then:
            store_i32 add node 4 tail_ptr
            ok<List, str> List node
        else:
            err<List, str> "oom"

fn main <()*>()> ():
    let tail_ptr <i32> alloc_raw 4
    let tail <List> List tail_ptr
    match cons tail:
        Result::Ok ready:
            let raw <i32> field::get ready "ptr"
            let next <i32> load_i32 add raw 4
            dealloc_raw next 4
            dealloc_raw raw 8
        Result::Err _e:
            ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("cons__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "owner stored into a raw node field must move under the new raw storage owner: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_returns_aggregate_with_raw_cell_owner_stored_through_field_alias() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *

struct HeaderBox:
    hdr <i32>

fn make_box <()*>HeaderBox> ():
    let hdr <i32> alloc_raw 16
    if:
        cond:
            eq hdr 0
        then:
            #intrinsic "unreachable" <> ()
        else:
            let entries <i32> alloc_raw 8
            if:
                cond:
                    eq entries 0
                then:
                    dealloc_raw hdr 16
                    #intrinsic "unreachable" <> ()
                else:
                    store_i32 add hdr 8 entries
                    HeaderBox hdr

fn replace_entries <(HeaderBox)*>HeaderBox> (box):
    let hdr <i32> field::get box "hdr"
    let old_entries <i32> load_i32 add hdr 8
    let new_entries <i32> alloc_raw 8
    if:
        cond:
            eq new_entries 0
        then:
            dealloc_raw old_entries 8
            dealloc_raw hdr 16
            #intrinsic "unreachable" <> ()
        else:
            dealloc_raw old_entries 8
            store_i32 add hdr 8 new_entries
            box

fn main <()*>()> ():
    let box0 <HeaderBox> make_box
    let box1 <HeaderBox> replace_entries box0
    let hdr <i32> field::get box1 "hdr"
    let entries <i32> load_i32 add hdr 8
    dealloc_raw entries 8
    dealloc_raw hdr 16
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("make_box__")
                || function.starts_with("replace_entries__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "raw cell owner stored through an aggregate field alias must be returned with the aggregate owner: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_keeps_raw_address_load_as_nonowning_view() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *

struct HeaderBox:
    hdr <i32>

fn make_box <()*>HeaderBox> ():
    let hdr <i32> alloc_raw 16
    if:
        cond:
            eq hdr 0
        then:
            #intrinsic "unreachable" <> ()
        else:
            let entries <i32> alloc_raw 8
            if:
                cond:
                    eq entries 0
                then:
                    dealloc_raw hdr 16
                    #intrinsic "unreachable" <> ()
                else:
                    store_i32 add hdr 8 entries
                    HeaderBox hdr

fn touch_entries <()*>HeaderBox> ():
    let ready <HeaderBox> make_box
    let hdr <i32> field::get ready "hdr"
    let entries <i32> load_i32 add hdr 8
    store_i32 entries 123
    ready

fn main <()*>()> ():
    let box1 <HeaderBox> touch_entries
    let hdr <i32> field::get box1 "hdr"
    let entries <i32> load_i32 add hdr 8
    dealloc_raw entries 8
    dealloc_raw hdr 16
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("make_box__")
                || function.starts_with("touch_entries__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "loading a raw address from an owned raw cell for probing must keep ownership with the aggregate: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
    assert!(
        resource.dump_text().contains("raw_address_view"),
        "address arithmetic used for probing must lower to an explicit non-owning raw address view:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_keeps_aggregate_raw_cell_root_through_loop_address_views() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *

struct HeaderBox:
    hdr <i32>

fn make_box <()*>HeaderBox> ():
    let hdr <i32> alloc_raw 16
    if:
        cond:
            eq hdr 0
        then:
            #intrinsic "unreachable" <> ()
        else:
            let entries <i32> alloc_raw 8
            if:
                cond:
                    eq entries 0
                then:
                    dealloc_raw hdr 16
                    #intrinsic "unreachable" <> ()
                else:
                    store_i32 add hdr 8 entries
                    HeaderBox hdr

fn slot_ptr <(i32)->i32> (entries):
    add entries 0

fn touch_entries_loop <()*>HeaderBox> ():
    let ready <HeaderBox> make_box
    let hdr <i32> field::get ready "hdr"
    let entries <i32> load_i32 add hdr 8
    let mut placed <bool> false
    while not placed:
        do:
            let slot <i32> slot_ptr entries
            store_i32 slot 123
            set placed true
    ready

fn main <()*>()> ():
    let box1 <HeaderBox> touch_entries_loop
    let hdr <i32> field::get box1 "hdr"
    let entries <i32> load_i32 add hdr 8
    dealloc_raw entries 8
    dealloc_raw hdr 16
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("make_box__")
                || function.starts_with("touch_entries_loop__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "loop address views must not re-root a returned aggregate raw cell owner under a local alias: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_moves_aliased_raw_cell_owner_into_enum_payload() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/result" as *

struct HeaderBox:
    hdr <i32>

fn make_box <()*>HeaderBox> ():
    let hdr <i32> alloc_raw 16
    if:
        cond:
            eq hdr 0
        then:
            #intrinsic "unreachable" <> ()
        else:
            let entries <i32> alloc_raw 8
            if:
                cond:
                    eq entries 0
                then:
                    dealloc_raw hdr 16
                    #intrinsic "unreachable" <> ()
                else:
                    store_i32 add hdr 8 entries
                    HeaderBox hdr

fn wrap_box <()*>Result<HeaderBox, i32>> ():
    let ready <HeaderBox> make_box
    let hdr <i32> field::get ready "hdr"
    let entries <i32> load_i32 add hdr 8
    store_i32 entries 7
    Result::Ok ready

fn main <()*>()> ():
    match wrap_box:
        Result::Ok box1:
            let hdr <i32> field::get box1 "hdr"
            let entries <i32> load_i32 add hdr 8
            dealloc_raw entries 8
            dealloc_raw hdr 16
        Result::Err _code:
            #intrinsic "unreachable" <> ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("make_box__")
                || function.starts_with("wrap_box__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "aliased raw cell owner must move with an aggregate when it is wrapped in an enum payload: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_consumes_only_used_aggregate_owner_projection() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *

struct Pair:
    left <i32>
    right <i32>

fn consume_left_return_right <(Pair)*>i32> (pair):
    dealloc_raw field::get pair "left" 4
    field::get pair "right"

fn main <()*>()> ():
    let left <i32> alloc_raw 4
    let right <i32> alloc_raw 4
    let pair <Pair> Pair left right
    let retained <i32> consume_left_return_right pair
    dealloc_raw retained 4
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("consume_left_return_right__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "aggregate call summary must consume only the owner projection not returned to the caller: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
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
fn resource_ir_borrow_check_releases_non_returned_call_argument_borrow_token() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_param = Place::local("token".to_string(), i32_ty);
    let x = Place::local("x".to_string(), i32_ty);
    let shared = Place::temporary(ResourceId(0), i32_ty);
    let call_out = Place::temporary(ResourceId(1), unit_ty);
    let replacement = Place::temporary(ResourceId(2), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "observe".to_string(),
                params: vec![ResourceLocal {
                    name: "token".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: helper_param,
                }],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![],
                    terminator: ResourceTerminator::Return { value: None, span },
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
                        ResourceOp::Borrow {
                            source: x.clone(),
                            output: shared.clone(),
                            kind: BorrowKind::Shared,
                            span,
                        },
                        ResourceOp::Call {
                            output: call_out,
                            target: ResourceCallTarget::User {
                                name: "observe".to_string(),
                                type_args: vec![],
                            },
                            args: vec![shared],
                            effect: EffectOp::UserCall {
                                name: "observe".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::Assign {
                            target: x,
                            value: replacement,
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

    let report = check_resource_borrow_lifetimes(&resource);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_borrow_check_keeps_local_call_argument_borrow_token_live() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_param = Place::local("token".to_string(), i32_ty);
    let x = Place::local("x".to_string(), i32_ty);
    let local_ref = Place::local("r".to_string(), i32_ty);
    let call_out = Place::temporary(ResourceId(0), unit_ty);
    let replacement = Place::temporary(ResourceId(1), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "observe".to_string(),
                params: vec![ResourceLocal {
                    name: "token".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: helper_param,
                }],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![],
                    terminator: ResourceTerminator::Return { value: None, span },
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
                        ResourceOp::Borrow {
                            source: x.clone(),
                            output: local_ref.clone(),
                            kind: BorrowKind::Shared,
                            span,
                        },
                        ResourceOp::Call {
                            output: call_out,
                            target: ResourceCallTarget::User {
                                name: "observe".to_string(),
                                type_args: vec![],
                            },
                            args: vec![local_ref],
                            effect: EffectOp::UserCall {
                                name: "observe".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::Assign {
                            target: x,
                            value: replacement,
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
}

#[test]
fn resource_ir_borrow_check_keeps_returned_call_argument_borrow_token_live() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_param = Place::local("token".to_string(), i32_ty);
    let x = Place::local("x".to_string(), i32_ty);
    let shared = Place::temporary(ResourceId(0), i32_ty);
    let returned = Place::temporary(ResourceId(1), i32_ty);
    let replacement = Place::temporary(ResourceId(2), i32_ty);
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
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::Borrow {
                            source: x.clone(),
                            output: shared.clone(),
                            kind: BorrowKind::Shared,
                            span,
                        },
                        ResourceOp::Call {
                            output: returned,
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
                        ResourceOp::Assign {
                            target: x,
                            value: replacement,
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
fn resource_ir_borrow_check_releases_non_returned_indirect_call_argument_borrow_token() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_param = Place::local("token".to_string(), i32_ty);
    let x = Place::local("x".to_string(), i32_ty);
    let shared = Place::temporary(ResourceId(0), i32_ty);
    let callee = Place::temporary(ResourceId(1), i32_ty);
    let call_out = Place::temporary(ResourceId(2), unit_ty);
    let replacement = Place::temporary(ResourceId(3), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "observe".to_string(),
                params: vec![ResourceLocal {
                    name: "token".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: helper_param,
                }],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![],
                    terminator: ResourceTerminator::Return { value: None, span },
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
                        ResourceOp::Borrow {
                            source: x.clone(),
                            output: shared.clone(),
                            kind: BorrowKind::Shared,
                            span,
                        },
                        ResourceOp::FunctionValue {
                            output: callee.clone(),
                            name: "observe".to_string(),
                            effect: EffectOp::UserCall {
                                name: "observe".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::IndirectCall {
                            output: call_out,
                            callee,
                            params: vec![i32_ty],
                            result: unit_ty,
                            args: vec![shared],
                            effect: EffectOp::UserCall {
                                name: "observe".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::Assign {
                            target: x,
                            value: replacement,
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

    let report = check_resource_borrow_lifetimes(&resource);
    assert_eq!(report.diagnostics, vec![]);
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
fn resource_ir_borrow_check_clears_stale_function_alias_on_assignment() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let fn_ty = types.function(vec![], vec![i32_ty], i32_ty, Effect::Pure);
    let span = Span::dummy();
    let x = Place::local("x".to_string(), i32_ty);
    let shared = Place::temporary(ResourceId(0), i32_ty);
    let known_function = Place::temporary(ResourceId(1), fn_ty);
    let f = Place::local("f".to_string(), fn_ty);
    let unknown_function = Place::temporary(ResourceId(2), fn_ty);
    let returned = Place::temporary(ResourceId(3), i32_ty);
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
                    ResourceOp::FunctionValue {
                        output: known_function.clone(),
                        name: "known_no_token_return".to_string(),
                        effect: EffectOp::Pure,
                        span,
                    },
                    ResourceOp::DeclareLocal {
                        place: f.clone(),
                        mutable: true,
                        initializer: Some(known_function),
                        span,
                    },
                    ResourceOp::Assign {
                        target: f.clone(),
                        value: unknown_function,
                        span,
                    },
                    ResourceOp::IndirectCall {
                        output: returned.clone(),
                        callee: f,
                        params: vec![i32_ty],
                        result: i32_ty,
                        args: vec![shared],
                        effect: EffectOp::Unknown {
                            reason: "assigned unknown callback".to_string(),
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
fn resource_ir_borrow_check_rejects_assign_over_borrowed_field_projection() {
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
            field_names: vec!["value".to_string()],
        },
    );
    let span = Span::dummy();
    let wrapper = Place::local("wrapper".to_string(), wrapper_ty);
    let field = wrapper.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let shared = Place::temporary(ResourceId(0), i32_ty);
    let replacement = Place::temporary(ResourceId(1), wrapper_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Borrow {
                source: field,
                output: shared,
                kind: BorrowKind::Shared,
                span,
            },
            ResourceOp::Assign {
                target: wrapper,
                value: replacement,
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
            active: BorrowState::Shared { .. },
            ..
        } if function == "main"
    )));
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
                    field_offsets: vec![0],
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
                condition_fact: None,
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
                    field_offsets: vec![0],
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
                condition_fact: None,
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

    let report = check_resource_owner_obligations(&resource, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceOwnerDiagnostic::OwnerUnavailable {
            function,
            operation: ResourceOwnerOperation::Dealloc,
            state: OwnerState::MaybeFreed { .. },
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
                condition_fact: None,
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

#[test]
fn resource_ir_cell_check_preserves_raw_cell_across_untouched_loop() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> 16
    store<LocalToken> p LocalToken @token_id
    let mut i <i32> 0
    while lt i 2:
        do:
            set i add i 1
    let out <LocalToken> load<LocalToken> p
    i
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "main" || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.is_empty(),
        "loop must preserve untouched raw cell diagnostics: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_realloc_transfers_copy_raw_cells() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *

fn checked_realloc_slot <()->i32> ():
    let slot <i32> alloc_raw 4
    store_i32 slot 99
    let grown <i32> realloc_raw slot 4 8
    if:
        lt 0 grown
        then:
            let v <i32> load_i32 grown
            dealloc_raw grown 8
            v
        else:
            let v <i32> load_i32 slot
            dealloc_raw slot 4
            v

fn main <()->i32> ():
    checked_realloc_slot
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let realloc_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function.starts_with("checked_realloc_slot__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        realloc_diagnostics.is_empty(),
        "checked realloc success must transfer initialized Copy raw cells and failure must keep the old cells: {:#?}\nresource:\n{}",
        realloc_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_raw_fill_helpers_initialize_copy_cells() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *

fn fill_bytes <()->i32> ():
    let p <i32> alloc_raw 8
    memset_u8 p 8 65
    let a <i32> load_u8 add p 0
    let b <i32> load_u8 add p 7
    dealloc_raw p 8
    add a b

fn fill_words <()->i32> ():
    let p <i32> alloc_raw 16
    fill_i32 p 4 42
    let a <i32> load_i32 add p 0
    let b <i32> load_i32 add p 12
    dealloc_raw p 16
    add a b

fn main <()->i32> ():
    add fill_bytes fill_words
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let fill_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function.starts_with("fill_bytes__")
                        || function.starts_with("fill_words__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        fill_diagnostics.is_empty(),
        "raw fill helpers must initialize Copy raw cells for caller-visible loads: {:#?}\nresource:\n{}",
        fill_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_external_fd_read_initializes_iovec_buffers() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let fd = Place::temporary(ResourceId(0), i32_ty);
    let iov_count = Place::temporary(ResourceId(1), i32_ty);
    let buf = Place::temporary(ResourceId(2), i32_ty);
    let iov = Place::temporary(ResourceId(3), i32_ty);
    let nread = Place::temporary(ResourceId(4), i32_ty);
    let store_buf = Place::temporary(ResourceId(5), unit_ty);
    let iov_len_cell = iov.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset { bytes: Some(4) }),
        i32_ty,
    );
    let store_iov_len = Place::temporary(ResourceId(6), unit_ty);
    let zero = Place::temporary(ResourceId(7), i32_ty);
    let store_nread = Place::temporary(ResourceId(8), unit_ty);
    let errno = Place::temporary(ResourceId(9), i32_ty);
    let loaded_nread = Place::temporary(ResourceId(10), i32_ty);
    let loaded_byte = Place::temporary(ResourceId(11), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: fd.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: iov_count.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: zero.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: buf.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: iov.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: nread.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_buf,
                args: vec![iov.clone(), buf.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_iov_len,
                args: vec![iov_len_cell, iov_count.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_nread,
                args: vec![nread.clone(), zero],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("fd_read"),
                },
                args: vec![fd, iov, iov_count, nread.clone()],
                effect: EffectOp::ExternalIo {
                    operation: String::from("fd_read"),
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded_nread,
                args: vec![nread],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded_byte,
                args: vec![buf],
                span,
            },
        ],
    );
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "fd_read must initialize nread and iovec-backed byte cells: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_fd_pwrite_initializes_nwritten_not_offset() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let fd = Place::temporary(ResourceId(0), i32_ty);
    let iov_count = Place::temporary(ResourceId(1), i32_ty);
    let iov = Place::temporary(ResourceId(2), i32_ty);
    let offset = Place::temporary(ResourceId(3), i32_ty);
    let nwritten = Place::temporary(ResourceId(4), i32_ty);
    let errno = Place::temporary(ResourceId(5), i32_ty);
    let loaded_nwritten = Place::temporary(ResourceId(6), i32_ty);
    let loaded_offset = Place::temporary(ResourceId(7), i32_ty);
    let offset_cell = offset
        .clone()
        .with_projection(PlaceProjection::Deref, i32_ty);
    let nwritten_cell = nwritten
        .clone()
        .with_projection(PlaceProjection::Deref, i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: fd.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: iov_count.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: offset.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: iov.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: nwritten.clone(),
                args: vec![],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("fd_pwrite"),
                },
                args: vec![fd, iov, iov_count, offset.clone(), nwritten.clone()],
                effect: EffectOp::ExternalIo {
                    operation: String::from("fd_pwrite"),
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded_nwritten,
                args: vec![nwritten],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded_offset,
                args: vec![offset],
                span,
            },
        ],
    );
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        !report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                place,
                ..
            } if place == &nwritten_cell
        )),
        "fd_pwrite must initialize the nwritten out cell: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                place,
                state: CellState::Uninit,
                ..
            } if place == &offset_cell
        )),
        "fd_pwrite must not treat the scalar offset argument as an out pointer: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_returned_raw_header_preserves_initialized_pointee() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *

fn make_header <()->i32> ():
    let data <i32> alloc_raw 4
    store_i32 data 65
    let header <i32> alloc_raw 4
    store_i32 header data
    header

fn main <()->i32> ():
    let header <i32> make_header
    let data <i32> load_i32 header
    let value <i32> load_i32 data
    dealloc_raw data 4
    dealloc_raw header 4
    value
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.is_empty(),
        "returned raw header must carry initialized raw cells and pointer-valued cell aliases: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_raw_fill_does_not_initialize_non_copy_cell() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let ptr = Place::temporary(ResourceId(0), i32_ty);
    let len = Place::temporary(ResourceId(1), i32_ty);
    let fill_value = Place::temporary(ResourceId(2), i32_ty);
    let fill_out = Place::temporary(ResourceId(3), unit_ty);
    let loaded = Place::temporary(ResourceId(4), owned_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: ptr.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: fill_value.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Fill,
                output: fill_out,
                args: vec![ptr.clone(), len, fill_value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded,
                args: vec![ptr],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                place,
                state: CellState::Uninit,
                ..
            } if place.ty == owned_ty
        )),
        "raw fill must not construct a non-Copy cell: {:#?}",
        report.diagnostics
    );
}

#[test]
fn resource_ir_cell_check_preserves_mem_ptr_disjoint_offsets() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let q <MemPtr<LocalToken>> mem_ptr_add<LocalToken> p 8
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    store<LocalToken> mem_ptr_addr q LocalToken @token_id
    let a <LocalToken> load<LocalToken> mem_ptr_addr p
    let b <LocalToken> load<LocalToken> mem_ptr_addr q
    0
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "main" || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.is_empty(),
        "MemPtr literal disjoint offsets must keep separate raw cells: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_preserves_mem_ptr_alias_after_region_token() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let token <RegionToken<LocalToken>> region_new<LocalToken> p size_of<LocalToken>
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let a <LocalToken> load<LocalToken> mem_ptr_addr p
    let r <Result<(),str>> dealloc_region<LocalToken> token
    0
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "main" || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.is_empty(),
        "RegionToken construction must not break MemPtr raw alias loads: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_preserves_region_token_ptr_helper_alias_after_token_move() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn token_ptr <(RegionToken<LocalToken>)->MemPtr<LocalToken>> (token):
    get token "ptr"

fn main <()->i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let token <RegionToken<LocalToken>> region_new<LocalToken> p size_of<LocalToken>
    store<LocalToken> mem_ptr_addr p LocalToken @token_id
    let q <MemPtr<LocalToken>> token_ptr token
    let a <LocalToken> load<LocalToken> mem_ptr_addr q
    0
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "main" || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.is_empty(),
        "RegionToken helper-derived MemPtr must keep pointee cell state separate from token value moves: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
    assert!(
        resource.dump_text().contains("raw_address_alias"),
        "RegionToken ptr helper must expose raw address alias:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_preserves_str_addr_helper_parameter_raw_load() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *

fn string_addr <(str)->i32> (s):
    #intrinsic "str_addr" <> (s)

fn read_len <(str)->i32> (s):
    load_i32 string_addr s

fn main <()->i32> ():
    read_len "abc"
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let read_len_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "read_len" || function.starts_with("read_len__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        read_len_diagnostics.is_empty(),
        "str_addr helper must alias initialized string backing storage: {:#?}\nresource:\n{}",
        read_len_diagnostics,
        resource.dump_text()
    );
    assert!(
        resource.dump_text().contains("raw_address_alias"),
        "str_addr helper lowering must expose a raw address alias:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_preserves_direct_arithmetic_external_raw_load() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/math" as *
#import "core/result" as *

fn read_check <(i32,i32)->i32> (checks_data, i):
    let r <Result<(),str>> load<Result<(),str>> add checks_data mul i size_of<Result<(),str>>
    0

fn main <()->i32> ():
    read_check 16 0
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let read_check_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "read_check" || function.starts_with("read_check__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        read_check_diagnostics.is_empty(),
        "direct raw address arithmetic must preserve the external storage root: {:#?}\nresource:\n{}",
        read_check_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_preserves_external_raw_address_field_load() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *

struct List<.T>:
    ptr <i32>

fn read_head <.T> <(List<.T>)->.T> (lst):
    let lst_ptr <i32> field::get lst "ptr"
    load<.T> lst_ptr

fn main <()->i32> ():
    0
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let read_head_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "read_head" || function.starts_with("read_head__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        read_head_diagnostics.is_empty(),
        "raw address field read from an external aggregate must alias initialized backing storage: {:#?}\nresource:\n{}",
        read_head_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_preserves_result_payload_raw_address_field() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/result" as *

struct Boxed:
    ptr <i32>

fn pass <(Boxed)->Result<Boxed, str>> (box):
    ok<Boxed, str> box

fn read_after_result <(Boxed)->i32> (box):
    match pass box:
        Result::Ok ready:
            let ptr <i32> field::get ready "ptr"
            load_i32 ptr
        Result::Err _e:
            0

fn main <()->i32> ():
    0
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "read_after_result" || function.starts_with("read_after_result__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "Result::Ok payload bind must preserve raw address field aliases: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_preserves_literal_arithmetic_helper_zero_offset() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn slot_ptr <.T,.V> <(i32,i32)->i32> (base, idx):
    add base mul idx add size_of<.T> size_of<.V>

fn main <()->i32> ():
    let p <i32> 16
    store<LocalToken> slot_ptr<LocalToken,i32> p 0 LocalToken @token_id
    store_i32 add p size_of<LocalToken> 123
    let a <LocalToken> load<LocalToken> p
    dealloc_raw p 16
    0
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "main" || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.is_empty(),
        "literal zero raw address helper offset must alias the base address: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
    assert!(
        resource.dump_text().contains("raw_address_view"),
        "explicit arithmetic helper offsets, including literal zero, must be represented as raw address views:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_keeps_unknown_arithmetic_helper_offset_conservative() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn choose_offset <(bool)->i32> (flag):
    if flag 0 size_of<LocalToken>

fn slot_ptr <.T,.V> <(i32,i32)->i32> (base, idx):
    add base mul idx add size_of<.T> size_of<.V>

fn main <()->i32> ():
    let p <i32> 16
    let off <i32> choose_offset true
    store<LocalToken> p LocalToken @token_id
    store<LocalToken> slot_ptr<LocalToken,i32> p off LocalToken @token_id
    0
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "main" || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        !main_diagnostics.is_empty(),
        "non-literal raw address helper offset must remain conservative:\nresource:\n{}",
        resource.dump_text()
    );
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
