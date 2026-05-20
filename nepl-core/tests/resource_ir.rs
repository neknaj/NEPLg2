use nepl_core::ast::Effect;
use nepl_core::diagnostic::Severity;
use nepl_core::diagnostic_codes::{
    DiagnosticCode, ResourceDiagnosticCode, ResourceLowerDiagnosticCode,
};
use nepl_core::hir::{
    FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirLine, HirModule, HirParam,
};
use nepl_core::loader::Loader;
use nepl_core::resource::{
    check_hir_resource_safety_shadow, check_resource_borrow_lifetimes,
    check_resource_effect_boundaries, check_resource_effect_boundaries_typed,
    check_resource_initialized_moves, check_resource_owner_obligations,
    compare_hir_resource_lowering, compare_hir_resource_lowering_typed,
    compute_resource_drop_elaboration_plan, compute_resource_drop_plan, lower_hir_module,
    lower_hir_module_skeleton, resolve_resource_drop_point_assignment,
    resolve_resource_drop_point_end_scope, resolve_resource_drop_point_path, AggregateKind,
    BorrowKind, BorrowState, CellState, CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp,
    CollectionSlotLifecycleRefutation, CollectionSlotState, EffectOp, ExternalIoOp, NondetOp,
    OwnerState, Place, PlaceProjection, PlaceRoot, RawAddressAliasKind, RawAddressViewKind,
    RawMemoryOp, ResourceAutoDrop, ResourceAutoDropKind, ResourceBlock, ResourceBlockId,
    ResourceBorrowDiagnostic, ResourceBorrowOperation, ResourceCallTarget, ResourceCheckDeferred,
    ResourceCheckDiagnostic, ResourceCheckOperation, ResourceCheckReport, ResourceConditionFact,
    ResourceCoverageDiagnostic, ResourceCoverageKind, ResourceCoveragePlaceOperation,
    ResourceDropElaborationHirBridgeError, ResourceDropElaborationPlanError, ResourceDropPoint,
    ResourceDropPointPath, ResourceDropPointResolutionError, ResourceDropPointStep,
    ResourceDropRequirement, ResourceEffectBoundaryDiagnostic, ResourceEffectCallKind,
    ResourceExprKind, ResourceFunction, ResourceFunctionCheck, ResourceI32RelationOp, ResourceId,
    ResourceLocal, ResourceModule, ResourceOffset, ResourceOp, ResourceOwnerDiagnostic,
    ResourceOwnerOperation, ResourceTerminator, StorageOrigin, UnknownEffectReason,
};
use nepl_core::source_map::CompilerMemoryType;
use nepl_core::span::{FileId, Span};
use nepl_core::types::{TypeCtx, TypeId, TypeKind};
use nepl_core::{BuildProfile, CompileOptions, CompileTarget};
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
        checked
            .diagnostics
            .iter()
            .all(|diagnostic| !matches!(diagnostic.severity, Severity::Error)),
        "typecheck diagnostics: {:#?}",
        checked.diagnostics
    );
    (checked.module.expect("typechecked module"), checked.types)
}

fn compile_resource_source_with_target(
    source: &str,
    target: CompileTarget,
) -> Result<(), nepl_core::CoreError> {
    compile_resource_source_with_path(source, target, PathBuf::from("/virtual/entry.nepl"))
}

fn compile_resource_source_with_raw_boundary(
    source: &str,
    target: CompileTarget,
) -> Result<(), nepl_core::CoreError> {
    compile_resource_source_as_compiler_owned(source, target)
}

fn compile_resource_source_as_compiler_owned(
    source: &str,
    target: CompileTarget,
) -> Result<(), nepl_core::CoreError> {
    compile_resource_source_with_path(
        source,
        target,
        stdlib_root().join("__resource_ir_boundary_test.nepl"),
    )
}

fn compile_resource_source_with_path(
    source: &str,
    target: CompileTarget,
    path: PathBuf,
) -> Result<(), nepl_core::CoreError> {
    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline(path, source.to_string())
        .expect("load source with stdlib");
    nepl_core::compile_module_with_source_map(
        loaded.module,
        Some(&loaded.source_map),
        CompileOptions {
            target: Some(target),
            verbose: false,
            profile: Some(BuildProfile::Debug),
        },
    )
    .map(|_| ())
}

fn assert_compile_resource_source_reports_code(
    source: &str,
    target: CompileTarget,
    expected_code: &str,
) {
    let err = compile_resource_source_with_target(source, target)
        .expect_err("source should be rejected by Resource IR static checks");
    let nepl_core::CoreError::Diagnostics(diagnostics) = err else {
        panic!("expected diagnostics error");
    };
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code.as_str() == expected_code),
        "expected diagnostic code {expected_code}, diagnostics: {diagnostics:#?}"
    );
}

fn type_ctx_with_copy_i32() -> TypeCtx {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.i32());
    types
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
            origin_name: "main".to_string(),
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
            "    expr LiteralI32(1) out=tmp1:t1 ty=t1 span=0:0-0\n",
            "    declare %x:t1 mut=true init=tmp1:t1 span=0:0-0\n",
            "    expr Let out=tmp2:t0 ty=t0 span=0:0-0\n",
            "    read %x:t1 -> tmp3:t1 span=0:0-0\n",
            "    expr LocalRead out=tmp3:t1 ty=t1 span=0:0-0\n",
            "    end_scope [%x:t1] result=tmp3:t1 span=0:0-0\n",
            "    end_scope [%arg:t1] result=tmp3:t1 span=0:0-0\n",
            "    terminator return tmp3:t1 span=0:0-0\n"
        )
    );
}

#[test]
fn resource_ir_lowering_preserves_scope_local_declaration_order() {
    let unit_ty = TypeId(0);
    let i32_ty = TypeId(1);
    let span = Span::dummy();
    let let_i32 = |name: &str, value: i32| HirLine {
        expr: HirExpr {
            ty: unit_ty,
            kind: HirExprKind::Let {
                name: name.to_string(),
                mutable: false,
                value: Box::new(HirExpr {
                    ty: i32_ty,
                    kind: HirExprKind::LiteralI32(value),
                    span,
                }),
            },
            span,
        },
        drop_result: true,
    };
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            origin_name: "main".to_string(),
            func_ty: TypeId(2),
            params: vec![],
            result: i32_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![
                    let_i32("p", 1),
                    let_i32("left", 2),
                    HirLine {
                        expr: HirExpr {
                            ty: i32_ty,
                            kind: HirExprKind::Var("left".to_string()),
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
    let locals = resource.functions[0].blocks[0]
        .ops
        .iter()
        .find_map(|op| match op {
            ResourceOp::EndScope { locals, .. } if !locals.is_empty() => Some(locals),
            _ => None,
        })
        .expect("block end_scope should contain declared locals");
    let names = locals
        .iter()
        .map(|place| match &place.root {
            PlaceRoot::Local(name) => name.as_str(),
            other => panic!("expected local place, got {other:?}"),
        })
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["p", "left"]);
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
            origin_name: "main".to_string(),
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
            origin_name: "main".to_string(),
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
fn resource_ir_layout_intrinsics_use_shared_core_intrinsic_kind() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/math" as *
#import "core/mem" as *

struct Pair:
    left <i32>
    right <i32>

fn pair_size <()->i32> ():
    size_of<Pair>

fn main <()->i32> ():
    let size <i32> pair_size
    let align <i32> align_of<Pair>
    add size align
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let dump = resource.dump_text();
    assert!(
        dump.contains("expr LiteralI32(8)"),
        "size_of<Pair> wrapper call must lower to a Resource IR scalar fact:\n{}",
        dump
    );
    assert!(
        dump.contains("expr LiteralI32(4)"),
        "align_of<Pair> intrinsic must lower to a Resource IR scalar fact:\n{}",
        dump
    );
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
            origin_name: "main".to_string(),
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
    assert!(dump.contains("effect internal_alloc(alloc)"));
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
            origin_name: "main".to_string(),
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
            } if function == "main"
                && *operation == ResourceCoveragePlaceOperation::BorrowSource
                && *diagnostic_span == span
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
fn resource_lowering_diagnostics_own_lower_incomplete_code() {
    let expected = DiagnosticCode::Resource(ResourceDiagnosticCode::Lower(
        ResourceLowerDiagnosticCode::Incomplete,
    ));
    let span = Span::dummy();

    let coverage = ResourceCoverageDiagnostic::CountMismatch {
        function: "main".to_string(),
        kind: ResourceCoverageKind::Borrow,
        hir: 1,
        resource: 0,
        span,
    };
    assert_eq!(coverage.diagnostic_code(), expected);
    assert_eq!(
        ResourceCoveragePlaceOperation::BorrowSource.as_str(),
        "borrow.source"
    );

    let plan_error = ResourceDropElaborationPlanError::MissingFunctionCheck {
        function: "main".to_string(),
    };
    assert_eq!(plan_error.diagnostic_code(), expected);

    let bridge_error = ResourceDropElaborationHirBridgeError::MissingSourceFunction {
        function: "main$T".to_string(),
        origin_name: "main".to_string(),
    };
    assert_eq!(bridge_error.diagnostic_code(), expected);
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
            origin_name: "main".to_string(),
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
            origin_name: "main".to_string(),
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
    assert_eq!(report.functions[0].counts.internal_memory_ops.alloc, 1);
    assert_eq!(report.functions[0].counts.internal_memory_ops.total(), 1);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function,
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_effect_check_propagates_internal_alloc_return_summary() {
    let i32_ty = TypeId(1);
    let span = Span::dummy();
    let size = Place::temporary(ResourceId(0), i32_ty);
    let raw = Place::temporary(ResourceId(1), i32_ty);
    let from_call = Place::temporary(ResourceId(2), i32_ty);
    let module = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "make_raw".to_string(),
                origin_name: "make_raw".to_string(),
                type_params: Vec::new(),
                params: vec![],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![
                        ResourceOp::Expr {
                            kind: ResourceExprKind::LiteralI32(4),
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
                    ],
                    terminator: ResourceTerminator::Return {
                        value: Some(raw),
                        span,
                    },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                origin_name: "main".to_string(),
                type_params: Vec::new(),
                params: vec![],
                result: i32_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![ResourceOp::Call {
                        output: from_call.clone(),
                        target: ResourceCallTarget::User {
                            name: "make_raw".to_string(),
                            type_args: vec![],
                        },
                        args: vec![],
                        effect: EffectOp::Pure,
                        span,
                    }],
                    terminator: ResourceTerminator::Return {
                        value: Some(from_call),
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
fn resource_ir_cell_check_fill_bytes_initializes_u8_cells() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.u8());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let buf = Place::temporary(ResourceId(0), i32_ty);
    let len = Place::temporary(ResourceId(1), i32_ty);
    let value = Place::temporary(ResourceId(2), i32_ty);
    let fill_out = Place::temporary(ResourceId(3), unit_ty);
    let loaded = Place::temporary(ResourceId(4), i32_ty);
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
                kind: ResourceExprKind::LiteralI32(4),
                output: len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(7),
                output: value.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::FillBytes,
                output: fill_out,
                args: vec![buf.clone(), len, value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::LoadU8,
                output: loaded,
                args: vec![buf],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(
        report.diagnostics,
        vec![],
        "FillBytes must initialize u8 raw cells for LoadU8 even though LoadU8 returns i32:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_fill_bytes_does_not_initialize_i32_cells() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.u8());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let buf = Place::temporary(ResourceId(0), i32_ty);
    let len = Place::temporary(ResourceId(1), i32_ty);
    let value = Place::temporary(ResourceId(2), i32_ty);
    let fill_out = Place::temporary(ResourceId(3), unit_ty);
    let loaded = Place::temporary(ResourceId(4), i32_ty);
    let loaded_cell = buf.clone().with_projection(PlaceProjection::Deref, i32_ty);
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
                kind: ResourceExprKind::LiteralI32(4),
                output: len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(7),
                output: value.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::FillBytes,
                output: fill_out,
                args: vec![buf.clone(), len, value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded,
                args: vec![buf],
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
            } if place == &loaded_cell
        )),
        "FillBytes must not prove an i32 typed cell initialized only because the byte value is i32: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_store_u8_initializes_u8_cells() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.u8());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let buf = Place::temporary(ResourceId(0), i32_ty);
    let value = Place::temporary(ResourceId(1), i32_ty);
    let store_out = Place::temporary(ResourceId(2), unit_ty);
    let loaded = Place::temporary(ResourceId(3), i32_ty);
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
                kind: ResourceExprKind::LiteralI32(7),
                output: value.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::StoreU8,
                output: store_out,
                args: vec![buf.clone(), value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::LoadU8,
                output: loaded,
                args: vec![buf],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(
        report.diagnostics,
        vec![],
        "StoreU8 must initialize a u8 raw cell even though the stored value expression is i32:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_effect_check_preserves_raw_slot_pointer_alias_stored_in_aggregate_field() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let wrapper_ty = types.register_named(
        "PtrBox".to_string(),
        TypeKind::Struct {
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
                        source_name: "p".to_string(),
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
                origin_name: "slot_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
fn typed_resource_ir_effect_check_reports_raw_alloc_escape_through_returned_i32_slot_alias() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
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
                origin_name: "slot_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            kind: ResourceExprKind::Literal,
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

    let report = check_resource_effect_boundaries_typed(&module, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function,
            ..
        } if function == "main"
    )));
}

#[test]
fn typed_resource_ir_effect_check_reports_raw_alloc_escape_through_indirect_returned_i32_slot_alias(
) {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let fn_ty = types.function(vec![], vec![i32_ty], i32_ty, Effect::Pure);
    let span = Span::dummy();
    let id_param = Place::local("slot".to_string(), i32_ty);
    let main_slot = Place::local("slot".to_string(), i32_ty);
    let callee = Place::temporary(ResourceId(0), fn_ty);
    let alias = Place::temporary(ResourceId(1), i32_ty);
    let size = Place::temporary(ResourceId(2), i32_ty);
    let raw = Place::temporary(ResourceId(3), i32_ty);
    let loaded = Place::temporary(ResourceId(4), i32_ty);
    let module = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "slot_id".to_string(),
                origin_name: "slot_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            output: alias.clone(),
                            callee,
                            params: vec![i32_ty],
                            result: i32_ty,
                            args: vec![main_slot.clone()],
                            effect: EffectOp::IndirectCall {
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Literal,
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
                            output: Place::temporary(ResourceId(5), unit_ty),
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

    let report = check_resource_effect_boundaries_typed(&module, &types);
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
            origin_name: "make_box".to_string(),
            type_params: Vec::new(),
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
            origin_name: "read_box_field".to_string(),
            type_params: Vec::new(),
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
            origin_name: "read_copied_box_field".to_string(),
            type_params: Vec::new(),
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
                origin_name: "raw_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
fn typed_resource_ir_effect_check_keeps_i32_raw_identity_parameter_summary() {
    let types = TypeCtx::new();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let param = Place::local("p".to_string(), i32_ty);
    let size = Place::temporary(ResourceId(0), i32_ty);
    let raw = Place::temporary(ResourceId(1), i32_ty);
    let forwarded = Place::temporary(ResourceId(2), i32_ty);
    let module = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "raw_id".to_string(),
                origin_name: "raw_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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

    let report = check_resource_effect_boundaries_typed(&module, &types);
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
                origin_name: "raw_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            source_name: "f".to_string(),
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
                                reason: UnknownEffectReason::FunctionValueWithoutKnownEffect,
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
fn resource_ir_lowering_initializes_bare_callable_var_references() {
    let source = r#"
#entry main
#indent 4
#target wasm

fn id <(i32)->i32> (x):
    x

fn get_op <(bool)->(i32)->i32> (con):
    if con:
        then:
            id;
            @id
        else:
            id
            @id

fn main <()->i32> ():
    let f get_op true
    f 1
"#;
    let (hir, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&hir, &types);
    fn count_named_function_values(ops: &[ResourceOp], name_prefix: &str) -> usize {
        ops.iter()
            .map(|op| match op {
                ResourceOp::FunctionValue { name, .. } if name.starts_with(name_prefix) => 1,
                ResourceOp::Branch {
                    then_ops, else_ops, ..
                } => {
                    count_named_function_values(then_ops, name_prefix)
                        + count_named_function_values(else_ops, name_prefix)
                }
                ResourceOp::Loop {
                    condition_ops,
                    body_ops,
                    ..
                } => {
                    count_named_function_values(condition_ops, name_prefix)
                        + count_named_function_values(body_ops, name_prefix)
                }
                ResourceOp::Match { arms, .. } => arms
                    .iter()
                    .map(|arm| count_named_function_values(&arm.ops, name_prefix))
                    .sum(),
                _ => 0,
            })
            .sum()
    }

    let id_function_values = resource
        .functions
        .iter()
        .flat_map(|function| function.blocks.iter())
        .map(|block| count_named_function_values(&block.ops, "id"))
        .sum::<usize>();
    assert!(
        id_function_values >= 4,
        "bare callable references and explicit @ references should both lower as function values:\n{}",
        resource.dump_text()
    );

    let report = check_resource_initialized_moves(&resource, &types);
    let callable_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| match diagnostic {
            ResourceCheckDiagnostic::CellUnavailable { function, .. } => {
                function.starts_with("id")
                    || function.starts_with("get_op")
                    || function.starts_with("main")
            }
            ResourceCheckDiagnostic::CollectionSlotRefuted { .. } => false,
        })
        .collect::<Vec<_>>();
    assert_eq!(callable_diagnostics, Vec::<&ResourceCheckDiagnostic>::new());
}

#[test]
fn resource_ir_lowering_preserves_nonzero_i32_relation_condition_fact() {
    let source = r#"
#entry main
#indent 4
#target wasm

#import "core/math" as *

fn main <(i32,i32)->i32> (i, len):
    if lt i len:
        then:
            i
        else:
            len
"#;
    let (hir, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&hir, &types);
    let main = resource
        .functions
        .iter()
        .find(|function| function.origin_name == "main")
        .expect("main resource function");
    let condition_fact = main
        .blocks
        .iter()
        .flat_map(|block| block.ops.iter())
        .find_map(|op| match op {
            ResourceOp::Branch {
                condition_fact: Some(fact),
                ..
            } => Some(fact),
            _ => None,
        })
        .expect("branch condition fact");

    let ResourceConditionFact::I32Relation { left, op, right } = condition_fact else {
        panic!(
            "lt i len must be preserved as an i32 relation fact:\n{}",
            resource.dump_text()
        );
    };
    assert_eq!(*op, ResourceI32RelationOp::Lt);
    assert!(matches!(&left.root, PlaceRoot::Local(name) if name == "i"));
    assert!(matches!(&right.root, PlaceRoot::Local(name) if name == "len"));
    assert!(
        resource.dump_text().contains("fact=i32_relation(%i:"),
        "relation fact should be visible in Resource IR dump:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_lowering_preserves_loop_i32_relation_condition_fact() {
    let source = r#"
#entry main
#indent 4
#target wasm

#import "core/math" as *

fn main <(i32)->i32> (len):
    let mut i <i32> 0
    while lt i len:
        do:
            set i add i 1
    i
"#;
    let (hir, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&hir, &types);
    let main = resource
        .functions
        .iter()
        .find(|function| function.origin_name == "main")
        .expect("main resource function");
    let condition_fact = main
        .blocks
        .iter()
        .flat_map(|block| block.ops.iter())
        .find_map(|op| match op {
            ResourceOp::Loop {
                condition_fact: Some(fact),
                ..
            } => Some(fact),
            _ => None,
        })
        .expect("loop condition fact");

    let ResourceConditionFact::I32Relation { left, op, right } = condition_fact else {
        panic!(
            "while lt i len must be preserved as an i32 relation fact:\n{}",
            resource.dump_text()
        );
    };
    assert_eq!(*op, ResourceI32RelationOp::Lt);
    assert!(matches!(&left.root, PlaceRoot::Local(name) if name == "i"));
    assert!(matches!(&right.root, PlaceRoot::Local(name) if name == "len"));
    assert!(
        resource.dump_text().contains("loop cond=")
            && resource.dump_text().contains("fact=i32_relation(%i:"),
        "loop relation fact should be visible in Resource IR dump:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_lowering_keeps_known_conjuncts_in_partial_loop_condition_fact() {
    let source = r#"
#entry main
#indent 4
#target wasm

#import "core/math" as *

fn main <(bool,i32)->i32> (flag, len):
    let mut i <i32> 0
    while and flag lt i len:
        do:
            set i add i 1
    i
"#;
    let (hir, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&hir, &types);
    let main = resource
        .functions
        .iter()
        .find(|function| function.origin_name == "main")
        .expect("main resource function");
    let condition_fact = main
        .blocks
        .iter()
        .flat_map(|block| block.ops.iter())
        .find_map(|op| match op {
            ResourceOp::Loop {
                condition_fact: Some(fact),
                ..
            } => Some(fact),
            _ => None,
        })
        .expect("loop condition fact");

    assert!(
        condition_fact_contains_i32_relation(condition_fact, "i", ResourceI32RelationOp::Lt, "len"),
        "known lt conjunct should survive unsupported boolean conjuncts:\n{}",
        resource.dump_text()
    );
}

fn condition_fact_contains_i32_relation(
    fact: &ResourceConditionFact,
    left_name: &str,
    expected_op: ResourceI32RelationOp,
    right_name: &str,
) -> bool {
    match fact {
        ResourceConditionFact::I32Relation { left, op, right } => {
            *op == expected_op
                && matches!(&left.root, PlaceRoot::Local(name) if name == left_name)
                && matches!(&right.root, PlaceRoot::Local(name) if name == right_name)
        }
        ResourceConditionFact::Any(facts) | ResourceConditionFact::All(facts) => {
            facts.iter().any(|fact| {
                condition_fact_contains_i32_relation(fact, left_name, expected_op, right_name)
            })
        }
        ResourceConditionFact::EqZero { .. }
        | ResourceConditionFact::NeZero { .. }
        | ResourceConditionFact::Positive { .. }
        | ResourceConditionFact::NonPositive { .. }
        | ResourceConditionFact::Negative { .. }
        | ResourceConditionFact::NonNegative { .. } => false,
    }
}

#[test]
fn resource_ir_lowering_uses_wasi_import_symbol_for_external_io_effect() {
    let source = r#"
#target wasi
#entry main
#indent 4

#import "core/math" as *
#import "core/cast" as *

#extern "wasi_snapshot_preview1" "fd_readdir" fn wasi_fd_readdir <(i32,i32,i32,i64,i32)*>i32>

fn main <()*>i32> ():
    let cookie <i64> <i64> cast 0
    wasi_fd_readdir 0 0 0 cookie 0
"#;
    let (hir, types) = typecheck_resource_source_with_target(source, CompileTarget::Wasi);
    let resource = lower_hir_module(&hir, &types);
    assert!(
        resource
            .dump_text()
            .contains("effect=external_io(fd_readdir)"),
        "WASI extern imports must lower by imported symbol, not local wrapper name:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_lowers_dedicated_memory_helpers_once_per_call() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *

fn main <()->i32> ():
    let p <MemPtr<u8>> mem_ptr_wrap<u8> 16
    let q <MemPtr<u8>> mem_ptr_add<u8> p 1
    let raw <i32> 17
    let token <RegionToken<u8>> region_new<u8> raw 8
    let view <MemPtr<u8>> region_ptr<u8> &token
    let raw_ref <&i32> region_token_raw_ref<u8> &token
    0
"#;
    let (hir, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&hir, &types);
    let main = resource
        .functions
        .iter()
        .find(|function| function.origin_name == "main")
        .expect("main resource function");
    let raw_address_facts = main
        .blocks
        .iter()
        .flat_map(|block| block.ops.iter())
        .filter(|op| {
            matches!(
                op,
                ResourceOp::RawAddressAlias { .. } | ResourceOp::RawAddressView { .. }
            )
        })
        .count();
    assert_eq!(
        raw_address_facts,
        5,
        "each dedicated memory helper call must emit exactly one raw-address fact:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_lowering_preserves_symbolic_mem_ptr_add_offset() {
    let source = r#"
#entry main
#indent 4
#target wasm

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn main <(i32)->i32> (idx):
    let raw <i32> alloc_raw 16
    let ptr <MemPtr<i32>> mem_ptr_wrap raw
    let slot <MemPtr<i32>> mem_ptr_add ptr idx
    mem_ptr_addr slot
"#;
    let (hir, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&hir, &types);
    let main = resource
        .functions
        .iter()
        .find(|function| function.origin_name == "main")
        .expect("main resource function");
    let has_symbolic_offset = main
        .blocks
        .iter()
        .flat_map(|block| block.ops.iter())
        .any(|op| {
            matches!(
                op,
                ResourceOp::RawAddressView { source, .. }
                    if source.projections.iter().any(|projection| {
                        matches!(
                            projection,
                            PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place })
                                if matches!(place.root, PlaceRoot::Temporary(_))
                        )
                    })
            )
        });

    assert!(
        has_symbolic_offset,
        "dynamic mem_ptr_add offset must keep the symbolic index place instead of collapsing to unknown:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_lowering_marks_mem_ptr_addr_as_non_owning_projection() {
    let source = r#"
#entry main
#indent 4
#target core

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn ptr_addr <(MemPtr<u8>)->i32> (p):
    mem_ptr_addr p

fn main <()->i32> ():
    ptr_addr mem_ptr_wrap<u8> 32
"#;
    let (hir, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&hir, &types);
    let ptr_addr = resource
        .functions
        .iter()
        .find(|function| function.origin_name == "ptr_addr")
        .expect("ptr_addr resource function");
    let has_non_owning_view = ptr_addr
        .blocks
        .iter()
        .flat_map(|block| block.ops.iter())
        .any(|op| {
            matches!(
                op,
                ResourceOp::RawAddressView {
                    kind: RawAddressViewKind::NonOwningProjection,
                    source,
                    ..
                } if source.projections.iter().any(|projection| {
                    matches!(projection, PlaceProjection::Field { index: 0, .. })
                })
            )
        });
    let has_plain_alias = ptr_addr
        .blocks
        .iter()
        .flat_map(|block| block.ops.iter())
        .any(|op| {
            matches!(
                op,
                ResourceOp::RawAddressAlias { source, .. }
                    if source.projections.iter().any(|projection| {
                        matches!(projection, PlaceProjection::Field { index: 0, .. })
                    })
            )
        });

    assert!(
        has_non_owning_view,
        "mem_ptr_addr must lower MemPtr.raw extraction as a non-owning projection view:\n{}",
        resource.dump_text()
    );
    assert!(
        !has_plain_alias,
        "mem_ptr_addr must not lower MemPtr.raw extraction as a transferable raw alias:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_effect_check_uses_known_function_alias_stored_in_aggregate_field() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let fn_ty = types.function(vec![], vec![i32_ty], i32_ty, Effect::Pure);
    let wrapper_ty = types.register_named(
        "CallbackBox".to_string(),
        TypeKind::Struct {
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
                origin_name: "return_zero".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                        ResourceOp::Expr {
                            kind: nepl_core::resource::ResourceExprKind::Literal,
                            output: raw.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::IndirectCall {
                            output: forwarded.clone(),
                            callee,
                            params: vec![i32_ty],
                            result: i32_ty,
                            args: vec![raw],
                            effect: EffectOp::IndirectCall {
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
                origin_name: "safe_zero".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            source_name: "f".to_string(),
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
                                reason: UnknownEffectReason::AssignedCallbackWithoutKnownEffect,
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
                origin_name: "apply".to_string(),
                type_params: Vec::new(),
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
                            reason: UnknownEffectReason::FunctionParameterWithoutKnownEffect,
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
            origin_name: "main".to_string(),
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
    assert_eq!(report.functions[0].counts.unsafe_memory_ops.store, 1);
    assert_eq!(report.functions[0].counts.unsafe_memory_ops.total(), 1);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction {
            function,
            operation,
            ..
        } if function == "main" && *operation == RawMemoryOp::Store
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary {
            function,
            operation,
            ..
        } if function == "main" && *operation == RawMemoryOp::Store
    )));
}

#[test]
fn resource_ir_effect_check_rejects_raw_memory_outside_boundary_in_impure_function() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let resource = manual_resource_module_with_effect(
        Effect::Impure,
        unit_ty,
        span,
        vec![
            ResourceOp::CallEffect {
                effect: EffectOp::UnsafeMemory {
                    operation: RawMemoryOp::Store,
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: Place::temporary(ResourceId(0), types.i32()),
                args: vec![Place::temporary(ResourceId(1), types.i32())],
                span,
            },
        ],
    );

    let report = check_resource_effect_boundaries(&resource);
    assert_eq!(report.functions[0].counts.unsafe_memory_ops.store, 1);
    assert_eq!(report.functions[0].counts.unsafe_memory_ops.total(), 1);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawMemoryOutsideBoundary {
            function,
            operation,
            ..
        } if function == "main" && *operation == RawMemoryOp::Store
    )));
    assert!(!report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::UnsafeMemoryInPureFunction { .. }
    )));
}

#[test]
fn compile_rejects_checked_mem_ptr_wrapper_with_forged_positive_address() {
    let source = r#"
#entry main
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/result" as *

fn main <()*>i32> ():
    let p <MemPtr<i32>> mem_ptr_wrap<i32> 16
    match store_i32 p 7:
        Result::Ok _:
            1
        Result::Err _:
            0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.raw.memory_outside_boundary",
    );
}

#[test]
fn compile_accepts_checked_mem_ptr_wrapper_with_null_sentinel() {
    let source = r#"
#entry main
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/option" as *

fn main <()*>i32> ():
    let p <MemPtr<i32>> mem_ptr_wrap<i32> 0
    match load_i32 p:
        Option::None:
            1
        Option::Some _:
            0
"#;

    compile_resource_source_as_compiler_owned(source, CompileTarget::Wasm)
        .expect("null MemPtr sentinel must be allowed to reach checked load guard");
}

#[test]
fn compile_accepts_checked_mem_ptr_wrapper_from_region_provenance() {
    let source = r#"
#entry main
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_region<i32> 1:
        Result::Ok region:
            let p <MemPtr<i32>> region_ptr &region
            match store_i32 p 7:
                Result::Ok _:
                    match dealloc_region<i32> region:
                        Result::Ok _:
                            1
                        Result::Err _:
                            0
                Result::Err _:
                    match dealloc_region<i32> region:
                        Result::Ok _:
                            0
                        Result::Err _:
                            0
        Result::Err _:
            0
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect("RegionToken-derived MemPtr must prove checked store provenance");
}

#[test]
fn resource_ir_effect_check_rejects_mem_ptr_return_identity_escape() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let mem_ptr_ty = types.register_named(
        "MemPtr".to_string(),
        TypeKind::Struct {
            name: "MemPtr".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["raw".to_string()],
        },
    );
    let span = Span::dummy();
    let size = Place::temporary(ResourceId(0), i32_ty);
    let raw = Place::temporary(ResourceId(1), i32_ty);
    let ptr = Place::temporary(ResourceId(2), mem_ptr_ty);
    let module = ResourceModule {
        functions: vec![ResourceFunction {
            name: "leak_ptr".to_string(),
            origin_name: "leak_ptr".to_string(),
            type_params: Vec::new(),
            params: vec![],
            result: mem_ptr_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::Expr {
                        kind: ResourceExprKind::LiteralI32(1),
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
                        output: ptr.clone(),
                        kind: AggregateKind::Struct {
                            name: "MemPtr".to_string(),
                            field_offsets: vec![0],
                        },
                        inputs: vec![raw],
                        span,
                    },
                ],
                terminator: ResourceTerminator::Return {
                    value: Some(ptr),
                    span,
                },
                span,
            }],
            span,
        }],
        entry: None,
        string_literals: vec![],
    };

    let report = check_resource_effect_boundaries_typed(&module, &types);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
            function,
            place,
            ..
        } if function == "leak_ptr" && place.ty == i32_ty
    )));
}

#[test]
fn compile_accepts_checked_region_pointer_from_region_provenance() {
    let source = r#"
#entry main
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()*>()> ():
    match alloc_region<u8> 1:
        Result::Ok token:
            let p <MemPtr<u8>> region_ptr &token
            match store_u8 p 7:
                Result::Ok _:
                    match dealloc_region<u8> token:
                        Result::Ok _:
                            ()
                        Result::Err _:
                            ()
                Result::Err _:
                    match dealloc_region<u8> token:
                        Result::Ok _:
                            ()
                        Result::Err _:
                            ()
        Result::Err _:
            ()
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect("region-derived MemPtr must prove checked store provenance");
}

#[test]
fn compile_accepts_checked_region_ptr_at_from_region_provenance() {
    let source = r#"
#entry main
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()*>()> ():
    match alloc_region<i32> 1:
        Result::Ok token:
            match region_ptr_at<i32,i32> &token 0:
                Result::Ok p:
                    match store_i32 p 9:
                        Result::Ok _:
                            match dealloc_region<i32> token:
                                Result::Ok _:
                                    ()
                                Result::Err _:
                                    ()
                        Result::Err _:
                            match dealloc_region<i32> token:
                                Result::Ok _:
                                    ()
                                Result::Err _:
                                    ()
                Result::Err _:
                    match dealloc_region<i32> token:
                        Result::Ok _:
                            ()
                        Result::Err _:
                            ()
        Result::Err _:
            ()
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect("region_ptr_at Ok payload must prove checked store provenance");
}

#[test]
fn compile_accepts_callback_returned_region_pointer_without_owner_transfer() {
    let source = r#"
#entry main
#target std

#import "core/mem" as *
#import "core/result" as *

fn id_ptr <(MemPtr<u8>)->MemPtr<u8>> (p):
    p

fn apply_ptr <(MemPtr<u8>, (MemPtr<u8>)->MemPtr<u8>)->MemPtr<u8>> (p, f):
    f p

fn borrowed_region_ptr_via_callback_param <(&RegionToken<u8>, (MemPtr<u8>)->MemPtr<u8>)->MemPtr<u8>> (token, f):
    let p <MemPtr<u8>> region_ptr token
    apply_ptr p f

fn main <()*>()> ():
    match alloc_region<u8> 1:
        Result::Ok token:
            let _p <MemPtr<u8>> borrowed_region_ptr_via_callback_param &token @id_ptr
            match dealloc_region<u8> token:
                Result::Ok _:
                    ()
                Result::Err _:
                    ()
        Result::Err _:
            ()
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasm).expect(
        "callback-returned region pointer must not carry owner state or block token dealloc",
    );
}

#[test]
fn resource_ir_lowering_does_not_treat_mem_ptr_store_wrapper_as_direct_raw_memory() {
    let source = r#"
#entry main
#target std

#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_region<i32> 1:
        Result::Ok region:
            let p <MemPtr<i32>> region_ptr &region
            match store_i32 p 1:
                Result::Ok _:
                    match dealloc_region region:
                        Result::Ok _:
                            1
                        Result::Err _:
                            0
                Result::Err _:
                    match dealloc_region region:
                        Result::Ok _:
                            0
                        Result::Err _:
                            0
        Result::Err _:
            0
"#;
    let (hir, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&hir, &types);
    let main = resource
        .functions
        .iter()
        .find(|function| function.origin_name == "main")
        .expect("main resource function should be lowered");

    let mut call_effects = Vec::new();
    for block in &main.blocks {
        collect_call_effects(&block.ops, &mut call_effects);
    }
    let mut direct_raw_ops = Vec::new();
    for block in &main.blocks {
        collect_direct_raw_memory_ops(&block.ops, &mut direct_raw_ops);
    }

    assert!(call_effects.iter().any(|effect| matches!(
        effect,
        EffectOp::UnsafeMemory {
            operation: RawMemoryOp::Store
        }
    )));
    assert!(!direct_raw_ops.contains(&RawMemoryOp::Store));
}

fn collect_call_effects<'a>(ops: &'a [ResourceOp], effects: &mut Vec<&'a EffectOp>) {
    for op in ops {
        match op {
            ResourceOp::CallEffect { effect, .. } => effects.push(effect),
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                collect_call_effects(then_ops, effects);
                collect_call_effects(else_ops, effects);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                collect_call_effects(condition_ops, effects);
                collect_call_effects(body_ops, effects);
            }
            ResourceOp::Match { arms, .. } => {
                for arm in arms {
                    collect_call_effects(&arm.ops, effects);
                }
            }
            _ => {}
        }
    }
}

fn collect_direct_raw_memory_ops(ops: &[ResourceOp], operations: &mut Vec<RawMemoryOp>) {
    for op in ops {
        match op {
            ResourceOp::RawMemory { operation, .. } => operations.push(*operation),
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                collect_direct_raw_memory_ops(then_ops, operations);
                collect_direct_raw_memory_ops(else_ops, operations);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                collect_direct_raw_memory_ops(condition_ops, operations);
                collect_direct_raw_memory_ops(body_ops, operations);
            }
            ResourceOp::Match { arms, .. } => {
                for arm in arms {
                    collect_direct_raw_memory_ops(&arm.ops, operations);
                }
            }
            _ => {}
        }
    }
}

#[test]
fn resource_ir_effect_check_counts_host_effect_operations() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::CallEffect {
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdRead,
                },
                span,
            },
            ResourceOp::CallEffect {
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdWrite,
                },
                span,
            },
            ResourceOp::CallEffect {
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdWrite,
                },
                span,
            },
            ResourceOp::CallEffect {
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::PathOpen,
                },
                span,
            },
            ResourceOp::CallEffect {
                effect: EffectOp::Nondet {
                    operation: NondetOp::RandomGet,
                },
                span,
            },
            ResourceOp::CallEffect {
                effect: EffectOp::Nondet {
                    operation: NondetOp::ClockTimeGet,
                },
                span,
            },
        ],
    );

    let report = check_resource_effect_boundaries(&resource);
    let counts = report.functions[0].counts;
    assert_eq!(counts.external_io_ops.fd_read, 1);
    assert_eq!(counts.external_io_ops.fd_write, 2);
    assert_eq!(counts.external_io_ops.path_open, 1);
    assert_eq!(counts.external_io_ops.total(), 4);
    assert_eq!(counts.nondet_ops.random_get, 1);
    assert_eq!(counts.nondet_ops.clock_time_get, 1);
    assert_eq!(counts.nondet_ops.total(), 2);
}

#[test]
fn resource_ir_effect_check_rejects_direct_host_effects_in_pure_function() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::CallEffect {
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdWrite,
                },
                span,
            },
            ResourceOp::CallEffect {
                effect: EffectOp::Nondet {
                    operation: NondetOp::RandomGet,
                },
                span,
            },
        ],
    );

    let report = check_resource_effect_boundaries(&resource);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::ImpureCallInPureFunction {
            function,
            call: ResourceEffectCallKind::ExternalIo {
                operation: ExternalIoOp::FdWrite,
            },
            ..
        } if function == "main"
    )));
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::ImpureCallInPureFunction {
            function,
            call: ResourceEffectCallKind::Nondet {
                operation: NondetOp::RandomGet,
            },
            ..
        } if function == "main"
    )));
}

#[test]
fn resource_ir_effect_check_reports_unknown_effect_as_lowering_incomplete() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![ResourceOp::CallEffect {
            effect: EffectOp::Unknown {
                reason: UnknownEffectReason::SyntheticTestFixture,
            },
            span,
        }],
    );

    let report = check_resource_effect_boundaries(&resource);
    assert_eq!(report.functions[0].counts.unknown_ops, 1);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
    diagnostic,
        ResourceEffectBoundaryDiagnostic::UnknownEffect {
            function,
            reason,
            ..
        } if function == "main" && *reason == UnknownEffectReason::SyntheticTestFixture
    )));
}

#[test]
fn resource_ir_lowering_treats_compiler_field_load_as_field_read() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let pair_ty = types.register_named(
        "Pair".to_string(),
        TypeKind::Struct {
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
            origin_name: "main".to_string(),
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
    let coverage = compare_hir_resource_lowering_typed(&module, &resource, &types);
    assert_eq!(coverage.diagnostics, vec![]);
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
            origin_name: "main".to_string(),
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/field" as *
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
    let p <i32> alloc_raw 16
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
        origin_name: "main".to_string(),
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
                            effect: Effect::Impure,
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
        origin_name: "callee".to_string(),
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
            effect: EffectOp::IndirectCall {
                effect: Effect::Impure,
            },
            ..
        } if params == &vec![i32_ty] && result == &i32_ty && args.len() == 1
    )));

    let dump = resource.dump_text();
    assert!(dump.contains("function_value callee"));
    assert!(dump.contains("call user(callee<>)"));
    assert!(dump.contains("effect=call(callee,Impure)"));
    assert!(dump.contains("indirect_call"));
    assert!(dump.contains("effect=indirect_call(Impure)"));
    assert!(!dump.contains("unknown(indirect call)"));
}

#[test]
fn resource_ir_lowering_carries_typechecked_indirect_call_effect() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/math" as *

fn square <(i32)->i32> (x):
    mul x x

fn apply <(i32, (i32)->i32)->i32> (value, callback):
    callback value

fn main <()->i32> ():
    apply 5 @square
"#;
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let apply = resource
        .functions
        .iter()
        .find(|function| function.name == "apply" || function.name.starts_with("apply__"))
        .expect("apply function should lower");
    let ops = &apply.blocks[apply.entry_block.0].ops;

    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::CallEffect {
            effect: EffectOp::IndirectCall {
                effect: Effect::Pure,
            },
            ..
        }
    )));
    assert!(ops.iter().any(|op| matches!(
        op,
        ResourceOp::IndirectCall {
            effect: EffectOp::IndirectCall {
                effect: Effect::Pure,
            },
            ..
        }
    )));

    let dump = resource.dump_text();
    assert!(dump.contains("effect indirect_call(Pure)"));
    assert!(!dump.contains("unknown(indirect call)"));
}

#[test]
fn resource_ir_effect_check_rejects_impure_indirect_call_in_pure_function() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let fn_ty = types.function(vec![], vec![i32_ty], i32_ty, Effect::Impure);
    let span = Span::dummy();
    let callback = Place::local("callback".to_string(), fn_ty);
    let arg = Place::temporary(ResourceId(0), i32_ty);
    let returned = Place::temporary(ResourceId(1), i32_ty);
    let module = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            origin_name: "main".to_string(),
            type_params: Vec::new(),
            params: vec![ResourceLocal {
                name: "callback".to_string(),
                ty: fn_ty,
                mutable: false,
                place: callback.clone(),
            }],
            result: i32_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::Expr {
                        kind: ResourceExprKind::Literal,
                        output: arg.clone(),
                        ty: i32_ty,
                        span,
                    },
                    ResourceOp::CallEffect {
                        effect: EffectOp::IndirectCall {
                            effect: Effect::Impure,
                        },
                        span,
                    },
                    ResourceOp::IndirectCall {
                        output: returned.clone(),
                        callee: callback,
                        params: vec![i32_ty],
                        result: i32_ty,
                        args: vec![arg],
                        effect: EffectOp::IndirectCall {
                            effect: Effect::Impure,
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

    let report = check_resource_effect_boundaries(&module);
    assert!(report.diagnostics.iter().any(|diagnostic| matches!(
        diagnostic,
        ResourceEffectBoundaryDiagnostic::ImpureCallInPureFunction {
            function,
            call: ResourceEffectCallKind::Indirect,
            ..
        } if function == "main"
    )));
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
            origin_name: "main".to_string(),
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
fn resource_ir_check_auto_drops_live_non_copy_local_at_scope_end() {
    let (types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            origin_name: "main".to_string(),
            func_ty: TypeId(9),
            params: vec![],
            result: unit_ty,
            effect: Effect::Pure,
            body: HirBody::Block(HirBlock {
                lines: vec![HirLine {
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

    let resource = lower_hir_module(&module, &types);
    let drop_plan = compute_resource_drop_plan(&resource, &types);
    assert_eq!(drop_plan.functions[0].auto_drops.len(), 1);
    assert_eq!(drop_plan.functions[0].drop_points.len(), 1);
    assert_eq!(drop_plan.functions[0].drop_points[0].auto_drops.len(), 1);
    assert_eq!(
        drop_plan.functions[0].drop_points[0].path.block,
        ResourceBlockId(0)
    );
    assert!(matches!(
        drop_plan.functions[0].drop_points[0].path.steps.last(),
        Some(ResourceDropPointStep::Op { .. })
    ));
    assert!(matches!(
        drop_plan.functions[0].auto_drops[0].kind,
        ResourceAutoDropKind::ScopeLocal
    ));
    assert!(matches!(
        drop_plan.functions[0].auto_drops[0].requirement,
        ResourceDropRequirement::StateOnly
    ));
    let end_scope = resolve_resource_drop_point_end_scope(
        resource_function(&resource, &drop_plan.functions[0].name),
        &drop_plan.functions[0].drop_points[0].path,
    )
    .expect("drop point path must resolve to the EndScope it describes");
    assert!(end_scope
        .locals
        .iter()
        .any(|place| matches!(&place.root, PlaceRoot::Local(name) if name == "x")));
    let invalid_path = ResourceDropPointPath {
        block: ResourceBlockId(0),
        steps: vec![ResourceDropPointStep::Op { index: usize::MAX }],
    };
    assert!(matches!(
        resolve_resource_drop_point_path(
            resource_function(&resource, &drop_plan.functions[0].name),
            &invalid_path
        ),
        Err(ResourceDropPointResolutionError::OpIndexOutOfBounds {
            index: usize::MAX,
            ..
        })
    ));
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    assert_eq!(report.functions[0].auto_drop_points.len(), 1);
    assert!(report.functions[0].auto_drop_points[0]
        .auto_drops
        .iter()
        .any(|drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name == "x")));
    assert!(report.functions[0].final_cells.iter().any(|entry| {
        matches!(&entry.place.root, PlaceRoot::Local(name) if name == "x")
            && entry.state == CellState::Dropped
    }));
}

#[test]
fn resource_ir_scope_auto_drop_keeps_same_type_shadowed_locals_distinct() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (_self):
        ()

fn consume <(Guard)->i32> (_g):
    0

fn main <()->i32> ():
    let x <Guard> Guard 1
    let _ <i32> if true:
        then:
            let x <Guard> Guard 2
            0
        else:
            0
    consume x
"#;
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let drop_plan = compute_resource_drop_plan(&resource, &types);
    let main_drop_plan = drop_plan
        .functions
        .iter()
        .find(|function| function.name.starts_with("main"))
        .expect("main drop plan should exist");
    assert!(main_drop_plan.drop_points.iter().any(|point| {
        point
            .auto_drops
            .iter()
            .any(|drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name == "x"))
    }));
    let inner_drop_point = main_drop_plan
        .drop_points
        .iter()
        .find(|point| {
            point.auto_drops.iter().any(
                |drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name.starts_with("x#")),
            ) && point
                .path
                .steps
                .iter()
                .any(|step| matches!(step, ResourceDropPointStep::BranchThen))
        })
        .expect("inner shadowed local should be auto-dropped at the then branch EndScope");
    assert!(main_drop_plan.drop_points.iter().any(|point| {
        point.auto_drops.iter().any(
            |drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name.starts_with("x#")),
        ) && point
            .path
            .steps
            .iter()
            .any(|step| matches!(step, ResourceDropPointStep::BranchThen))
    }));
    let inner_end_scope = resolve_resource_drop_point_end_scope(
        resource_function(&resource, &main_drop_plan.name),
        &inner_drop_point.path,
    )
    .unwrap_or_else(|error| {
        panic!(
            "nested branch drop point path must resolve to an EndScope: {error:?}, path={:?}",
            inner_drop_point.path
        )
    });
    assert!(inner_end_scope
        .locals
        .iter()
        .any(|place| matches!(&place.root, PlaceRoot::Local(name) if name.starts_with("x#"))));
    assert!(main_drop_plan
        .auto_drops
        .iter()
        .any(|drop| { matches!(&drop.place.root, PlaceRoot::Local(name) if name == "x") }));
    assert!(main_drop_plan.auto_drops.iter().any(|drop| {
        matches!(&drop.place.root, PlaceRoot::Local(name) if name.starts_with("x#"))
    }));
    assert!(main_drop_plan.auto_drops.iter().any(|drop| {
        matches!(
            (&drop.place.root, &drop.requirement),
            (PlaceRoot::Local(name), ResourceDropRequirement::WholeValue) if name == "x"
        )
    }));
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let main_check = report
        .functions
        .iter()
        .find(|function| function.name == main_drop_plan.name)
        .expect("main resource check should exist");
    assert!(main_check.auto_drop_points.iter().any(|point| {
        point.auto_drops.iter().any(
            |drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name.starts_with("x#")),
        )
    }));
    assert!(!main_check.auto_drop_points.iter().any(|point| {
        point
            .auto_drops
            .iter()
            .any(|drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name == "x"))
    }));
    let dump = resource.dump_text();
    assert!(dump.contains("%x#"));
}

#[test]
fn resource_ir_match_payload_bind_shadow_uses_declared_place() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude

enum LocalResult:
    Ok <i32>
    Err <i32>

fn make_err <()->LocalResult> ():
    LocalResult::Err 7

fn forward_shadow <(i32)->LocalResult> (e):
    match make_err:
        LocalResult::Err e:
            LocalResult::Err e
        LocalResult::Ok value:
            LocalResult::Ok value

fn main <()->i32> ():
    match forward_shadow 1:
        LocalResult::Err code:
            code
        LocalResult::Ok value:
            value
"#;
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let dump = resource.dump_text();
    assert!(
        dump.contains("bind=%e#0:") && dump.contains("source=e"),
        "shadowed match payload bind must use the declared Resource IR place:\n{}",
        dump
    );
}

#[test]
fn resource_ir_match_payload_bind_shadow_keeps_source_name_for_drop_bridge() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

enum GuardResult:
    Ok <i32>
    Err <Guard>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (_self):
        ()

fn make_guard_err <()->GuardResult> ():
    GuardResult::Err Guard 2

fn run <(Guard)->i32> (g):
    match make_guard_err:
        GuardResult::Err g:
            0
        GuardResult::Ok value:
            value

fn main <()->i32> ():
    run Guard 1
"#;
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let plan = compute_resource_drop_elaboration_plan(&resource, &report)
        .expect("drop elaboration plan should preserve shadowed match payload source names");
    nepl_core::resource::validate_resource_drop_elaboration_hir_bridge(&module, &plan)
        .expect("shadowed match payload drop should bridge through the source bind name");
    let dump = resource.dump_text();
    assert!(
        dump.contains("bind=%g#0:") && dump.contains("source=g"),
        "shadowed non-Copy match bind should keep source name separately:\n{}",
        dump
    );
}

#[test]
fn resource_ir_live_auto_drop_points_include_function_parameters() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (_self):
        ()

fn ignore <(Guard)->i32> (_g):
    1

fn main <()->i32> ():
    ignore Guard 7
"#;
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let ignore_check = report
        .functions
        .iter()
        .find(|function| function.name.starts_with("ignore"))
        .expect("ignore resource check should exist");
    let param_drop_point = ignore_check
        .auto_drop_points
        .iter()
        .find(|point| {
            point
                .auto_drops
                .iter()
                .any(|drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name == "_g"))
        })
        .expect("unused non-Copy parameter should be a live auto-drop point");
    let ignore_function = resource
        .functions
        .iter()
        .find(|function| function.name == ignore_check.name)
        .expect("ignore resource function should exist");
    let end_scope = resolve_resource_drop_point_end_scope(ignore_function, &param_drop_point.path)
        .expect("parameter drop point should resolve to its EndScope");
    assert!(end_scope
        .locals
        .iter()
        .any(|place| matches!(&place.root, PlaceRoot::Local(name) if name == "_g")));
    let elaboration_plan = compute_resource_drop_elaboration_plan(&resource, &report)
        .expect("parameter live drop point should be valid for drop elaboration");
    let ignore_plan = elaboration_plan
        .functions
        .iter()
        .find(|function| function.name == ignore_check.name)
        .expect("ignore drop elaboration function should exist");
    assert!(ignore_plan
        .auto_drops
        .iter()
        .any(|drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name == "_g")));
    assert!(ignore_plan
        .auto_drops
        .iter()
        .any(|drop| drop.source_name == "_g"
            && matches!(&drop.place.root, PlaceRoot::Local(name) if name == "_g")));
    assert!(ignore_check.final_cells.iter().any(|entry| {
        matches!(&entry.place.root, PlaceRoot::Local(name) if name == "_g")
            && entry.state == CellState::Dropped
    }));
}

#[test]
fn resource_ir_drop_elaboration_plan_preserves_monomorphized_function_origin() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (_self):
        ()

fn ignore <.T> <(.T)->i32> (_value):
    1

fn main <()->i32> ():
    ignore<Guard> Guard 7
"#;
    let (module, mut types) = typecheck_resource_source(source);
    let mono = nepl_core::monomorphize::monomorphize(&mut types, module);
    let (module, unresolved_trait_calls) = mono.into_parts();
    assert!(
        unresolved_trait_calls.is_empty(),
        "unresolved trait calls: {:#?}",
        unresolved_trait_calls
    );
    let ignore_hir = module
        .functions
        .iter()
        .find(|function| function.origin_name == "ignore")
        .expect("monomorphized ignore HIR function should keep source origin");
    assert_ne!(
        ignore_hir.name, ignore_hir.origin_name,
        "generic instantiation must keep the mangled function name separate from source origin"
    );

    let resource = lower_hir_module(&module, &types);
    let ignore_resource = resource
        .functions
        .iter()
        .find(|function| function.name == ignore_hir.name)
        .expect("Resource IR function should be lowered from the monomorphized HIR function");
    assert_eq!(ignore_resource.origin_name, "ignore");
    assert!(
        resource
            .dump_text()
            .contains(&format!("fn {} origin=ignore", ignore_resource.name)),
        "Resource IR dump should expose explicit function origin metadata:\n{}",
        resource.dump_text()
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let elaboration_plan = compute_resource_drop_elaboration_plan(&resource, &report)
        .expect("drop elaboration should accept checked facts for monomorphized functions");
    let ignore_plan = elaboration_plan
        .functions
        .iter()
        .find(|function| function.name == ignore_resource.name)
        .expect("drop elaboration plan should include the monomorphized ignore function");
    assert_eq!(ignore_plan.origin_name, "ignore");
    assert!(ignore_plan.auto_drops.iter().any(|drop| {
        drop.source_name == "_value"
            && matches!(&drop.place.root, PlaceRoot::Local(name) if name == "_value")
    }));
}

#[test]
fn resource_ir_drop_elaboration_hir_bridge_accepts_monomorphized_origin() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude

struct Guard:
    id <i32>

fn ignore <.T> <(.T)->i32> (_value):
    1

fn main <()->i32> ():
    ignore<Guard> Guard 7
"#;
    let (source_module, _) = typecheck_resource_source(source);
    let (module, mut types) = typecheck_resource_source(source);
    let mono = nepl_core::monomorphize::monomorphize(&mut types, module);
    let (module, unresolved_trait_calls) = mono.into_parts();
    assert!(
        unresolved_trait_calls.is_empty(),
        "unresolved trait calls: {:#?}",
        unresolved_trait_calls
    );
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let plan = compute_resource_drop_elaboration_plan(&resource, &report)
        .expect("checked Resource IR drop plan should be valid");

    nepl_core::resource::validate_resource_drop_elaboration_hir_bridge(&source_module, &plan)
        .expect("monomorphized plan should bridge back to source HIR origin and binding");
}

#[test]
fn resource_ir_drop_elaboration_hir_bridge_rejects_missing_source_origin() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude

struct Guard:
    id <i32>

fn ignore <.T> <(.T)->i32> (_value):
    1

fn main <()->i32> ():
    ignore<Guard> Guard 7
"#;
    let (source_module, _) = typecheck_resource_source(source);
    let (module, mut types) = typecheck_resource_source(source);
    let mono = nepl_core::monomorphize::monomorphize(&mut types, module);
    let (module, unresolved_trait_calls) = mono.into_parts();
    assert!(unresolved_trait_calls.is_empty());
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let mut plan = compute_resource_drop_elaboration_plan(&resource, &report).unwrap();
    let function = plan
        .functions
        .iter_mut()
        .find(|function| !function.auto_drops.is_empty())
        .expect("test plan should contain a live drop function");
    let resource_function_name = function.name.clone();
    function.origin_name = "__missing_origin".to_string();

    let errors =
        nepl_core::resource::validate_resource_drop_elaboration_hir_bridge(&source_module, &plan)
            .expect_err("missing source origin must be rejected");
    assert!(errors.iter().any(|error| matches!(
        error,
        ResourceDropElaborationHirBridgeError::MissingSourceFunction {
            function,
            origin_name,
        } if function == &resource_function_name && origin_name == "__missing_origin"
    )));
}

#[test]
fn resource_ir_drop_elaboration_hir_bridge_rejects_missing_source_binding() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude

struct Guard:
    id <i32>

fn ignore <.T> <(.T)->i32> (_value):
    1

fn main <()->i32> ():
    ignore<Guard> Guard 7
"#;
    let (source_module, _) = typecheck_resource_source(source);
    let (module, mut types) = typecheck_resource_source(source);
    let mono = nepl_core::monomorphize::monomorphize(&mut types, module);
    let (module, unresolved_trait_calls) = mono.into_parts();
    assert!(unresolved_trait_calls.is_empty());
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let mut plan = compute_resource_drop_elaboration_plan(&resource, &report).unwrap();
    let function = plan
        .functions
        .iter_mut()
        .find(|function| !function.auto_drops.is_empty())
        .expect("test plan should contain a live drop function");
    let resource_function_name = function.name.clone();
    let origin_name = function.origin_name.clone();
    function.drop_points[0].auto_drops[0].source_name = "__missing_binding".to_string();

    let errors =
        nepl_core::resource::validate_resource_drop_elaboration_hir_bridge(&source_module, &plan)
            .expect_err("missing source binding must be rejected");
    assert!(errors.iter().any(|error| matches!(
        error,
        ResourceDropElaborationHirBridgeError::MissingSourceBinding {
            function,
            origin_name: error_origin_name,
            source_name,
            ..
        } if function == &resource_function_name
            && error_origin_name == &origin_name
            && source_name == "__missing_binding"
    )));
}

#[test]
fn resource_ir_drop_elaboration_plan_records_assignment_overwrite_drop_facts() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (_self):
        ()

fn main <()->i32> ():
    let mut g <Guard> Guard 0
    set g Guard 1
    0
"#;
    let (source_module, _) = typecheck_resource_source(source);
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let plan = compute_resource_drop_elaboration_plan(&resource, &report)
        .expect("assignment overwrite should build a checked drop elaboration plan");
    let main_plan = plan
        .functions
        .iter()
        .find(|function| function.name.starts_with("main"))
        .expect("main plan should exist");
    let overwrite_point = main_plan
        .drop_points
        .iter()
        .find(|point| {
            point.auto_drops.iter().any(|drop| {
                drop.source_name == "g"
                    && matches!(drop.kind, ResourceAutoDropKind::AssignmentOverwrite)
            })
        })
        .expect("assignment overwrite should record a dedicated drop point");
    let overwrite_drop = overwrite_point
        .auto_drops
        .iter()
        .find(|drop| matches!(drop.kind, ResourceAutoDropKind::AssignmentOverwrite))
        .expect("assignment drop point should contain an assignment overwrite drop");
    assert_eq!(overwrite_drop.source_name, "g");
    assert!(matches!(
        overwrite_drop.requirement,
        ResourceDropRequirement::WholeValue
    ));
    let assignment = resolve_resource_drop_point_assignment(
        resource_function(&resource, &main_plan.name),
        &overwrite_point.path,
    )
    .expect("assignment overwrite drop point must resolve to ResourceOp::Assign");
    assert_eq!(assignment.target, &overwrite_drop.place);

    nepl_core::resource::validate_resource_drop_elaboration_hir_bridge(&source_module, &plan)
        .expect("assignment overwrite drop point should bridge to source HIR set binding");
}

#[test]
fn resource_ir_assignment_overwrite_records_partial_drop_after_field_move() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude
#import "core/field" as field
#import "core/traits/drop" as *

struct GuardA:
    id <i32>
struct GuardB:
    id <i32>
struct Pair:
    left <GuardA>
    right <GuardB>

impl Drop for GuardA:
    fn drop <(&GuardA)*>()> (_self):
        ()

impl Drop for GuardB:
    fn drop <(&GuardB)*>()> (_self):
        ()

fn main <()->i32> ():
    let mut p <Pair> Pair (GuardA 0) (GuardB 0)
    let left <GuardA> field::get p "left"
    set p Pair (GuardA 1) (GuardB 1)
    0
"#;
    let (source_module, _) = typecheck_resource_source(source);
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let plan = compute_resource_drop_elaboration_plan(&resource, &report)
        .expect("partial assignment overwrite should build a checked drop elaboration plan");
    let main_plan = plan
        .functions
        .iter()
        .find(|function| function.name.starts_with("main"))
        .expect("main plan should exist");
    let overwrite_drop = main_plan
        .auto_drops
        .iter()
        .find(|drop| {
            drop.source_name == "p"
                && matches!(drop.kind, ResourceAutoDropKind::AssignmentOverwrite)
        })
        .expect("partially moved assignment target should drop remaining initialized descendants");
    match &overwrite_drop.requirement {
        ResourceDropRequirement::Structural {
            fields,
            dynamic_enum_fields,
        } => {
            assert_eq!(fields.len(), 1);
            assert!(dynamic_enum_fields.is_empty());
            assert_eq!(types.type_to_string(fields[0].ty), "GuardB");
        }
        other => panic!("partial overwrite must only drop the remaining GuardB field: {other:?}"),
    }
    nepl_core::resource::validate_resource_drop_elaboration_hir_bridge(&source_module, &plan)
        .expect("partial assignment overwrite drop should bridge to source HIR set binding");
}

#[test]
fn resource_ir_drop_elaboration_plan_skips_moved_assignment_targets() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (_self):
        ()

fn consume <(Guard)->i32> (_g):
    0

fn main <()->i32> ():
    let mut g <Guard> Guard 0
    let _ <i32> consume g
    set g Guard 1
    0
"#;
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let plan = compute_resource_drop_elaboration_plan(&resource, &report)
        .expect("moved assignment target should still build a plan");
    let main_plan = plan
        .functions
        .iter()
        .find(|function| function.name.starts_with("main"))
        .expect("main plan should exist");

    assert!(
        !main_plan.auto_drops.iter().any(|drop| {
            drop.source_name == "g"
                && matches!(drop.kind, ResourceAutoDropKind::AssignmentOverwrite)
        }),
        "moved target reinitialization must not emit an overwrite drop: {:#?}",
        main_plan.auto_drops
    );
    assert!(
        main_plan.auto_drops.iter().any(|drop| {
            drop.source_name == "g" && matches!(drop.kind, ResourceAutoDropKind::ScopeLocal)
        }),
        "the newly assigned value should still be dropped at scope exit"
    );
}

#[test]
fn resource_drop_insertion_consumes_checked_scope_and_assignment_points() {
    fn count_trait_drop_calls(expr: &HirExpr) -> usize {
        match &expr.kind {
            HirExprKind::Call { callee, args } => {
                let own = usize::from(matches!(
                    callee,
                    FuncRef::Trait { method, .. } if method.as_str() == "drop"
                ));
                own + args.iter().map(count_trait_drop_calls).sum::<usize>()
            }
            HirExprKind::CallIndirect { callee, args, .. } => {
                count_trait_drop_calls(callee)
                    + args.iter().map(count_trait_drop_calls).sum::<usize>()
            }
            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                count_trait_drop_calls(cond)
                    + count_trait_drop_calls(then_branch)
                    + count_trait_drop_calls(else_branch)
            }
            HirExprKind::While { cond, body } => {
                count_trait_drop_calls(cond) + count_trait_drop_calls(body)
            }
            HirExprKind::Match { scrutinee, arms } => {
                count_trait_drop_calls(scrutinee)
                    + arms
                        .iter()
                        .map(|arm| count_trait_drop_calls(&arm.body))
                        .sum::<usize>()
            }
            HirExprKind::Intrinsic { args, .. } => {
                args.iter().map(count_trait_drop_calls).sum::<usize>()
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                payload.as_deref().map(count_trait_drop_calls).unwrap_or(0)
            }
            HirExprKind::StructConstruct { fields, .. } => {
                fields.iter().map(count_trait_drop_calls).sum()
            }
            HirExprKind::TupleConstruct { items } => items.iter().map(count_trait_drop_calls).sum(),
            HirExprKind::Block(block) => block
                .lines
                .iter()
                .map(|line| count_trait_drop_calls(&line.expr))
                .sum(),
            HirExprKind::Let { value, .. }
            | HirExprKind::Set { value, .. }
            | HirExprKind::AddrOf(value)
            | HirExprKind::Deref(value) => count_trait_drop_calls(value),
            HirExprKind::FnValue(_)
            | HirExprKind::Var(_)
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Unit
            | HirExprKind::Drop { .. } => 0,
        }
    }
    fn collect_user_calls(expr: &HirExpr, out: &mut Vec<String>) {
        match &expr.kind {
            HirExprKind::Call { callee, args } => {
                if let FuncRef::User(name, _, _) = callee {
                    out.push(name.clone());
                }
                for arg in args {
                    collect_user_calls(arg, out);
                }
            }
            HirExprKind::CallIndirect { callee, args, .. } => {
                collect_user_calls(callee, out);
                for arg in args {
                    collect_user_calls(arg, out);
                }
            }
            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                collect_user_calls(cond, out);
                collect_user_calls(then_branch, out);
                collect_user_calls(else_branch, out);
            }
            HirExprKind::While { cond, body } => {
                collect_user_calls(cond, out);
                collect_user_calls(body, out);
            }
            HirExprKind::Match { scrutinee, arms } => {
                collect_user_calls(scrutinee, out);
                for arm in arms {
                    collect_user_calls(&arm.body, out);
                }
            }
            HirExprKind::Intrinsic { args, .. } => {
                for arg in args {
                    collect_user_calls(arg, out);
                }
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                if let Some(payload) = payload {
                    collect_user_calls(payload, out);
                }
            }
            HirExprKind::StructConstruct { fields, .. } => {
                for field in fields {
                    collect_user_calls(field, out);
                }
            }
            HirExprKind::TupleConstruct { items } => {
                for item in items {
                    collect_user_calls(item, out);
                }
            }
            HirExprKind::Block(block) => {
                for line in &block.lines {
                    collect_user_calls(&line.expr, out);
                }
            }
            HirExprKind::Let { value, .. }
            | HirExprKind::Set { value, .. }
            | HirExprKind::AddrOf(value)
            | HirExprKind::Deref(value) => collect_user_calls(value, out),
            HirExprKind::FnValue(_)
            | HirExprKind::Var(_)
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Unit
            | HirExprKind::Drop { .. } => {}
        }
    }

    let source = r#"
#entry main
#indent 4
#target core
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (_self):
        ()

fn main <()->i32> ():
    let mut g <Guard> Guard 0
    set g Guard 1
    0
"#;
    let (module, mut types) = typecheck_resource_source(source);
    let mono = nepl_core::monomorphize::monomorphize(&mut types, module);
    let (mut module, unresolved_trait_calls) = mono.into_parts();
    assert!(unresolved_trait_calls.is_empty());
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let plan = compute_resource_drop_elaboration_plan(&resource, &report)
        .expect("checked drop plan should be valid");
    let main_plan = plan
        .functions
        .iter()
        .find(|function| function.origin_name == "main")
        .expect("main drop plan should exist");
    assert!(main_plan
        .auto_drops
        .iter()
        .any(|drop| matches!(drop.kind, ResourceAutoDropKind::AssignmentOverwrite)));
    assert!(main_plan
        .auto_drops
        .iter()
        .any(|drop| matches!(drop.kind, ResourceAutoDropKind::ScopeLocal)));

    nepl_core::passes::insert_resource_drops(&mut module, &mut types, &plan)
        .expect("checked Resource IR drop plan should be consumed by HIR drop insertion");
    let main = module
        .functions
        .iter()
        .find(|function| function.origin_name == "main")
        .expect("main HIR function should remain present");
    let HirBody::Block(block) = &main.body else {
        panic!("main should be a block body");
    };
    let drop_calls = block
        .lines
        .iter()
        .map(|line| count_trait_drop_calls(&line.expr))
        .sum::<usize>();
    assert_eq!(
        drop_calls, 2,
        "assignment overwrite and scope exit should both be generated from checked drop facts"
    );

    let mono = nepl_core::monomorphize::monomorphize(&mut types, module);
    let (module, unresolved_trait_calls) = mono.into_parts();
    assert!(
        unresolved_trait_calls.is_empty(),
        "generated Drop trait calls must be resolvable by the final monomorphize pass: {:#?}",
        unresolved_trait_calls
    );
    let function_names = module
        .functions
        .iter()
        .map(|function| function.name.clone())
        .collect::<std::collections::BTreeSet<_>>();
    let mut user_calls = Vec::new();
    for function in &module.functions {
        if let HirBody::Block(block) = &function.body {
            for line in &block.lines {
                collect_user_calls(&line.expr, &mut user_calls);
            }
        }
    }
    assert!(
        user_calls.iter().all(|name| function_names.contains(name)),
        "final monomorphize must keep generated Drop impl call targets: calls={:?}, functions={:?}",
        user_calls,
        function_names
    );
}

#[test]
fn resource_ir_drop_elaboration_plan_uses_checked_live_drop_facts() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (_self):
        ()

fn consume <(Guard)->i32> (_g):
    0

fn main <()->i32> ():
    let x <Guard> Guard 1
    let _ <i32> if true:
        then:
            let x <Guard> Guard 2
            0
        else:
            0
    consume x
"#;
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let plan = compute_resource_drop_elaboration_plan(&resource, &report)
        .expect("checked live drop facts should build a codegen-facing plan");
    let main_plan = plan
        .functions
        .iter()
        .find(|function| function.name.starts_with("main"))
        .expect("main drop elaboration plan should exist");

    assert!(main_plan.auto_drops.iter().any(|drop| {
        matches!(&drop.place.root, PlaceRoot::Local(name) if name.starts_with("x#"))
    }));
    assert!(main_plan.auto_drops.iter().any(|drop| {
        drop.source_name == "x"
            && matches!(&drop.place.root, PlaceRoot::Local(name) if name.starts_with("x#"))
    }));
    assert!(!main_plan
        .auto_drops
        .iter()
        .any(|drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name == "x")));
    let inner_point = main_plan
        .drop_points
        .iter()
        .find(|point| {
            point.auto_drops.iter().any(
                |drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name.starts_with("x#")),
            )
        })
        .expect("inner live drop point should be present");
    resolve_resource_drop_point_end_scope(
        resource_function(&resource, &main_plan.name),
        &inner_point.path,
    )
    .expect("drop elaboration point must resolve to an EndScope");
}

#[test]
fn resource_ir_drop_elaboration_plan_rejects_invalid_checked_paths() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (_self):
        ()

fn ignore <(Guard)->i32> (_g):
    1

fn main <()->i32> ():
    ignore Guard 7
"#;
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let mut report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let ignore_check = report
        .functions
        .iter_mut()
        .find(|function| function.name.starts_with("ignore"))
        .expect("ignore resource check should exist");
    ignore_check.auto_drop_points[0].path = ResourceDropPointPath {
        block: ResourceBlockId(0),
        steps: vec![ResourceDropPointStep::Op { index: 9999 }],
    };

    let errors = compute_resource_drop_elaboration_plan(&resource, &report)
        .expect_err("invalid live drop point path must be rejected");
    assert!(errors.iter().any(|error| matches!(
        error,
        ResourceDropElaborationPlanError::InvalidDropPointPath {
            function,
            error:
                ResourceDropPointResolutionError::OpIndexOutOfBounds {
                    index: 9999,
                    ..
                },
            ..
        } if function.starts_with("ignore")
    )));
}

#[test]
fn resource_ir_drop_elaboration_plan_rejects_places_outside_end_scope() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (_self):
        ()

fn ignore <(Guard)->i32> (_g):
    1

fn main <()->i32> ():
    ignore Guard 7
"#;
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let mut report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
    let ignore_check = report
        .functions
        .iter_mut()
        .find(|function| function.name.starts_with("ignore"))
        .expect("ignore resource check should exist");
    let bad_place = Place::local(
        "not_in_this_scope".to_string(),
        ignore_check.auto_drop_points[0].auto_drops[0].place.ty,
    );
    ignore_check.auto_drop_points[0].auto_drops[0].place = bad_place.clone();

    let errors = compute_resource_drop_elaboration_plan(&resource, &report)
        .expect_err("live drop point place outside EndScope locals must be rejected");
    assert!(errors.iter().any(|error| matches!(
        error,
        ResourceDropElaborationPlanError::DropPlaceOutsideEndScope {
            function,
            place,
            ..
        } if function.starts_with("ignore") && place == &bad_place
    )));
}

#[test]
fn resource_ir_drop_elaboration_plan_rejects_missing_source_binding() {
    let (types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let place = Place::local("x#0".to_string(), owned_ty);
    let path = ResourceDropPointPath {
        block: ResourceBlockId(0),
        steps: vec![ResourceDropPointStep::Op { index: 0 }],
    };
    let resource = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            origin_name: "main".to_string(),
            type_params: Vec::new(),
            params: vec![],
            result: unit_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![ResourceOp::EndScope {
                    locals: vec![place.clone()],
                    result: None,
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
    let report = ResourceCheckReport {
        functions: vec![ResourceFunctionCheck {
            name: "main".to_string(),
            final_cells: vec![],
            final_collection_slots: vec![],
            auto_drop_points: vec![ResourceDropPoint {
                path: path.clone(),
                span,
                auto_drops: vec![ResourceAutoDrop {
                    place: place.clone(),
                    kind: ResourceAutoDropKind::ScopeLocal,
                    requirement: ResourceDropRequirement::StateOnly,
                    span,
                }],
            }],
            deferred: ResourceCheckDeferred::default(),
        }],
        diagnostics: vec![],
        deferred: ResourceCheckDeferred::default(),
    };

    let errors = compute_resource_drop_elaboration_plan(&resource, &report)
        .expect_err("drop elaboration must reject places without source binding names");
    assert!(errors.iter().any(|error| matches!(
        error,
        ResourceDropElaborationPlanError::MissingDropBinding {
            function,
            path: error_path,
            place: error_place,
            ..
        } if function == "main" && error_path == &path && error_place == &place
    )));
}

#[test]
fn resource_ir_drop_plan_classifies_structural_and_dynamic_payload_drops() {
    let source = r#"
#entry main
#indent 4
#target core
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (_self):
        ()

struct Holder:
    guard <Guard>
    count <i32>

enum MaybeGuard:
    Some <Guard>
    None

fn main <()->i32> ():
    let h <Holder> Holder (Guard 1) 2
    let e <MaybeGuard> MaybeGuard::Some Guard 3
    0
"#;
    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let drop_plan = compute_resource_drop_plan(&resource, &types);
    let main_drop_plan = drop_plan
        .functions
        .iter()
        .find(|function| function.name.starts_with("main"))
        .expect("main drop plan should exist");
    let top_scope_point = main_drop_plan
        .drop_points
        .iter()
        .find(|point| {
            point
                .auto_drops
                .iter()
                .any(|drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name == "h"))
                && point
                    .auto_drops
                    .iter()
                    .any(|drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name == "e"))
        })
        .expect("top-level EndScope should keep Holder and MaybeGuard in one drop point");
    assert_eq!(top_scope_point.auto_drops.len(), 2);
    assert!(!top_scope_point.path.steps.iter().any(|step| matches!(
        step,
        ResourceDropPointStep::BranchThen
            | ResourceDropPointStep::BranchElse
            | ResourceDropPointStep::LoopBody
            | ResourceDropPointStep::LoopCondition
            | ResourceDropPointStep::MatchArm { .. }
    )));
    let top_end_scope = resolve_resource_drop_point_end_scope(
        resource_function(&resource, &main_drop_plan.name),
        &top_scope_point.path,
    )
    .expect("top-level drop point path must resolve to an EndScope");
    assert!(top_end_scope
        .locals
        .iter()
        .any(|place| matches!(&place.root, PlaceRoot::Local(name) if name == "h")));
    assert!(top_end_scope
        .locals
        .iter()
        .any(|place| matches!(&place.root, PlaceRoot::Local(name) if name == "e")));

    let holder_drop = main_drop_plan
        .auto_drops
        .iter()
        .find(|drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name == "h"))
        .expect("Holder local should be an auto-drop candidate");
    match &holder_drop.requirement {
        ResourceDropRequirement::Structural {
            fields,
            dynamic_enum_fields,
        } => {
            assert_eq!(fields.len(), 1);
            assert!(dynamic_enum_fields.is_empty());
            assert_eq!(fields[0].offset, 0);
            assert_eq!(types.type_to_string(fields[0].ty), "Guard");
        }
        other => panic!("Holder must require structural field drop, got {other:?}"),
    }

    let enum_drop = main_drop_plan
        .auto_drops
        .iter()
        .find(|drop| matches!(&drop.place.root, PlaceRoot::Local(name) if name == "e"))
        .expect("MaybeGuard local should be an auto-drop candidate");
    assert!(matches!(
        enum_drop.requirement,
        ResourceDropRequirement::DynamicEnumPayload
    ));
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
            params: vec![],
            result: unit_ty,
            effect: Effect::Pure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::DeclareLocal {
                        place: x.clone(),
                        source_name: "x".to_string(),
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
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
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
                PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
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
fn resource_ir_cell_check_allows_zero_initialized_runtime_literal_raw_load() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let address = Place::temporary(ResourceId(0), i32_ty);
    let loaded = Place::temporary(ResourceId(1), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(4),
                output: address.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded,
                args: vec![address],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
            name: "MemPtr".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["raw".to_string()],
        },
    );
    let vec_ty = types.register_named(
        "VecLike".to_string(),
        TypeKind::Struct {
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
        PlaceProjection::StorageOffset(ResourceOffset::Known(8)),
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
        PlaceProjection::StorageOffset(ResourceOffset::Unknown),
        i32_ty,
    );
    let loaded = Place::temporary(ResourceId(5), i32_ty);
    let resource = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
                        synthetic: false,
                        span,
                    },
                    ResourceOp::RawAddressAlias {
                        source: data_ref_address,
                        target: data_ref.clone(),
                        kind: RawAddressAliasKind::Transparent,
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
                        source_name: "v_data".to_string(),
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
                        kind: RawAddressAliasKind::Transparent,
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
                source_name: "p".to_string(),
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
                origin_name: "slot_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
                params: vec![],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: ptr.clone(),
                            args: vec![],
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
                        ResourceOp::RawAddressAlias {
                            source: ptr.clone(),
                            target: returned.clone(),
                            kind: RawAddressAliasKind::Transparent,
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
                origin_name: "make_slot".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                origin_name: "make_slot".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            source_name: "p".to_string(),
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
fn resource_ir_cell_check_summarizes_unit_helper_argument_raw_cell_initialization() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let helper_param = Place::local("p".to_string(), i32_ty);
    let value = Place::temporary(ResourceId(0), i32_ty);
    let store_out = Place::temporary(ResourceId(1), unit_ty);
    let caller_buf = Place::temporary(ResourceId(2), i32_ty);
    let call_output = Place::temporary(ResourceId(3), unit_ty);
    let loaded = Place::temporary(ResourceId(4), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "fill_slot".to_string(),
                origin_name: "fill_slot".to_string(),
                type_params: Vec::new(),
                params: vec![ResourceLocal {
                    name: "p".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: helper_param.clone(),
                }],
                result: unit_ty,
                effect: Effect::Impure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: vec![
                        ResourceOp::Expr {
                            kind: ResourceExprKind::Literal,
                            output: value.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Store,
                            output: store_out,
                            args: vec![helper_param, value],
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return { value: None, span },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                origin_name: "main".to_string(),
                type_params: Vec::new(),
                params: vec![],
                result: unit_ty,
                effect: Effect::Impure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: caller_buf.clone(),
                            args: vec![],
                            span,
                        },
                        ResourceOp::Call {
                            output: call_output,
                            target: ResourceCallTarget::User {
                                name: "fill_slot".to_string(),
                                type_args: vec![],
                            },
                            args: vec![caller_buf.clone()],
                            effect: EffectOp::UserCall {
                                name: "fill_slot".to_string(),
                                effect: Effect::Impure,
                            },
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Load,
                            output: loaded,
                            args: vec![caller_buf],
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
        "unit helper argument raw cell initialization should reach caller: {:#?}\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_keeps_conditional_unit_helper_argument_init_conservative() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.bool());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let helper_param = Place::local("p".to_string(), i32_ty);
    let condition = Place::temporary(ResourceId(0), bool_ty);
    let then_value = Place::temporary(ResourceId(1), i32_ty);
    let then_store = Place::temporary(ResourceId(2), unit_ty);
    let then_unit = Place::temporary(ResourceId(3), unit_ty);
    let else_unit = Place::temporary(ResourceId(4), unit_ty);
    let branch_output = Place::temporary(ResourceId(5), unit_ty);
    let caller_buf = Place::temporary(ResourceId(6), i32_ty);
    let call_output = Place::temporary(ResourceId(7), unit_ty);
    let loaded = Place::temporary(ResourceId(8), i32_ty);
    let expected_cell = caller_buf
        .clone()
        .with_projection(PlaceProjection::Deref, i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "maybe_fill_slot".to_string(),
                origin_name: "maybe_fill_slot".to_string(),
                type_params: Vec::new(),
                params: vec![ResourceLocal {
                    name: "p".to_string(),
                    ty: i32_ty,
                    mutable: false,
                    place: helper_param.clone(),
                }],
                result: unit_ty,
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
                            output: branch_output,
                            condition,
                            condition_fact: None,
                            then_ops: vec![
                                ResourceOp::Expr {
                                    kind: ResourceExprKind::Literal,
                                    output: then_value.clone(),
                                    ty: i32_ty,
                                    span,
                                },
                                ResourceOp::RawMemory {
                                    operation: RawMemoryOp::Store,
                                    output: then_store,
                                    args: vec![helper_param, then_value],
                                    span,
                                },
                                ResourceOp::Expr {
                                    kind: ResourceExprKind::Literal,
                                    output: then_unit.clone(),
                                    ty: unit_ty,
                                    span,
                                },
                            ],
                            then_value: then_unit,
                            else_ops: vec![ResourceOp::Expr {
                                kind: ResourceExprKind::Literal,
                                output: else_unit.clone(),
                                ty: unit_ty,
                                span,
                            }],
                            else_value: else_unit,
                            span,
                        },
                    ],
                    terminator: ResourceTerminator::Return { value: None, span },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                origin_name: "main".to_string(),
                type_params: Vec::new(),
                params: vec![],
                result: unit_ty,
                effect: Effect::Impure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: caller_buf.clone(),
                            args: vec![],
                            span,
                        },
                        ResourceOp::Call {
                            output: call_output,
                            target: ResourceCallTarget::User {
                                name: "maybe_fill_slot".to_string(),
                                type_args: vec![],
                            },
                            args: vec![caller_buf.clone()],
                            effect: EffectOp::UserCall {
                                name: "maybe_fill_slot".to_string(),
                                effect: Effect::Impure,
                            },
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Load,
                            output: loaded,
                            args: vec![caller_buf],
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
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                function,
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                place,
                state: CellState::Uninit,
                ..
            } if function == "main" && place == &expected_cell
        )),
        "conditional unit helper argument initialization must not be summarized unconditionally: {:#?}\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_applies_result_ok_param_raw_cell_initialization() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()->i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok region:
            let p <MemPtr<i32>> region_ptr &region
            match store_i32 p 123:
                Result::Err _e:
                    0
                Result::Ok _:
                    let v <i32> match load_i32 p:
                        Option::None:
                            0
                        Option::Some x:
                            x
                    match dealloc_region region:
                        Result::Err _e:
                            0
                        Result::Ok _:
                            v
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
        "Result::Ok-gated RegionToken MemPtr store must initialize the caller raw cell before load: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_applies_result_ok_region_ptr_direct_store_initialization() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()->i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok region:
            let p <MemPtr<i32>> region_ptr &region
            match store_i32 p 123:
                Result::Err _e:
                    0
                Result::Ok _:
                    let v <i32> match load_i32 p:
                        Option::None:
                            0
                        Option::Some x:
                            x
                    match dealloc_region region:
                        Result::Err _drop:
                            0
                        Result::Ok _:
                            v
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
        "Result::Ok-gated RegionToken MemPtr store must initialize the direct raw cell before load: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_applies_result_ok_region_ptr_at_direct_store_initialization() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()->i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok region:
            match region_ptr_at<i32,i32> &region 0:
                Result::Err _ptr_err:
                    0
                Result::Ok p:
                    match store_i32 p 123:
                        Result::Err _store_err:
                            0
                        Result::Ok _:
                            let v <i32> match load_i32 p:
                                Option::None:
                                    0
                                Option::Some x:
                                    x
                            match dealloc_region region:
                                Result::Err _drop:
                                    0
                                Result::Ok _:
                                    v
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
        "Result::Ok-gated region_ptr_at MemPtr store must initialize the direct raw cell before load: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_does_not_apply_result_err_param_raw_cell_initialization() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn maybe_store <(bool,MemPtr<i32>,i32)->Result<(),str>> (flag, p, v):
    if:
        flag
        then:
            let raw <i32> mem_ptr_addr p
            store_i32 raw v
            Result<(),str>::Ok ()
        else:
            Result<(),str>::Err "skip"

fn main <()->i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok region:
            let p <MemPtr<i32>> region_ptr &region
            match maybe_store false p 123:
                Result::Err _e:
                    let v <i32> match load_i32 p:
                        Option::None:
                            0
                        Option::Some x:
                            x
                    match dealloc_region region:
                        Result::Err _drop:
                            0
                        Result::Ok _:
                            v
                Result::Ok _:
                    match dealloc_region region:
                        Result::Err _drop:
                            0
                        Result::Ok _:
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
        main_diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                state: CellState::Uninit,
                ..
            }
        )),
        "Result::Err arm must not receive Result::Ok-gated raw cell initialization: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_skips_unreachable_mem_ptr_load_some_requirement() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/option" as *

fn main <()->i32> ():
    let p <MemPtr<i32>> mem_ptr_wrap 0
    match load_i32 p:
        Option::None:
            1
        Option::Some _v:
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
        "statically invalid MemPtr load can only return Option::None: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_keeps_reachable_mem_ptr_load_some_requirement() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()->i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok region:
            let p <MemPtr<i32>> region_ptr &region
            let v <i32> match load_i32 p:
                Option::None:
                    0
                Option::Some x:
                    x
            match dealloc_region region:
                Result::Err _drop:
                    0
                Result::Ok _:
                    v
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                state: CellState::Uninit,
                ..
            }
        )),
        "reachable Option::Some from unknown valid MemPtr must still require initialized cell: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_applies_result_ok_mem_ptr_dealloc_consumption() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>()> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            ()
        Result::Ok region:
            let p <MemPtr<i32>> region_ptr &region
            match store_i32 p 123:
                Result::Err _store:
                    match dealloc_region region:
                        Result::Ok _:
                            ()
                        Result::Err _:
                            ()
                Result::Ok _store:
                    let _v <i32> match load_i32 p:
                        Option::None:
                            0
                        Option::Some x:
                            x
                    match dealloc_region region:
                        Result::Ok _:
                            ()
                        Result::Err _drop:
                            ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function == "main" || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.is_empty(),
        "dealloc_region must consume the RegionToken owner only in Result::Ok and preserve it in Result::Err: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_proves_checked_dealloc_err_unreachable_for_computed_alloc_size() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn release_arg_vector <(i32,i32)->i32> (idx, argc):
    if lt idx 0:
        then:
            0
        else:
            if ge idx argc:
                then:
                    0
                else:
                    let argv_size <i32> mul argc 4
                    match alloc_region_bytes<u8> argv_size:
                        Result::Err _e:
                            0
                        Result::Ok argv:
                            match dealloc_region<u8> argv:
                                Result::Ok _:
                                    1
                                Result::Err _:
                                    2

fn main <()->i32> ():
    release_arg_vector 0 1
"#;

    let (module, mut types) = typecheck_resource_source(source);
    let mono = nepl_core::monomorphize::monomorphize(&mut types, module);
    let (module, unresolved_trait_calls) = mono.into_parts();
    assert!(
        unresolved_trait_calls.is_empty(),
        "unresolved trait calls: {:#?}",
        unresolved_trait_calls
    );
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
            function == "release_arg_vector" || function.starts_with("release_arg_vector__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "Result::Ok from alloc_region_bytes must carry enough scalar facts to prove checked dealloc_region Err unreachable for the same computed size: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_applies_result_ok_raw_dealloc_consumption() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc 8:
        Result::Err _e:
            0
        Result::Ok p:
            store_i32 p 77
            let ok <i32> if eq load_i32 p 77 1 0
            match dealloc p 8:
                Result::Err _e:
                    0
                Result::Ok _:
                    ok
"#;

    let (module, mut types) = typecheck_resource_source(source);
    let mono = nepl_core::monomorphize::monomorphize(&mut types, module);
    let (module, unresolved_trait_calls) = mono.into_parts();
    assert!(
        unresolved_trait_calls.is_empty(),
        "unresolved trait calls: {:#?}",
        unresolved_trait_calls
    );
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function == "main" || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.is_empty(),
        "dealloc must consume the raw owner only in Result::Ok and preserve cleanup in Result::Err: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
    assert!(
        compile_resource_source_with_raw_boundary(source, CompileTarget::Wasi).is_ok(),
        "compiler pipeline must accept the same checked raw dealloc pattern"
    );
}

#[test]
fn resource_ir_owner_check_resolves_unwrap_ok_raw_dealloc_consumption() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>()> ():
    let data <i32> unwrap_ok alloc 8
    store_i32 data 77;
    unwrap_ok dealloc data 8;
"#;

    let (module, mut types) = typecheck_resource_source(source);
    let mono = nepl_core::monomorphize::monomorphize(&mut types, module);
    let (module, unresolved_trait_calls) = mono.into_parts();
    assert!(
        unresolved_trait_calls.is_empty(),
        "unresolved trait calls: {:#?}",
        unresolved_trait_calls
    );
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function == "main" || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.is_empty(),
        "unwrap_ok must resolve the Result::Ok owner effects from alloc/dealloc: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
    compile_resource_source_with_raw_boundary(source, CompileTarget::Wasm)
        .expect("compiler pipeline must accept unwrap_ok alloc/dealloc owner flow");
}

#[test]
fn resource_ir_owner_check_accepts_borrowed_region_ptr_at_then_region_dealloc() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            0
        Result::Ok token:
            match region_ptr_at<i32,i32> &token 0:
                Result::Err _e:
                    match dealloc_region token:
                        Result::Err _drop:
                            0
                        Result::Ok _:
                            0
                Result::Ok p:
                    match store_i32 p 321:
                        Result::Err _e:
                            match dealloc_region token:
                                Result::Err _drop:
                                    0
                                Result::Ok _:
                                    0
                        Result::Ok _:
                            let v <i32> match load_i32 p:
                                Option::None:
                                    0
                                Option::Some x:
                                    x
                            match dealloc_region token:
                                Result::Err _e:
                                    0
                                Result::Ok _:
                                    v
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect("borrowed region_ptr_at must preserve RegionToken owner for dealloc_region");
}

#[test]
fn resource_ir_owner_check_accepts_borrowed_region_ptr_retag_then_region_dealloc() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_region<u8> 16:
        Result::Err _e:
            0
        Result::Ok token:
            let p_u8 <MemPtr<u8>> region_ptr &token
            let p_i32 <MemPtr<i32>> mem_ptr_wrap mem_ptr_addr p_u8
            match fill_i32 p_i32 4 7:
                Result::Err _e:
                    match dealloc_region token:
                        Result::Err _drop:
                            0
                        Result::Ok _:
                            0
                Result::Ok _:
                    let ok <i32> match load_i32 p_i32:
                        Option::None:
                            0
                        Option::Some v:
                            if eq v 7 1 0
                    match dealloc_region token:
                        Result::Err _e:
                            0
                        Result::Ok _:
                            ok
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect("borrowed region_ptr MemPtr retag must remain non-owning until token dealloc");
}

#[test]
fn resource_ir_cell_check_accepts_retagged_mem_ptr_after_byte_and_word_fill() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_region<u8> 16:
        Result::Err _e:
            0
        Result::Ok token:
            let p_u8 <MemPtr<u8>> region_ptr &token
            let p_i32 <MemPtr<i32>> mem_ptr_wrap mem_ptr_addr p_u8
            match fill_u8 p_u8 16 0:
                Result::Err _e:
                    match dealloc_region token:
                        Result::Err _drop:
                            0
                        Result::Ok _:
                            0
                Result::Ok _:
                    match fill_i32 p_i32 4 7:
                        Result::Err _e:
                            match dealloc_region token:
                                Result::Err _drop:
                                    0
                                Result::Ok _:
                                    0
                        Result::Ok _:
                            let ok <i32> match load_i32 p_i32:
                                Option::None:
                                    0
                                Option::Some v:
                                    if eq v 7 1 0
                            match dealloc_region token:
                                Result::Err _e:
                                    0
                                Result::Ok _:
                                    ok
"#;

    let (module, types) = typecheck_resource_source_with_target(source, CompileTarget::Wasm);
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
        "retagged MemPtr fill_i32 must initialize the typed cell read by load_i32: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );

    compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect("borrowed region_ptr retag with byte and word fill must compile");
}

#[test]
fn resource_ir_owner_check_rejects_raw_dealloc_after_region_dealloc_consumes_token() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>()> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            ()
        Result::Ok region:
            let raw <i32> *region_token_raw_ref &region
            let r <Result<(),str>> dealloc_region region
            dealloc_raw raw 4
            match r:
                Result::Ok _:
                    ()
                Result::Err _drop:
                    ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                    if function == "main" || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::Dealloc,
                state: OwnerState::NoFreeObligation,
                ..
            }
        )),
        "dealloc_region consumes the RegionToken owner; the borrowed raw view must not carry a second free obligation: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_rejects_old_raw_dealloc_after_realloc_region_consumes_token() {
    let source = r#"
#entry main
#indent 4
#target std

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn main <()*>()> ():
    match alloc_region<i32> 1:
        Result::Err _e:
            ()
        Result::Ok region:
            let old_raw <i32> *region_token_raw_ref &region
            let r <Result<RegionToken<i32>,RegionReallocError<i32>>> realloc_region_bytes_keep<i32> region 8
            dealloc_raw old_raw 4
            match r:
                Result::Ok q:
                    match dealloc_region q:
                        Result::Ok _:
                            ()
                        Result::Err _drop:
                            ()
                Result::Err _grow:
                    let old <RegionToken<i32>> region_realloc_error_region<i32> _grow
                    match dealloc_region old:
                        Result::Ok _:
                            ()
                        Result::Err _:
                            ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                    if function == "main" || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::Dealloc,
                state: OwnerState::NoFreeObligation,
                ..
            }
        )),
        "realloc_region consumes the old RegionToken owner; the borrowed old raw view must not carry a second free obligation before Result refinement: {:#?}\nresource:\n{}",
        main_diagnostics,
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
        PlaceProjection::StorageOffset(ResourceOffset::Known(8)),
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
                kind: ResourceExprKind::LiteralI32(4),
                output: fill_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
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
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let loaded = Place::temporary(ResourceId(8), i32_ty);
    let resource = ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "make_header".to_string(),
                origin_name: "make_header".to_string(),
                type_params: Vec::new(),
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
                            kind: ResourceExprKind::LiteralI32(4),
                            output: fill_len.clone(),
                            ty: i32_ty,
                            span,
                        },
                        ResourceOp::Expr {
                            kind: ResourceExprKind::LiteralI32(0),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            source_name: "sc".to_string(),
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
                            source_name: "buf".to_string(),
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
fn resource_ir_cell_check_preserves_dynamic_fill_origin_across_local_reads() {
    let source = r#"
#entry main
#indent 4
#target wasi

#import "core/math" as *
#import "core/result" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn main <()->i32> ():
    let n <i32> 4
    let pref_len <i32> add n 1
    let pref <i32> unwrap_ok alloc mul pref_len 4
    fill_i32 pref pref_len 0
    let i <i32> 1
    let im1 <i32> sub i 1
    let prev_off <i32> mul im1 4
    let prev_ptr <i32> add pref prev_off
    let prev <i32> if and ge im1 0 lt im1 pref_len:
        then:
            load_i32 prev_ptr
        else:
            0
    prev
"#;

    let (module, types) = typecheck_resource_source_with_target(source, CompileTarget::Wasi);
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
        "dynamic fill through one local read must initialize guarded scaled dynamic loads through later reads of the same raw address: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_preserves_dynamic_fill_across_impure_i32_reads() {
    let source = r#"
#entry main
#indent 4
#target wasi

#import "core/math" as *
#import "core/result" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn next <()*>i32> ():
    1

fn main <()*>i32> ():
    let n <i32> 4
    let pref_len <i32> add n 1
    let pref <i32> unwrap_ok alloc mul pref_len 4
    fill_i32 pref pref_len 0
    let l1 <i32> next
    let r1 <i32> next
    let l <i32> sub l1 1
    let left_off <i32> mul l 4
    let right_off <i32> mul r1 4
    let left_ptr <i32> add pref left_off
    let right_ptr <i32> add pref right_off
    let diff <i32> if:
        and and ge l 0 lt l pref_len and ge r1 0 lt r1 pref_len
        then:
            let left <i32> load_i32 left_ptr
            let right <i32> load_i32 right_ptr
            sub right left
        else:
            0
    diff
"#;

    let (module, types) = typecheck_resource_source_with_target(source, CompileTarget::Wasi);
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
        "impure i32-producing calls must not erase guarded initialized Copy facts for unrelated raw buffers: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
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
                origin_name: "slot_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
                params: vec![],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(1),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(1),
                    ops: vec![
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Alloc,
                            output: ptr.clone(),
                            args: vec![],
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
                        ResourceOp::RawAddressAlias {
                            source: ptr.clone(),
                            target: returned.clone(),
                            kind: RawAddressAliasKind::Transparent,
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
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: ptr.clone(),
                args: vec![],
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
                    operation: RawMemoryOp::Store,
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
fn resource_ir_cell_check_store_reinitializes_moved_raw_cell() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let ptr = Place::temporary(ResourceId(0), i32_ty);
    let first_value = Place::temporary(ResourceId(1), owned_ty);
    let first_store = Place::temporary(ResourceId(2), unit_ty);
    let first_load = Place::temporary(ResourceId(3), owned_ty);
    let second_value = Place::temporary(ResourceId(4), owned_ty);
    let second_store = Place::temporary(ResourceId(5), unit_ty);
    let second_load = Place::temporary(ResourceId(6), owned_ty);
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
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: first_load,
                args: vec![ptr.clone()],
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
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: second_load,
                args: vec![ptr],
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
fn resource_ir_owner_check_rejects_raw_dealloc_extent_mismatch() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/math" as *

fn main <()*>()> ():
    let p <i32> alloc_raw 4
    dealloc_raw p 8
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::DeallocExtent,
                state: OwnerState::Live { .. },
                ..
            }
        )),
        "raw dealloc must prove that the size argument matches the allocation extent: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_rejects_raw_realloc_old_extent_mismatch() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/math" as *

fn main <()*>()> ():
    let p <i32> alloc_raw 4
    let q <i32> realloc_raw p 8 16
    if:
        lt 0 q
        then:
            dealloc_raw q 16
        else:
            dealloc_raw p 4
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::ReallocExtent,
                state: OwnerState::Live { .. },
                ..
            }
        )),
        "raw realloc must prove that old_size matches the current allocation extent: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_accepts_symbolic_raw_dealloc_extent_match() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/math" as *

fn main <(i32)*>()> (len):
    let bytes <i32> mul len 4
    let p <i32> alloc_raw bytes
    dealloc_raw p bytes
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let main_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function == "main" || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.is_empty(),
        "same symbolic byte extent must remain provable through local reads: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_rejects_region_new_extent_mismatch_through_summary() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/result" as *

fn main <()*>()> ():
    let raw <i32> alloc_raw 4
    let token <RegionToken<i32>> region_new<i32> raw 8
    match dealloc_region<i32> token:
        Result::Ok _:
            ()
        Result::Err _drop:
            dealloc_raw raw 4
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::ConstructInput,
                state: OwnerState::Live { .. },
                ..
            }
        )),
        "region_new summary must preserve the allocation extent requirement before dealloc_region: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_rejects_region_new_extent_mismatch_before_realloc_summary() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/result" as *

fn main <()*>()> ():
    let raw <i32> alloc_raw 4
    let token <RegionToken<i32>> region_new<i32> raw 8
    match realloc_region_bytes_keep<i32> token 16:
        Result::Ok q:
            match dealloc_region<i32> q:
                Result::Ok _:
                    ()
                Result::Err _:
                    ()
        Result::Err e:
            let old <RegionToken<i32>> region_realloc_error_region<i32> e
            match dealloc_region old:
                Result::Ok _:
                    ()
                Result::Err _:
                    dealloc_raw raw 4
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::ConstructInput,
                state: OwnerState::Live { .. },
                ..
            }
        )),
        "region_new summary must preserve the old-size allocation extent requirement before realloc_region: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_preserves_region_realloc_result_owner() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/result" as *

fn main <()*>()> ():
    match alloc_region_bytes<i32> 4:
        Result::Err _e:
            ()
        Result::Ok region:
            match realloc_region_bytes_keep<i32> region 8:
                Result::Ok grown:
                    match dealloc_region<i32> grown:
                        Result::Ok _:
                            ()
                        Result::Err _:
                            #intrinsic "unreachable" <> ()
                Result::Err e:
                    let old <RegionToken<i32>> region_realloc_error_region<i32> e
                    match dealloc_region<i32> old:
                        Result::Ok _:
                            ()
                        Result::Err _:
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
            function.starts_with("realloc_region_bytes_keep__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "RegionToken realloc summary must return the owner through both Result variants: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_transfers_raw_pointer_read_before_dealloc() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let p = Place::local("p".to_string(), i32_ty);
    let load_address = Place::temporary(ResourceId(0), i32_ty);
    let loaded = Place::temporary(ResourceId(1), i32_ty);
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
                args: vec![load_address.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(3), unit_ty),
                args: vec![load_address],
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
            ResourceOp::RawAddressAlias {
                source: p.clone(),
                target: alias.clone(),
                kind: RawAddressAliasKind::Transparent,
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
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                function,
                operation: ResourceOwnerOperation::Dealloc,
                place,
                state: OwnerState::Freed,
                ..
            } if function == "main" && matches!(&place.root, PlaceRoot::Local(name) if name == "p")
        )),
        "expected stale alias to resolve to the freed owner, got {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
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
                source_name: "p".to_string(),
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
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let p = Place::local("p".to_string(), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: p.clone(),
                args: vec![Place::i32_constant(4, i32_ty)],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(0), unit_ty),
                args: vec![p.clone(), Place::i32_constant(4, i32_ty)],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(1), unit_ty),
                args: vec![p.clone(), Place::i32_constant(4, i32_ty)],
                span,
            },
        ],
    );
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                function,
                operation: ResourceOwnerOperation::Dealloc,
                place,
                state: OwnerState::Freed,
                ..
            } if function == "main" && matches!(&place.root, PlaceRoot::Local(name) if name == "p")
        )),
        "expected double dealloc Freed diagnostic, got {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
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
                origin_name: "alloc_owner".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
fn resource_ir_owner_check_does_not_treat_plain_i32_identity_as_owner_return() {
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
                origin_name: "owner_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            args: vec![p.clone()],
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
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerLeaked {
                function,
                place,
                ..
            } if function == "main" && place == &p
        )),
        "plain i32 identity helpers must not implicitly transfer raw ownership: {:#?}",
        report.diagnostics
    );
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
                origin_name: "bool_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
fn resource_ir_owner_summary_does_not_treat_plain_i32_struct_fields_as_owners() {
    let source = r#"
#entry main
#indent 4
#target core

struct SourceSpan:
    file_id <i32>
    start <i32>
    end <i32>

struct Token:
    kind <i32>
    span <SourceSpan>

fn make_token <(i32,i32,i32)->Token> (file_id, start, end):
    let span <SourceSpan> SourceSpan file_id start end
    Token 0 span

fn main <()*>()> ():
    let token <Token> make_token 0 1 2
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
            function.starts_with("make_token__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "plain i32 span fields must not become owner obligations: {diagnostics:#?}\nresource:\n{}",
        resource.dump_text()
    );
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
                origin_name: "copy_loop".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                origin_name: "consume_owner".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                origin_name: "dealloc_raw".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                                operation: RawMemoryOp::Dealloc,
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
                origin_name: "alloc_owner".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                origin_name: "make_owner".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
fn resource_ir_owner_check_transfers_aggregate_owner_returned_by_function_value() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let wrapper_ty = types.register_named(
        "Wrapper".to_string(),
        TypeKind::Struct {
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
    let callee = Place::temporary(ResourceId(2), i32_ty);
    let returned = Place::temporary(ResourceId(3), wrapper_ty);
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
                name: "owner_wrapper_id".to_string(),
                origin_name: "owner_wrapper_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                        ResourceOp::FunctionValue {
                            output: callee.clone(),
                            name: "owner_wrapper_id".to_string(),
                            effect: EffectOp::UserCall {
                                name: "owner_wrapper_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::IndirectCall {
                            output: returned.clone(),
                            callee,
                            params: vec![wrapper_ty],
                            result: wrapper_ty,
                            args: vec![wrapper],
                            effect: EffectOp::UserCall {
                                name: "owner_wrapper_id".to_string(),
                                effect: Effect::Pure,
                            },
                            span,
                        },
                        ResourceOp::RawMemory {
                            operation: RawMemoryOp::Dealloc,
                            output: Place::temporary(ResourceId(4), unit_ty),
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
                    reason: UnknownEffectReason::CallbackParameterWithoutKnownEffect,
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
fn resource_ir_owner_check_preserves_non_owning_arg_to_unknown_callback() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let owner = Place::temporary(ResourceId(0), i32_ty);
    let view = Place::temporary(ResourceId(1), i32_ty);
    let callee = Place::local("callback".to_string(), i32_ty);
    let returned = Place::temporary(ResourceId(2), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: owner.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawAddressView {
                source: owner.clone(),
                target: view.clone(),
                kind: RawAddressViewKind::NonOwningProjection,
                span,
            },
            ResourceOp::IndirectCall {
                output: returned,
                callee,
                params: vec![i32_ty],
                result: i32_ty,
                args: vec![view],
                effect: EffectOp::Unknown {
                    reason: UnknownEffectReason::CallbackParameterWithoutKnownEffect,
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(3), unit_ty),
                args: vec![owner],
                span,
            },
        ],
    );

    let report = check_resource_owner_obligations(&resource, &types);
    assert_eq!(
        report.diagnostics,
        vec![],
        "passing a non-owning raw address view to an unknown callback must not require a free obligation: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_rejects_unknown_callback_return_with_non_owning_candidate() {
    let types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let owned_return_candidate = Place::temporary(ResourceId(0), i32_ty);
    let view_owner = Place::temporary(ResourceId(1), i32_ty);
    let view = Place::temporary(ResourceId(2), i32_ty);
    let callee = Place::local("callback".to_string(), i32_ty);
    let returned = Place::temporary(ResourceId(3), i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: owned_return_candidate.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: view_owner.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawAddressView {
                source: view_owner.clone(),
                target: view.clone(),
                kind: RawAddressViewKind::NonOwningProjection,
                span,
            },
            ResourceOp::IndirectCall {
                output: returned.clone(),
                callee,
                params: vec![i32_ty, i32_ty],
                result: i32_ty,
                args: vec![owned_return_candidate, view],
                effect: EffectOp::Unknown {
                    reason: UnknownEffectReason::CallbackParameterWithoutKnownEffect,
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(4), unit_ty),
                args: vec![returned.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: Place::temporary(ResourceId(5), unit_ty),
                args: vec![view_owner],
                span,
            },
        ],
    );

    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                place,
                state: OwnerState::NoFreeObligation,
                operation: ResourceOwnerOperation::Dealloc,
                ..
            } if place == &returned
        )),
        "unknown callback output must not become a definite owner when a non-owning same-type argument could be returned: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_moves_owner_into_constructed_aggregate() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let wrapper_ty = types.register_named(
        "Wrapper".to_string(),
        TypeKind::Struct {
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
                origin_name: "make_wrapper".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                origin_name: "id_wrapper".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
fn resource_ir_owner_check_accepts_checks_print_machine_report_return() {
    let source = r#"
#entry main
#indent 4
#target std

#import "std/test" as *

fn main <()* >i32> ():
    let mut report checks_new
    set report checks_push report assert "initial" true
    let shown checks_print_machine report
    checks_exit_code shown
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
            function.starts_with("checks_print_machine__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "checks_print_machine must return the same live TestReport owner without leaking temporary projections: {:#?}\nresource:\n{}",
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
fn resource_ir_owner_check_safe_realloc_variant_return_preserves_err_owner() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *
#import "core/option" as *

fn grow_or_keep <(bool)->i32> (grow):
    let p <i32> alloc_raw 4
    store_i32 p 77
    let new_size <i32> if:
        grow
        then:
            8
        else:
            0
    match realloc p 4 new_size:
        Result::Err _e:
            let v <i32> load_i32 p
            dealloc_raw p 4
            v
        Result::Ok q:
            let v <i32> load_i32 q
            dealloc_raw q new_size
            v

fn main <()->i32> ():
    let a <i32> grow_or_keep true
    let b <i32> grow_or_keep false
    add a b
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
            function.starts_with("grow_or_keep__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "realloc Err must keep the original owner while Ok transfers it to the payload: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_accepts_realloc_owner_replacement_assignment() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn grow_replace_and_free <()*>()> ():
    match alloc_region_bytes<i32> 4:
        Result::Err _e:
            ()
        Result::Ok region0:
            let mut region <RegionToken<i32>> region0
            match realloc_region_bytes_keep<i32> region 8:
                Result::Ok grown:
                    set region grown
                    match dealloc_region<i32> region:
                        Result::Ok _:
                            ()
                        Result::Err _drop:
                            ()
                Result::Err _grow:
                    set region region_realloc_error_region<i32> _grow
                    match dealloc_region<i32> region:
                        Result::Ok _:
                            ()
                        Result::Err _drop:
                            ()

fn main <()*>()> ():
    grow_replace_and_free
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
            function.starts_with("grow_replace_and_free__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "realloc Ok payload assignment must replace the old handle without losing the same owner obligation: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_accepts_loop_realloc_owner_replacement() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn loop_grow_replace_and_free <()*>()> ():
    match alloc_region_bytes<i32> 4:
        Result::Err _e:
            ()
        Result::Ok region0:
            let mut region <RegionToken<i32>> region0
            let mut cap <i32> 4
            let mut done <i32> 0
            while eq done 0:
                do:
                    match realloc_region_bytes_keep<i32> region 8:
                        Result::Ok grown:
                            set region grown
                            set cap 8
                            set done 1
                        Result::Err _grow:
                            set region region_realloc_error_region<i32> _grow
                            set done 1
            match dealloc_region<i32> region:
                Result::Ok _:
                    ()
                Result::Err _drop:
                    #intrinsic "unreachable" <> ()

fn main <()*>()> ():
    loop_grow_replace_and_free
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
            function.starts_with("loop_grow_replace_and_free__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "loop-carried realloc replacement must keep the owner obligation live until final dealloc: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_preserves_branch_result_variant_owner_return() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/diag/error" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn make_branch_result <(bool)*>Result<ByteBuf, StdErrorKind>> (ok_flag):
    match alloc_region_bytes<u8> 3:
        Result::Err _e:
            Result<ByteBuf, StdErrorKind>::Err StdErrorKind::OutOfMemory
        Result::Ok out:
            let res <Result<ByteBuf, StdErrorKind>> if:
                cond:
                    ok_flag
                then:
                    Result<ByteBuf, StdErrorKind>::Ok io_bytebuf_finish_region out 3
                else:
                    match dealloc_region<u8> out:
                        Result::Ok _:
                            ()
                        Result::Err _:
                            #intrinsic "unreachable" <> ()
                    Result<ByteBuf, StdErrorKind>::Err StdErrorKind::InvalidOperation
            res

fn main <()*>()> ():
    match make_branch_result true:
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
            function.starts_with("make_branch_result__")
                || function.starts_with("io_bytebuf_free__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "branch output Result::Ok payload must retain the owner return effect through a local binding and function return: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_preserves_branch_result_from_owner_returning_helper() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/diag/error" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn finish_bytes <(RegionToken<u8>,i32)*>Result<ByteBuf, StdErrorKind>> (region, len):
    Result<ByteBuf, StdErrorKind>::Ok io_bytebuf_finish_region region len

fn make_helper_branch_result <(bool)*>Result<ByteBuf, StdErrorKind>> (ok_flag):
    match alloc_region_bytes<u8> 3:
        Result::Err _e:
            Result<ByteBuf, StdErrorKind>::Err StdErrorKind::OutOfMemory
        Result::Ok out:
            let res <Result<ByteBuf, StdErrorKind>> if:
                cond:
                    ok_flag
                then:
                    finish_bytes out 3
                else:
                    match dealloc_region<u8> out:
                        Result::Ok _:
                            ()
                        Result::Err _:
                            #intrinsic "unreachable" <> ()
                    Result<ByteBuf, StdErrorKind>::Err StdErrorKind::InvalidOperation
            res

fn main <()*>()> ():
    match make_helper_branch_result true:
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
            function.starts_with("finish_bytes__")
                || function.starts_with("make_helper_branch_result__")
                || function.starts_with("io_bytebuf_free__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "branch output must retain a helper's Result::Ok owner return effect through the local result binding: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_prefers_live_return_owner_over_moved_source_alias() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/diag/error" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn finish_or_error <(RegionToken<u8>,i32,bool)*>Result<ByteBuf, StdErrorKind>> (region, len, ok_flag):
    if:
        cond:
            ok_flag
        then:
            Result<ByteBuf, StdErrorKind>::Ok io_bytebuf_finish_region region len
        else:
            match dealloc_region<u8> region:
                Result::Ok _:
                    ()
                Result::Err _:
                    #intrinsic "unreachable" <> ()
            Result<ByteBuf, StdErrorKind>::Err StdErrorKind::InvalidOperation

fn make_helper_result <(bool)*>Result<ByteBuf, StdErrorKind>> (ok_flag):
    match alloc_region_bytes<u8> 3:
        Result::Err _e:
            Result<ByteBuf, StdErrorKind>::Err StdErrorKind::OutOfMemory
        Result::Ok out:
            let res <Result<ByteBuf, StdErrorKind>> finish_or_error out 3 ok_flag;
            res

fn main <()*>()> ():
    match make_helper_result true:
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
            function.starts_with("finish_or_error__")
                || function.starts_with("make_helper_result__")
                || function.starts_with("io_bytebuf_free__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "returned owner aliases must resolve to the live Result::Ok payload, not the moved source handle: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_does_not_reconsume_unconditional_variant_argument() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/diag/error" as *
#import "alloc/io" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *
#import "std/text" as *

fn make_text_bytes <()* >Result<ByteBuf, StdErrorKind>> ():
    match alloc_region_bytes<u8> 2:
        Result::Err _e:
            Result<ByteBuf, StdErrorKind>::Err StdErrorKind::OutOfMemory
        Result::Ok region:
            let out <MemPtr<u8>> region_ptr &region
            store_u8 out 'o'
            store_u8 mem_ptr_add out 1 'k'
            Result<ByteBuf, StdErrorKind>::Ok io_bytebuf_finish_region region 2

fn main <()* >str> ():
    match make_text_bytes:
        Result::Ok bytes:
            match text_bytebuf_to_utf8_str_result bytes:
                Result::Ok text:
                    text
                Result::Err _e:
                    ""
        Result::Err _e:
            ""
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
            function.starts_with("make_text_bytes__")
                || function.starts_with("text_bytebuf_to_utf8_str_result__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "variant effects must not reconsume a ByteBuf argument already consumed by the callee summary: {:#?}\nresource:\n{}",
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
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
fn resource_ir_owner_check_vec_push_error_owner_does_not_leak_through_result_err() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *
#import "core/result" as *

fn push_or_error <(Vec<i32>,i32)*>Result<Vec<i32>,str>> (items, item):
    match push<i32> items item:
        Result::Ok out:
            Result<Vec<i32>,str>::Ok out
        Result::Err err:
            let returned <Vec<i32>> vec_push_error_vec<i32> err
            free<i32> returned
            Result<Vec<i32>,str>::Err "oom"

fn main <()*>()> ():
    let items <Vec<i32>> unwrap_ok new<i32>
    match push_or_error items 1:
        Result::Ok out:
            free<i32> out
        Result::Err _err:
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
            function.starts_with("push_or_error__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "Vec.push failure payload must not leave an inactive Result::Ok owner live on the Err return path: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_transfers_nested_btree_insert_error_owner_through_helper() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/btreemap" as *
#import "alloc/diag/error" as *
#import "core/result" as *

fn must_map <(Result<BTreeMap<i32,i32>, BTreeMapInsertError<i32,i32>>)*>BTreeMap<i32,i32>> (r):
    match r:
        Result::Ok m:
            m
        Result::Err e:
            let _d <Diag> btreemap_insert_error_diag<i32,i32> &e
            btreemap_insert_error_owner<i32,i32> e

fn main <()*>()> ():
    let map0 <BTreeMap<i32,i32>> unwrap_ok<BTreeMap<i32,i32>, Diag> new<i32,i32>
    let map1 <BTreeMap<i32,i32>> must_map insert<i32,i32> map0 1 10
    free<i32,i32> map1
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
            function.starts_with("must_map__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "nested BTreeMap insert error owner must transfer through helper summary: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_recursive_vec_result_err_does_not_keep_inactive_ok_owner() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/result" as *

fn step_a <(i32,Vec<i32>)*>Result<Vec<i32>,str>> (n, items):
    if:
        le n 0
        then:
            Result<Vec<i32>,str>::Ok items
        else:
            match push<i32> items n:
                Result::Ok out:
                    step_b sub n 1 out
                Result::Err err:
                    let returned <Vec<i32>> vec_push_error_vec<i32> err
                    free<i32> returned
                    Result<Vec<i32>,str>::Err "oom"

fn step_b <(i32,Vec<i32>)*>Result<Vec<i32>,str>> (n, items):
    if:
        le n 0
        then:
            Result<Vec<i32>,str>::Ok items
        else:
            step_a n items

fn main <()*>()> ():
    let items <Vec<i32>> unwrap_ok new<i32>
    match step_a 2 items:
        Result::Ok out:
            free<i32> out
        Result::Err _err:
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
            function.starts_with("step_a__")
                || function.starts_with("step_b__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "recursive Result<Vec<_>, _> summaries must not keep inactive Ok payload owners live on Err paths: {:#?}\nresource:\n{}",
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
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
fn resource_ir_owner_check_rejects_region_ptr_raw_owner_return() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn alloc_addr <()*>Result<i32, str>> ():
    match alloc_region_bytes<u8> 8:
        Result::Err _e:
            err<i32, str> "oom"
        Result::Ok region:
            let node_ptr <MemPtr<u8>> region_ptr &region
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
        diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerMaybeLeaked {
                function,
                ..
            } if function.starts_with("alloc_addr__")
        )),
        "mem_ptr_addr must not transfer a RegionToken owner into a raw Result::Ok address: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
    assert!(
        diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                function,
                operation: ResourceOwnerOperation::Dealloc,
                state: OwnerState::NoFreeObligation,
                ..
            } if function.starts_with("main__")
        )),
        "dealloc_raw on a raw projection must be rejected because the raw i32 has no free obligation: {:#?}\nresource:\n{}",
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/field" as *
#import "core/result" as *

fn finish_region <(RegionToken<u8>)->str> (region):
    let base_raw <i32> get region "raw"
    string_from_addr_unchecked base_raw

fn string_from_addr_unchecked <(i32)->str> (addr):
    #intrinsic "str_from_addr_unchecked" <> (addr)

fn main <()* >str> ():
    match alloc_region_bytes<u8> 4:
        Result::Ok region:
            finish_region region
        Result::Err e:
            e
"#;

    compile_resource_source_as_compiler_owned(source, CompileTarget::Wasm)
        .expect("str_from_addr_unchecked must transfer the raw allocation owner into returned str");
}

#[test]
fn resource_ir_owner_check_preserves_str_owner_through_str_addr_helper() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/result" as *

fn string_addr <(str)->i32> (s):
    #intrinsic "str_addr" <> (s)

fn main <()* >str> ():
    match concat_result "a" "b":
        Result::Ok s:
            let _addr <i32> string_addr s
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
            function.starts_with("string_addr__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "str_addr helper must return a non-owning raw address view and keep the str owner with the source: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_rejects_dealloc_through_str_addr_helper_view() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn string_addr <(str)->i32> (s):
    #intrinsic "str_addr" <> (s)

fn main <()*>()> ():
    match concat_result "a" "b":
        Result::Ok s:
            let addr <i32> string_addr s
            dealloc_raw addr len s
        Result::Err _e:
            ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                if function == "main" || function.starts_with("main__")
        )),
        "str_addr helper must produce only a non-owning raw address view; dealloc through it must be rejected: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_rejects_dealloc_through_result_wrapped_str_addr_view() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn string_addr <(str)->i32> (s):
    #intrinsic "str_addr" <> (s)

fn string_addr_result <(str)->Result<i32,str>> (s):
    let addr <i32> string_addr s
    Result<i32,str>::Ok addr

fn main <()*>()> ():
    match concat_result "a" "b":
        Result::Ok s:
            match string_addr_result s:
                Result::Ok addr:
                    dealloc_raw addr len s
                Result::Err _e:
                    ()
        Result::Err _e:
            ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                if function == "main" || function.starts_with("main__")
        )),
        "a non-owning str_addr view wrapped in Result::Ok must not become a raw owner: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_rejects_region_token_forged_from_str_addr_view() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn string_addr <(str)->i32> (s):
    #intrinsic "str_addr" <> (s)

fn forge_region_from_str <(str)*>Result<(), str>> (s):
    let raw <i32> string_addr s
    let token <RegionToken<u8>> region_new<u8> raw 1
    dealloc_region token

fn main <()*>()> ():
    match forge_region_from_str "abc":
        Result::Ok _:
            ()
        Result::Err _e:
            ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                function,
                state: OwnerState::NoFreeObligation,
                ..
            } if function.starts_with("main__")
                || function.starts_with("forge_region_from_str__")
        )),
        "region_new must not turn a non-owning str_addr raw identity into a RegionToken owner: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_rejects_region_token_forged_from_fixed_raw() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn forge_fixed_region <()* >Result<(), str>> ():
    let token <RegionToken<u8>> region_new<u8> 16 1
    dealloc_region token

fn main <()*>()> ():
    match forge_fixed_region:
        Result::Ok _:
            ()
        Result::Err _e:
            ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                function,
                state: OwnerState::NoFreeObligation,
                ..
            } if function.starts_with("main__")
                || function.starts_with("forge_fixed_region__")
        )),
        "region_new must not turn a fixed raw address into a RegionToken owner: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_rejects_returned_region_token_forged_from_fixed_raw() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn forge_fixed_region <()* >RegionToken<u8>> ():
    region_new<u8> 16 1

fn main <()*>()> ():
    let token <RegionToken<u8>> forge_fixed_region
    match dealloc_region token:
        Result::Ok _:
            ()
        Result::Err _e:
            ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                function,
                state: OwnerState::NoFreeObligation,
                ..
            } if function.starts_with("main__")
        )),
        "returned region_new token from fixed raw address must carry its owned storage obligation to the caller: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_accepts_returned_allocated_region_token() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn make_region <()* >RegionToken<u8>> ():
    match alloc_region<u8> 1:
        Result::Ok token:
            token
        Result::Err _e:
            #intrinsic "unreachable" <> ()

fn main <()*>()> ():
    let token <RegionToken<u8>> make_region
    match dealloc_region token:
        Result::Ok _:
            ()
        Result::Err _e:
            ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "returned allocated RegionToken owner must remain deallocatable by the caller: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn typecheck_rejects_region_token_struct_constructor_outside_memory_boundary() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn string_addr <(str)->i32> (s):
    #intrinsic "str_addr" <> (s)

fn forge_region_from_str <(str)*>Result<(), str>> (s):
    let raw <i32> string_addr s
    let token <RegionToken<u8>> RegionToken raw 1
    dealloc_region token

fn main <()*>()> ():
    match forge_region_from_str "abc":
        Result::Ok _:
            ()
        Result::Err _e:
            ()
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "type.owner_token.constructor_restricted",
    );
}

#[test]
fn typecheck_marks_imported_compiler_memory_types_in_type_context() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *

fn main <()->i32> ():
    0
"#;

    let (_module, types) = typecheck_resource_source(source);
    let mem_ptr = types.lookup_named("MemPtr").expect("MemPtr type");
    let region_token = types.lookup_named("RegionToken").expect("RegionToken type");

    assert_eq!(
        types.compiler_memory_type(mem_ptr),
        Some(CompilerMemoryType::RawPointer)
    );
    assert_eq!(
        types.compiler_memory_type(region_token),
        Some(CompilerMemoryType::OwnerToken)
    );
}

#[test]
fn typecheck_requires_struct_shape_for_compiler_memory_type_registration() {
    let source = r#"
#no_prelude
#entry main
#indent 4
#target std

pub struct MemPtr:
    raw <i32>
    tag <i32>

fn make <()->MemPtr> ():
    MemPtr 1 2

fn main <()->i32> ():
    let _p <MemPtr> make
    0
"#;

    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline(
            stdlib_root().join("malformed_mem_ptr.nepl"),
            source.to_string(),
        )
        .expect("load malformed same-name memory type source");
    let checked = nepl_core::typecheck::typecheck(
        &loaded.module,
        CompileTarget::Wasm,
        BuildProfile::Debug,
        Some(&loaded.source_map),
    );

    assert!(
        checked.diagnostics.is_empty(),
        "malformed compiler-owned MemPtr must not be rejected or registered without shape proof: {:#?}",
        checked.diagnostics
    );
    let mem_ptr = checked.types.lookup_named("MemPtr").expect("MemPtr type");
    assert_eq!(checked.types.compiler_memory_type(mem_ptr), None);
}

#[test]
fn typecheck_allows_user_struct_named_region_token() {
    let source = r#"
#no_prelude
#entry main
#indent 4
#target std

struct RegionToken:
    value <i32>

fn make <()->RegionToken> ():
    RegionToken 3

fn main <()->i32> ():
    0
"#;

    let _ = typecheck_resource_source(source);
}

#[test]
fn typecheck_rejects_region_token_field_access_outside_memory_boundary() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/field" as *

fn reveal_ptr <(RegionToken<u8>)->MemPtr<u8>> (token):
    get token "ptr"

fn main <()->i32> ():
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "type.owner_token.field_access_restricted",
    );
}

#[test]
fn typecheck_allows_region_token_field_access_with_owner_field_boundary() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/field" as field

fn reveal_raw <(RegionToken<u8>)->i32> (token):
    field::get token "raw"

fn main <()->i32> ():
    0
"#;

    compile_resource_source_as_compiler_owned(source, CompileTarget::Wasm)
        .expect("owner field source proof must allow direct RegionToken field projection");
}

#[test]
fn typecheck_rejects_mem_ptr_field_access_outside_compiler_memory_field_boundary() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/field" as field

fn reveal_raw <(MemPtr<u8>)->i32> (ptr):
    field::get ptr "raw"

fn main <()->i32> ():
    0
"#;

    let err = compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect_err("ordinary source must not allow raw pointer field projection");
    let nepl_core::CoreError::Diagnostics(diagnostics) = err else {
        panic!("expected diagnostics error");
    };
    assert!(
        diagnostics.iter().any(
            |diagnostic| diagnostic.code.as_str() == "type.raw_pointer.field_access_restricted"
        ),
        "expected raw pointer field access restriction, diagnostics: {diagnostics:#?}"
    );
}

#[test]
fn typecheck_rejects_compiler_owned_aggregate_mem_ptr_payload_field_access() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/field" as field

struct PtrHolder:
    ptr <MemPtr<u8>>

fn reveal_ptr <(PtrHolder)->MemPtr<u8>> (holder):
    field::get holder "ptr"

fn main <()->i32> ():
    0
"#;

    let err = compile_resource_source_as_compiler_owned(source, CompileTarget::Wasm)
        .expect_err("compiler-owned owner field proof must not allow MemPtr payload extraction");
    let nepl_core::CoreError::Diagnostics(diagnostics) = err else {
        panic!("expected diagnostics error");
    };
    assert!(
        diagnostics.iter().any(
            |diagnostic| diagnostic.code.as_str() == "type.raw_pointer.field_access_restricted"
        ),
        "expected raw pointer field access restriction, diagnostics: {diagnostics:#?}"
    );
}

#[test]
fn typecheck_rejects_nested_owner_backed_aggregate_constructor_outside_boundary() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *

struct VecBox:
    items <Vec<i32>>

fn box_vec <(Vec<i32>)->VecBox> (items):
    VecBox items

fn main <()->i32> ():
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "type.owner_aggregate.constructor_restricted",
    );
}

#[test]
fn typecheck_allows_owner_backed_constructor_inside_compiler_owned_source() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *

struct VecBox:
    items <Vec<i32>>

fn box_vec <(Vec<i32>)->VecBox> (items):
    VecBox items

fn main <()->i32> ():
    0
"#;

    compile_resource_source_as_compiler_owned(source, CompileTarget::Wasm)
        .expect("compiler-owned source proof must allow the exact owner-backed constructor site");
}

#[test]
fn typecheck_rejects_generic_owner_backed_aggregate_constructor_after_application() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *

struct OwnerBox<.T>:
    item <.T>

fn box_vec <(Vec<i32>)->OwnerBox<Vec<i32>>> (items):
    OwnerBox<Vec<i32>> items

fn main <()->i32> ():
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "type.owner_aggregate.constructor_restricted",
    );
}

#[test]
fn typecheck_rejects_hashmap_owner_storage_reconstruction_outside_boundary() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *

fn rebuild <(HashMapStorage<i32,i32>,DefaultHash32)->HashMap<i32,i32,DefaultHash32>> (storage, hasher):
    HashMap<i32,i32,DefaultHash32> 0 4 0 storage hasher

fn main <()->i32> ():
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "type.owner_aggregate.constructor_restricted",
    );
}

#[test]
fn typecheck_rejects_hashmap_owner_storage_field_projection_outside_boundary() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/hashmap" as *
#import "core/field" as field
#import "core/traits/hash" as *

fn steal_storage <(HashMap<i32,i32,DefaultHash32>)->HashMapStorage<i32,i32>> (map):
    field::get map "storage"

fn main <()->i32> ():
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "type.owner_aggregate.field_access_restricted",
    );
}

#[test]
fn typecheck_rejects_nested_owner_backed_aggregate_field_projection_outside_boundary() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *
#import "core/field" as field

struct VecBox:
    items <Vec<i32>>

fn steal_items <(VecBox)->Vec<i32>> (box):
    field::get box "items"

fn main <()->i32> ():
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "type.owner_aggregate.field_access_restricted",
    );
}

#[test]
fn typecheck_allows_owner_backed_aggregate_scalar_metadata_field_projection() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *
#import "core/field" as field

fn vec_len_from_ref <(&Vec<i32>)->i32> (v):
    *field::get_ref v "len"

fn main <()->i32> ():
    0
"#;

    let _ = typecheck_resource_source(source);
}

#[test]
fn typecheck_allows_user_struct_named_region_token_field_access() {
    let source = r#"
#no_prelude
#entry main
#indent 4
#target std

struct RegionToken:
    value <i32>

fn read_value <(RegionToken)->i32> (token):
    #intrinsic "get_field" <> (token,"value")

fn main <()->i32> ():
    0
"#;

    let _ = typecheck_resource_source(source);
}

#[test]
fn resource_ir_owner_check_uses_proven_region_token_identity_for_construct_extent() {
    let mut types = TypeCtx::new();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let fake_region = types.register_named(
        "RegionToken".to_string(),
        TypeKind::Struct {
            name: "RegionToken".to_string(),
            type_params: vec![],
            fields: vec![i32_ty, i32_ty],
            field_names: vec!["raw".to_string(), "size".to_string()],
        },
    );
    let span = Span::dummy();
    let module = HirModule {
        functions: vec![HirFunction {
            doc: None,
            name: "main".to_string(),
            origin_name: "main".to_string(),
            func_ty: TypeId(100),
            params: vec![],
            result: unit_ty,
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
                            kind: HirExprKind::Let {
                                name: "token".to_string(),
                                mutable: false,
                                value: Box::new(HirExpr {
                                    ty: fake_region,
                                    kind: HirExprKind::StructConstruct {
                                        name: "RegionToken".to_string(),
                                        type_args: vec![],
                                        fields: vec![
                                            HirExpr {
                                                ty: i32_ty,
                                                kind: HirExprKind::Var("p".to_string()),
                                                span,
                                            },
                                            HirExpr {
                                                ty: i32_ty,
                                                kind: HirExprKind::LiteralI32(8),
                                                span,
                                            },
                                        ],
                                    },
                                    span,
                                }),
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

    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        !report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::ConstructInput,
                ..
            }
        )),
        "same-name user RegionToken must not receive owner-token construct extent checks: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn typecheck_rejects_mem_ptr_struct_constructor_outside_memory_boundary() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn make <()->MemPtr<u8>> ():
    MemPtr 0

fn main <()->i32> ():
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "type.raw_pointer.constructor_restricted",
    );
}

#[test]
fn typecheck_allows_user_struct_named_mem_ptr() {
    let source = r#"
#no_prelude
#entry main
#indent 4
#target std

struct MemPtr:
    raw <i32>

fn make <()->MemPtr> ():
    MemPtr 3

fn main <()->i32> ():
    0
"#;

    let _ = typecheck_resource_source(source);
}

#[test]
fn typecheck_rejects_mem_ptr_field_access_outside_memory_boundary() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/field" as *

fn reveal_raw <()->i32> ():
    let p <MemPtr<u8>> mem_ptr_wrap 16
    get p "raw"

fn main <()->i32> ():
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "type.raw_pointer.field_access_restricted",
    );
}

#[test]
fn typecheck_allows_user_struct_named_mem_ptr_field_access() {
    let source = r#"
#no_prelude
#entry main
#indent 4
#target std

struct MemPtr:
    raw <i32>

fn read_raw <(MemPtr)->i32> (p):
    #intrinsic "get_field" <> (p,"raw")

fn main <()->i32> ():
    0
"#;

    let _ = typecheck_resource_source(source);
}

#[test]
fn resource_ir_owner_check_rejects_region_token_forged_from_region_ptr_helper() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn borrowed_region_ptr <(&RegionToken<u8>)->MemPtr<u8>> (token):
    region_ptr token

fn forge_region_from_region_ptr <(RegionToken<u8>)*>Result<(), str>> (token):
    let p <MemPtr<u8>> borrowed_region_ptr &token
    let raw <i32> mem_ptr_addr p
    let forged <RegionToken<u8>> region_new<u8> raw 1
    dealloc_region forged

fn main <()*>()> ():
    match alloc_region<u8> 1:
        Result::Err _e:
            ()
        Result::Ok token:
            match forge_region_from_region_ptr token:
                Result::Ok _:
                    ()
                Result::Err _e:
                    ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                function,
                state: OwnerState::NoFreeObligation,
                ..
            } if function.starts_with("main__")
                || function.starts_with("forge_region_from_region_ptr__")
        )),
        "region_ptr returned through a helper must remain a non-owning projection and cannot be forged into a RegionToken owner: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.raw.memory_outside_boundary",
    );
}

#[test]
fn resource_ir_owner_check_accepts_region_ptr_through_known_identity_callback() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn id_ptr <(MemPtr<u8>)->MemPtr<u8>> (p):
    p

fn apply_ptr <(MemPtr<u8>, (MemPtr<u8>)->MemPtr<u8>)->MemPtr<u8>> (p, f):
    f p

fn borrowed_region_ptr_via_callback <(&RegionToken<u8>)->MemPtr<u8>> (token):
    let p <MemPtr<u8>> region_ptr token
    apply_ptr p @id_ptr

fn main <()*>()> ():
    match alloc_region<u8> 1:
        Result::Err _e:
            ()
        Result::Ok token:
            let p <MemPtr<u8>> borrowed_region_ptr_via_callback &token
            match store_u8 p 7:
                Result::Err _e:
                    match dealloc_region token:
                        Result::Ok _:
                            ()
                        Result::Err _drop:
                            ()
                Result::Ok _:
                    match dealloc_region token:
                        Result::Ok _:
                            ()
                        Result::Err _drop:
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
            function.starts_with("apply_ptr__")
                || function.starts_with("borrowed_region_ptr_via_callback__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "known identity callback must return the borrowed region_ptr as a non-owning MemPtr without consuming an owner: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
    compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect("known identity callback should preserve a borrowed MemPtr view");
}

#[test]
fn resource_ir_owner_check_preserves_region_ptr_through_callback_parameter() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn id_ptr <(MemPtr<u8>)->MemPtr<u8>> (p):
    p

fn apply_ptr <(MemPtr<u8>, (MemPtr<u8>)->MemPtr<u8>)->MemPtr<u8>> (p, f):
    f p

fn borrowed_region_ptr_via_callback_param <(&RegionToken<u8>, (MemPtr<u8>)->MemPtr<u8>)->MemPtr<u8>> (token, f):
    let p <MemPtr<u8>> region_ptr token
    apply_ptr p f

fn main <()*>()> ():
    match alloc_region<u8> 1:
        Result::Err _e:
            ()
        Result::Ok token:
            let p <MemPtr<u8>> borrowed_region_ptr_via_callback_param &token @id_ptr
            match store_u8 p 7:
                Result::Err _e:
                    match dealloc_region token:
                        Result::Ok _:
                            ()
                        Result::Err _drop:
                            ()
                Result::Ok _:
                    match dealloc_region token:
                        Result::Ok _:
                            ()
                        Result::Err _drop:
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
            function.starts_with("apply_ptr__")
                || function.starts_with("borrowed_region_ptr_via_callback_param__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "callback parameter identity must preserve borrowed region_ptr as a non-owning MemPtr without requiring a free obligation: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
    compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect("callback parameter identity should preserve a borrowed MemPtr view");
}

#[test]
fn resource_ir_owner_check_rejects_region_token_forged_from_higher_order_region_ptr() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn id_ptr <(MemPtr<u8>)->MemPtr<u8>> (p):
    p

fn apply_ptr <(MemPtr<u8>, (MemPtr<u8>)->MemPtr<u8>)->MemPtr<u8>> (p, f):
    f p

fn borrowed_region_ptr_via_callback <(&RegionToken<u8>)->MemPtr<u8>> (token):
    let p <MemPtr<u8>> region_ptr token
    apply_ptr p @id_ptr

fn forge_region_from_callback_ptr <(RegionToken<u8>)*>Result<(), str>> (token):
    let p <MemPtr<u8>> borrowed_region_ptr_via_callback &token
    let raw <i32> mem_ptr_addr p
    let forged <RegionToken<u8>> region_new<u8> raw 1
    dealloc_region forged

fn main <()*>()> ():
    match alloc_region<u8> 1:
        Result::Err _e:
            ()
        Result::Ok token:
            match forge_region_from_callback_ptr token:
                Result::Ok _:
                    ()
                Result::Err _e:
                    ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let apply_span = source
        .find("apply_ptr p @id_ptr")
        .expect("test source must contain callback application");
    let dealloc_span = source
        .find("dealloc_region forged")
        .expect("test source must contain forged dealloc");
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                function,
                state: OwnerState::NoFreeObligation,
                span,
                ..
            } if (function.starts_with("main__")
                || function.starts_with("forge_region_from_callback_ptr__"))
                && span.start as usize >= dealloc_span
        )),
        "higher-order returned region_ptr must remain non-owning and fail at forged dealloc: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
    assert!(
        !report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                state: OwnerState::NoFreeObligation,
                span,
                ..
            } if (span.start as usize) >= apply_span
                && (span.start as usize) < dealloc_span
        )),
        "known identity callback must not be reported as owner consumption before the forged dealloc: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.raw.memory_outside_boundary",
    );
}

#[test]
fn resource_ir_owner_check_rejects_region_token_forged_from_region_ptr_at_ok_payload() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn forge_region_from_region_ptr_at <(RegionToken<u8>)*>Result<(), str>> (token):
    match region_ptr_at<u8,u8> &token 0:
        Result::Err e:
            Result<(), str>::Err e
        Result::Ok p:
            let raw <i32> mem_ptr_addr p
            let forged <RegionToken<u8>> region_new<u8> raw 1
            dealloc_region forged

fn main <()*>()> ():
    match alloc_region<u8> 1:
        Result::Err _e:
            ()
        Result::Ok token:
            match forge_region_from_region_ptr_at token:
                Result::Ok _:
                    ()
                Result::Err _e:
                    ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                function,
                state: OwnerState::NoFreeObligation,
                ..
            } if function.starts_with("main__")
                || function.starts_with("forge_region_from_region_ptr_at__")
        )),
        "region_ptr_at Ok payload must remain a non-owning projection and cannot be forged into a RegionToken owner: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.raw.memory_outside_boundary",
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
fn resource_ir_effect_check_accepts_owned_str_return_identity() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/result" as *

fn make_text <()* >str> ():
    match concat_result "a" "b":
        Result::Ok text:
            text
        Result::Err e:
            e

fn main <()* >i32> ():
    len make_text
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect("owned str return must not be reported as raw address identity escape");
}

#[test]
fn resource_ir_owner_check_accepts_string_from_mem_unchecked_result_transfer() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string/storage" as *
#import "alloc/string/access" as *
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
fn resource_ir_owner_check_keeps_source_after_string_from_mem_copy() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string/storage" as *
#import "alloc/string/access" as *
#import "core/result" as *

fn main <()* >i32> ():
    let src <str> "abc"
    match string_from_mem_unchecked_result string_data_ptr src len src:
        Result::Ok _copied:
            len src
        Result::Err _e:
            len src
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                    if function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "string_from_mem_unchecked_result copies bytes and must not reserve the source str: {:#?}",
        diagnostics
    );
}

#[test]
fn resource_ir_owner_check_rejects_string_from_mem_oversized_region_span() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string/storage" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_region<u8> 1:
        Result::Err _e:
            0
        Result::Ok region:
            let p <MemPtr<u8>> region_ptr &region
            let copied_len <i32> match string_from_mem_unchecked_result p 100:
                Result::Ok copied:
                    1
                Result::Err _e:
                    0
            match dealloc_region region:
                Result::Ok _:
                    copied_len
                Result::Err _e:
                    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.owner.unavailable",
    );
}

#[test]
fn resource_ir_owner_check_rejects_cstr_bounded_oversized_region_span() {
    let source = r#"
#entry main
#indent 4
#target std
#import "std/env/cliarg/cstr" as *
#import "core/mem" as *
#import "core/result" as *

fn main <()*>i32> ():
    match alloc_region<u8> 1:
        Result::Err _e:
            0
        Result::Ok region:
            let p <MemPtr<u8>> region_ptr &region
            let status <i32> match cstr_to_str_bounded_result p 100:
                Result::Ok _s:
                    1
                Result::Err _e:
                    0
            match dealloc_region<u8> region:
                Result::Ok _:
                    status
                Result::Err _e:
                    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.owner.unavailable",
    );
}

#[test]
fn resource_ir_owner_check_accepts_string_from_mem_string_source_span() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string/storage" as *
#import "alloc/string/access" as *
#import "core/result" as *

fn main <()*>i32> ():
    let src <str> "A"
    match string_from_mem_unchecked_result string_data_ptr src 1:
        Result::Ok copied:
            1
        Result::Err _e:
            0
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect("string-backed span should satisfy raw memory span summary proof");
}

#[test]
fn resource_ir_owner_check_keeps_string_source_after_slice_result_consumer() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/result" as *

fn consume_slice <(Result<str,str>)->()> (slice):
    match slice:
        Result::Ok _text:
            ()
        Result::Err _e:
            ()

fn main <()* >i32> ():
    let s <str> "Aあ💯"
    consume_slice str_slice_chars_result s 1 3
    len s
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                    if function.starts_with("consume_slice__") || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "copying a string slice result must not reserve the source str after the result is consumed: {:#?}",
        diagnostics
    );
}

#[test]
fn resource_ir_owner_check_keeps_string_source_after_byte_slice_result_consumer() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/result" as *

fn consume_slice <(Result<str,str>)->()> (slice):
    match slice:
        Result::Ok _text:
            ()
        Result::Err _e:
            ()

fn main <()* >i32> ():
    let s <str> "Aあ💯"
    consume_slice str_slice_result s 1 4
    len s
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                    if function.starts_with("consume_slice__") || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "copying a byte slice result must not reserve the source str after the result is consumed: {:#?}",
        diagnostics
    );
}

#[test]
fn resource_ir_owner_check_keeps_string_source_after_char_index_result_consumer() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/result" as *

fn consume_index <(Result<i32,str>)->()> (index):
    match index:
        Result::Ok _byte:
            ()
        Result::Err _e:
            ()

fn main <()* >i32> ():
    let s <str> "Aあ💯"
    consume_index str_char_byte_index_result s 1
    len s
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                    if function.starts_with("consume_index__") || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "char index lookup must not reserve the source str after its Result is consumed: {:#?}",
        diagnostics
    );
}

#[test]
fn resource_ir_owner_check_forwards_nested_byte_builder_result_owner() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/io" as *
#import "core/result" as *

fn make_bytes <()* >Result<ByteBuf, StdErrorKind>> ():
    match byte_builder_new:
        Result::Err e:
            Result<ByteBuf, StdErrorKind>::Err e
        Result::Ok b0:
            match byte_builder_push_char_utf8 b0 'A':
                Result::Err e:
                    Result<ByteBuf, StdErrorKind>::Err e
                Result::Ok b1:
                    match byte_builder_push_char_utf8 b1 'あ':
                        Result::Err e:
                            Result<ByteBuf, StdErrorKind>::Err e
                        Result::Ok b2:
                            byte_builder_finish b2

fn main <()* >()> ():
    match make_bytes:
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
            matches!(
                diagnostic,
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                    if function.starts_with("make_bytes__") || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "nested Result<ByteBuilder, _> forwarding must transfer the returned builder owner into the Ok payload: {:#?}",
        diagnostics
    );
}

#[test]
fn resource_ir_effect_check_accepts_byte_builder_finish_owner_return() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/io" as *
#import "core/result" as *

fn make_bytes <()* >Result<ByteBuf, StdErrorKind>> ():
    match byte_builder_new:
        Result::Err e:
            Result<ByteBuf, StdErrorKind>::Err e
        Result::Ok b0:
            match byte_builder_push_u8 b0 65:
                Result::Err e:
                    Result<ByteBuf, StdErrorKind>::Err e
                Result::Ok b1:
                    byte_builder_finish b1

fn main <()* >()> ():
    match make_bytes:
        Result::Ok bytes:
            io_bytebuf_free bytes
        Result::Err _e:
            ()
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasi)
        .expect("owner-protected ByteBuf return must not be treated as raw MemPtr escape");
}

#[test]
fn resource_ir_effect_check_accepts_vec_owner_result_return_identity() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as v
#import "core/result" as *
#import "std/fs/path/normalize/range_stack" as range_stack

fn main <()* >()> ():
    match v::new<i32>:
        Result::Err _e:
            ()
        Result::Ok stack:
            match range_stack::fs_normalize_range_push stack 0 1:
                Result::Err _e:
                    ()
                Result::Ok next:
                    v::free<i32> next
"#;

    let (module, mut types) = typecheck_resource_source(source);
    let monomorphized = nepl_core::monomorphize::monomorphize(&mut types, module).module;
    let resource = lower_hir_module(&monomorphized, &types);
    let report = check_resource_effect_boundaries_typed(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceEffectBoundaryDiagnostic::RawAddressEscapeFromInternalAlloc {
                    function,
                    ..
                } if function.starts_with("fs_normalize_range_push__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "Result<Vec<_>, _> owner return must not be treated as raw identity escape: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_accepts_vec_free_region_token_cleanup() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as v
#import "core/result" as *

fn main <()* >()> ():
    match v::new<i32>:
        Result::Err _e:
            ()
        Result::Ok stack:
            v::free<i32> stack
"#;
    let (module, mut types) = typecheck_resource_source(source);
    let monomorphized = nepl_core::monomorphize::monomorphize(&mut types, module).module;
    let resource = lower_hir_module(&monomorphized, &types);
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
            function.starts_with("free__Vec_T") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "Vec::free must close the RegionToken owner obligation through source-level dealloc_region: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_accepts_fs_normalize_range_push_result_owner_cleanup() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as v
#import "core/result" as *
#import "std/fs/path/normalize/range_stack" as range_stack

fn main <()* >()> ():
    match v::new<i32>:
        Result::Err _e:
            ()
        Result::Ok stack:
            match range_stack::fs_normalize_range_push stack 0 1:
                Result::Err _e:
                    ()
                Result::Ok next:
                    v::free<i32> next
"#;
    let (module, mut types) = typecheck_resource_source(source);
    let monomorphized = nepl_core::monomorphize::monomorphize(&mut types, module).module;
    let resource = lower_hir_module(&monomorphized, &types);
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
            function.starts_with("fs_normalize_range_push__")
                || function.starts_with("free__Vec_T")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "fs_normalize_range_push must return Vec owner payloads that remain freeable by source-level Vec::free cleanup: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_accepts_fs_normalize_build_ranges_builder_cleanup() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as v
#import "alloc/string/builder" as *
#import "core/result" as *
#import "std/fs/path/normalize/build" as normalize_build
#import "std/fs/path/normalize/range_stack" as range_stack

fn main <()* >()> ():
    match v::new<i32>:
        Result::Err _e:
            ()
        Result::Ok stack0:
            match range_stack::fs_normalize_range_push stack0 0 1:
                Result::Err _e:
                    ()
                Result::Ok stack:
                    let result <Result<StringBuilder,i32>> normalize_build::fs_normalize_build_ranges_builder "a" &stack
                    v::free<i32> stack
                    match result:
                        Result::Err _e:
                            ()
                        Result::Ok sb:
                            string_builder_free sb
"#;
    let (module, mut types) = typecheck_resource_source(source);
    let monomorphized = nepl_core::monomorphize::monomorphize(&mut types, module).module;
    let resource = lower_hir_module(&monomorphized, &types);
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
            function.starts_with("fs_normalize_build_ranges_builder__")
                || function.starts_with("string_builder_free__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "fs_normalize_build_ranges_builder must return a StringBuilder owner payload that remains freeable by source-level cleanup: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_accepts_string_builder_free_cleanup() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string/builder" as *
#import "core/result" as *

fn main <()* >()> ():
    match string_builder_new_result:
        Result::Err _e:
            ()
        Result::Ok sb:
            string_builder_free sb
"#;
    let (module, mut types) = typecheck_resource_source(source);
    let monomorphized = nepl_core::monomorphize::monomorphize(&mut types, module).module;
    let resource = lower_hir_module(&monomorphized, &types);
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
            function.starts_with("string_builder_free__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "StringBuilder::free must consume the nested ByteBuilder RegionToken owner: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_effect_check_accepts_string_builder_build_str_return() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "alloc/string/builder" as *
#import "core/result" as *

fn main <()* >i32> ():
    match string_builder_new_result:
        Result::Err _e:
            0
        Result::Ok b0:
            match sb_append_result b0 "A":
                Result::Err _e:
                    0
                Result::Ok b1:
                    match sb_build_result b1:
                        Result::Err _e:
                            0
                        Result::Ok out:
                            len out
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect("StringBuilder build must return owned str without raw identity escape");
}

#[test]
fn resource_ir_owner_check_accepts_string_builder_build_wrapper_str_observer() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *

fn main <()* >i32> ():
    let mut sb <StringBuilder> string_builder_new;
    set sb sb_append sb "Error: ";
    set sb sb_append_i32 sb 404;
    set sb sb_append sb " Not Found";
    let res <str> sb_build sb;
    len res
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
            function.starts_with("main__")
                || function.starts_with("sb_append_i32__")
                || function.starts_with("sb_append_i32_result__")
                || function.starts_with("sb_build__")
                || function.starts_with("sb_build_result__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "StringBuilder sb_build wrapper result must not leak nested Result<str,str> temporary owners: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );

    compile_resource_source_with_target(source, CompileTarget::Wasm).expect(
        "StringBuilder sb_build wrapper result must compile under the full Resource IR gate",
    );
}

#[test]
fn resource_ir_owner_check_byte_builder_free_closes_region_by_token_size() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/io/bytebuilder" as *
#import "core/result" as *

fn main <()* >()> ():
    match byte_builder_new:
        Result::Ok b0:
            match byte_builder_push_u8 b0 65:
                Result::Ok b1:
                    byte_builder_free b1
                Result::Err _e:
                    ()
        Result::Err _e:
            ()
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasi)
        .expect("ByteBuilder free must consume the RegionToken owner when token size is positive");
}

#[test]
fn resource_ir_owner_check_forwards_byte_builder_owner_through_text_result_mapping() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/io" as *
#import "core/result" as *

fn byte_builder_text <()*>Result<str,str>> ():
    match byte_builder_new:
        Result::Err _e:
            Result<str,str>::Err "byte builder alloc"
        Result::Ok b0:
            match byte_builder_push_char_utf8 b0 'A':
                Result::Err _e:
                    Result<str,str>::Err "byte builder push A"
                Result::Ok b1:
                    match byte_builder_push_char_utf8 b1 'あ':
                        Result::Err _e:
                            Result<str,str>::Err "byte builder push hira"
                        Result::Ok b2:
                            match byte_builder_finish b2:
                                Result::Err _e:
                                    Result<str,str>::Err "byte builder finish"
                                Result::Ok bytes:
                                    match io_bytebuf_to_str_result bytes:
                                        Result::Err _e:
                                            Result<str,str>::Err "byte builder decode"
                                        Result::Ok text:
                                            Result<str,str>::Ok text

fn main <()* >()> ():
    match byte_builder_text:
        Result::Ok _text:
            ()
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
            matches!(
                diagnostic,
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                    if function.starts_with("byte_builder_text__") || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "ByteBuilder owner returned through char append must stay live when the surrounding function maps the final bytes into Result<str, str>: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_forwards_byte_builder_owner_through_leb32_loop() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/io" as *
#import "core/result" as *

fn main <()* >()> ():
    match byte_builder_new:
        Result::Ok b0:
            match byte_builder_push_leb_u32 b0 624485:
                Result::Ok b1:
                    match byte_builder_finish b1:
                        Result::Ok bytes:
                            io_bytebuf_free bytes
                        Result::Err _e:
                            ()
                Result::Err _e:
                    ()
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
            matches!(
                diagnostic,
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                    | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                    | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. }
                    if function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "ByteBuilder owner returned through LEB32 loop must be consumable by finish: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_keeps_byte_builder_string_source_usable() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/io" as *
#import "alloc/string" as *
#import "core/math" as *
#import "core/result" as *

fn main <()* >()> ():
    let text <str> "abcdefghij"
    match byte_builder_with_capacity 2:
        Result::Err _e:
            ()
        Result::Ok b0:
            match byte_builder_push_str b0 text:
                Result::Err _e:
                    ()
                Result::Ok b1:
                    let keep_len <i32> len text
                    match byte_builder_finish b1:
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
            matches!(
                diagnostic,
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                    | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                    | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. }
                    if function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "ByteBuilder string append must not consume or reserve the source str: {:#?}\nresource:\n{}",
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
#import "alloc/string/storage" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

fn make_nonempty <()* >Result<ByteBuf, StdErrorKind>> ():
    match alloc_region_bytes<u8> 3:
        Result::Ok region:
            let out <MemPtr<u8>> region_ptr &region
            let out_raw <i32> mem_ptr_addr out
            let data <MemPtr<u8>> string_data_ptr "abc"
            let data_raw <i32> mem_ptr_addr data
            mem_copy out_raw data_raw 3
            Result<ByteBuf, StdErrorKind>::Ok io_bytebuf_finish_region region 3
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
fn resource_ir_owner_check_vec_partition_returns_named_vec_owners() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *
#import "core/math" as *
#import "core/result" as *

fn is_even <(i32)->bool> (x):
    eq rem_s x 2 0

fn main <()*>i32> ():
    let xs <Vec<i32>>:
        new<i32>
        |> uwok
        |> push<i32> 1 |> uwok
        |> push<i32> 2 |> uwok
        |> push<i32> 3 |> uwok
    let parts <VecPartition<i32>> unwrap_ok<VecPartition<i32>, VecTransformError<i32>> partition<i32> xs @is_even
    let ok <bool> and eq vec_partition_matched_len<i32> &parts 1 eq vec_partition_rest_len<i32> &parts 2
    vec_partition_free<i32> parts
    if ok 0 1
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
            function.starts_with("partition__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "Vec.partition must return both named Vec owners without leaking intermediate storage: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_unwrap_ok_push_transfers_vec_owner() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *
#import "core/result" as *

fn main <()*>i32> ():
    let v0 <Vec<i32>> unwrap_ok new<i32>
    let v1 <Vec<i32>> unwrap_ok push<i32> v0 7
    free<i32> v1
    0
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
            function.starts_with("main__") || function.starts_with("push__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "unwrap_ok push must transfer the input Vec owner into the returned Vec payload: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_does_not_treat_raw_cell_payload_as_storage_owner() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.bool());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let predicate_ty = types.function(vec![], vec![i32_ty], bool_ty, Effect::Pure);
    let span = Span::dummy();
    let predicate = Place::local("predicate".to_string(), predicate_ty);
    let owner_raw = Place::temporary(ResourceId(0), i32_ty);
    let loaded_value = Place::temporary(ResourceId(1), i32_ty);
    let predicate_result = Place::temporary(ResourceId(2), bool_ty);
    let dealloc_result = Place::temporary(ResourceId(3), unit_ty);
    let resource = ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            origin_name: "main".to_string(),
            type_params: Vec::new(),
            params: vec![ResourceLocal {
                name: "predicate".to_string(),
                ty: predicate_ty,
                mutable: false,
                place: predicate.clone(),
            }],
            result: unit_ty,
            effect: Effect::Impure,
            entry_block: ResourceBlockId(0),
            blocks: vec![ResourceBlock {
                id: ResourceBlockId(0),
                ops: vec![
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Alloc,
                        output: owner_raw.clone(),
                        args: vec![],
                        span,
                    },
                    ResourceOp::StorageOrigin {
                        target: owner_raw.clone(),
                        origin: StorageOrigin::Owned,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Load,
                        output: loaded_value.clone(),
                        args: vec![owner_raw.clone()],
                        span,
                    },
                    ResourceOp::IndirectCall {
                        output: predicate_result,
                        callee: predicate,
                        params: vec![i32_ty],
                        result: bool_ty,
                        args: vec![loaded_value],
                        effect: EffectOp::IndirectCall {
                            effect: Effect::Pure,
                        },
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Dealloc,
                        output: dealloc_result,
                        args: vec![owner_raw],
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

    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceOwnerDiagnostic::OwnerUnavailable { function, operation, .. }
                    if function == "main"
                        && matches!(operation, ResourceOwnerOperation::CallArgument)
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "raw cell payload reads must not inherit the storage owner's free obligation across Deref: {:#?}\nresource:\n{}",
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
fn resource_ir_compiler_rejects_non_copy_move_from_live_shared_reference() {
    let source = r#"
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let x <LocalToken> LocalToken @token_id
    let r <&LocalToken> &x
    let y <LocalToken> x
    let z <&LocalToken> r
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.borrow.move_from_shared",
    );
}

#[test]
fn resource_ir_compiler_rejects_unique_and_shared_reference_overlap() {
    let source = r#"
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn use_both <(&mut LocalToken,&LocalToken)->i32> (_a, _b):
    0

fn main <()->i32> ():
    let x <LocalToken> LocalToken @token_id
    use_both &mut x &x
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.borrow.borrow_during_unique",
    );
}

#[test]
fn resource_ir_compiler_rejects_local_reference_return_escape() {
    let source = r#"
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn leak <()->&LocalToken> ():
    let t <LocalToken> LocalToken @token_id
    &t

fn main <()->i32> ():
    let r <&LocalToken> leak
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.borrow.return_escape",
    );
}

#[test]
fn resource_ir_compiler_rejects_branch_retained_borrow_move() {
    let source = r#"
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let x <LocalToken> LocalToken @token_id
    let y <LocalToken> LocalToken @token_id
    let mut r <&LocalToken> &x
    let cnd <bool> true
    if cnd:
        then:
            set r &y
            0
        else:
            0
    let moved <LocalToken> y
    let still_live <&LocalToken> r
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.borrow.move_from_shared",
    );
}

#[test]
fn resource_ir_compiler_rejects_match_payload_borrow_move() {
    let source = r#"
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

enum RefOpt:
    Some <&LocalToken>
    None

fn main <()->i32> ():
    let x <LocalToken> LocalToken @token_id
    let e <RefOpt> RefOpt::Some &x
    match e:
        RefOpt::Some r:
            let y <LocalToken> x
            let keep <&LocalToken> r
            0
        RefOpt::None:
            0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.borrow.move_from_shared",
    );
}

#[test]
fn resource_ir_compiler_rejects_inner_local_reference_assignment_escape() {
    let source = r#"
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let outer <LocalToken> LocalToken @token_id
    let mut r <&LocalToken> &outer
    block:
        let inner <LocalToken> LocalToken @token_id
        set r &inner
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.borrow.return_escape",
    );
}

#[test]
fn resource_ir_compiler_rejects_inner_local_struct_reference_assignment_escape() {
    let source = r#"
#entry main
#indent 4
#target core

struct LocalToken:
    raw <(i32)->i32>

struct RefBox:
    inner <&LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let outer <LocalToken> LocalToken @token_id
    let mut b <RefBox> RefBox &outer
    block:
        let inner <LocalToken> LocalToken @token_id
        let local <RefBox> RefBox &inner
        set b local
    0
"#;

    assert_compile_resource_source_reports_code(
        source,
        CompileTarget::Wasm,
        "resource.borrow.return_escape",
    );
}

#[test]
fn resource_ir_compiler_rejects_function_value_raw_writes_through_summaries() {
    let cases = [
        r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn clobber_i32 <(MemPtr<i32>)*>()> (p):
    let r <Result<(),str>> store_i32 p 0
    ()

fn apply_clobber <(MemPtr<i32>, (MemPtr<i32>)*>())*>()> (p, f):
    f p

fn forward_clobber <(MemPtr<i32>, (MemPtr<i32>)*>())*>()> (p, f):
    apply_clobber p f

fn main <()*>i32> ():
    let raw <i32> 16
    let pi <MemPtr<i32>> mem_ptr_wrap<i32> raw
    store<LocalToken> raw LocalToken @token_id
    forward_clobber pi @clobber_i32
    0
"#,
        r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

struct CallbackHolder:
    cb <(MemPtr<i32>)*>()>

fn clobber_i32 <(MemPtr<i32>)*>()> (p):
    let r <Result<(),str>> store_i32 p 0
    ()

fn call_holder <(MemPtr<i32>, CallbackHolder)*>()> (p, holder):
    let f <(MemPtr<i32>)*>()> field::get holder "cb"
    f p

fn main <()*>i32> ():
    let raw <i32> 16
    let pi <MemPtr<i32>> mem_ptr_wrap<i32> raw
    store<LocalToken> raw LocalToken @token_id
    let holder <CallbackHolder> CallbackHolder @clobber_i32
    call_holder pi holder
    0
"#,
        r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/option" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn clobber_i32 <(MemPtr<i32>)*>()> (p):
    let r <Result<(),str>> store_i32 p 0
    ()

fn call_option <(MemPtr<i32>, Option<(MemPtr<i32>)*>()>)*>()> (p, opt):
    match opt:
        Option::Some f:
            f p
        Option::None:
            ()

fn main <()*>i32> ():
    let raw <i32> 16
    let pi <MemPtr<i32>> mem_ptr_wrap<i32> raw
    store<LocalToken> raw LocalToken @token_id
    call_option pi Option::Some @clobber_i32
    0
"#,
    ];

    for source in cases {
        assert_compile_resource_source_reports_code(
            source,
            CompileTarget::Wasm,
            "resource.cell.initialized_conflict",
        );
    }
}

#[test]
fn resource_ir_borrow_check_allows_shared_read_until_release() {
    let types = type_ctx_with_copy_i32();
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
                synthetic: false,
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

    let report = check_resource_borrow_lifetimes(&resource, &types);
    assert_eq!(report.diagnostics, vec![]);
}

#[test]
fn resource_ir_borrow_check_reports_read_during_unique_borrow() {
    let types = type_ctx_with_copy_i32();
    let span = Span::dummy();
    let x = Place::local("x".to_string(), types.i32());
    let unique = Place::temporary(ResourceId(0), types.i32());
    let resource = manual_resource_module(
        types.unit(),
        span,
        vec![
            ResourceOp::Borrow {
                source: x.clone(),
                output: unique.clone(),
                kind: BorrowKind::Unique,
                synthetic: false,
                span,
            },
            ResourceOp::Read {
                source: x.clone(),
                output: Place::temporary(ResourceId(1), types.i32()),
                span,
            },
            ResourceOp::Read {
                source: unique,
                output: Place::temporary(ResourceId(2), types.i32()),
                span,
            },
        ],
    );

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
    let shared = Place::temporary(ResourceId(0), types.i32());
    let resource = manual_resource_module(
        types.unit(),
        span,
        vec![
            ResourceOp::Borrow {
                source: x.clone(),
                output: shared.clone(),
                kind: BorrowKind::Shared,
                synthetic: false,
                span,
            },
            ResourceOp::Borrow {
                source: x,
                output: Place::temporary(ResourceId(1), types.i32()),
                kind: BorrowKind::Unique,
                synthetic: false,
                span,
            },
            ResourceOp::Read {
                source: shared,
                output: Place::temporary(ResourceId(2), types.i32()),
                span,
            },
        ],
    );

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
                synthetic: false,
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
                synthetic: false,
                span,
            },
        ],
    );

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
                    synthetic: false,
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

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
                origin_name: "borrow_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            synthetic: false,
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

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
                origin_name: "observe".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            synthetic: false,
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

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
                origin_name: "observe".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            synthetic: false,
                            span,
                        },
                        ResourceOp::Call {
                            output: call_out,
                            target: ResourceCallTarget::User {
                                name: "observe".to_string(),
                                type_args: vec![],
                            },
                            args: vec![local_ref.clone()],
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
                        ResourceOp::Read {
                            source: local_ref,
                            output: Place::temporary(ResourceId(3), i32_ty),
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

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
                origin_name: "borrow_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            synthetic: false,
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
                        ResourceOp::Assign {
                            target: x,
                            value: replacement,
                            span,
                        },
                        ResourceOp::Read {
                            source: returned,
                            output: Place::temporary(ResourceId(3), i32_ty),
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

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
                origin_name: "borrow_id".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            synthetic: false,
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

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
                origin_name: "observe".to_string(),
                type_params: Vec::new(),
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
                origin_name: "main".to_string(),
                type_params: Vec::new(),
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
                            synthetic: false,
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

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
                        synthetic: false,
                        span,
                    },
                    ResourceOp::IndirectCall {
                        output: returned.clone(),
                        callee,
                        params: vec![i32_ty],
                        result: i32_ty,
                        args: vec![shared],
                        effect: EffectOp::Unknown {
                            reason: UnknownEffectReason::CallbackParameterWithoutKnownEffect,
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

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
                        synthetic: false,
                        span,
                    },
                    ResourceOp::IndirectCall {
                        output: returned.clone(),
                        callee,
                        params: vec![i32_ty],
                        result: bool_ty,
                        args: vec![shared],
                        effect: EffectOp::Unknown {
                            reason: UnknownEffectReason::CallbackParameterWithoutKnownEffect,
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

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
                        synthetic: false,
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
                        source_name: "f".to_string(),
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
                            reason: UnknownEffectReason::AssignedCallbackWithoutKnownEffect,
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

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
            origin_name: "main".to_string(),
            type_params: Vec::new(),
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
                        synthetic: false,
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

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
                output: shared.clone(),
                kind: BorrowKind::Shared,
                synthetic: false,
                span,
            },
            ResourceOp::Assign {
                target: wrapper,
                value: replacement,
                span,
            },
            ResourceOp::Read {
                source: shared,
                output: Place::temporary(ResourceId(2), i32_ty),
                span,
            },
        ],
    );

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
                source_name: "x".to_string(),
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
                source_name: "x".to_string(),
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
                condition_fact: None,
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
    let keep_alive = Place::temporary(ResourceId(6), i32_ty);
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
                    output: shared.clone(),
                    kind: BorrowKind::Shared,
                    synthetic: false,
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
            ResourceOp::Read {
                source: shared,
                output: keep_alive,
                span,
            },
        ],
    );

    let report = check_resource_borrow_lifetimes(&resource, &types);
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <i32> alloc_raw 16
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
fn resource_ir_cell_check_realloc_transfers_initialized_byte_ranges() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn id <(i32)->i32> (x):
    x

fn checked_realloc_byte_range <()->i32> ():
    let len <i32> 4
    let p <i32> alloc_raw len
    fill_u8 p len 65
    let grown <i32> realloc_raw p len 8
    if:
        lt 0 grown
        then:
            let i <i32> id 2
            let v <i32> if:
                and ge i 0 lt i len
                then:
                    load_u8 add grown i
                else:
                    0
            dealloc_raw grown 8
            v
        else:
            let i <i32> id 2
            let v <i32> if:
                and ge i 0 lt i len
                then:
                    load_u8 add p i
                else:
                    0
            dealloc_raw p len
            v

fn main <()->i32> ():
    checked_realloc_byte_range
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
                    if function.starts_with("checked_realloc_byte_range__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        realloc_diagnostics.is_empty(),
        "checked realloc success must transfer initialized byte ranges and failure must keep the old range: {:#?}\nresource:\n{}",
        realloc_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_realloc_transfers_initialized_element_ranges() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn id <(i32)->i32> (x):
    x

fn checked_realloc_element_range <()->i32> ():
    let len <i32> 4
    let p <i32> alloc_raw 16
    fill_i32 p len 42
    let grown <i32> realloc_raw p 16 32
    if:
        lt 0 grown
        then:
            let i <i32> id 2
            let off <i32> mul i 4
            let v <i32> if:
                and ge i 0 lt i len
                then:
                    load_i32 add grown off
                else:
                    0
            dealloc_raw grown 32
            v
        else:
            let i <i32> id 2
            let off <i32> mul i 4
            let v <i32> if:
                and ge i 0 lt i len
                then:
                    load_i32 add p off
                else:
                    0
            dealloc_raw p 16
            v

fn main <()->i32> ():
    checked_realloc_element_range
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
                    if function.starts_with("checked_realloc_element_range__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        realloc_diagnostics.is_empty(),
        "checked realloc success must transfer initialized element ranges and failure must keep the old range: {:#?}\nresource:\n{}",
        realloc_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_bulk_copy_transfers_initialized_byte_ranges() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn id <(i32)->i32> (x):
    x

fn copy_byte_range <()->i32> ():
    let len <i32> 4
    let src <i32> alloc_raw len
    let dst <i32> alloc_raw len
    memset_u8 src len 65
    mem_copy dst src len
    let i <i32> id 2
    let v <i32> if:
        and ge i 0 lt i len
        then:
            load_u8 add dst i
        else:
            0
    dealloc_raw src len
    dealloc_raw dst len
    v

fn main <()->i32> ():
    copy_byte_range
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let copy_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function.starts_with("copy_byte_range__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        copy_diagnostics.is_empty(),
        "bulk copy must transfer initialized byte ranges covered by the copied byte count: {:#?}\nresource:\n{}",
        copy_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_bulk_copy_transfers_initialized_copy_cells_with_extent() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn copy_i32_cell <()->i32> ():
    let bytes <i32> 4
    let src <i32> alloc_raw bytes
    let dst <i32> alloc_raw bytes
    store_i32 src 99
    mem_copy dst src bytes
    let v <i32> load_i32 dst
    dealloc_raw src bytes
    dealloc_raw dst bytes
    v

fn main <()->i32> ():
    copy_i32_cell
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let copy_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function.starts_with("copy_i32_cell__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        copy_diagnostics.is_empty(),
        "bulk copy must transfer initialized Copy cells only when the byte count covers the cell: {:#?}\nresource:\n{}",
        copy_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_bulk_move_transfers_initialized_element_ranges() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn id <(i32)->i32> (x):
    x

fn move_element_range <()->i32> ():
    let len <i32> 4
    let bytes <i32> mul len 4
    let src <i32> alloc_raw bytes
    let dst <i32> alloc_raw bytes
    fill_i32 src len 42
    mem_move dst src bytes
    let i <i32> id 2
    let off <i32> mul i 4
    let v <i32> if:
        and ge i 0 lt i len
        then:
            load_i32 add dst off
        else:
            0
    dealloc_raw src bytes
    dealloc_raw dst bytes
    v

fn main <()->i32> ():
    move_element_range
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let move_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function.starts_with("move_element_range__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        move_diagnostics.is_empty(),
        "bulk move must transfer initialized element ranges when the copied byte count proves the whole element prefix: {:#?}\nresource:\n{}",
        move_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_bulk_copy_does_not_transfer_uncovered_byte_ranges() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn main <()->i32> ():
    let src_len <i32> 8
    let copy_len <i32> 4
    let src <i32> alloc_raw src_len
    let dst <i32> alloc_raw src_len
    memset_u8 src src_len 65
    mem_copy dst src copy_len
    let value <i32> load_u8 add dst 7
    dealloc_raw src src_len
    dealloc_raw dst src_len
    value
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                state: CellState::Uninit,
                ..
            }
        )),
        "bulk copy must not transfer initialized byte evidence beyond the copied extent: {:#?}\nresource:\n{}",
        report.diagnostics,
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
    assert!(
        resource.dump_text().contains("raw_memory fill_bytes"),
        "memset_u8/fill_u8 must remain distinct from fill_i32 in Resource IR:\n{}",
        resource.dump_text()
    );
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
fn resource_ir_cell_check_byte_fill_requires_guard_for_symbolic_load() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn main <(i32,i32)->i32> (i, len):
    let p <i32> alloc_raw len
    memset_u8 p len 0
    let value <i32> load_u8 add p i
    dealloc_raw p len
    value
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                state: CellState::Uninit,
                ..
            }
        )),
        "symbolic byte load after memset_u8 must require a typed range guard: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_byte_fill_accepts_symbolic_load_with_range_guard() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn main <(i32,i32)->i32> (i, len):
    let p <i32> alloc_raw len
    memset_u8 p len 0
    let value <i32> if and ge i 0 lt i len:
        then:
            load_u8 add p i
        else:
            0
    dealloc_raw p len
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
        "symbolic byte load guarded by 0 <= i && i < len must use the byte fill range fact: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_word_fill_requires_guard_for_scaled_symbolic_load() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn main <(i32,i32)->i32> (i, len):
    let p <i32> alloc_raw mul len 4
    fill_i32 p len 0
    let off <i32> mul i 4
    let value <i32> load_i32 add p off
    dealloc_raw p mul len 4
    value
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                state: CellState::Uninit,
                ..
            }
        )),
        "symbolic i32 load after fill_i32 must require a typed element range guard: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_word_fill_accepts_scaled_symbolic_load_with_range_guard() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn main <(i32,i32)->i32> (i, len):
    let p <i32> alloc_raw mul len 4
    fill_i32 p len 0
    let off <i32> mul i 4
    let value <i32> if and ge i 0 lt i len:
        then:
            load_i32 add p off
        else:
            0
    dealloc_raw p mul len 4
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
        "scaled symbolic i32 load guarded by 0 <= i && i < len must use the fill_i32 element range fact: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_word_fill_non_copy_value_does_not_create_range_evidence() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let address = Place::temporary(ResourceId(0), i32_ty);
    let count = Place::temporary(ResourceId(1), i32_ty);
    let value = Place::temporary(ResourceId(2), owned_ty);
    let fill_out = Place::temporary(ResourceId(3), unit_ty);
    let loaded = Place::temporary(ResourceId(4), owned_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: address.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: count.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: value.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Fill,
                output: fill_out,
                args: vec![address.clone(), count, value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded.clone(),
                args: vec![address],
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
        "non-Copy raw fill must be discard-only and must not create initialized range evidence: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_external_fd_read_initializes_nread_cell() {
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
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let store_iov_len = Place::temporary(ResourceId(6), unit_ty);
    let zero = Place::temporary(ResourceId(7), i32_ty);
    let store_nread = Place::temporary(ResourceId(8), unit_ty);
    let errno = Place::temporary(ResourceId(9), i32_ty);
    let loaded_nread = Place::temporary(ResourceId(10), i32_ty);
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
                    operation: ExternalIoOp::FdRead,
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded_nread,
                args: vec![nread],
                span,
            },
        ],
    );
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "fd_read must initialize the nread out cell: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_fd_read_accepts_payload_load_guarded_by_nread() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.bool());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let fd = Place::temporary(ResourceId(0), i32_ty);
    let iov_count = Place::temporary(ResourceId(1), i32_ty);
    let cap = Place::temporary(ResourceId(2), i32_ty);
    let zero = Place::temporary(ResourceId(3), i32_ty);
    let index = Place::temporary(ResourceId(4), i32_ty);
    let buf = Place::temporary(ResourceId(5), i32_ty);
    let iov = Place::temporary(ResourceId(6), i32_ty);
    let nread_ptr = Place::temporary(ResourceId(7), i32_ty);
    let store_buf = Place::temporary(ResourceId(8), unit_ty);
    let iov_len_cell = iov.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let store_len = Place::temporary(ResourceId(9), unit_ty);
    let store_nread = Place::temporary(ResourceId(10), unit_ty);
    let errno = Place::temporary(ResourceId(11), i32_ty);
    let nread_value = Place::temporary(ResourceId(12), i32_ty);
    let condition = Place::temporary(ResourceId(13), bool_ty);
    let loaded_byte = Place::temporary(ResourceId(14), i32_ty);
    let else_value = Place::temporary(ResourceId(15), i32_ty);
    let branch_output = Place::temporary(ResourceId(16), i32_ty);
    let indexed_buf = buf.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
            place: Box::new(index.clone()),
        }),
        i32_ty,
    );
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
                kind: ResourceExprKind::LiteralI32(1),
                output: iov_count.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(8),
                output: cap.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
                output: zero.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: index.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: buf.clone(),
                args: vec![cap.clone()],
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
                output: nread_ptr.clone(),
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
                output: store_len,
                args: vec![iov_len_cell, cap],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_nread,
                args: vec![nread_ptr.clone(), zero],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("fd_read"),
                },
                args: vec![fd, iov, iov_count, nread_ptr.clone()],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdRead,
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: nread_value.clone(),
                args: vec![nread_ptr],
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: condition.clone(),
                ty: bool_ty,
                span,
            },
            ResourceOp::Branch {
                output: branch_output,
                condition,
                condition_fact: Some(ResourceConditionFact::All(vec![
                    ResourceConditionFact::NonNegative {
                        place: index.clone(),
                    },
                    ResourceConditionFact::I32Relation {
                        left: index,
                        op: ResourceI32RelationOp::Lt,
                        right: nread_value,
                    },
                ])),
                then_ops: vec![ResourceOp::RawMemory {
                    operation: RawMemoryOp::Load,
                    output: loaded_byte.clone(),
                    args: vec![indexed_buf],
                    span,
                }],
                then_value: loaded_byte,
                else_ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::LiteralI32(0),
                    output: else_value.clone(),
                    ty: i32_ty,
                    span,
                }],
                else_value,
                span,
            },
        ],
    );
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "fd_read payload load guarded by nread must use a bounded initialized range: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_fd_read_rejects_payload_load_guarded_only_by_capacity() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.bool());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let fd = Place::temporary(ResourceId(0), i32_ty);
    let iov_count = Place::temporary(ResourceId(1), i32_ty);
    let cap = Place::temporary(ResourceId(2), i32_ty);
    let zero = Place::temporary(ResourceId(3), i32_ty);
    let index = Place::temporary(ResourceId(4), i32_ty);
    let buf = Place::temporary(ResourceId(5), i32_ty);
    let iov = Place::temporary(ResourceId(6), i32_ty);
    let nread_ptr = Place::temporary(ResourceId(7), i32_ty);
    let store_buf = Place::temporary(ResourceId(8), unit_ty);
    let iov_len_cell = iov.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let store_len = Place::temporary(ResourceId(9), unit_ty);
    let store_nread = Place::temporary(ResourceId(10), unit_ty);
    let errno = Place::temporary(ResourceId(11), i32_ty);
    let condition = Place::temporary(ResourceId(12), bool_ty);
    let loaded_byte = Place::temporary(ResourceId(13), i32_ty);
    let else_value = Place::temporary(ResourceId(14), i32_ty);
    let branch_output = Place::temporary(ResourceId(15), i32_ty);
    let indexed_buf = buf.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
            place: Box::new(index.clone()),
        }),
        i32_ty,
    );
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
                kind: ResourceExprKind::LiteralI32(1),
                output: iov_count.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(8),
                output: cap.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
                output: zero.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: index.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: buf.clone(),
                args: vec![cap.clone()],
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
                output: nread_ptr.clone(),
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
                output: store_len,
                args: vec![iov_len_cell, cap.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_nread,
                args: vec![nread_ptr.clone(), zero],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("fd_read"),
                },
                args: vec![fd, iov, iov_count, nread_ptr],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdRead,
                },
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: condition.clone(),
                ty: bool_ty,
                span,
            },
            ResourceOp::Branch {
                output: branch_output,
                condition,
                condition_fact: Some(ResourceConditionFact::All(vec![
                    ResourceConditionFact::NonNegative {
                        place: index.clone(),
                    },
                    ResourceConditionFact::I32Relation {
                        left: index,
                        op: ResourceI32RelationOp::Lt,
                        right: cap,
                    },
                ])),
                then_ops: vec![ResourceOp::RawMemory {
                    operation: RawMemoryOp::Load,
                    output: loaded_byte.clone(),
                    args: vec![indexed_buf],
                    span,
                }],
                then_value: loaded_byte,
                else_ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::LiteralI32(0),
                    output: else_value.clone(),
                    ty: i32_ty,
                    span,
                }],
                else_value,
                span,
            },
        ],
    );
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                function,
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                state: CellState::Uninit,
                ..
            } if function == "main" || function.starts_with("main__")
        )),
        "fd_read must not initialize the whole iovec capacity without nread proof: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_fd_readdir_accepts_payload_load_guarded_by_used() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.bool());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let fd = Place::temporary(ResourceId(0), i32_ty);
    let cap = Place::temporary(ResourceId(1), i32_ty);
    let cookie = Place::temporary(ResourceId(2), i32_ty);
    let zero = Place::temporary(ResourceId(3), i32_ty);
    let index = Place::temporary(ResourceId(4), i32_ty);
    let buf = Place::temporary(ResourceId(5), i32_ty);
    let used_ptr = Place::temporary(ResourceId(6), i32_ty);
    let store_used = Place::temporary(ResourceId(7), unit_ty);
    let errno = Place::temporary(ResourceId(8), i32_ty);
    let used_value = Place::temporary(ResourceId(9), i32_ty);
    let condition = Place::temporary(ResourceId(10), bool_ty);
    let loaded_byte = Place::temporary(ResourceId(11), i32_ty);
    let else_value = Place::temporary(ResourceId(12), i32_ty);
    let branch_output = Place::temporary(ResourceId(13), i32_ty);
    let indexed_buf = buf.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
            place: Box::new(index.clone()),
        }),
        i32_ty,
    );
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
                kind: ResourceExprKind::LiteralI32(64),
                output: cap.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
                output: cookie.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
                output: zero.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: index.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: buf.clone(),
                args: vec![cap.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: used_ptr.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_used,
                args: vec![used_ptr.clone(), zero],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("fd_readdir"),
                },
                args: vec![fd, buf.clone(), cap, cookie, used_ptr.clone()],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdReaddir,
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: used_value.clone(),
                args: vec![used_ptr],
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: condition.clone(),
                ty: bool_ty,
                span,
            },
            ResourceOp::Branch {
                output: branch_output,
                condition,
                condition_fact: Some(ResourceConditionFact::All(vec![
                    ResourceConditionFact::NonNegative {
                        place: index.clone(),
                    },
                    ResourceConditionFact::I32Relation {
                        left: index,
                        op: ResourceI32RelationOp::Lt,
                        right: used_value,
                    },
                ])),
                then_ops: vec![ResourceOp::RawMemory {
                    operation: RawMemoryOp::LoadU8,
                    output: loaded_byte.clone(),
                    args: vec![indexed_buf],
                    span,
                }],
                then_value: loaded_byte,
                else_ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::LiteralI32(0),
                    output: else_value.clone(),
                    ty: i32_ty,
                    span,
                }],
                else_value,
                span,
            },
        ],
    );
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "fd_readdir payload load guarded by used byte count must use a bounded initialized range: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_fd_readdir_accepts_load_through_offset_raw_address_alias() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.bool());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let fd = Place::temporary(ResourceId(0), i32_ty);
    let cap = Place::temporary(ResourceId(1), i32_ty);
    let cookie = Place::temporary(ResourceId(2), i32_ty);
    let zero = Place::temporary(ResourceId(3), i32_ty);
    let off_init = Place::temporary(ResourceId(4), i32_ty);
    let buf = Place::temporary(ResourceId(5), i32_ty);
    let used_ptr = Place::temporary(ResourceId(6), i32_ty);
    let store_used = Place::temporary(ResourceId(7), unit_ty);
    let errno = Place::temporary(ResourceId(8), i32_ty);
    let used_value = Place::temporary(ResourceId(9), i32_ty);
    let used = Place::local(String::from("used"), i32_ty);
    let off = Place::local(String::from("off"), i32_ty);
    let condition = Place::temporary(ResourceId(10), bool_ty);
    let off_read = Place::temporary(ResourceId(11), i32_ty);
    let rec = Place::temporary(ResourceId(12), i32_ty);
    let loaded_cell = Place::temporary(ResourceId(13), i32_ty);
    let else_value = Place::temporary(ResourceId(14), i32_ty);
    let branch_output = Place::temporary(ResourceId(15), i32_ty);
    let cell_width = Place::temporary(ResourceId(16), i32_ty);
    let access_end = Place::temporary(ResourceId(17), i32_ty);
    let used_cell = used_ptr
        .clone()
        .with_projection(PlaceProjection::Deref, i32_ty);
    let indexed_buf = buf.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
            place: Box::new(off_read.clone()),
        }),
        i32_ty,
    );
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
                kind: ResourceExprKind::LiteralI32(64),
                output: cap.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
                output: cookie.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
                output: zero.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
                output: off_init.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: buf.clone(),
                args: vec![cap.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: used_ptr.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_used,
                args: vec![used_ptr.clone(), zero],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("fd_readdir"),
                },
                args: vec![fd, buf.clone(), cap, cookie, used_ptr.clone()],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdReaddir,
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: used_value.clone(),
                args: vec![used_ptr],
                span,
            },
            ResourceOp::DeclareLocal {
                place: used.clone(),
                source_name: String::from("used"),
                mutable: false,
                initializer: Some(used_value),
                span,
            },
            ResourceOp::DeclareLocal {
                place: off.clone(),
                source_name: String::from("off"),
                mutable: true,
                initializer: Some(off_init),
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(4),
                output: cell_width.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Call {
                output: access_end.clone(),
                target: ResourceCallTarget::User {
                    name: String::from("add__i32_i32__i32__pure"),
                    type_args: Vec::new(),
                },
                args: vec![off.clone(), cell_width],
                effect: EffectOp::Pure,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: condition.clone(),
                ty: bool_ty,
                span,
            },
            ResourceOp::Branch {
                output: branch_output,
                condition,
                condition_fact: Some(ResourceConditionFact::All(vec![
                    ResourceConditionFact::NonNegative { place: off.clone() },
                    ResourceConditionFact::I32Relation {
                        left: off.clone(),
                        op: ResourceI32RelationOp::Lt,
                        right: used_cell.clone(),
                    },
                    ResourceConditionFact::I32Relation {
                        left: access_end,
                        op: ResourceI32RelationOp::Le,
                        right: used_cell,
                    },
                ])),
                then_ops: vec![
                    ResourceOp::Read {
                        source: off,
                        output: off_read.clone(),
                        span,
                    },
                    ResourceOp::Call {
                        output: rec.clone(),
                        target: ResourceCallTarget::User {
                            name: String::from("add__i32_i32__i32__pure"),
                            type_args: Vec::new(),
                        },
                        args: vec![buf.clone(), off_read.clone()],
                        effect: EffectOp::Pure,
                        span,
                    },
                    ResourceOp::RawAddressAlias {
                        source: indexed_buf,
                        target: rec.clone(),
                        kind: RawAddressAliasKind::Transparent,
                        span,
                    },
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Load,
                        output: loaded_cell.clone(),
                        args: vec![rec],
                        span,
                    },
                ],
                then_value: loaded_cell,
                else_ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::LiteralI32(0),
                    output: else_value.clone(),
                    ty: i32_ty,
                    span,
                }],
                else_value,
                span,
            },
        ],
    );
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "fd_readdir load through raw-address alias must keep offset/used proof: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_fd_readdir_rejects_payload_load_guarded_only_by_capacity() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.bool());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let span = Span::dummy();
    let fd = Place::temporary(ResourceId(0), i32_ty);
    let cap = Place::temporary(ResourceId(1), i32_ty);
    let cookie = Place::temporary(ResourceId(2), i32_ty);
    let zero = Place::temporary(ResourceId(3), i32_ty);
    let index = Place::temporary(ResourceId(4), i32_ty);
    let buf = Place::temporary(ResourceId(5), i32_ty);
    let used_ptr = Place::temporary(ResourceId(6), i32_ty);
    let store_used = Place::temporary(ResourceId(7), unit_ty);
    let errno = Place::temporary(ResourceId(8), i32_ty);
    let condition = Place::temporary(ResourceId(9), bool_ty);
    let loaded_byte = Place::temporary(ResourceId(10), i32_ty);
    let else_value = Place::temporary(ResourceId(11), i32_ty);
    let branch_output = Place::temporary(ResourceId(12), i32_ty);
    let indexed_buf = buf.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
            place: Box::new(index.clone()),
        }),
        i32_ty,
    );
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
                kind: ResourceExprKind::LiteralI32(64),
                output: cap.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
                output: cookie.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
                output: zero.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: index.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: buf.clone(),
                args: vec![cap.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: used_ptr.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_used,
                args: vec![used_ptr.clone(), zero],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("fd_readdir"),
                },
                args: vec![fd, buf.clone(), cap.clone(), cookie, used_ptr],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdReaddir,
                },
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: condition.clone(),
                ty: bool_ty,
                span,
            },
            ResourceOp::Branch {
                output: branch_output,
                condition,
                condition_fact: Some(ResourceConditionFact::All(vec![
                    ResourceConditionFact::NonNegative {
                        place: index.clone(),
                    },
                    ResourceConditionFact::I32Relation {
                        left: index,
                        op: ResourceI32RelationOp::Lt,
                        right: cap,
                    },
                ])),
                then_ops: vec![ResourceOp::RawMemory {
                    operation: RawMemoryOp::LoadU8,
                    output: loaded_byte.clone(),
                    args: vec![indexed_buf],
                    span,
                }],
                then_value: loaded_byte,
                else_ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::LiteralI32(0),
                    output: else_value.clone(),
                    ty: i32_ty,
                    span,
                }],
                else_value,
                span,
            },
        ],
    );
    let report = check_resource_initialized_moves(&resource, &types);
    let uninit = report
        .diagnostics
        .iter()
        .any(|diagnostic| matches!(diagnostic, ResourceCheckDiagnostic::CellUnavailable { .. }));
    assert!(
        uninit,
        "fd_readdir must not initialize the whole output capacity without used-byte proof: {:#?}\nresource:\n{}",
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
    let buf = Place::temporary(ResourceId(8), i32_ty);
    let fill_value = Place::temporary(ResourceId(9), i32_ty);
    let fill_buf = Place::temporary(ResourceId(10), unit_ty);
    let store_buf = Place::temporary(ResourceId(11), unit_ty);
    let iov_len_cell = iov.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let store_len = Place::temporary(ResourceId(12), unit_ty);
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
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: fill_value.clone(),
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
                output: nwritten.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Fill,
                output: fill_buf,
                args: vec![buf.clone(), iov_count.clone(), fill_value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_buf,
                args: vec![iov.clone(), buf],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_len,
                args: vec![iov_len_cell, iov_count.clone()],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("fd_pwrite"),
                },
                args: vec![fd, iov, iov_count, offset.clone(), nwritten.clone()],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdPwrite,
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
fn resource_ir_cell_check_fd_write_accepts_initialized_iovec_buffer() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let fd = Place::temporary(ResourceId(0), i32_ty);
    let iov_count = Place::temporary(ResourceId(1), i32_ty);
    let len = Place::temporary(ResourceId(2), i32_ty);
    let fill_value = Place::temporary(ResourceId(3), i32_ty);
    let buf = Place::temporary(ResourceId(4), i32_ty);
    let iov = Place::temporary(ResourceId(5), i32_ty);
    let nwritten = Place::temporary(ResourceId(6), i32_ty);
    let fill_buf = Place::temporary(ResourceId(7), unit_ty);
    let store_buf = Place::temporary(ResourceId(8), unit_ty);
    let iov_len_cell = iov.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let store_len = Place::temporary(ResourceId(9), unit_ty);
    let errno = Place::temporary(ResourceId(10), i32_ty);
    let loaded_nwritten = Place::temporary(ResourceId(11), i32_ty);
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
                output: nwritten.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Fill,
                output: fill_buf,
                args: vec![buf.clone(), len.clone(), fill_value],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_buf,
                args: vec![iov.clone(), buf],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_len,
                args: vec![iov_len_cell, len],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("fd_write"),
                },
                args: vec![fd, iov, iov_count, nwritten.clone()],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdWrite,
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded_nwritten,
                args: vec![nwritten],
                span,
            },
        ],
    );
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "fd_write must accept initialized iovec descriptors and payload buffers: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_fd_write_reports_uninitialized_iovec_buffer() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let fd = Place::temporary(ResourceId(0), i32_ty);
    let iov_count = Place::temporary(ResourceId(1), i32_ty);
    let len = Place::temporary(ResourceId(2), i32_ty);
    let buf = Place::temporary(ResourceId(3), i32_ty);
    let iov = Place::temporary(ResourceId(4), i32_ty);
    let nwritten = Place::temporary(ResourceId(5), i32_ty);
    let store_buf = Place::temporary(ResourceId(6), unit_ty);
    let iov_len_cell = iov.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let store_len = Place::temporary(ResourceId(7), unit_ty);
    let errno = Place::temporary(ResourceId(8), i32_ty);
    let payload_cell = buf.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Unknown),
        i32_ty,
    );
    let payload_cell = payload_cell.with_projection(PlaceProjection::Deref, i32_ty);
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
                output: len.clone(),
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
                output: nwritten.clone(),
                args: vec![],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_buf,
                args: vec![iov.clone(), buf],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_len,
                args: vec![iov_len_cell, len],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("fd_write"),
                },
                args: vec![fd, iov, iov_count, nwritten],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdWrite,
                },
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
            } if place == &payload_cell
        )),
        "fd_write must reject uninitialized iovec payload buffers: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_fd_read_reports_uninitialized_iovec_descriptor() {
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
    let nread = Place::temporary(ResourceId(3), i32_ty);
    let errno = Place::temporary(ResourceId(4), i32_ty);
    let iov_buffer_cell = iov.clone().with_projection(PlaceProjection::Deref, i32_ty);
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
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("fd_read"),
                },
                args: vec![fd, iov, iov_count, nread],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdRead,
                },
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
            } if place == &iov_buffer_cell
        )),
        "fd_read must reject uninitialized iovec descriptor cells: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_fd_read_rejects_iovec_payload_extent_mismatch() {
    let (resource, types) = external_io_iov_owner_resource(ExternalIoOp::FdRead, 1, 8);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                state: OwnerState::Live { .. },
                ..
            }
        )),
        "fd_read must prove iovec payload length against the backing owner extent: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_fd_write_rejects_iovec_payload_extent_mismatch() {
    let (resource, types) = external_io_iov_owner_resource(ExternalIoOp::FdWrite, 1, 8);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                state: OwnerState::Live { .. },
                ..
            }
        )),
        "fd_write must prove iovec payload length against the backing owner extent: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_applies_fd_write_wrapper_iovec_payload_span_summary() {
    let (resource, types) = fd_write_wrapper_owner_resource(true);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "fd_write wrapper summaries must defer iovec payload proof to the caller and then prove it from caller stores: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_reports_fd_write_wrapper_missing_iovec_payload_store() {
    let (resource, types) = fd_write_wrapper_owner_resource(false);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                function,
                operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                state: OwnerState::NoFreeObligation,
                ..
            } if function == "main"
        )),
        "fd_write wrapper summaries must still reject callers that never initialize the iovec payload pointer: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_fd_write_rejects_iovec_descriptor_extent_mismatch() {
    let (resource, types) =
        external_io_iov_owner_resource_with_iov_storage(ExternalIoOp::FdWrite, 8, 8, 4);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                state: OwnerState::Live { .. },
                ..
            }
        )),
        "fd_write must prove the iovec descriptor span against the descriptor owner extent: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_fd_read_accepts_iovec_payload_extent_match() {
    let (resource, types) = external_io_iov_owner_resource(ExternalIoOp::FdRead, 8, 8);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "matching fd_read iovec payload length and owner extent must be accepted: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_fd_read_accepts_iovec_payload_subspan() {
    let (resource, types) = external_io_iov_owner_resource(ExternalIoOp::FdRead, 16, 8);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "fd_read iovec payload proof must accept host-visible subspans inside the backing owner extent: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_random_get_rejects_output_extent_mismatch() {
    let (resource, types) = direct_host_output_owner_resource(NondetOp::RandomGet, 1, 8);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                state: OwnerState::Live { .. },
                ..
            }
        )),
        "random_get must prove the output byte span against the backing owner extent: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_random_get_initializes_only_reported_byte_range() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.u8());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let len = Place::temporary(ResourceId(0), i32_ty);
    let buf = Place::temporary(ResourceId(1), i32_ty);
    let errno = Place::temporary(ResourceId(2), i32_ty);
    let out_of_range = buf.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(8)),
        i32_ty,
    );
    let loaded = Place::temporary(ResourceId(3), types.u8());
    let out_of_range_cell = out_of_range
        .clone()
        .with_projection(PlaceProjection::Deref, types.u8());
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(4),
                output: len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: buf.clone(),
                args: vec![len.clone()],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("random_get"),
                },
                args: vec![buf, len],
                effect: EffectOp::Nondet {
                    operation: NondetOp::RandomGet,
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::LoadU8,
                output: loaded,
                args: vec![out_of_range],
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
            } if place == &out_of_range_cell
        )),
        "random_get must not initialize bytes beyond the host-visible output length: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_path_open_rejects_path_extent_mismatch() {
    let (resource, types) = path_open_owner_resource(1, 8);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                state: OwnerState::Live { .. },
                ..
            }
        )),
        "path_open must prove the path input span against the backing owner extent: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_path_open_rejects_uninitialized_path_input() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let dirfd = Place::temporary(ResourceId(0), i32_ty);
    let path_len = Place::temporary(ResourceId(1), i32_ty);
    let zero = Place::temporary(ResourceId(2), i32_ty);
    let path = Place::temporary(ResourceId(3), i32_ty);
    let fd_out = Place::temporary(ResourceId(4), i32_ty);
    let errno = Place::temporary(ResourceId(5), i32_ty);
    let fd_out_len = Place::temporary(ResourceId(6), i32_ty);
    let path_cell = path
        .clone()
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::Unknown),
            i32_ty,
        )
        .with_projection(PlaceProjection::Deref, i32_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(3),
                output: dirfd.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(8),
                output: path_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
                output: zero.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(4),
                output: fd_out_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: path.clone(),
                args: vec![path_len.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: fd_out.clone(),
                args: vec![fd_out_len],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("path_open"),
                },
                args: vec![
                    dirfd,
                    zero.clone(),
                    path,
                    path_len,
                    zero.clone(),
                    zero.clone(),
                    zero.clone(),
                    zero.clone(),
                    fd_out,
                ],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::PathOpen,
                },
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
            } if place == &path_cell
        )),
        "path_open must reject uninitialized path bytes before the host reads them: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_path_open_accepts_string_data_ptr_with_len() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "alloc/string/storage" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/raw" as *
#import "core/cast" as *
#import "std/fs/raw" as *

fn open_probe <(str)*>i32> (path):
    let path_ptr <i32> mem_ptr_addr string_data_ptr path
    let path_len <i32> len path
    let rights <i64> cast 0
    wasi_path_open 3 0 path_ptr path_len 0 rights rights 0 0

fn main <()*>i32> ():
    open_probe "abc"
"#;

    let (module, types) = typecheck_resource_source_with_target(source, CompileTarget::Wasi);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "open_probe" || function.starts_with("open_probe__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "path_open must accept path pointer and byte length proven from the same str source: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_path_open_accepts_non_owning_string_data_ptr() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "alloc/string/storage" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/raw" as *
#import "core/cast" as *
#import "std/fs/raw" as *

fn open_probe <(str)*>i32> (path):
    let path_ptr <i32> mem_ptr_addr string_data_ptr path
    let path_len <i32> len path
    let rights <i64> cast 0
    wasi_path_open 3 0 path_ptr path_len 0 rights rights 0 0

fn main <()*>i32> ():
    open_probe "abc"
"#;

    let (module, types) = typecheck_resource_source_with_target(source, CompileTarget::Wasi);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                    | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                    | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. }
                    if function == "open_probe" || function.starts_with("open_probe__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "path_open owner check must not require a free obligation for a non-owning str data view: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_fs_open_with_flags_accepts_string_path() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/cast" as *
#import "core/result" as *
#import "std/fs/fd" as *

fn main <()*>i32> ():
    let rights <i64> cast 0
    match fs_open_with_flags "dir" 0 rights:
        Result::Ok fd:
            fd
        Result::Err e:
            e
"#;

    let (module, types) = typecheck_resource_source_with_target(source, CompileTarget::Wasi);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "fs_open_with_flags__str_i32_i64__Result_T_E_i32_i32__imp"
                        || function.starts_with("fs_open_with_flags__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "fs_open_with_flags must preserve string path pointer/length proof through its local control flow: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_args_get_accepts_sizes_get_dependent_extent_proof() {
    let (resource, types) = args_get_dependent_host_span_resource(true, true, false, false);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "args_get must accept pointer table and byte buffer extents derived from args_sizes_get proof: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_environ_get_accepts_sizes_get_dependent_extent_proof() {
    let (resource, types) = args_get_dependent_host_span_resource(true, true, false, true);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "environ_get must accept pointer table and byte buffer extents derived from environ_sizes_get proof: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_args_get_rejects_missing_sizes_get_proof() {
    let (resource, types) = args_get_dependent_host_span_resource(false, true, false, false);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                state: OwnerState::Live { .. },
                ..
            }
        )),
        "args_get must reject host output owners when no prior args_sizes_get proof exists: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_args_get_rejects_unscaled_pointer_table_extent() {
    let (resource, types) = args_get_dependent_host_span_resource(true, false, false, false);
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                state: OwnerState::Live { .. },
                ..
            }
        )),
        "args_get must require the pointer table owner extent to be argc * pointer_width, not argc bytes: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_args_get_without_sizes_get_does_not_initialize_unknown_offset() {
    let (resource, types) = args_get_dependent_host_span_resource(false, true, true, false);
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                state: CellState::Uninit,
                ..
            }
        )),
        "args_get without a sizes_get proof must not mark argv as unknown-offset initialized: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_args_sizes_get_accepts_known_offset_output_cell() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let len = Place::temporary(ResourceId(0), i32_ty);
    let meta = Place::temporary(ResourceId(1), i32_ty);
    let meta_second = meta.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let errno = Place::temporary(ResourceId(2), i32_ty);
    let free = Place::temporary(ResourceId(3), unit_ty);
    let resource = manual_resource_module_with_effect(
        Effect::Impure,
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(8),
                output: len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: meta.clone(),
                args: vec![len.clone()],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("args_sizes_get"),
                },
                args: vec![meta.clone(), meta_second],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::ArgsSizesGet,
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: free,
                args: vec![meta, len],
                span,
            },
        ],
    );

    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "args_sizes_get must prove a second i32 output at base+4 from the same 8-byte owner: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_args_sizes_get_rejects_known_offset_beyond_owner() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let len = Place::temporary(ResourceId(0), i32_ty);
    let meta = Place::temporary(ResourceId(1), i32_ty);
    let meta_second = meta.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let errno = Place::temporary(ResourceId(2), i32_ty);
    let free = Place::temporary(ResourceId(3), unit_ty);
    let resource = manual_resource_module_with_effect(
        Effect::Impure,
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(4),
                output: len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: meta.clone(),
                args: vec![len.clone()],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("args_sizes_get"),
                },
                args: vec![meta.clone(), meta_second],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::ArgsSizesGet,
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: free,
                args: vec![meta, len],
                span,
            },
        ],
    );

    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceOwnerDiagnostic::OwnerUnavailable {
                operation: ResourceOwnerOperation::ExternalIoPayloadExtent,
                state: OwnerState::Live { .. },
                ..
            }
        )),
        "args_sizes_get must reject base+4 i32 output when the owner has only 4 bytes: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_owner_check_args_get_accepts_host_size_return_summary() {
    let (resource, types) = args_get_host_size_return_summary_resource();
    let report = check_resource_owner_obligations(&resource, &types);
    assert!(
        report.diagnostics.is_empty(),
        "args_get must accept argc/buf_size facts returned through a typed struct summary: {:#?}",
        report.diagnostics
    );
}

#[test]
fn resource_ir_owner_check_cliarg_get_accepts_region_token_return_summary() {
    let source = r#"
#entry main
#indent 4
#target wasi
#import "core/option" as *
#import "std/env/cliarg/raw" as *

fn main <()*>i32> ():
    match cliarg_get_checked 0:
        Option::Some _arg:
            1
        Option::None:
            0
"#;
    let (module, types) = typecheck_resource_source_with_target(source, CompileTarget::Wasi);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let cliarg_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            function.starts_with("cliarg_get_checked__")
                || function.starts_with("cli_args_sizes_result__")
        })
        .collect::<Vec<_>>();
    assert!(
        cliarg_diagnostics.is_empty(),
        "cliarg_get_checked must prove args_get scratch RegionToken extents through function and Result summaries: {:#?}",
        cliarg_diagnostics
    );
}

fn external_io_iov_owner_resource(
    operation: ExternalIoOp,
    allocation_len: i32,
    iov_len: i32,
) -> (ResourceModule, TypeCtx) {
    external_io_iov_owner_resource_with_iov_storage(operation, allocation_len, iov_len, 8)
}

fn fd_write_wrapper_owner_resource(include_payload_store: bool) -> (ResourceModule, TypeCtx) {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let wrapper_fd = Place::local(String::from("fd"), i32_ty);
    let wrapper_iov = Place::local(String::from("iov"), i32_ty);
    let wrapper_iovcnt = Place::local(String::from("iovcnt"), i32_ty);
    let wrapper_nwritten = Place::local(String::from("nwritten"), i32_ty);
    let wrapper_errno = Place::temporary(ResourceId(100), i32_ty);
    let wrapper = ResourceFunction {
        name: String::from("fd_write_wrapper"),
        origin_name: String::from("fd_write_wrapper"),
        type_params: Vec::new(),
        params: vec![
            ResourceLocal {
                name: String::from("fd"),
                ty: i32_ty,
                mutable: false,
                place: wrapper_fd.clone(),
            },
            ResourceLocal {
                name: String::from("iov"),
                ty: i32_ty,
                mutable: false,
                place: wrapper_iov.clone(),
            },
            ResourceLocal {
                name: String::from("iovcnt"),
                ty: i32_ty,
                mutable: false,
                place: wrapper_iovcnt.clone(),
            },
            ResourceLocal {
                name: String::from("nwritten"),
                ty: i32_ty,
                mutable: false,
                place: wrapper_nwritten.clone(),
            },
        ],
        result: i32_ty,
        effect: Effect::Impure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![ResourceOp::Call {
                output: wrapper_errno.clone(),
                target: ResourceCallTarget::Builtin {
                    name: String::from("fd_write"),
                },
                args: vec![wrapper_fd, wrapper_iov, wrapper_iovcnt, wrapper_nwritten],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::FdWrite,
                },
                span,
            }],
            terminator: ResourceTerminator::Return {
                value: Some(wrapper_errno),
                span,
            },
            span,
        }],
        span,
    };
    let fd = Place::temporary(ResourceId(0), i32_ty);
    let iov_count = Place::temporary(ResourceId(1), i32_ty);
    let payload_len = Place::temporary(ResourceId(2), i32_ty);
    let iov_storage_len = Place::temporary(ResourceId(3), i32_ty);
    let out_storage_len = Place::temporary(ResourceId(4), i32_ty);
    let payload = Place::temporary(ResourceId(5), i32_ty);
    let payload_view = Place::temporary(ResourceId(6), i32_ty);
    let iov = Place::temporary(ResourceId(7), i32_ty);
    let out_ptr = Place::temporary(ResourceId(8), i32_ty);
    let iov_len_cell = iov.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let store_payload = Place::temporary(ResourceId(9), unit_ty);
    let store_len = Place::temporary(ResourceId(10), unit_ty);
    let errno = Place::temporary(ResourceId(11), i32_ty);
    let free_out = Place::temporary(ResourceId(12), unit_ty);
    let free_iov = Place::temporary(ResourceId(13), unit_ty);
    let free_payload = Place::temporary(ResourceId(14), unit_ty);
    let mut ops = vec![
        ResourceOp::Expr {
            kind: ResourceExprKind::LiteralI32(0),
            output: fd.clone(),
            ty: i32_ty,
            span,
        },
        ResourceOp::Expr {
            kind: ResourceExprKind::LiteralI32(1),
            output: iov_count.clone(),
            ty: i32_ty,
            span,
        },
        ResourceOp::Expr {
            kind: ResourceExprKind::LiteralI32(8),
            output: payload_len.clone(),
            ty: i32_ty,
            span,
        },
        ResourceOp::Expr {
            kind: ResourceExprKind::LiteralI32(8),
            output: iov_storage_len.clone(),
            ty: i32_ty,
            span,
        },
        ResourceOp::Expr {
            kind: ResourceExprKind::LiteralI32(4),
            output: out_storage_len.clone(),
            ty: i32_ty,
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Alloc,
            output: payload.clone(),
            args: vec![payload_len.clone()],
            span,
        },
        ResourceOp::RawAddressView {
            source: payload.clone(),
            target: payload_view.clone(),
            kind: RawAddressViewKind::NonOwningProjection,
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Alloc,
            output: iov.clone(),
            args: vec![iov_storage_len.clone()],
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Alloc,
            output: out_ptr.clone(),
            args: vec![out_storage_len.clone()],
            span,
        },
    ];
    if include_payload_store {
        ops.push(ResourceOp::RawMemory {
            operation: RawMemoryOp::Store,
            output: store_payload,
            args: vec![iov.clone(), payload_view],
            span,
        });
    }
    ops.extend([
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Store,
            output: store_len,
            args: vec![iov_len_cell, payload_len.clone()],
            span,
        },
        ResourceOp::Call {
            output: errno,
            target: ResourceCallTarget::User {
                name: String::from("fd_write_wrapper"),
                type_args: Vec::new(),
            },
            args: vec![fd, iov.clone(), iov_count, out_ptr.clone()],
            effect: EffectOp::UserCall {
                name: String::from("fd_write_wrapper"),
                effect: Effect::Impure,
            },
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Dealloc,
            output: free_out,
            args: vec![out_ptr, out_storage_len],
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Dealloc,
            output: free_iov,
            args: vec![iov, iov_storage_len],
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Dealloc,
            output: free_payload,
            args: vec![payload, payload_len],
            span,
        },
    ]);
    let main = ResourceFunction {
        name: String::from("main"),
        origin_name: String::from("main"),
        type_params: Vec::new(),
        params: vec![],
        result: unit_ty,
        effect: Effect::Impure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops,
            terminator: ResourceTerminator::Return { value: None, span },
            span,
        }],
        span,
    };
    (
        ResourceModule {
            functions: vec![wrapper, main],
            entry: Some(String::from("main")),
            string_literals: vec![],
        },
        types,
    )
}

fn external_io_iov_owner_resource_with_iov_storage(
    operation: ExternalIoOp,
    allocation_len: i32,
    iov_len: i32,
    iov_storage_bytes: i32,
) -> (ResourceModule, TypeCtx) {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let fd = Place::temporary(ResourceId(0), i32_ty);
    let iov_count = Place::temporary(ResourceId(1), i32_ty);
    let alloc_len = Place::temporary(ResourceId(2), i32_ty);
    let payload_len = Place::temporary(ResourceId(3), i32_ty);
    let iov_storage_len = Place::temporary(ResourceId(4), i32_ty);
    let out_storage_len = Place::temporary(ResourceId(5), i32_ty);
    let payload = Place::temporary(ResourceId(6), i32_ty);
    let payload_view = Place::temporary(ResourceId(7), i32_ty);
    let iov = Place::temporary(ResourceId(8), i32_ty);
    let out_ptr = Place::temporary(ResourceId(9), i32_ty);
    let store_payload = Place::temporary(ResourceId(10), unit_ty);
    let iov_len_cell = iov.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let store_len = Place::temporary(ResourceId(11), unit_ty);
    let errno = Place::temporary(ResourceId(12), i32_ty);
    let free_out = Place::temporary(ResourceId(13), unit_ty);
    let free_iov = Place::temporary(ResourceId(14), unit_ty);
    let free_payload = Place::temporary(ResourceId(15), unit_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
                output: fd.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(1),
                output: iov_count.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(allocation_len),
                output: alloc_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(iov_len),
                output: payload_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(iov_storage_bytes),
                output: iov_storage_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(4),
                output: out_storage_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: payload.clone(),
                args: vec![alloc_len.clone()],
                span,
            },
            ResourceOp::RawAddressView {
                source: payload.clone(),
                target: payload_view.clone(),
                kind: RawAddressViewKind::NonOwningProjection,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: iov.clone(),
                args: vec![iov_storage_len.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: out_ptr.clone(),
                args: vec![out_storage_len.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_payload,
                args: vec![iov.clone(), payload_view],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Store,
                output: store_len,
                args: vec![iov_len_cell, payload_len],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: operation.as_str().to_string(),
                },
                args: vec![fd, iov.clone(), iov_count, out_ptr.clone()],
                effect: EffectOp::ExternalIo { operation },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: free_out,
                args: vec![out_ptr, out_storage_len],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: free_iov,
                args: vec![iov, iov_storage_len],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: free_payload,
                args: vec![payload, alloc_len],
                span,
            },
        ],
    );
    (resource, types)
}

fn args_get_dependent_host_span_resource(
    include_sizes_get: bool,
    scale_pointer_table: bool,
    read_after_args_get: bool,
    use_environ: bool,
) -> (ResourceModule, TypeCtx) {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let four = Place::temporary(ResourceId(0), i32_ty);
    let argc_cell = Place::temporary(ResourceId(1), i32_ty);
    let argv_buf_size_cell = Place::temporary(ResourceId(2), i32_ty);
    let argc = Place::temporary(ResourceId(3), i32_ty);
    let argv_buf_size = Place::temporary(ResourceId(4), i32_ty);
    let table_bytes = Place::temporary(ResourceId(5), i32_ty);
    let argv = Place::temporary(ResourceId(6), i32_ty);
    let argv_buf = Place::temporary(ResourceId(7), i32_ty);
    let sizes_errno = Place::temporary(ResourceId(8), i32_ty);
    let args_errno = Place::temporary(ResourceId(9), i32_ty);
    let read_argv = Place::temporary(ResourceId(10), i32_ty);
    let free_argv_buf = Place::temporary(ResourceId(11), unit_ty);
    let free_argv = Place::temporary(ResourceId(12), unit_ty);
    let free_argv_buf_size_cell = Place::temporary(ResourceId(13), unit_ty);
    let free_argc_cell = Place::temporary(ResourceId(14), unit_ty);
    let sizes_name = if use_environ {
        "environ_sizes_get"
    } else {
        "args_sizes_get"
    };
    let sizes_operation = if use_environ {
        ExternalIoOp::EnvironSizesGet
    } else {
        ExternalIoOp::ArgsSizesGet
    };
    let get_name = if use_environ {
        "environ_get"
    } else {
        "args_get"
    };
    let get_operation = if use_environ {
        ExternalIoOp::EnvironGet
    } else {
        ExternalIoOp::ArgsGet
    };
    let mut ops = vec![
        ResourceOp::Expr {
            kind: ResourceExprKind::LiteralI32(4),
            output: four.clone(),
            ty: i32_ty,
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Alloc,
            output: argc_cell.clone(),
            args: vec![four.clone()],
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Alloc,
            output: argv_buf_size_cell.clone(),
            args: vec![four.clone()],
            span,
        },
    ];
    if include_sizes_get {
        ops.push(ResourceOp::Call {
            output: sizes_errno,
            target: ResourceCallTarget::Builtin {
                name: String::from(sizes_name),
            },
            args: vec![argc_cell.clone(), argv_buf_size_cell.clone()],
            effect: EffectOp::ExternalIo {
                operation: sizes_operation,
            },
            span,
        });
    }
    if include_sizes_get {
        ops.extend([
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: argc.clone(),
                args: vec![argc_cell.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: argv_buf_size.clone(),
                args: vec![argv_buf_size_cell.clone()],
                span,
            },
        ]);
    } else {
        ops.extend([
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(1),
                output: argc.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(4),
                output: argv_buf_size.clone(),
                ty: i32_ty,
                span,
            },
        ]);
    }
    if scale_pointer_table {
        ops.push(ResourceOp::Call {
            output: table_bytes.clone(),
            target: ResourceCallTarget::User {
                name: String::from("mul__i32_i32__i32__pure"),
                type_args: Vec::new(),
            },
            args: vec![argc.clone(), four.clone()],
            effect: EffectOp::Pure,
            span,
        });
    }
    let argv_extent = if scale_pointer_table {
        table_bytes.clone()
    } else {
        argc.clone()
    };
    ops.extend([
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Alloc,
            output: argv.clone(),
            args: vec![argv_extent.clone()],
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Alloc,
            output: argv_buf.clone(),
            args: vec![argv_buf_size.clone()],
            span,
        },
        ResourceOp::Call {
            output: args_errno,
            target: ResourceCallTarget::Builtin {
                name: String::from(get_name),
            },
            args: vec![argv.clone(), argv_buf.clone()],
            effect: EffectOp::ExternalIo {
                operation: get_operation,
            },
            span,
        },
    ]);
    if read_after_args_get {
        ops.push(ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            output: read_argv,
            args: vec![argv.clone()],
            span,
        });
    }
    ops.extend([
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Dealloc,
            output: free_argv_buf,
            args: vec![argv_buf, argv_buf_size],
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Dealloc,
            output: free_argv,
            args: vec![argv, argv_extent],
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Dealloc,
            output: free_argv_buf_size_cell,
            args: vec![argv_buf_size_cell, four.clone()],
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Dealloc,
            output: free_argc_cell,
            args: vec![argc_cell, four],
            span,
        },
    ]);
    (manual_resource_module(unit_ty, span, ops), types)
}

fn args_get_host_size_return_summary_resource() -> (ResourceModule, TypeCtx) {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let sizes_ty = types.register_named(
        "Sizes".to_string(),
        TypeKind::Struct {
            name: "Sizes".to_string(),
            type_params: vec![],
            fields: vec![i32_ty, i32_ty],
            field_names: vec!["argc".to_string(), "buf_size".to_string()],
        },
    );
    let span = Span::dummy();
    let helper_meta = Place::local(String::from("meta"), i32_ty);
    let helper_meta_second = helper_meta.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let helper_errno = Place::temporary(ResourceId(0), i32_ty);
    let helper_argc = Place::temporary(ResourceId(1), i32_ty);
    let helper_buf_size = Place::temporary(ResourceId(2), i32_ty);
    let helper_sizes = Place::temporary(ResourceId(3), sizes_ty);
    let helper = ResourceFunction {
        name: String::from("sizes_helper"),
        origin_name: String::from("sizes_helper"),
        type_params: Vec::new(),
        params: vec![ResourceLocal {
            name: String::from("meta"),
            ty: i32_ty,
            mutable: false,
            place: helper_meta.clone(),
        }],
        result: sizes_ty,
        effect: Effect::Impure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![
                ResourceOp::Call {
                    output: helper_errno,
                    target: ResourceCallTarget::Builtin {
                        name: String::from("args_sizes_get"),
                    },
                    args: vec![helper_meta.clone(), helper_meta_second.clone()],
                    effect: EffectOp::ExternalIo {
                        operation: ExternalIoOp::ArgsSizesGet,
                    },
                    span,
                },
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Load,
                    output: helper_argc.clone(),
                    args: vec![helper_meta],
                    span,
                },
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Load,
                    output: helper_buf_size.clone(),
                    args: vec![helper_meta_second],
                    span,
                },
                ResourceOp::Construct {
                    output: helper_sizes.clone(),
                    kind: AggregateKind::Struct {
                        name: String::from("Sizes"),
                        field_offsets: vec![0, 4],
                    },
                    inputs: vec![helper_argc, helper_buf_size],
                    span,
                },
            ],
            terminator: ResourceTerminator::Return {
                value: Some(helper_sizes),
                span,
            },
            span,
        }],
        span,
    };

    let len = Place::temporary(ResourceId(100), i32_ty);
    let meta = Place::temporary(ResourceId(101), i32_ty);
    let sizes = Place::temporary(ResourceId(102), sizes_ty);
    let argc_field = sizes.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let buf_size_field = sizes.clone().with_projection(
        PlaceProjection::Field {
            index: 1,
            offset_bytes: 4,
        },
        i32_ty,
    );
    let argc = Place::temporary(ResourceId(103), i32_ty);
    let buf_size = Place::temporary(ResourceId(104), i32_ty);
    let four = Place::temporary(ResourceId(105), i32_ty);
    let table_bytes = Place::temporary(ResourceId(106), i32_ty);
    let argv = Place::temporary(ResourceId(107), i32_ty);
    let argv_buf = Place::temporary(ResourceId(108), i32_ty);
    let args_errno = Place::temporary(ResourceId(109), i32_ty);
    let free_argv_buf = Place::temporary(ResourceId(110), unit_ty);
    let free_argv = Place::temporary(ResourceId(111), unit_ty);
    let free_meta = Place::temporary(ResourceId(112), unit_ty);
    let main = ResourceFunction {
        name: String::from("main"),
        origin_name: String::from("main"),
        type_params: Vec::new(),
        params: vec![],
        result: unit_ty,
        effect: Effect::Impure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: vec![
                ResourceOp::Expr {
                    kind: ResourceExprKind::LiteralI32(8),
                    output: len.clone(),
                    ty: i32_ty,
                    span,
                },
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Alloc,
                    output: meta.clone(),
                    args: vec![len.clone()],
                    span,
                },
                ResourceOp::Call {
                    output: sizes.clone(),
                    target: ResourceCallTarget::User {
                        name: String::from("sizes_helper"),
                        type_args: Vec::new(),
                    },
                    args: vec![meta.clone()],
                    effect: EffectOp::UserCall {
                        name: String::from("sizes_helper"),
                        effect: Effect::Impure,
                    },
                    span,
                },
                ResourceOp::Read {
                    source: argc_field,
                    output: argc.clone(),
                    span,
                },
                ResourceOp::Read {
                    source: buf_size_field,
                    output: buf_size.clone(),
                    span,
                },
                ResourceOp::Expr {
                    kind: ResourceExprKind::LiteralI32(4),
                    output: four.clone(),
                    ty: i32_ty,
                    span,
                },
                ResourceOp::Call {
                    output: table_bytes.clone(),
                    target: ResourceCallTarget::User {
                        name: String::from("mul__i32_i32__i32__pure"),
                        type_args: Vec::new(),
                    },
                    args: vec![argc, four],
                    effect: EffectOp::Pure,
                    span,
                },
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Alloc,
                    output: argv.clone(),
                    args: vec![table_bytes.clone()],
                    span,
                },
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Alloc,
                    output: argv_buf.clone(),
                    args: vec![buf_size.clone()],
                    span,
                },
                ResourceOp::Call {
                    output: args_errno,
                    target: ResourceCallTarget::Builtin {
                        name: String::from("args_get"),
                    },
                    args: vec![argv.clone(), argv_buf.clone()],
                    effect: EffectOp::ExternalIo {
                        operation: ExternalIoOp::ArgsGet,
                    },
                    span,
                },
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Dealloc,
                    output: free_argv_buf,
                    args: vec![argv_buf, buf_size],
                    span,
                },
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Dealloc,
                    output: free_argv,
                    args: vec![argv, table_bytes],
                    span,
                },
                ResourceOp::RawMemory {
                    operation: RawMemoryOp::Dealloc,
                    output: free_meta,
                    args: vec![meta, len],
                    span,
                },
            ],
            terminator: ResourceTerminator::Return { value: None, span },
            span,
        }],
        span,
    };

    (
        ResourceModule {
            functions: vec![helper, main],
            entry: Some(String::from("main")),
            string_literals: vec![],
        },
        types,
    )
}

fn path_open_owner_resource(allocation_len: i32, path_len_bytes: i32) -> (ResourceModule, TypeCtx) {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let dirfd = Place::temporary(ResourceId(0), i32_ty);
    let alloc_len = Place::temporary(ResourceId(1), i32_ty);
    let path_len = Place::temporary(ResourceId(2), i32_ty);
    let fd_out_len = Place::temporary(ResourceId(3), i32_ty);
    let zero = Place::temporary(ResourceId(4), i32_ty);
    let fill = Place::temporary(ResourceId(5), i32_ty);
    let path = Place::temporary(ResourceId(6), i32_ty);
    let fd_out = Place::temporary(ResourceId(7), i32_ty);
    let fill_path = Place::temporary(ResourceId(8), unit_ty);
    let errno = Place::temporary(ResourceId(9), i32_ty);
    let free_fd_out = Place::temporary(ResourceId(10), unit_ty);
    let free_path = Place::temporary(ResourceId(11), unit_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(3),
                output: dirfd.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(allocation_len),
                output: alloc_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(path_len_bytes),
                output: path_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(4),
                output: fd_out_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(0),
                output: zero.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(47),
                output: fill.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: path.clone(),
                args: vec![alloc_len.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: fd_out.clone(),
                args: vec![fd_out_len.clone()],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::FillBytes,
                output: fill_path,
                args: vec![path.clone(), alloc_len.clone(), fill],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: String::from("path_open"),
                },
                args: vec![
                    dirfd,
                    zero.clone(),
                    path.clone(),
                    path_len,
                    zero.clone(),
                    zero.clone(),
                    zero.clone(),
                    zero,
                    fd_out.clone(),
                ],
                effect: EffectOp::ExternalIo {
                    operation: ExternalIoOp::PathOpen,
                },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: free_fd_out,
                args: vec![fd_out, fd_out_len],
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: free_path,
                args: vec![path, alloc_len],
                span,
            },
        ],
    );
    (resource, types)
}

fn direct_host_output_owner_resource(
    operation: NondetOp,
    allocation_len: i32,
    output_len: i32,
) -> (ResourceModule, TypeCtx) {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let alloc_len = Place::temporary(ResourceId(0), i32_ty);
    let requested_len = Place::temporary(ResourceId(1), i32_ty);
    let buf = Place::temporary(ResourceId(2), i32_ty);
    let errno = Place::temporary(ResourceId(3), i32_ty);
    let free_buf = Place::temporary(ResourceId(4), unit_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(allocation_len),
                output: alloc_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(output_len),
                output: requested_len.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Alloc,
                output: buf.clone(),
                args: vec![alloc_len.clone()],
                span,
            },
            ResourceOp::Call {
                output: errno,
                target: ResourceCallTarget::Builtin {
                    name: operation.as_str().to_string(),
                },
                args: vec![buf.clone(), requested_len],
                effect: EffectOp::Nondet { operation },
                span,
            },
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Dealloc,
                output: free_buf,
                args: vec![buf, alloc_len],
                span,
            },
        ],
    );
    (resource, types)
}

#[test]
fn resource_ir_cell_check_returned_raw_header_preserves_initialized_pointee() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
fn resource_ir_cell_check_returned_raw_header_preserves_guarded_byte_range() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn id <(i32)->i32> (x):
    x

fn make_header <(i32)->i32> (len):
    let data <i32> alloc_raw len
    fill_u8 data len 65
    let header <i32> alloc_raw 8
    store_i32 header data
    store_i32 add header 4 len
    header

fn main <()->i32> ():
    let header <i32> make_header 4
    let data <i32> load_i32 header
    let len <i32> load_i32 add header 4
    let i <i32> id 2
    let value <i32> if:
        and ge i 0 lt i len
        then:
            load_u8 add data i
        else:
            0
    dealloc_raw data len
    dealloc_raw header 8
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
        "returned raw header must summarize byte range only when caller proves offset < len: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_returned_raw_header_rejects_unguarded_byte_range() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

fn id <(i32)->i32> (x):
    x

fn make_header <(i32)->i32> (len):
    let data <i32> alloc_raw len
    fill_u8 data len 65
    let header <i32> alloc_raw 8
    store_i32 header data
    store_i32 add header 4 len
    header

fn main <()->i32> ():
    let header <i32> make_header 4
    let data <i32> load_i32 header
    let len <i32> load_i32 add header 4
    let i <i32> id 2
    let value <i32> load_u8 add data i
    dealloc_raw data len
    dealloc_raw header 8
    value
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                function,
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                state: CellState::Uninit,
                ..
            } if function.starts_with("main__")
        )),
        "returned raw header byte range must still reject unguarded symbolic loads: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_returned_growing_scanner_header_preserves_capacity_byte_range() {
    let source = r#"
#entry main
#indent 4
#target wasi
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "std/stdio" as *

fn id <(i32)->i32> (x):
    x

fn scanner_read_all <()*>i32> ():
    let mut cap <i32> 4
    let mut buf <i32> alloc_raw cap
    memset_u8 buf cap 0
    let iov <i32> alloc_raw 8
    let nread_ptr <i32> alloc_raw 4
    let mut len <i32> 0
    let mut done <bool> false
    while not done:
        do:
            if le cap len:
                then:
                    let next_cap <i32> mul cap 2
                    let grown <i32> realloc_raw buf cap next_cap
                    if eq grown 0:
                        then:
                            set done true
                        else:
                            set buf grown
                            memset_u8 add buf cap sub next_cap cap 0
                            set cap next_cap
                else:
                    ()
            if not done:
                then:
                    store_i32 iov add buf len
                    store_i32 add iov 4 sub cap len
                    store_i32 nread_ptr 0
                    let errno <i32> fd_read 0 iov 1 nread_ptr
                    let got <i32> load_i32 nread_ptr
                    if or ne errno 0 eq got 0:
                        then:
                            set done true
                        else:
                            set len add len got
                else:
                    ()
    dealloc_raw iov 8
    dealloc_raw nread_ptr 4
    let sc <i32> alloc_raw 16
    store_i32 sc buf
    store_i32 add sc 4 len
    store_i32 add sc 8 0
    store_i32 add sc 12 cap
    sc

fn main <()*>i32> ():
    let sc <i32> scanner_read_all
    let data <i32> load_i32 sc
    let len <i32> load_i32 add sc 4
    let cap <i32> load_i32 add sc 12
    let i <i32> id 2
    let value <i32> if and and ge i 0 lt i len lt i cap:
        then:
            load_u8 add data i
        else:
            0
    dealloc_raw data cap
    dealloc_raw sc 16
    value
"#;

    let (module, types) = typecheck_resource_source_with_target(source, CompileTarget::Wasi);
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
        "returned grow-loop scanner header must preserve initialized byte range up to returned capacity: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_returned_aggregate_preserves_guarded_byte_range() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct Scanner:
    buf <i32>
    len <i32>

fn id <(i32)->i32> (x):
    x

fn make_scanner <(i32)->Scanner> (len):
    let data <i32> alloc_raw len
    fill_u8 data len 65
    Scanner data len

fn main <()->i32> ():
    let sc <Scanner> make_scanner 4
    let data <i32> field::get sc "buf"
    let len <i32> field::get sc "len"
    let i <i32> id 2
    let value <i32> if:
        and ge i 0 lt i len
        then:
            load_u8 add data i
        else:
            0
    dealloc_raw data len
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
        "returned aggregate fields must carry guarded byte range summaries: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_returned_aggregate_rejects_unguarded_byte_range() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct Scanner:
    buf <i32>
    len <i32>

fn id <(i32)->i32> (x):
    x

fn make_scanner <(i32)->Scanner> (len):
    let data <i32> alloc_raw len
    fill_u8 data len 65
    Scanner data len

fn main <()->i32> ():
    let sc <Scanner> make_scanner 4
    let data <i32> field::get sc "buf"
    let len <i32> field::get sc "len"
    let i <i32> id 2
    let value <i32> load_u8 add data i
    dealloc_raw data len
    value
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                function,
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                state: CellState::Uninit,
                ..
            } if function.starts_with("main__")
        )),
        "returned aggregate byte range must still reject unguarded symbolic loads: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_aggregate_assignment_clears_stale_byte_range() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct Scanner:
    buf <i32>
    len <i32>

fn id <(i32)->i32> (x):
    x

fn make_filled <(i32)->Scanner> (len):
    let data <i32> alloc_raw len
    fill_u8 data len 65
    Scanner data len

fn make_unfilled <(i32)->Scanner> (len):
    let data <i32> alloc_raw len
    Scanner data len

fn main <()->i32> ():
    let mut sc <Scanner> make_filled 4
    set sc make_unfilled 4
    let data <i32> field::get sc "buf"
    let len <i32> field::get sc "len"
    let i <i32> id 2
    let value <i32> if:
        and ge i 0 lt i len
        then:
            load_u8 add data i
        else:
            0
    dealloc_raw data len
    value
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                function,
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                state: CellState::Uninit,
                ..
            } if function.starts_with("main__")
        )),
        "aggregate assignment must clear stale initialized range facts: {:#?}\nresource:\n{}",
        report.diagnostics,
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
fn resource_ir_cell_check_raw_fill_with_non_copy_value_does_not_initialize_range() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    types.register_copy_impl_target(types.i32());
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let ptr = Place::temporary(ResourceId(0), i32_ty);
    let len = Place::temporary(ResourceId(1), i32_ty);
    let fill_value = Place::temporary(ResourceId(2), owned_ty);
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
                ty: owned_ty,
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
        "raw fill with a non-Copy value must not create repeated initialized owner cells: {:#?}",
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
fn resource_ir_cell_check_rejects_double_non_copy_load_through_mem_ptr_alias() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let r1 <i32> mem_ptr_addr p
    let r2 <i32> mem_ptr_addr p
    store<LocalToken> r1 LocalToken @token_id
    let a <LocalToken> load<LocalToken> r1
    let b <LocalToken> load<LocalToken> r2
    0
"#;

    assert_compile_resource_source_reports_code(source, CompileTarget::Wasm, "resource.cell.moved");
}

#[test]
fn resource_ir_cell_check_preserves_mem_ptr_alias_after_region_token() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let raw <i32> mem_ptr_addr p
    let token <RegionToken<LocalToken>> region_new<LocalToken> raw size_of<LocalToken>
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn token_ptr <(RegionToken<LocalToken>)->MemPtr<LocalToken>> (token):
    region_ptr &token

fn main <()->i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let raw <i32> mem_ptr_addr p
    let token <RegionToken<LocalToken>> region_new<LocalToken> raw size_of<LocalToken>
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
fn resource_ir_cell_check_preserves_borrowed_region_ptr_at_known_offset_alias() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn main <()->i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let raw <i32> mem_ptr_addr p
    let token <RegionToken<LocalToken>> region_new<LocalToken> raw size_of<LocalToken>
    match region_ptr_at<LocalToken,LocalToken> &token 0:
        Result::Ok q:
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let a <LocalToken> load<LocalToken> mem_ptr_addr p
            let b <LocalToken> load<LocalToken> mem_ptr_addr q
            0
        Result::Err _e:
            0
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                state: CellState::Moved,
                ..
            }
        )),
        "borrowed region_ptr_at Ok payload must alias the token raw cell and report moved, not uninit: {:#?}\nresource:\n{}",
        report.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_rejects_borrowed_region_ptr_at_unknown_offset_dealloc_with_live_cell() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn choose_offset <(bool)->i32> (flag):
    if flag 0 4

fn main <()->i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    let raw <i32> mem_ptr_addr p
    let token <RegionToken<LocalToken>> region_new<LocalToken> raw size_of<LocalToken>
    let off <i32> choose_offset true
    match region_ptr_at<LocalToken,LocalToken> &token off:
        Result::Ok q:
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let r <Result<(),str>> dealloc_region<LocalToken> token
            0
        Result::Err _e:
            0
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    assert!(
        report.diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                operation: ResourceCheckOperation::RawMemoryDeallocCell,
                state: CellState::Initialized(_),
                ..
            }
        )),
        "borrowed region_ptr_at unknown-offset payload must retain initialized cell conflict on dealloc_region: {:#?}\nresource:\n{}",
        report.diagnostics,
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
        resource
            .dump_text()
            .contains("raw_address_view non_owning_projection"),
        "str_addr helper lowering must expose a non-owning raw address view:\n{}",
        resource.dump_text()
    );
}

#[test]
fn resource_ir_lowering_preserves_string_data_ptr_offset_from_str_addr_wrapper() {
    let source = r#"
#entry main
#indent 4
#target core
#import "alloc/string/storage" as *
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/raw" as *

fn str_data_addr_probe <(str)->i32> (s):
    mem_ptr_addr string_data_ptr s

fn main <()->i32> ():
    str_data_addr_probe "abc"
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let dump = resource.dump_text();
    assert!(
        dump.contains("raw_address_view non_owning_projection tmp1:t1[+4] -> tmp2:t1.field0@0"),
        "string_data_ptr should lower through string_addr to a known +4 raw view:\n{}",
        dump
    );
    let report = check_resource_initialized_moves(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "str_data_addr_probe" || function.starts_with("str_data_addr_probe__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "string_data_ptr raw address must keep initialized str storage evidence: {:#?}\nresource:\n{}",
        diagnostics,
        dump
    );
}

#[test]
fn resource_ir_lowering_coverage_accepts_transparent_addr_of_raw_address_helper() {
    let source = r#"
#entry main
#indent 4
#target core
#import "alloc/string/storage" as *
#import "core/mem" as *
#import "core/result" as *

fn main <()->i32> ():
    match string_alloc_region 3:
        Result::Ok region:
            let data <MemPtr<u8>> string_region_data_ptr &region
            0
        Result::Err _e:
            1
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let coverage = compare_hir_resource_lowering_typed(&module, &resource, &types);
    assert!(
        coverage.diagnostics.is_empty(),
        "transparent raw address helper called with &local must not add an untracked deref projection: {:#?}\nresource:\n{}",
        coverage.diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_lowering_preserves_transparent_region_ptr_wrapper() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/result" as *

fn region_data_ptr <(&RegionToken<u8>)->MemPtr<u8>> (region):
    region_ptr region

fn main <()->i32> ():
    match alloc_region_bytes<u8> 4:
        Result::Ok region:
            let data <MemPtr<u8>> region_data_ptr &region
            0
        Result::Err _e:
            1
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let dump = resource.dump_text();
    assert!(
        dump.contains("raw_address_view non_owning_projection"),
        "region_ptr wrapper return must lower to a non-owning raw view:\n{}",
        dump
    );
    let coverage = compare_hir_resource_lowering_typed(&module, &resource, &types);
    assert!(
        coverage.diagnostics.is_empty(),
        "transparent region_ptr wrapper called with &local must match HIR coverage: {:#?}\nresource:\n{}",
        coverage.diagnostics,
        dump
    );
}

#[test]
fn resource_ir_cell_check_preserves_mem_ptr_parameter_offset_raw_load() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/option" as *

fn byte_at <(MemPtr<u8>,i32)->i32> (data, idx):
    let ptr <MemPtr<u8>> mem_ptr_add data idx
    match load_u8 ptr:
        Option::Some b:
            b
        Option::None:
            0

fn main <()->i32> ():
    0
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let byte_at_diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            matches!(
                diagnostic,
                ResourceCheckDiagnostic::CellUnavailable { function, .. }
                    if function == "byte_at" || function.starts_with("byte_at__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        byte_at_diagnostics.is_empty(),
        "MemPtr parameter offset view must keep external initialized storage evidence through typed load wrappers: {:#?}\nresource:\n{}",
        byte_at_diagnostics,
        resource.dump_text()
    );
    assert!(
        resource
            .dump_text()
            .contains("raw_address_view mem_ptr_offset"),
        "mem_ptr_add parameter view must lower as an explicit MemPtr offset:\n{}",
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
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
fn resource_ir_cell_check_preserves_nested_copy_field_after_raw_aggregate_load() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct Span:
    file_id <i32>
    start <i32>

impl Clone for Span:
    fn clone <(&Span)->Span> (self):
        *self

impl Copy for Span:
    fn copy_mark <(Span)->Span> (self):
        self

struct Item:
    kind <i32>
    span <Span>

impl Clone for Item:
    fn clone <(&Item)->Item> (self):
        *self

impl Copy for Item:
    fn copy_mark <(Item)->Item> (self):
        self

fn main <()->i32> ():
    let p <i32> 16
    store<Item> p Item 7 Span 42 3
    let item <Item> load<Item> p
    let got <i32> item.span.file_id
    got
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
        "nested Copy field read from a raw-loaded aggregate must inherit initialized cell evidence: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );

    compile_resource_source_with_raw_boundary(source, CompileTarget::Wasm)
        .expect("nested Copy field read from raw-loaded aggregate should pass Resource IR checks");
}

#[test]
fn resource_ir_cell_check_preserves_external_raw_address_field_load() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

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
fn resource_ir_cell_check_moves_numeric_tuple_fields_independently() {
    let source = r#"
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as v
#import "core/field" as *
#import "core/math" as *
#import "core/result" as *

fn pair_with_empty <.T> <(Vec<.T>)->Result<.Pair, StdErrorKind>> (left):
    let right <Vec<.T>> uwok v::new<.T>;
    Result::Ok<.Pair, StdErrorKind> Tuple:
        left
        right

fn main <()*>i32> ():
    let xs <Vec<i32>>:
        v::new<i32>
        |> uwok
        |> v::push<i32> 1 |> uwok
    let parts unwrap_ok pair_with_empty<i32> xs;
    let evens <Vec<i32>> get parts 0;
    let rest <Vec<i32>> get parts 1;
    let n <i32> v::len<i32> &evens;
    v::free<i32> evens;
    v::free<i32> rest;
    if eq n 1 1 0
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
                ResourceCheckDiagnostic::CellUnavailable { function, .. } if function == "main"
            )
        })
        .collect::<Vec<_>>();
    assert!(
        main_diagnostics.is_empty(),
        "numeric tuple get must move only the selected field, not the whole tuple: {:#?}\nresource:\n{}",
        main_diagnostics,
        resource.dump_text()
    );

    let main = resource
        .functions
        .iter()
        .find(|function| function.name == "main" || function.name.starts_with("main__"))
        .expect("main function should lower to Resource IR");
    let main_ops = &main.blocks[main.entry_block.0].ops;
    for expected_index in [0usize, 1usize] {
        assert!(
            main_ops.iter().any(|op| matches!(
                op,
                ResourceOp::Read { source, .. }
                    if matches!(&source.root, PlaceRoot::Local(name) if name == "parts")
                        && source.projections.iter().any(|projection| matches!(
                            projection,
                            PlaceProjection::TupleField { index, .. } if *index == expected_index
                        ))
            )),
            "numeric tuple get must lower to an explicit field read for index {expected_index}:\n{}",
            resource.dump_text()
        );
    }

    compile_resource_source_with_target(source, CompileTarget::Wasm)
        .expect("numeric tuple field moves should pass the full Resource IR pipeline");
}

#[test]
fn resource_ir_cell_check_preserves_result_payload_raw_address_field() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
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
fn resource_ir_cell_check_preserves_custom_enum_payload_raw_address_field() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct Boxed:
    ptr <i32>

enum MaybeBox:
    Ready <Boxed>
    Empty

fn pass_custom <(Boxed)->MaybeBox> (box):
    MaybeBox::Ready box

fn read_after_custom <(Boxed)->i32> (box):
    match pass_custom box:
        MaybeBox::Ready ready:
            let ptr <i32> field::get ready "ptr"
            load_i32 ptr
        MaybeBox::Empty:
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
                    if function == "read_after_custom" || function.starts_with("read_after_custom__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "custom enum payload bind must preserve raw address field aliases through Resource IR value projection proof: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_cell_check_preserves_direct_result_payload_raw_address_alias() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn ok_ptr <(MemPtr<LocalToken>)->Result<MemPtr<LocalToken>,str>> (p):
    Result<MemPtr<LocalToken>,str>::Ok p

fn main <()*>i32> ():
    let p <MemPtr<LocalToken>> mem_ptr_wrap<LocalToken> 16
    match ok_ptr p:
        Result::Ok q:
            store<LocalToken> mem_ptr_addr p LocalToken @token_id
            let a <LocalToken> load<LocalToken> mem_ptr_addr p
            let b <LocalToken> load<LocalToken> mem_ptr_addr q
            0
        Result::Err _e:
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
                    if function == "main" || function.starts_with("main__")
            )
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.iter().any(|diagnostic| matches!(
            diagnostic,
            ResourceCheckDiagnostic::CellUnavailable {
                operation: ResourceCheckOperation::RawMemoryLoadCell,
                state: CellState::Moved,
                ..
            }
        )),
        "direct Result::Ok match payload must alias the returned MemPtr raw cell and report moved, not uninit: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_field_get_ref_deref_uses_borrowed_field_cell() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/field" as field

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

struct Pair:
    token <LocalToken>
    count <i32>

fn main <()->i32> ():
    let p <Pair> Pair (LocalToken @token_id) 7
    let count <i32> *field::get_ref &p "count"
    count
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| match diagnostic {
            ResourceCheckDiagnostic::CellUnavailable { function, .. } => {
                function.starts_with("main__")
            }
            ResourceCheckDiagnostic::CollectionSlotRefuted { .. } => false,
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "field::get_ref deref should read the borrowed field cell without uninit diagnostics: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_match_never_arm_does_not_poison_initialized_state() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/result" as *

fn main <()->i32> ():
    let mut acc <i32> 0
    let r <Result<i32,str>> Result::Ok 7
    match r:
        Result::Ok v:
            set acc v
            acc
        Result::Err _:
            #intrinsic "unreachable" <> ()
"#;

    let (module, types) = typecheck_resource_source(source);
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_initialized_moves(&resource, &types);
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| match diagnostic {
            ResourceCheckDiagnostic::CellUnavailable { function, .. } => {
                function.starts_with("main__")
            }
            ResourceCheckDiagnostic::CollectionSlotRefuted { .. } => false,
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "never-valued match arms must not participate in initialized-state merge: {:#?}\nresource:\n{}",
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

fn token_id <(i32)->i32> (x):
    x

fn slot_ptr <.T,.V> <(i32,i32)->i32> (base, idx):
    add base mul idx add size_of<.T> size_of<.V>

fn main <()->i32> ():
    let p <i32> alloc_raw 16
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
fn resource_ir_cell_check_preserves_untracked_literal_helper_zero_offset_for_first_store() {
    let source = r#"
#entry main
#indent 4
#target core
#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
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
        "literal zero helper view must retain its stable raw address origin until first raw store proves the cell: {:#?}\nresource:\n{}",
        main_diagnostics,
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
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
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

#[test]
fn resource_ir_collection_slot_rejects_double_move() {
    let (types, owned_ty) = types_with_non_copy_owned();
    let span = Span::dummy();
    let storage = Place::local("buffer".to_string(), owned_ty);
    let slot = storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
        owned_ty,
    );
    let resource = manual_resource_module(
        types.unit(),
        span,
        vec![
            ResourceOp::CollectionSlotLifecycle {
                target: slot.clone(),
                event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned_ty },
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: slot.clone(),
                event: CollectionSlotLifecycleEvent::MoveOut {
                    expected_ty: owned_ty,
                },
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: slot.clone(),
                event: CollectionSlotLifecycleEvent::MoveOut {
                    expected_ty: owned_ty,
                },
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);

    assert_eq!(
        report.diagnostics,
        vec![ResourceCheckDiagnostic::CollectionSlotRefuted {
            function: "main".to_string(),
            target: slot,
            reason: CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::MoveOut,
                state: CollectionSlotState::Moved(owned_ty),
            },
            span,
        }]
    );
}

#[test]
fn resource_ir_collection_slot_merges_branch_liveness_before_storage_dealloc() {
    let (types, owned_ty) = types_with_non_copy_owned();
    let i32_ty = types.i32();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let storage = Place::local("buffer".to_string(), owned_ty);
    let slot = storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
        owned_ty,
    );
    let condition = Place::temporary(ResourceId(600), i32_ty);
    let output = Place::temporary(ResourceId(601), unit_ty);
    let then_value = Place::temporary(ResourceId(602), unit_ty);
    let else_value = Place::temporary(ResourceId(603), unit_ty);
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(1),
                output: condition.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Branch {
                output,
                condition,
                condition_fact: None,
                then_ops: vec![
                    ResourceOp::CollectionSlotLifecycle {
                        target: slot.clone(),
                        event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned_ty },
                        span,
                    },
                    ResourceOp::CollectionSlotLifecycle {
                        target: slot.clone(),
                        event: CollectionSlotLifecycleEvent::MoveOut {
                            expected_ty: owned_ty,
                        },
                        span,
                    },
                    ResourceOp::Expr {
                        kind: ResourceExprKind::Literal,
                        output: then_value.clone(),
                        ty: unit_ty,
                        span,
                    },
                ],
                then_value,
                else_ops: vec![
                    ResourceOp::CollectionSlotLifecycle {
                        target: slot.clone(),
                        event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned_ty },
                        span,
                    },
                    ResourceOp::Expr {
                        kind: ResourceExprKind::Literal,
                        output: else_value.clone(),
                        ty: unit_ty,
                        span,
                    },
                ],
                else_value,
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: storage,
                event: CollectionSlotLifecycleEvent::StorageDealloc,
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);

    assert!(matches!(
        report.diagnostics.as_slice(),
        [ResourceCheckDiagnostic::CollectionSlotRefuted {
            target,
            reason:
                CollectionSlotLifecycleRefutation::MaybeLiveSlotDuringStorageDealloc {
                    slot_ty: Some(found_ty),
                },
            ..
        }] if *target == slot && *found_ty == owned_ty
    ));
    assert!(report.functions[0]
        .final_collection_slots
        .iter()
        .any(|entry| entry.slot == slot
            && entry.state == CollectionSlotState::MaybeInitialized(Some(owned_ty))));
}

#[test]
fn resource_ir_collection_slot_call_summary_moves_slot_before_storage_dealloc() {
    let (types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let storage = Place::local("buffer".to_string(), owned_ty);
    let resource = collection_slot_summary_resource(
        unit_ty,
        owned_ty,
        span,
        "drain_slot",
        vec![
            ResourceOp::CollectionSlotLifecycle {
                target: slot_for_param(owned_ty),
                event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned_ty },
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: slot_for_param(owned_ty),
                event: CollectionSlotLifecycleEvent::MoveOut {
                    expected_ty: owned_ty,
                },
                span,
            },
        ],
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: storage.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::Call {
                output: Place::temporary(ResourceId(700), unit_ty),
                target: ResourceCallTarget::User {
                    name: "drain_slot".to_string(),
                    type_args: vec![],
                },
                args: vec![storage.clone()],
                effect: EffectOp::Pure,
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: storage,
                event: CollectionSlotLifecycleEvent::StorageDealloc,
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);

    assert!(
        report.diagnostics.is_empty(),
        "callee summary should replay initialize+move before caller storage dealloc: {:#?}",
        report.diagnostics
    );
}

#[test]
fn resource_ir_collection_storage_relocate_transfers_live_slot_to_new_storage() {
    let (types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let old_storage = Place::local("old_buffer".to_string(), owned_ty);
    let new_storage = Place::local("new_buffer".to_string(), owned_ty);
    let old_slot = old_storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
        owned_ty,
    );
    let new_slot = new_storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
        owned_ty,
    );
    let resource = manual_resource_module(
        unit_ty,
        span,
        vec![
            ResourceOp::CollectionSlotLifecycle {
                target: old_slot.clone(),
                event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned_ty },
                span,
            },
            ResourceOp::CollectionStorageRelocate {
                old_storage: old_storage.clone(),
                new_storage: new_storage.clone(),
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: old_storage,
                event: CollectionSlotLifecycleEvent::StorageDealloc,
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: new_storage,
                event: CollectionSlotLifecycleEvent::StorageDealloc,
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);

    assert_eq!(
        report.diagnostics,
        vec![ResourceCheckDiagnostic::CollectionSlotRefuted {
            function: "main".to_string(),
            target: new_slot.clone(),
            reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc {
                slot_ty: owned_ty,
            },
            span,
        }]
    );
    assert!(report.functions[0]
        .final_collection_slots
        .iter()
        .any(|entry| entry.slot == new_slot
            && entry.state == CollectionSlotState::Initialized(owned_ty)));
}

#[test]
fn resource_ir_collection_storage_relocate_call_summary_transfers_slot() {
    let (types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let old_storage = Place::local("old_buffer".to_string(), owned_ty);
    let new_storage = Place::local("new_buffer".to_string(), owned_ty);
    let old_slot = old_storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
        owned_ty,
    );
    let new_slot = new_storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
        owned_ty,
    );
    let resource = collection_slot_relocate_summary_resource(
        unit_ty,
        owned_ty,
        span,
        vec![ResourceOp::CollectionStorageRelocate {
            old_storage: Place::local("old_storage".to_string(), owned_ty),
            new_storage: Place::local("new_storage".to_string(), owned_ty),
            span,
        }],
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: old_storage.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: new_storage.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: old_slot,
                event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned_ty },
                span,
            },
            ResourceOp::Call {
                output: Place::temporary(ResourceId(709), unit_ty),
                target: ResourceCallTarget::User {
                    name: "relocate_storage".to_string(),
                    type_args: vec![],
                },
                args: vec![old_storage.clone(), new_storage.clone()],
                effect: EffectOp::Pure,
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: new_storage,
                event: CollectionSlotLifecycleEvent::StorageDealloc,
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);

    assert_eq!(
        report.diagnostics,
        vec![ResourceCheckDiagnostic::CollectionSlotRefuted {
            function: "main".to_string(),
            target: new_slot,
            reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc {
                slot_ty: owned_ty,
            },
            span,
        }]
    );
}

#[test]
fn resource_ir_collection_slot_call_summary_rejects_live_slot_during_storage_dealloc() {
    let (types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let storage = Place::local("buffer".to_string(), owned_ty);
    let slot = storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
        owned_ty,
    );
    let resource = collection_slot_summary_resource(
        unit_ty,
        owned_ty,
        span,
        "init_slot",
        vec![ResourceOp::CollectionSlotLifecycle {
            target: slot_for_param(owned_ty),
            event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned_ty },
            span,
        }],
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: storage.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::Call {
                output: Place::temporary(ResourceId(710), unit_ty),
                target: ResourceCallTarget::User {
                    name: "init_slot".to_string(),
                    type_args: vec![],
                },
                args: vec![storage.clone()],
                effect: EffectOp::Pure,
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: storage,
                event: CollectionSlotLifecycleEvent::StorageDealloc,
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);

    assert_eq!(
        report.diagnostics,
        vec![ResourceCheckDiagnostic::CollectionSlotRefuted {
            function: "main".to_string(),
            target: slot,
            reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc {
                slot_ty: owned_ty,
            },
            span,
        }]
    );
}

#[test]
fn resource_ir_collection_slot_call_summary_merges_callee_branch_effects() {
    let (types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let storage = Place::local("buffer".to_string(), owned_ty);
    let slot = storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
        owned_ty,
    );
    let condition = Place::temporary(ResourceId(715), i32_ty);
    let output = Place::temporary(ResourceId(716), unit_ty);
    let then_value = Place::temporary(ResourceId(717), unit_ty);
    let else_value = Place::temporary(ResourceId(718), unit_ty);
    let resource = collection_slot_summary_resource(
        unit_ty,
        owned_ty,
        span,
        "maybe_init_slot",
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::LiteralI32(1),
                output: condition.clone(),
                ty: i32_ty,
                span,
            },
            ResourceOp::Branch {
                output,
                condition,
                condition_fact: None,
                then_ops: vec![
                    ResourceOp::CollectionSlotLifecycle {
                        target: slot_for_param(owned_ty),
                        event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned_ty },
                        span,
                    },
                    ResourceOp::Expr {
                        kind: ResourceExprKind::Literal,
                        output: then_value.clone(),
                        ty: unit_ty,
                        span,
                    },
                ],
                then_value,
                else_ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::Literal,
                    output: else_value.clone(),
                    ty: unit_ty,
                    span,
                }],
                else_value,
                span,
            },
        ],
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: storage.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::Call {
                output: Place::temporary(ResourceId(719), unit_ty),
                target: ResourceCallTarget::User {
                    name: "maybe_init_slot".to_string(),
                    type_args: vec![],
                },
                args: vec![storage.clone()],
                effect: EffectOp::Pure,
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: storage,
                event: CollectionSlotLifecycleEvent::StorageDealloc,
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);

    assert!(matches!(
        report.diagnostics.as_slice(),
        [ResourceCheckDiagnostic::CollectionSlotRefuted {
            target,
            reason:
                CollectionSlotLifecycleRefutation::MaybeLiveSlotDuringStorageDealloc {
                    slot_ty: Some(found_ty),
                },
            ..
        }] if *target == slot && *found_ty == owned_ty
    ));
}

#[test]
fn resource_ir_collection_slot_indirect_call_summary_applies_function_alias() {
    let (mut types, owned_ty) = types_with_non_copy_owned();
    let unit_ty = types.unit();
    let function_ty = types.function(vec![], vec![owned_ty], unit_ty, Effect::Pure);
    let span = Span::dummy();
    let storage = Place::local("buffer".to_string(), owned_ty);
    let callee = Place::temporary(ResourceId(720), function_ty);
    let resource = collection_slot_summary_resource(
        unit_ty,
        owned_ty,
        span,
        "drain_slot",
        vec![
            ResourceOp::CollectionSlotLifecycle {
                target: slot_for_param(owned_ty),
                event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned_ty },
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: slot_for_param(owned_ty),
                event: CollectionSlotLifecycleEvent::MoveOut {
                    expected_ty: owned_ty,
                },
                span,
            },
        ],
        vec![
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: storage.clone(),
                ty: owned_ty,
                span,
            },
            ResourceOp::FunctionValue {
                output: callee.clone(),
                name: "drain_slot".to_string(),
                effect: EffectOp::Pure,
                span,
            },
            ResourceOp::IndirectCall {
                output: Place::temporary(ResourceId(721), unit_ty),
                callee,
                params: vec![owned_ty],
                result: unit_ty,
                args: vec![storage.clone()],
                effect: EffectOp::Pure,
                span,
            },
            ResourceOp::CollectionSlotLifecycle {
                target: storage,
                event: CollectionSlotLifecycleEvent::StorageDealloc,
                span,
            },
        ],
    );

    let report = check_resource_initialized_moves(&resource, &types);

    assert!(
        report.diagnostics.is_empty(),
        "function alias indirect call should apply collection slot summary: {:#?}",
        report.diagnostics
    );
}

fn types_with_non_copy_owned() -> (TypeCtx, TypeId) {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    let owned_ty = types.register_named(
        "Owned".to_string(),
        TypeKind::Struct {
            name: "Owned".to_string(),
            type_params: vec![],
            fields: vec![],
            field_names: vec![],
        },
    );
    (types, owned_ty)
}

fn slot_for_param(owned_ty: TypeId) -> Place {
    Place::local("slot_storage".to_string(), owned_ty).with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
        owned_ty,
    )
}

fn collection_slot_summary_resource(
    unit_ty: TypeId,
    owned_ty: TypeId,
    span: Span,
    helper_name: &str,
    helper_ops: Vec<ResourceOp>,
    main_ops: Vec<ResourceOp>,
) -> ResourceModule {
    let param_place = Place::local("slot_storage".to_string(), owned_ty);
    ResourceModule {
        functions: vec![
            ResourceFunction {
                name: helper_name.to_string(),
                origin_name: helper_name.to_string(),
                type_params: Vec::new(),
                params: vec![ResourceLocal {
                    name: "slot_storage".to_string(),
                    ty: owned_ty,
                    mutable: true,
                    place: param_place,
                }],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: helper_ops,
                    terminator: ResourceTerminator::Return { value: None, span },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                origin_name: "main".to_string(),
                type_params: Vec::new(),
                params: vec![],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: main_ops,
                    terminator: ResourceTerminator::Return { value: None, span },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    }
}

fn collection_slot_relocate_summary_resource(
    unit_ty: TypeId,
    owned_ty: TypeId,
    span: Span,
    helper_ops: Vec<ResourceOp>,
    main_ops: Vec<ResourceOp>,
) -> ResourceModule {
    ResourceModule {
        functions: vec![
            ResourceFunction {
                name: "relocate_storage".to_string(),
                origin_name: "relocate_storage".to_string(),
                type_params: Vec::new(),
                params: vec![
                    ResourceLocal {
                        name: "old_storage".to_string(),
                        ty: owned_ty,
                        mutable: true,
                        place: Place::local("old_storage".to_string(), owned_ty),
                    },
                    ResourceLocal {
                        name: "new_storage".to_string(),
                        ty: owned_ty,
                        mutable: true,
                        place: Place::local("new_storage".to_string(), owned_ty),
                    },
                ],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: helper_ops,
                    terminator: ResourceTerminator::Return { value: None, span },
                    span,
                }],
                span,
            },
            ResourceFunction {
                name: "main".to_string(),
                origin_name: "main".to_string(),
                type_params: Vec::new(),
                params: vec![],
                result: unit_ty,
                effect: Effect::Pure,
                entry_block: ResourceBlockId(0),
                blocks: vec![ResourceBlock {
                    id: ResourceBlockId(0),
                    ops: main_ops,
                    terminator: ResourceTerminator::Return { value: None, span },
                    span,
                }],
                span,
            },
        ],
        entry: Some("main".to_string()),
        string_literals: vec![],
    }
}

fn manual_resource_module(unit_ty: TypeId, span: Span, ops: Vec<ResourceOp>) -> ResourceModule {
    manual_resource_module_with_effect(Effect::Pure, unit_ty, span, ops)
}

fn manual_resource_module_with_effect(
    effect: Effect,
    unit_ty: TypeId,
    span: Span,
    ops: Vec<ResourceOp>,
) -> ResourceModule {
    ResourceModule {
        functions: vec![ResourceFunction {
            name: "main".to_string(),
            origin_name: "main".to_string(),
            type_params: Vec::new(),
            params: vec![],
            result: unit_ty,
            effect,
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

fn resource_function<'a>(resource: &'a ResourceModule, name: &str) -> &'a ResourceFunction {
    resource
        .functions
        .iter()
        .find(|function| function.name == name)
        .expect("resource function should exist")
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
            origin_name: "main".to_string(),
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
            origin_name: "main".to_string(),
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

#[test]
fn resource_ir_owner_check_accepts_stdio_fd_write_scratch_cleanup() {
    let source = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *
#import "std/stdio/write" as *

fn main <()*>()> ():
    match stdio_write_fd_byte_result 1 82:
        Result::Ok _:
            ()
        Result::Err _:
            ()
"#;

    let (module, mut types) = typecheck_resource_source_with_target(source, CompileTarget::Wasi);
    let mono = nepl_core::monomorphize::monomorphize(&mut types, module);
    let (module, unresolved_trait_calls) = mono.into_parts();
    assert!(
        unresolved_trait_calls.is_empty(),
        "unresolved trait calls: {:#?}",
        unresolved_trait_calls
    );
    let resource = lower_hir_module(&module, &types);
    let resource_dump = resource.dump_text();
    let stdio_dump = resource_dump
        .split("\nfn ")
        .filter(|section| {
            section.starts_with("stdio_write_fd_mem_result__")
                || section.starts_with("stdio_fd_write_from_result__")
        })
        .collect::<Vec<_>>()
        .join("\nfn ");
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
            function.starts_with("stdio_write_fd_mem_result__")
                || function.starts_with("stdio_fd_write_from_result__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "stdio fd_write scratch owners must be released on all paths: {:#?}\nresource:\nfn {}",
        diagnostics,
        stdio_dump
    );
}

#[test]
fn resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup() {
    let source = r#"
#entry main
#indent 4
#target wasi

#import "core/mem" as *
#import "core/mem/internal" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *
#import "std/fs/fd" as *
#import "std/fs/read" as *
#import "std/stdio/read" as *
#import "std/stdio/write" as *

fn main <()*>()> ():
    match fs_open_read "tests/fixtures/fs/read_sample.txt":
        Result::Ok _fd:
            ()
        Result::Err _e:
            ()
    match fs_read_fd_bytes 0:
        Result::Ok _bytes:
            ()
        Result::Err _e:
            ()
    match stdio_read_all_bytes_result:
        Result::Ok _bytes:
            ()
        Result::Err _e:
            ()
    match stdio_write_fd_byte_result 1 82:
        Result::Ok _:
            ()
        Result::Err _:
            ()
"#;

    let (module, mut types) = typecheck_resource_source_with_target(source, CompileTarget::Wasi);
    let mono = nepl_core::monomorphize::monomorphize(&mut types, module);
    let (module, unresolved_trait_calls) = mono.into_parts();
    assert!(
        unresolved_trait_calls.is_empty(),
        "unresolved trait calls: {:#?}",
        unresolved_trait_calls
    );
    let resource = lower_hir_module(&module, &types);
    let report = check_resource_owner_obligations(&resource, &types);
    let prefixes = [
        "fs_open_with_flags__",
        "fs_read_fd_bytes__",
        "stdio_read_all_bytes_result__",
        "stdio_write_fd_byte_result__",
        "stdio_write_fd_mem_result__",
    ];
    let diagnostics = report
        .diagnostics
        .iter()
        .filter(|diagnostic| {
            let function = match diagnostic {
                ResourceOwnerDiagnostic::OwnerUnavailable { function, .. }
                | ResourceOwnerDiagnostic::OwnerLeaked { function, .. }
                | ResourceOwnerDiagnostic::OwnerMaybeLeaked { function, .. } => function,
            };
            prefixes.iter().any(|prefix| function.starts_with(prefix))
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "fs/stdio scratch owners must be released on all paths: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
}

#[test]
fn resource_ir_compiler_accepts_stdio_string_temporaries() {
    let source = r#"
#entry main
#indent 4
#target std
#import "std/stdio" as *
#import "alloc/string/integer/format" as string_integer

fn main <()*>()> ():
    print_i32 12;
    let text <str> string_integer::from_i32 34;
    print text;
    print text;
    let style <AnsiTextStyle> ansi_bold_color_style AnsiColor::Green;
    print ansi_text_style_code style;
    ()
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasi)
        .expect("stdio string temporaries must compile under Resource IR owner gate");
}

#[test]
fn resource_ir_compiler_accepts_vec_get_copy_str_option_return() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as v
#import "alloc/string" as *
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *

struct Parsed:
    input <Option<str>>

fn parse <(&Vec<str>)->Parsed> (args):
    Parsed v::get<str> args 0

fn main <()* >i32> ():
    let args <Vec<str>>:
        unwrap_ok v::new<str>
        |> v::push<str> "alpha" |> uwok
    let parsed <Parsed> parse &args
    let input_ref <&Option<str>> field::get_ref &parsed "input"
    let ok <bool> match *input_ref:
        Option::Some text:
            str_eq text "alpha"
        Option::None:
            false
    v::free<str> args
    if ok 0 1
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasi)
        .expect("Vec.get<str> must copy the element value without moving the Vec storage owner");
}

#[test]
fn resource_ir_owner_variant_reservation_ignores_copy_payload_sources() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/math" as *
#import "core/result" as *
#import "core/traits/copy" as *

struct SpanLike:
    file_id <i32>
    start <i32>
    end <i32>

impl Clone for SpanLike:
    fn clone <(&SpanLike)->SpanLike> (self):
        *self

impl Copy for SpanLike:
    fn copy_mark <(SpanLike)->SpanLike> (self):
        self

fn maybe_span <(SpanLike,bool)*>Result<i32,SpanLike>> (span, ok):
    if:
        ok
        then:
            Result<i32,SpanLike>::Ok 1
        else:
            Result<i32,SpanLike>::Err span

fn maybe_text <(str,bool)*>Result<i32,str>> (text, ok):
    if:
        ok
        then:
            Result<i32,str>::Ok 1
        else:
            Result<i32,str>::Err text

fn main <()*>i32> ():
    let span <SpanLike> SpanLike 0 10 14
    let text <str> "alpha"
    let span_ok <bool> match maybe_span span true:
        Result::Ok _:
            eq span.start 10
        Result::Err _:
            false
    let text_ok <bool> match maybe_text text true:
        Result::Ok _:
            str_eq text "alpha"
        Result::Err _:
            false
    if and span_ok text_ok 0 1
"#;

    compile_resource_source_with_target(source, CompileTarget::Wasi).expect(
        "Copy payload sources inside unresolved Result variants must not be owner-reserved",
    );
}

#[test]
fn resource_ir_owner_summary_ignores_copy_diagnostic_label_i32_payloads() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/field" as field
#import "core/math" as *
#import "core/option" as *
#import "core/result" as *
#import "core/traits/copy" as *

struct SpanLike:
    file_id <i32>
    start <i32>
    end <i32>

impl Clone for SpanLike:
    fn clone <(&SpanLike)->SpanLike> (self):
        *self

impl Copy for SpanLike:
    fn copy_mark <(SpanLike)->SpanLike> (self):
        self

struct DiagnosticLabel:
    span <SpanLike>
    message <str>

impl Clone for DiagnosticLabel:
    fn clone <(&DiagnosticLabel)->DiagnosticLabel> (self):
        *self

impl Copy for DiagnosticLabel:
    fn copy_mark <(DiagnosticLabel)->DiagnosticLabel> (self):
        self

struct Diagnostic:
    code <i32>
    primary <Option<DiagnosticLabel>>
    message <str>

impl Clone for Diagnostic:
    fn clone <(&Diagnostic)->Diagnostic> (self):
        *self

impl Copy for Diagnostic:
    fn copy_mark <(Diagnostic)->Diagnostic> (self):
        self

fn label_new <(SpanLike,str)->DiagnosticLabel> (span, message):
    DiagnosticLabel span message

fn diagnostic_with_primary <(i32,SpanLike,str)->Diagnostic> (code, span, message):
    Diagnostic code some<DiagnosticLabel> label_new span "label" message

fn fail_with_label <(SpanLike)->Result<i32,Diagnostic>> (span):
    Result<i32,Diagnostic>::Err diagnostic_with_primary 7 span "failed"

fn main <()*>i32> ():
    let span <SpanLike> SpanLike 0 10 14
    match fail_with_label span:
        Result::Ok _value:
            1
        Result::Err diag:
            let code_ok <bool> eq field::get diag "code" 7
            let label_ok <bool> match field::get diag "primary":
                Option::Some label:
                    str_eq field::get label "message" "label"
                Option::None:
                    false
            if and code_ok label_ok 0 1
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
            function.starts_with("label_new__")
                || function.starts_with("diagnostic_with_primary__")
                || function.starts_with("fail_with_label__")
                || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "Copy diagnostic label span fields must remain metadata, not raw owner obligations: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
    compile_resource_source_with_target(source, CompileTarget::Wasi)
        .expect("nested Copy diagnostic labels must compile without resource.owner.maybe_leak");
}

#[test]
fn resource_ir_owner_summary_keeps_copy_str_views_after_selfhost_path_resolution() {
    let source = r#"
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/math" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/module/stdlib_map" as *

fn main <()*>i32> ():
    let map <SelfhostModulePathMap> selfhost_module_path_map_new "user" "stdlib"
    let span <SelfhostSourceSpan> source_span_empty 0 0
    let current_path <str> "user/app/main.nepl"
    let resolved_ok <bool> match selfhost_module_path_resolve_import &map current_path span "./util":
        Result::Ok resolved:
            str_eq resolved.path "user/app/util.nepl"
        Result::Err _diag:
            false
    let current_still_available <bool> str_eq current_path "user/app/main.nepl"
    if and resolved_ok current_still_available 0 1
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
            function.starts_with("selfhost_module_path_") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "selfhost path resolution must not consume Copy str view arguments in owner summaries: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
    compile_resource_source_with_target(source, CompileTarget::Wasi).expect(
        "selfhost path resolution must not consume Copy str view arguments in owner summaries",
    );
}

#[test]
fn resource_ir_owner_summary_returns_branch_report_with_copy_str_payloads() {
    let source = r#"
#entry main
#indent 4
#target std
#import "core/result" as *
#import "std/test" as *

fn push_err <(TestReport,bool)*>TestReport> (checks, ok):
    if:
        ok
        then:
            checks_push checks check true
        else:
            checks_push checks Result<(),str>::Err "bad"

fn main <()*>i32> ():
    let checks <TestReport> checks_new
    let shown <TestReport> checks_print_report push_err checks false
    checks_exit_code shown
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
            function.starts_with("push_err__") || function.starts_with("main__")
        })
        .collect::<Vec<_>>();
    assert!(
        diagnostics.is_empty(),
        "branching std/test report flow must return active report owners and ignore inactive Copy str payload reservations: {:#?}\nresource:\n{}",
        diagnostics,
        resource.dump_text()
    );
    compile_resource_source_with_target(source, CompileTarget::Wasi).expect(
        "branching std/test report flow must return active report owners and ignore inactive Copy str payload reservations",
    );
}
