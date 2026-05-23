extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::DiagnosticCode;
use crate::hir::{HirBlock, HirBody, HirExpr, HirExprKind, HirModule};
use crate::intrinsic_kinds::{CoreIntrinsicKind, FieldAccessorKind, ScalarIntrinsicKind};
use crate::resource_primitives::{CollectionSlotBorrowPrimitive, CollectionSlotLifecyclePrimitive};
use crate::scalar_primitives::I32ArithmeticPrimitive;
use crate::types::TypeCtx;
use crate::wasm_shared;
use wasm_encoder::ValType;

use crate::diagnostic_codes::{BackendDiagnosticCode, TypeDiagnosticCode, WasmDiagnosticCode};

type WasmSig = (Vec<ValType>, Vec<ValType>);

fn is_supported_llvm_intrinsic(name: &str) -> bool {
    match CoreIntrinsicKind::from_intrinsic_name(name) {
        Some(CoreIntrinsicKind::CallsiteSpan) => false,
        Some(_) => true,
        None => {
            FieldAccessorKind::from_intrinsic_name(name).is_some()
                || ScalarIntrinsicKind::from_intrinsic_name(name).is_some()
                || I32ArithmeticPrimitive::from_codegen_intrinsic_name(name).is_some()
                || CollectionSlotLifecyclePrimitive::from_intrinsic_name(name).is_some()
                || CollectionSlotBorrowPrimitive::from_intrinsic_name(name).is_some()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::is_supported_llvm_intrinsic;
    use crate::scalar_primitives::I32ArithmeticPrimitive;

    #[test]
    fn llvm_intrinsic_support_uses_i32_arithmetic_codegen_subset() {
        assert!(is_supported_llvm_intrinsic(
            I32ArithmeticPrimitive::Add
                .codegen_intrinsic_name()
                .expect("add is a backend intrinsic")
        ));
        assert!(!is_supported_llvm_intrinsic("sub"));
        assert!(!is_supported_llvm_intrinsic("mul"));
    }
}

fn wasm_error(
    code: WasmDiagnosticCode,
    message: impl Into<String>,
    span: crate::span::Span,
) -> Diagnostic {
    Diagnostic::error_with_code(
        DiagnosticCode::Backend(BackendDiagnosticCode::Wasm(code)),
        message,
        span,
    )
}

fn type_error(
    code: TypeDiagnosticCode,
    message: impl Into<String>,
    span: crate::span::Span,
) -> Diagnostic {
    Diagnostic::error_with_code(DiagnosticCode::Type(code), message, span)
}

pub fn precheck_wasm_codegen(ctx: &TypeCtx, module: &HirModule) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let wasm_sig_set = wasm_shared::collect_wasm_signature_set(ctx, module);

    for ext in &module.externs {
        if wasm_shared::wasm_sig_ids(ctx, ext.result, &ext.params).is_none() {
            out.push(wasm_error(
                WasmDiagnosticCode::ExternSignatureUnsupported,
                "unsupported extern signature for wasm",
                ext.span,
            ));
        }
    }

    let reachable_functions = wasm_shared::collect_reachable_wasm_functions(module);
    for f in &module.functions {
        if !reachable_functions.contains(&f.name) {
            continue;
        }
        if wasm_shared::wasm_sig(ctx, f.result, &f.params).is_none()
            && !wasm_shared::should_skip_wasm_codegen_for_generic(ctx, f)
        {
            out.push(wasm_error(
                WasmDiagnosticCode::FunctionSignatureUnsupported,
                "unsupported function signature for wasm",
                f.span,
            ));
        }
        if !wasm_shared::should_skip_wasm_codegen_for_generic(ctx, f) {
            let result_kind = ctx.get(ctx.resolve_id(f.result));
            if !matches!(result_kind, crate::types::TypeKind::Unit) {
                if let HirBody::Block(block) = &f.body {
                    if !block_produces_value(ctx, block) {
                        out.push(wasm_error(
                            WasmDiagnosticCode::ReturnValueMissing,
                            "function expected to return value",
                            f.span,
                        ));
                    }
                }
            }
            if matches!(f.body, HirBody::LlvmIr(_)) {
                out.push(wasm_error(
                    WasmDiagnosticCode::LlvmIrBodyUnsupported,
                    "llvm ir block cannot be compiled by wasm backend",
                    f.span,
                ));
            }
            if let HirBody::Block(block) = &f.body {
                precheck_wasm_indirect_signature(ctx, block, &wasm_sig_set, &mut out);
            }
            out.extend(wasm_shared::precheck_raw_wasm_body(ctx, f));
        }
    }

    out
}

pub fn precheck_llvm_codegen(
    ctx: &TypeCtx,
    module: &HirModule,
    reachable: &BTreeSet<String>,
) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    for f in &module.functions {
        if !reachable.contains(&f.name) {
            continue;
        }
        if let HirBody::Block(block) = &f.body {
            let result_kind = ctx.get(ctx.resolve_id(f.result));
            if !matches!(result_kind, crate::types::TypeKind::Unit)
                && !block_produces_value(ctx, block)
            {
                out.push(type_error(
                    TypeDiagnosticCode::ReturnTypeMismatch,
                    "function expected to return value",
                    f.span,
                ));
            }
            precheck_llvm_expr_tree(block, &mut out);
        }
    }
    out
}

fn precheck_llvm_expr_tree(block: &HirBlock, out: &mut Vec<Diagnostic>) {
    for line in &block.lines {
        check_llvm_expr(&line.expr, out);
    }
}

fn check_llvm_expr(expr: &HirExpr, out: &mut Vec<Diagnostic>) {
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. } => {
            if !is_supported_llvm_intrinsic(name) {
                out.push(type_error(
                    TypeDiagnosticCode::IntrinsicUnknown,
                    "unknown codegen intrinsic for llvm",
                    expr.span,
                ));
            }
            for arg in args {
                check_llvm_expr(arg, out);
            }
        }
        HirExprKind::Call { args, .. } => {
            for arg in args {
                check_llvm_expr(arg, out);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            check_llvm_expr(callee, out);
            for arg in args {
                check_llvm_expr(arg, out);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            check_llvm_expr(cond, out);
            check_llvm_expr(then_branch, out);
            check_llvm_expr(else_branch, out);
        }
        HirExprKind::While { cond, body } => {
            check_llvm_expr(cond, out);
            check_llvm_expr(body, out);
        }
        HirExprKind::Match { scrutinee, arms } => {
            check_llvm_expr(scrutinee, out);
            for arm in arms {
                check_llvm_expr(&arm.body, out);
            }
        }
        HirExprKind::Block(b) => precheck_llvm_expr_tree(b, out),
        HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
            check_llvm_expr(value, out);
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                check_llvm_expr(payload, out);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for field in fields {
                check_llvm_expr(field, out);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                check_llvm_expr(item, out);
            }
        }
        HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => check_llvm_expr(inner, out),
        HirExprKind::Drop { .. } => {}
        HirExprKind::Unit
        | HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Var(_)
        | HirExprKind::FnValue(_) => {}
    }
}

fn block_produces_value(ctx: &TypeCtx, block: &HirBlock) -> bool {
    let mut last_non_drop_line_ty_is_value = false;
    for line in &block.lines {
        if line.drop_result {
            continue;
        }
        let ty = ctx.get(ctx.resolve_id(line.expr.ty));
        last_non_drop_line_ty_is_value = !matches!(ty, crate::types::TypeKind::Unit);
    }
    last_non_drop_line_ty_is_value
}

fn precheck_wasm_indirect_signature(
    ctx: &TypeCtx,
    block: &HirBlock,
    wasm_sig_set: &BTreeSet<WasmSig>,
    out: &mut Vec<Diagnostic>,
) {
    for line in &block.lines {
        check_indirect_sig_expr(ctx, &line.expr, wasm_sig_set, out);
    }
}

fn check_indirect_sig_expr(
    ctx: &TypeCtx,
    expr: &HirExpr,
    wasm_sig_set: &BTreeSet<WasmSig>,
    out: &mut Vec<Diagnostic>,
) {
    let mut stack = Vec::new();
    stack.push(expr);
    while let Some(expr) = stack.pop() {
        match &expr.kind {
            HirExprKind::CallIndirect {
                callee,
                params,
                result,
                args,
                ..
            } => {
                if let Some(sig) = wasm_shared::wasm_sig_ids(ctx, *result, params) {
                    if !wasm_sig_set.contains(&sig) {
                        out.push(wasm_error(
                            WasmDiagnosticCode::IndirectSignatureMissing,
                            "missing wasm signature for indirect call",
                            expr.span,
                        ));
                    }
                } else {
                    out.push(wasm_error(
                        WasmDiagnosticCode::IndirectSignatureUnsupported,
                        "unsupported indirect call signature for wasm",
                        expr.span,
                    ));
                }
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
                stack.push(callee);
            }
            HirExprKind::Call { args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::Intrinsic { name, args, .. } => {
                if !wasm_shared::is_supported_wasm_intrinsic(name) {
                    out.push(wasm_error(
                        WasmDiagnosticCode::IntrinsicUnknown,
                        "unknown codegen intrinsic",
                        expr.span,
                    ));
                }
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                stack.push(else_branch);
                stack.push(then_branch);
                stack.push(cond);
            }
            HirExprKind::While { cond, body } => {
                stack.push(body);
                stack.push(cond);
            }
            HirExprKind::Match { scrutinee, arms } => {
                for arm in arms.iter().rev() {
                    stack.push(&arm.body);
                }
                stack.push(scrutinee);
            }
            HirExprKind::Block(block) => {
                for line in block.lines.iter().rev() {
                    stack.push(&line.expr);
                }
            }
            HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
                stack.push(value);
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                if let Some(payload) = payload {
                    stack.push(payload);
                }
            }
            HirExprKind::StructConstruct { fields, .. } => {
                for field in fields.iter().rev() {
                    stack.push(field);
                }
            }
            HirExprKind::TupleConstruct { items } => {
                for item in items.iter().rev() {
                    stack.push(item);
                }
            }
            HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
                stack.push(inner);
            }
            HirExprKind::Drop { .. } => {}
            HirExprKind::Unit
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Var(_)
            | HirExprKind::FnValue(_) => {}
        }
    }
}
