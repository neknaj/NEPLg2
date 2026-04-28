use alloc::boxed::Box;
use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::{HirExpr, HirExprKind};
use crate::types::TypeKind;

use super::{AssignKind, BlockChecker, StackEntry};

fn assignment_apply_dump_enabled() -> bool {
    #[cfg(target_os = "none")]
    {
        false
    }
    #[cfg(not(target_os = "none"))]
    {
        std::env::var("NEPL_DUMP_HIR").is_ok()
    }
}

macro_rules! assignment_apply_log {
    ($($arg:tt)*) => {{
        #[cfg(target_os = "none")]
        {
            let _ = core::format_args!($($arg)*);
        }
        #[cfg(not(target_os = "none"))]
        {
            std::eprintln!($($arg)*);
        }
    }};
}

macro_rules! assignment_apply_dump {
    ($($arg:tt)*) => {
        if assignment_apply_dump_enabled() {
            assignment_apply_log!($($arg)*);
        }
    };
}

impl<'a> BlockChecker<'a> {
    pub(super) fn apply_assignment_function(
        &mut self,
        func: StackEntry,
        args: Vec<StackEntry>,
        assign: AssignKind,
    ) -> Option<StackEntry> {
        if args.len() != 1 {
            self.diagnostics.push(
                Diagnostic::error("assignment expects one argument", func.expr.span)
                    .with_id(DiagnosticId::TypeAssignmentArityMismatch),
            );
            return None;
        }
        if let AssignKind::AddrOf(mutable) = assign {
            if crate::log::is_verbose() {
                assignment_apply_log!(
                    "apply_function: Reducing AddrOf, inner={:?}",
                    args[0].expr.kind
                );
            }
            let inner_ty = args[0].ty;
            let res_ty = self.ctx.reference(inner_ty, mutable);
            return Some(StackEntry {
                ty: res_ty,
                expr: HirExpr {
                    ty: res_ty,
                    kind: HirExprKind::AddrOf(Box::new(args[0].expr.clone())),
                    span: func.expr.span,
                },
                type_args: Vec::new(),
                assign: None,
                auto_call: true,
            });
        } else if matches!(assign, AssignKind::Deref) {
            let arg_ty = self.ctx.resolve(args[0].ty);
            let inner_ty = match self.ctx.get(arg_ty) {
                TypeKind::Reference(inner, _) => inner,
                _ => {
                    self.diagnostics.push(
                        Diagnostic::error(
                            format!(
                                "cannot dereference non-reference type: {}",
                                self.ctx.type_to_string(arg_ty)
                            ),
                            args[0].expr.span,
                        )
                        .with_id(DiagnosticId::TypeInvalidDeref),
                    );
                    self.ctx.never()
                }
            };
            return Some(StackEntry {
                ty: inner_ty,
                expr: HirExpr {
                    ty: inner_ty,
                    kind: HirExprKind::Deref(Box::new(args[0].expr.clone())),
                    span: func.expr.span,
                },
                type_args: Vec::new(),
                assign: None,
                auto_call: true,
            });
        }

        let name = match &func.expr.kind {
            HirExprKind::Var(n) => n.clone(),
            _ => "_".to_string(),
        };
        if let Some(b) = self.env.lookup_mut(&name) {
            let b_ty = b.ty;
            let b_mut = b.mutable;
            let b_defined = b.defined;
            if let Err(_) = self.ctx.unify(b_ty, args[0].ty) {
                self.diagnostics.push(
                    Diagnostic::error("type mismatch in assignment", func.expr.span)
                        .with_id(DiagnosticId::TypeAssignmentTypeMismatch),
                );
            }
            match assign {
                AssignKind::Let => {
                    b.defined = true;
                    b.ty = b_ty;
                    assignment_apply_dump!("typecheck: marking binding defined {}", name);
                    Some(StackEntry {
                        ty: self.ctx.unit(),
                        expr: HirExpr {
                            ty: self.ctx.unit(),
                            kind: HirExprKind::Let {
                                name: name.clone(),
                                mutable: b_mut,
                                value: Box::new(args[0].expr.clone()),
                            },
                            span: func.expr.span,
                        },
                        type_args: Vec::new(),
                        assign: None,
                        auto_call: true,
                    })
                }
                AssignKind::Set => {
                    if !b_defined {
                        self.diagnostics.push(
                            Diagnostic::error("cannot set undefined variable", func.expr.span)
                                .with_id(DiagnosticId::TypeUndefinedVariable),
                        );
                    }
                    if !b_mut {
                        self.diagnostics.push(
                            Diagnostic::error("variable is not mutable", func.expr.span)
                                .with_id(DiagnosticId::TypeImmutableMutation),
                        );
                    }
                    Some(StackEntry {
                        ty: self.ctx.unit(),
                        expr: HirExpr {
                            ty: self.ctx.unit(),
                            kind: HirExprKind::Set {
                                name: name.clone(),
                                value: Box::new(args[0].expr.clone()),
                            },
                            span: func.expr.span,
                        },
                        type_args: Vec::new(),
                        assign: None,
                        auto_call: true,
                    })
                }
                _ => unreachable!(),
            }
        } else {
            self.diagnostics.push(
                Diagnostic::error(
                    format!("undefined variable for assignment: {}", name),
                    func.expr.span,
                )
                .with_id(DiagnosticId::TypeAssignmentUndefinedVariable),
            );
            None
        }
    }
}
