use nepl_core::ast::Effect;
use nepl_core::codegen_wasm;
use nepl_core::diagnostic_codes::DiagnosticCode;
use nepl_core::hir::{HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirLine, HirModule};
use nepl_core::span::Span;
use nepl_core::types::{TypeCtx, TypeId};

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

fn first_error_code(
    err: Result<codegen_wasm::CodegenResult, Vec<nepl_core::diagnostic::Diagnostic>>,
) -> DiagnosticCode {
    let diagnostics = err.expect_err("codegen should return diagnostics");
    diagnostics
        .first()
        .and_then(|diag| diag.code)
        .expect("diagnostic code should be attached")
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
        DiagnosticCode::Backend(nepl_core::diagnostic_codes::BackendDiagnosticCode::Wasm(
            nepl_core::diagnostic_codes::WasmDiagnosticCode::FunctionSignatureUnsupported,
        ))
    );
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
        DiagnosticCode::Backend(nepl_core::diagnostic_codes::BackendDiagnosticCode::Wasm(
            nepl_core::diagnostic_codes::WasmDiagnosticCode::VariableUnknown
        ))
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
        DiagnosticCode::Backend(nepl_core::diagnostic_codes::BackendDiagnosticCode::Wasm(
            nepl_core::diagnostic_codes::WasmDiagnosticCode::StringLiteralNotFound,
        ))
    );
}
