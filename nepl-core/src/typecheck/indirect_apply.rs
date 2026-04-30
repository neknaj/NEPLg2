use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::hir::{HirExpr, HirExprKind};
use crate::types::{TypeId, TypeKind};

use super::diagnostics::type_error;
use super::env::BindingKind;
use super::{BlockChecker, StackEntry};

pub(super) fn apply_indirect_function_call(
    checker: &mut BlockChecker<'_>,
    func: StackEntry,
    args: Vec<StackEntry>,
    result: TypeId,
    expected_ret: Option<TypeId>,
) -> Option<StackEntry> {
    let allow_indirect = match &func.expr.kind {
        HirExprKind::FnValue(name) => {
            let has_capture = checker.env.lookup_all_callables_by_symbol(name).iter().any(
                |b| matches!(&b.kind, BindingKind::Func { captures, .. } if !captures.is_empty()),
            );
            if has_capture {
                checker.diagnostics.push(type_error(
                    TypeDiagnosticCode::FunctionValueCapturingUnsupported,
                    "capturing function cannot be used as a function value yet",
                    func.expr.span,
                ));
                false
            } else {
                true
            }
        }
        HirExprKind::Var(name) => {
            if !matches!(checker.ctx.get(func.ty), TypeKind::Function { .. }) {
                false
            } else {
                let has_capture = checker
                    .env
                    .lookup_all_callables(name)
                    .iter()
                    .any(|b| matches!(&b.kind, BindingKind::Func { captures, .. } if !captures.is_empty()));
                if has_capture {
                    checker.diagnostics.push(type_error(
                        TypeDiagnosticCode::FunctionValueCapturingUnsupported,
                        "capturing function cannot be passed as a function value yet",
                        func.expr.span,
                    ));
                    false
                } else {
                    true
                }
            }
        }
        _ => matches!(checker.ctx.get(func.ty), TypeKind::Function { .. }),
    };
    if !allow_indirect {
        checker.diagnostics.push(type_error(
            TypeDiagnosticCode::IndirectCallRequiresFunctionValue,
            "indirect call requires a function value",
            func.expr.span,
        ));
        return None;
    }
    let callee_effect = match checker.ctx.get(func.ty) {
        TypeKind::Function { effect, .. } => effect,
        _ => {
            checker.diagnostics.push(type_error(
                TypeDiagnosticCode::IndirectCallRequiresFunctionValue,
                "indirect call requires a function value",
                func.expr.span,
            ));
            return None;
        }
    };

    let resolved_params: Vec<TypeId> = args.iter().map(|a| checker.ctx.resolve_id(a.ty)).collect();
    let mut resolved_result = checker.ctx.resolve_id(result);
    if let Some(expected) = expected_ret {
        if checker.ctx.unify(resolved_result, expected).is_ok() {
            resolved_result = checker.ctx.resolve_id(expected);
        }
    }
    Some(StackEntry {
        ty: resolved_result,
        expr: HirExpr {
            ty: resolved_result,
            kind: HirExprKind::CallIndirect {
                callee: Box::new(func.expr.clone()),
                params: resolved_params,
                result: resolved_result,
                effect: callee_effect,
                args: args.into_iter().map(|a| a.expr).collect(),
            },
            span: func.expr.span,
        },
        type_args: Vec::new(),
        assign: None,
        auto_call: true,
    })
}
