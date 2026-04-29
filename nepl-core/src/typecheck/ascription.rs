use alloc::format;

use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::hir::HirExprKind;
use crate::span::Span;
use crate::types::TypeId;

use super::diagnostics::type_error;
use super::{BlockChecker, StackEntry};

impl<'a> BlockChecker<'a> {
    pub(super) fn apply_ascription(
        &mut self,
        stack: &mut [StackEntry],
        target: TypeId,
        span: Span,
    ) {
        if let Some(top) = stack.last_mut() {
            match self.char_literal_context_type(top, target) {
                Some(Ok(resolved)) => {
                    top.ty = resolved;
                    top.expr.ty = resolved;
                    return;
                }
                Some(Err(())) => {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::AnnotationMismatch,
                        "char literal does not fit in u8",
                        span,
                    ));
                    return;
                }
                None => {}
            }
            if let Err(_) = self.ctx.unify(top.ty, target) {
                let actual_ty = self.ctx.type_to_string(top.ty);
                let expected_ty = self.ctx.type_to_string(target);
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::AnnotationMismatch,
                    format!(
                        "type annotation mismatch (expected {}, got {})",
                        expected_ty, actual_ty
                    ),
                    span,
                ));
            } else {
                let resolved = self.ctx.resolve_id(target);
                top.ty = resolved;
                top.expr.ty = resolved;
            }
        }
    }

    pub(super) fn char_literal_value(&self, entry: &StackEntry) -> Option<i32> {
        if !self.ctx.same_type(entry.ty, self.ctx.char()) {
            return None;
        }
        match &entry.expr.kind {
            HirExprKind::LiteralI32(value) => Some(*value),
            _ => None,
        }
    }

    pub(super) fn char_literal_context_type(
        &self,
        entry: &StackEntry,
        target: TypeId,
    ) -> Option<Result<TypeId, ()>> {
        let value = self.char_literal_value(entry)?;
        if self.ctx.same_type(target, self.ctx.i32()) {
            return Some(Ok(self.ctx.resolve_id(target)));
        }
        if self.ctx.same_type(target, self.ctx.u8()) {
            return Some(
                (0..=255)
                    .contains(&value)
                    .then(|| self.ctx.resolve_id(target))
                    .ok_or(()),
            );
        }
        None
    }

    pub(super) fn char_literal_matches_context(&self, entry: &StackEntry, target: TypeId) -> bool {
        matches!(self.char_literal_context_type(entry, target), Some(Ok(_)))
    }
}
