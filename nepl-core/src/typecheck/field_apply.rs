use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::{HirExpr, HirExprKind};
use crate::span::Span;
use crate::types::TypeKind;

use super::hir_finalize::{add_i32_offset_expr, raw_aggregate_load_addr_expr};
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
                    self.diagnostics.push(
                        Diagnostic::error(
                            "get_ref expects a reference to a composite value",
                            obj.span,
                        )
                        .with_id(DiagnosticId::TypeInvalidFieldAccess),
                    );
                    self.ctx.never()
                }
            }
        } else {
            obj.ty
        };
        let Some((f_ty, offset)) =
            self.resolve_field_access_with_mode(access_base_ty, f_idx, span, false)
        else {
            return FieldAccessorApplyResult::NotHandled;
        };
        if field_accessor == FieldAccessorKind::Get && args.len() == 2 {
            let addr_expr = if let Some(raw_addr) = raw_aggregate_load_addr_expr(&obj, &self.ctx) {
                add_i32_offset_expr(raw_addr, offset, idx.span, span, self.ctx.i32())
            } else {
                add_i32_offset_expr(obj, offset, idx.span, span, self.ctx.i32())
            };
            return FieldAccessorApplyResult::Handled(Some(StackEntry {
                ty: f_ty,
                expr: HirExpr {
                    ty: f_ty,
                    kind: HirExprKind::Intrinsic {
                        name: "load".to_string(),
                        type_args: vec![f_ty],
                        args: vec![addr_expr],
                    },
                    span,
                },
                type_args: Vec::new(),
                assign: None,
                auto_call: true,
            }));
        } else if field_accessor == FieldAccessorKind::GetRef && args.len() == 2 {
            let ref_ty = self.ctx.reference(f_ty, false);
            let addr_expr = if offset == 0 {
                HirExpr {
                    ty: ref_ty,
                    kind: obj.kind,
                    span,
                }
            } else {
                HirExpr {
                    ty: ref_ty,
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
                ty: ref_ty,
                expr: addr_expr,
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
