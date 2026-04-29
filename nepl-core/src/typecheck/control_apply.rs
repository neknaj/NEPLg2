use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::hir::{HirExpr, HirExprKind};

use super::diagnostics::type_error;
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
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::IfArityMismatch,
                "if expects three arguments",
                func.expr.span,
            ));
            return SpecialApplyResult::Handled(None);
        }
        if self.ctx.unify(args[0].ty, self.ctx.bool()).is_err() {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::IfConditionMismatch,
                "if condition must be bool",
                args[0].expr.span,
            ));
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
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::WhileArityMismatch,
                "while expects two arguments",
                func.expr.span,
            ));
            return SpecialApplyResult::Handled(None);
        }
        if self.ctx.unify(args[0].ty, self.ctx.bool()).is_err() {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::WhileConditionMismatch,
                "while condition must be bool",
                args[0].expr.span,
            ));
        }
        if self.ctx.unify(args[1].ty, self.ctx.unit()).is_err() {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::WhileBodyMismatch,
                "while body must be unit",
                args[1].expr.span,
            ));
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
