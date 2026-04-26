extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::{HirBlock, HirBody, HirExpr, HirExprKind, HirModule};
use crate::types::TypeCtx;
use crate::wasm_shared;
use wasm_encoder::ValType;

type WasmSig = (Vec<ValType>, Vec<ValType>);
const LLVM_SUPPORTED_INTRINSICS: &[&str] = &[
    "size_of",
    "align_of",
    "load",
    "store",
    "get_field",
    "get_field_ref",
    "set_field",
    "unreachable",
    "add",
    "f32_to_i32",
    "i32_to_u8",
    "i32_to_u32",
    "u8_to_i32",
    "u32_to_i32",
    "i64_to_u64",
    "u64_to_i64",
    "str_addr",
    "str_from_addr_unchecked",
];

pub fn precheck_wasm_codegen(ctx: &TypeCtx, module: &HirModule) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let wasm_sig_set = wasm_shared::collect_wasm_signature_set(ctx, module);

    for ext in &module.externs {
        if wasm_shared::wasm_sig_ids(ctx, ext.result, &ext.params).is_none() {
            out.push(
                Diagnostic::error("unsupported extern signature for wasm", ext.span)
                    .with_id(DiagnosticId::CodegenWasmUnsupportedExternSignature),
            );
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
            out.push(
                Diagnostic::error("unsupported function signature for wasm", f.span)
                    .with_id(DiagnosticId::CodegenWasmUnsupportedFunctionSignature),
            );
        }
        if !wasm_shared::should_skip_wasm_codegen_for_generic(ctx, f) {
            let result_kind = ctx.get(ctx.resolve_id(f.result));
            if !matches!(result_kind, crate::types::TypeKind::Unit) {
                if let HirBody::Block(block) = &f.body {
                    if !block_produces_value(ctx, block) {
                        out.push(
                            Diagnostic::error("function expected to return value", f.span)
                                .with_id(DiagnosticId::CodegenWasmMissingReturnValue),
                        );
                    }
                }
            }
            if matches!(f.body, HirBody::LlvmIr(_)) {
                out.push(
                    Diagnostic::error("llvm ir block cannot be compiled by wasm backend", f.span)
                        .with_id(DiagnosticId::CodegenWasmLlvmIrBodyNotSupported),
                );
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
                out.push(
                    Diagnostic::error("function expected to return value", f.span)
                        .with_id(DiagnosticId::TypeReturnTypeMismatch),
                );
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
            if !LLVM_SUPPORTED_INTRINSICS
                .iter()
                .any(|n| *n == name.as_str())
            {
                out.push(
                    Diagnostic::error("unknown codegen intrinsic for llvm", expr.span)
                        .with_id(DiagnosticId::TypeUnknownIntrinsic),
                );
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
            } => {
                if let Some(sig) = wasm_shared::wasm_sig_ids(ctx, *result, params) {
                    if !wasm_sig_set.contains(&sig) {
                        out.push(
                            Diagnostic::error(
                                "missing wasm signature for indirect call",
                                expr.span,
                            )
                            .with_id(DiagnosticId::CodegenWasmMissingIndirectSignature),
                        );
                    }
                } else {
                    out.push(
                        Diagnostic::error(
                            "unsupported indirect call signature for wasm",
                            expr.span,
                        )
                        .with_id(DiagnosticId::CodegenWasmUnsupportedIndirectSignature),
                    );
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
                    out.push(
                        Diagnostic::error("unknown codegen intrinsic", expr.span)
                            .with_id(DiagnosticId::CodegenWasmUnknownIntrinsic),
                    );
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
