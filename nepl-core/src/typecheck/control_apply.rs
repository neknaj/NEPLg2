use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::DiagnosticCode;
use crate::hir::{HirExpr, HirExprKind};

use super::{BlockChecker, StackEntry};

pub(super) enum SpecialApplyResult {
    NotHandled,
    Handled(Option<StackEntry>),
}

impl<'a> BlockChecker<'a> {
    pub(super) fn apply_control_special_function(
        &mut self,
        func: &StackEntry,
        args: &[StackEntry],
    ) -> SpecialApplyResult {
        match &func.expr.kind {
            HirExprKind::Var(name) if name == "if" => self.apply_if_function(func, args),
            HirExprKind::Var(name) if name == "while" => self.apply_while_function(func, args),
            _ => SpecialApplyResult::NotHandled,
        }
    }

    fn apply_if_function(&mut self, func: &StackEntry, args: &[StackEntry]) -> SpecialApplyResult {
        if args.len() != 3 {
            self.diagnostics.push(
                Diagnostic::error("if expects three arguments", func.expr.span).with_code(
                    DiagnosticCode::Type(
                        crate::diagnostic_codes::TypeDiagnosticCode::IfArityMismatch,
                    ),
                ),
            );
            return SpecialApplyResult::Handled(None);
        }
        if self.ctx.unify(args[0].ty, self.ctx.bool()).is_err() {
            self.diagnostics.push(
                Diagnostic::error("if condition must be bool", args[0].expr.span).with_code(
                    DiagnosticCode::Type(
                        crate::diagnostic_codes::TypeDiagnosticCode::IfConditionMismatch,
                    ),
                ),
            );
        }
        let branch_ty = self.ctx.unify(args[1].ty, args[2].ty).unwrap_or(args[1].ty);
        SpecialApplyResult::Handled(Some(StackEntry {
            ty: branch_ty,
            expr: HirExpr {
                ty: branch_ty,
                kind: HirExprKind::If {
                    cond: Box::new(args[0].expr.clone()),
                    then_branch: Box::new(args[1].expr.clone()),
                    else_branch: Box::new(args[2].expr.clone()),
                },
                span: func.expr.span,
            },
            type_args: Vec::new(),
            assign: None,
            auto_call: true,
        }))
    }

    fn apply_while_function(
        &mut self,
        func: &StackEntry,
        args: &[StackEntry],
    ) -> SpecialApplyResult {
        if args.len() != 2 {
            self.diagnostics.push(
                Diagnostic::error("while expects two arguments", func.expr.span).with_code(
                    DiagnosticCode::Type(
                        crate::diagnostic_codes::TypeDiagnosticCode::WhileArityMismatch,
                    ),
                ),
            );
            return SpecialApplyResult::Handled(None);
        }
        if self.ctx.unify(args[0].ty, self.ctx.bool()).is_err() {
            self.diagnostics.push(
                Diagnostic::error("while condition must be bool", args[0].expr.span).with_code(
                    DiagnosticCode::Type(
                        crate::diagnostic_codes::TypeDiagnosticCode::WhileConditionMismatch,
                    ),
                ),
            );
        }
        if self.ctx.unify(args[1].ty, self.ctx.unit()).is_err() {
            self.diagnostics.push(
                Diagnostic::error("while body must be unit", args[1].expr.span).with_code(
                    DiagnosticCode::Type(
                        crate::diagnostic_codes::TypeDiagnosticCode::WhileBodyMismatch,
                    ),
                ),
            );
        }
        SpecialApplyResult::Handled(Some(StackEntry {
            ty: self.ctx.unit(),
            expr: HirExpr {
                ty: self.ctx.unit(),
                kind: HirExprKind::While {
                    cond: Box::new(args[0].expr.clone()),
                    body: Box::new(args[1].expr.clone()),
                },
                span: func.expr.span,
            },
            type_args: Vec::new(),
            assign: None,
            auto_call: true,
        }))
    }
}
