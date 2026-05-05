use nepl_core::ast::Effect;
use nepl_core::codegen_wasm;
use nepl_core::diagnostic_codes::{
    BackendDiagnosticCode, DiagnosticCode, TypeDiagnosticCode, WasmDiagnosticCode,
};
use nepl_core::hir::{
    HirBlock, HirBody, HirExpr, HirExprKind, HirExtern, HirFunction, HirLine, HirModule, HirParam,
};
use nepl_core::passes::codegen_precheck::{precheck_llvm_codegen, precheck_wasm_codegen};
use nepl_core::span::Span;
use nepl_core::types::{TypeCtx, TypeId};
use std::collections::BTreeSet;

fn empty_module(functions: Vec<HirFunction>, entry: Option<&str>) -> HirModule {
    HirModule {
        functions,
        entry: entry.map(str::to_string),
        externs: Vec::new(),
        string_literals: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
    }
}

fn zero_arg_function(ctx: &mut TypeCtx, name: &str, result: TypeId, expr: HirExpr) -> HirFunction {
    let func_ty = ctx.function(Vec::new(), Vec::new(), result, Effect::Pure);
    HirFunction {
        doc: None,
        name: name.to_string(),
        func_ty,
        params: Vec::new(),
        result,
        effect: Effect::Pure,
        body: HirBody::Block(HirBlock {
            lines: vec![HirLine {
                expr,
                drop_result: false,
            }],
            ty: result,
            span: Span::dummy(),
        }),
        span: Span::dummy(),
    }
}

fn one_arg_function(
    ctx: &mut TypeCtx,
    name: &str,
    param_name: &str,
    param_ty: TypeId,
    result: TypeId,
    expr: HirExpr,
) -> HirFunction {
    let func_ty = ctx.function(Vec::new(), vec![param_ty], result, Effect::Pure);
    HirFunction {
        doc: None,
        name: name.to_string(),
        func_ty,
        params: vec![HirParam {
            name: param_name.to_string(),
            ty: param_ty,
            mutable: false,
        }],
        result,
        effect: Effect::Pure,
        body: HirBody::Block(HirBlock {
            lines: vec![HirLine {
                expr,
                drop_result: false,
            }],
            ty: result,
            span: Span::dummy(),
        }),
        span: Span::dummy(),
    }
}

fn first_error_code(
    err: Result<codegen_wasm::CodegenResult, Vec<nepl_core::diagnostic::Diagnostic>>,
) -> DiagnosticCode {
    let diagnostics = err.expect_err("codegen should return diagnostics");
    diagnostics
        .first()
        .and_then(|diag| diag.code)
        .expect("diagnostic code should be attached")
}

fn first_diagnostic_code(diagnostics: Vec<nepl_core::diagnostic::Diagnostic>) -> DiagnosticCode {
    diagnostics
        .first()
        .and_then(|diag| diag.code)
        .expect("diagnostic code should be attached")
}

fn wasm_code(code: WasmDiagnosticCode) -> DiagnosticCode {
    DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(code))
}

#[test]
fn wasm_codegen_reports_unsupported_function_signature_without_panicking() {
    let mut ctx = TypeCtx::new();
    let never_ty = ctx.never();
    let unit_ty = ctx.unit();
    let function = zero_arg_function(
        &mut ctx,
        "main",
        never_ty,
        HirExpr {
            ty: unit_ty,
            kind: HirExprKind::Unit,
            span: Span::dummy(),
        },
    );

    let module = empty_module(vec![function], Some("main"));

    assert_eq!(
        first_error_code(codegen_wasm::generate_wasm(&ctx, &module)),
        wasm_code(WasmDiagnosticCode::FunctionSignatureUnsupported)
    );
}

#[test]
fn wasm_codegen_ignores_entry_unreachable_bad_function() {
    let mut ctx = TypeCtx::new();
    let i32_ty = ctx.i32();
    let never_ty = ctx.never();
    let unit_ty = ctx.unit();
    let main = zero_arg_function(
        &mut ctx,
        "main",
        i32_ty,
        HirExpr {
            ty: i32_ty,
            kind: HirExprKind::LiteralI32(0),
            span: Span::dummy(),
        },
    );
    let unreachable_bad = zero_arg_function(
        &mut ctx,
        "unreachable_bad",
        never_ty,
        HirExpr {
            ty: unit_ty,
            kind: HirExprKind::Unit,
            span: Span::dummy(),
        },
    );
    let module = empty_module(vec![main, unreachable_bad], Some("main"));

    assert!(precheck_wasm_codegen(&ctx, &module).is_empty());
    let generated = codegen_wasm::generate_wasm(&ctx, &module)
        .expect("wasm codegen should ignore entry-unreachable functions");
    let bytes = generated.bytes.expect("wasm bytes should be emitted");
    assert!(!bytes.is_empty());
}

#[test]
fn wasm_codegen_reports_unknown_variable_without_panicking() {
    let mut ctx = TypeCtx::new();
    let i32_ty = ctx.i32();
    let function = zero_arg_function(
        &mut ctx,
        "main",
        i32_ty,
        HirExpr {
            ty: i32_ty,
            kind: HirExprKind::Var("missing".to_string()),
            span: Span::dummy(),
        },
    );

    let module = empty_module(vec![function], Some("main"));

    assert_eq!(
        first_error_code(codegen_wasm::generate_wasm(&ctx, &module)),
        wasm_code(WasmDiagnosticCode::VariableUnknown)
    );
}

#[test]
fn wasm_codegen_reports_missing_string_literal_without_panicking() {
    let mut ctx = TypeCtx::new();
    let str_ty = ctx.str();
    let function = zero_arg_function(
        &mut ctx,
        "main",
        str_ty,
        HirExpr {
            ty: str_ty,
            kind: HirExprKind::LiteralStr(0),
            span: Span::dummy(),
        },
    );

    let module = empty_module(vec![function], Some("main"));

    assert_eq!(
        first_error_code(codegen_wasm::generate_wasm(&ctx, &module)),
        wasm_code(WasmDiagnosticCode::StringLiteralNotFound)
    );
}

#[test]
fn wasm_precheck_reports_unsupported_extern_signature_code() {
    let ctx = TypeCtx::new();
    let never_ty = ctx.never();
    let module = HirModule {
        functions: Vec::new(),
        entry: None,
        externs: vec![HirExtern {
            module: "env".to_string(),
            name: "host".to_string(),
            local_name: "host".to_string(),
            params: Vec::new(),
            result: never_ty,
            effect: Effect::Impure,
            span: Span::dummy(),
        }],
        string_literals: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
    };

    assert_eq!(
        first_diagnostic_code(precheck_wasm_codegen(&ctx, &module)),
        wasm_code(WasmDiagnosticCode::ExternSignatureUnsupported)
    );
}

#[test]
fn wasm_precheck_reports_missing_return_value_code() {
    let mut ctx = TypeCtx::new();
    let i32_ty = ctx.i32();
    let unit_ty = ctx.unit();
    let function = zero_arg_function(
        &mut ctx,
        "main",
        i32_ty,
        HirExpr {
            ty: unit_ty,
            kind: HirExprKind::Unit,
            span: Span::dummy(),
        },
    );
    let module = empty_module(vec![function], Some("main"));

    assert_eq!(
        first_diagnostic_code(precheck_wasm_codegen(&ctx, &module)),
        wasm_code(WasmDiagnosticCode::ReturnValueMissing)
    );
}

#[test]
fn wasm_precheck_reports_indirect_signature_unsupported_code() {
    let mut ctx = TypeCtx::new();
    let i32_ty = ctx.i32();
    let never_ty = ctx.never();
    let unit_ty = ctx.unit();
    let callee_ty = ctx.function(vec![i32_ty], Vec::new(), never_ty, Effect::Pure);
    let function = zero_arg_function(
        &mut ctx,
        "main",
        unit_ty,
        HirExpr {
            ty: never_ty,
            kind: HirExprKind::CallIndirect {
                callee: Box::new(HirExpr {
                    ty: callee_ty,
                    kind: HirExprKind::FnValue("callee".to_string()),
                    span: Span::dummy(),
                }),
                params: vec![i32_ty],
                result: never_ty,
                effect: Effect::Pure,
                args: vec![HirExpr {
                    ty: i32_ty,
                    kind: HirExprKind::LiteralI32(1),
                    span: Span::dummy(),
                }],
            },
            span: Span::dummy(),
        },
    );
    let module = empty_module(vec![function], Some("main"));

    assert_eq!(
        first_diagnostic_code(precheck_wasm_codegen(&ctx, &module)),
        wasm_code(WasmDiagnosticCode::IndirectSignatureUnsupported)
    );
}

#[test]
fn wasm_precheck_reports_indirect_signature_missing_code() {
    let mut ctx = TypeCtx::new();
    let i32_ty = ctx.i32();
    let unit_ty = ctx.unit();
    let callee_ty = ctx.function(Vec::new(), vec![i32_ty], i32_ty, Effect::Pure);
    let function = one_arg_function(
        &mut ctx,
        "main",
        "f",
        callee_ty,
        unit_ty,
        HirExpr {
            ty: i32_ty,
            kind: HirExprKind::CallIndirect {
                callee: Box::new(HirExpr {
                    ty: callee_ty,
                    kind: HirExprKind::Var("f".to_string()),
                    span: Span::dummy(),
                }),
                params: vec![i32_ty],
                result: i32_ty,
                effect: Effect::Pure,
                args: vec![HirExpr {
                    ty: i32_ty,
                    kind: HirExprKind::LiteralI32(1),
                    span: Span::dummy(),
                }],
            },
            span: Span::dummy(),
        },
    );
    let module = empty_module(vec![function], Some("main"));

    assert_eq!(
        first_diagnostic_code(precheck_wasm_codegen(&ctx, &module)),
        wasm_code(WasmDiagnosticCode::IndirectSignatureMissing)
    );
}

#[test]
fn wasm_codegen_reports_indirect_signature_missing_without_panicking() {
    let mut ctx = TypeCtx::new();
    let i32_ty = ctx.i32();
    let unit_ty = ctx.unit();
    let callee_ty = ctx.function(Vec::new(), vec![i32_ty], i32_ty, Effect::Pure);
    let function = one_arg_function(
        &mut ctx,
        "main",
        "f",
        callee_ty,
        unit_ty,
        HirExpr {
            ty: i32_ty,
            kind: HirExprKind::CallIndirect {
                callee: Box::new(HirExpr {
                    ty: callee_ty,
                    kind: HirExprKind::Var("f".to_string()),
                    span: Span::dummy(),
                }),
                params: vec![i32_ty],
                result: i32_ty,
                effect: Effect::Pure,
                args: vec![HirExpr {
                    ty: i32_ty,
                    kind: HirExprKind::LiteralI32(1),
                    span: Span::dummy(),
                }],
            },
            span: Span::dummy(),
        },
    );
    let module = empty_module(vec![function], Some("main"));

    assert_eq!(
        first_error_code(codegen_wasm::generate_wasm(&ctx, &module)),
        wasm_code(WasmDiagnosticCode::IndirectSignatureMissing)
    );
}

#[test]
fn wasm_precheck_reports_unknown_intrinsic_code() {
    let mut ctx = TypeCtx::new();
    let unit_ty = ctx.unit();
    let function = zero_arg_function(
        &mut ctx,
        "main",
        unit_ty,
        HirExpr {
            ty: unit_ty,
            kind: HirExprKind::Intrinsic {
                name: "unknown_wasm_codegen_intrinsic".to_string(),
                type_args: Vec::new(),
                args: Vec::new(),
            },
            span: Span::dummy(),
        },
    );
    let module = empty_module(vec![function], Some("main"));

    assert_eq!(
        first_diagnostic_code(precheck_wasm_codegen(&ctx, &module)),
        wasm_code(WasmDiagnosticCode::IntrinsicUnknown)
    );
}

#[test]
fn llvm_precheck_reports_unknown_intrinsic_type_code() {
    let mut ctx = TypeCtx::new();
    let unit_ty = ctx.unit();
    let function = zero_arg_function(
        &mut ctx,
        "main",
        unit_ty,
        HirExpr {
            ty: unit_ty,
            kind: HirExprKind::Intrinsic {
                name: "unknown_llvm_codegen_intrinsic".to_string(),
                type_args: Vec::new(),
                args: Vec::new(),
            },
            span: Span::dummy(),
        },
    );
    let module = empty_module(vec![function], Some("main"));
    let reachable = BTreeSet::from(["main".to_string()]);

    assert_eq!(
        first_diagnostic_code(precheck_llvm_codegen(&ctx, &module, &reachable)),
        DiagnosticCode::Type(TypeDiagnosticCode::IntrinsicUnknown)
    );
}
