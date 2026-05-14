use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::hir::{HirExpr, HirExprKind};
use crate::span::Span;
use crate::types::TypeKind;

use super::diagnostics::type_error;
use super::{BlockChecker, FieldAccessorKind, FieldIdx, StackEntry};

pub(super) enum FieldAccessorApplyResult {
    NotHandled,
    Handled(Option<StackEntry>),
}

impl<'a> BlockChecker<'a> {
    pub(super) fn apply_field_accessor_function(
        &mut self,
        field_accessor: FieldAccessorKind,
        args: &[StackEntry],
        span: Span,
    ) -> FieldAccessorApplyResult {
        if args.len() < 2 {
            return FieldAccessorApplyResult::NotHandled;
        }
        let obj = args[0].expr.clone();
        let idx = &args[1].expr;
        let field_idx = match &idx.kind {
            HirExprKind::LiteralI32(v) => Some(FieldIdx::Index(*v as usize)),
            HirExprKind::LiteralStr(sid) => {
                let name = self.string_table.get(*sid).unwrap().clone();
                Some(FieldIdx::Name(name))
            }
            _ => None,
        };
        let Some(f_idx) = field_idx else {
            return FieldAccessorApplyResult::NotHandled;
        };
        let access_base_ty = if field_accessor == FieldAccessorKind::GetRef {
            let resolved_obj_ty = self.ctx.resolve(obj.ty);
            match self.ctx.get(resolved_obj_ty) {
                TypeKind::Reference(inner, _) => inner,
                _ => {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::FieldInvalidAccess,
                        "get_ref expects a reference to a composite value",
                        obj.span,
                    ));
                    self.ctx.never()
                }
            }
        } else {
            obj.ty
        };
        let Some((f_ty, offset)) =
            self.resolve_field_access_with_mode(access_base_ty, f_idx, span, true)
        else {
            return FieldAccessorApplyResult::NotHandled;
        };
        if field_accessor == FieldAccessorKind::Get && args.len() == 2 {
            return FieldAccessorApplyResult::Handled(Some(StackEntry {
                ty: f_ty,
                expr: HirExpr {
                    ty: f_ty,
                    kind: HirExprKind::Intrinsic {
                        name: "get_field".to_string(),
                        type_args: Vec::new(),
                        args: vec![obj, idx.clone()],
                    },
                    span,
                },
                type_args: Vec::new(),
                assign: None,
                auto_call: true,
            }));
        } else if field_accessor == FieldAccessorKind::GetRef && args.len() == 2 {
            let ref_ty = self.ctx.reference(f_ty, false);
            return FieldAccessorApplyResult::Handled(Some(StackEntry {
                ty: ref_ty,
                expr: HirExpr {
                    ty: ref_ty,
                    kind: HirExprKind::Intrinsic {
                        name: "get_field_ref".to_string(),
                        type_args: Vec::new(),
                        args: vec![obj, idx.clone()],
                    },
                    span,
                },
                type_args: Vec::new(),
                assign: None,
                auto_call: true,
            }));
        } else if field_accessor == FieldAccessorKind::Put && args.len() == 3 {
            let val = args[2].expr.clone();
            let _ = self.ctx.unify(val.ty, f_ty);
            let addr_expr = if offset == 0 {
                obj
            } else {
                HirExpr {
                    ty: self.ctx.i32(),
                    kind: HirExprKind::Intrinsic {
                        name: "add".to_string(),
                        type_args: vec![self.ctx.i32()],
                        args: vec![
                            obj,
                            HirExpr {
                                ty: self.ctx.i32(),
                                kind: HirExprKind::LiteralI32(offset as i32),
                                span: idx.span,
                            },
                        ],
                    },
                    span,
                }
            };
            return FieldAccessorApplyResult::Handled(Some(StackEntry {
                ty: self.ctx.unit(),
                expr: HirExpr {
                    ty: self.ctx.unit(),
                    kind: HirExprKind::Intrinsic {
                        name: "store".to_string(),
                        type_args: vec![f_ty],
                        args: vec![addr_expr, val],
                    },
                    span,
                },
                type_args: Vec::new(),
                assign: None,
                auto_call: true,
            }));
        }
        FieldAccessorApplyResult::NotHandled
    }
}
