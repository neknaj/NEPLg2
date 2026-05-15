use alloc::format;

use crate::ast::{Block, Effect, Stmt};
use crate::diagnostic_codes::EffectDiagnosticCode;
use crate::effects::{
    intrinsic_effect, intrinsic_is_raw_memory_effect, raw_body_direct_callees,
    raw_body_memory_operations, raw_memory_op_from_name, RawBodyMemoryOp, RawMemoryOp,
};
use crate::hir::HirBody;
use crate::span::Span;

use super::diagnostics::effect_error;
use super::env::BindingKind;
use super::syntax_helpers::gate_allows;
use super::BlockChecker;

impl<'a> BlockChecker<'a> {
    pub(super) fn validate_raw_body_effect(&mut self, body: &HirBody, span: Span) -> bool {
        if matches!(self.current_effect, Effect::Pure) {
            let memory_ops = raw_body_memory_operations(body);
            if let Some(operation) = memory_ops
                .iter()
                .copied()
                .find(|operation| !self.raw_body_memory_operation_allowed(*operation, span))
            {
                self.diagnostics.push(effect_error(
                    EffectDiagnosticCode::PureCallsImpure,
                    format!(
                        "pure raw body cannot access raw memory instruction '{}'",
                        operation.as_str()
                    ),
                    span,
                ));
                return false;
            }
            for callee in raw_body_direct_callees(body) {
                if let Some(operation) = raw_memory_op_from_name(&callee) {
                    if !self.raw_memory_operation_allowed(operation, span) {
                        self.diagnostics.push(effect_error(
                            EffectDiagnosticCode::PureCallsImpure,
                            format!("pure raw body cannot call raw memory helper '{}'", callee),
                            span,
                        ));
                        return false;
                    }
                    continue;
                }
                if self.raw_callee_is_impure(&callee) {
                    self.diagnostics.push(effect_error(
                        EffectDiagnosticCode::PureCallsImpure,
                        "pure context cannot call impure function",
                        span,
                    ));
                    return false;
                }
            }
        }
        true
    }

    pub(super) fn raw_memory_structural_boundary_allowed(&self, span: Span) -> bool {
        let Some(source_map) = self.source_map else {
            return false;
        };
        source_map.raw_memory_structural_boundary_allowed(span.file_id)
    }

    pub(super) fn raw_memory_operation_allowed(&self, operation: RawMemoryOp, span: Span) -> bool {
        let Some(source_map) = self.source_map else {
            return false;
        };
        source_map.raw_memory_operation_boundary_allowed(span.file_id, operation)
    }

    pub(super) fn raw_body_memory_operation_allowed(
        &self,
        operation: RawBodyMemoryOp,
        span: Span,
    ) -> bool {
        let Some(source_map) = self.source_map else {
            return false;
        };
        source_map.raw_body_memory_operation_boundary_allowed(span.file_id, operation)
    }

    pub(super) fn owner_aggregate_constructor_boundary_allowed(
        &self,
        span: Span,
        name: &str,
    ) -> bool {
        let Some(source_map) = self.source_map else {
            return false;
        };
        source_map.owner_aggregate_constructor_boundary_allowed(span.file_id, name)
    }

    pub(super) fn owner_aggregate_field_boundary_allowed(&self, span: Span) -> bool {
        let Some(source_map) = self.source_map else {
            return false;
        };
        source_map.owner_aggregate_field_boundary_allowed(span.file_id)
    }

    pub(super) fn raw_memory_intrinsic_allowed(&self, name: &str, span: Span) -> bool {
        intrinsic_is_raw_memory_effect(name)
            && raw_memory_op_from_name(name)
                .is_some_and(|operation| self.raw_memory_operation_allowed(operation, span))
    }

    pub(super) fn raw_callee_is_impure(&self, callee: &str) -> bool {
        if callee.starts_with("llvm.") {
            return false;
        }
        if let Some(effect) = self.raw_callee_declared_effect(callee) {
            return matches!(effect, Effect::Impure);
        }
        matches!(intrinsic_effect(callee), Effect::Impure)
    }

    pub(super) fn raw_callee_declared_effect(&self, callee: &str) -> Option<Effect> {
        let mut saw_pure = false;
        for binding in self
            .env
            .lookup_all_callables(callee)
            .into_iter()
            .chain(self.env.lookup_all_callables_by_symbol(callee).into_iter())
        {
            if let BindingKind::Func { effect, .. } = &binding.kind {
                if matches!(effect, Effect::Impure) {
                    return Some(Effect::Impure);
                }
                saw_pure = true;
            }
        }
        if saw_pure {
            Some(Effect::Pure)
        } else {
            None
        }
    }

    pub(super) fn select_target_raw_body(&mut self, block: &Block) -> Option<HirBody> {
        let mut pending_if: Option<bool> = None;
        let mut selected: Option<HirBody> = None;
        for stmt in &block.items {
            if let Stmt::Directive(d) = stmt {
                if let Some(allowed) = gate_allows(d, self.target, self.profile) {
                    pending_if = Some(allowed);
                    continue;
                }
            }
            let allowed = pending_if.unwrap_or(true);
            pending_if = None;
            if !allowed {
                continue;
            }
            match stmt {
                Stmt::Wasm(w) => {
                    if selected.is_some() {
                        self.diagnostics.push(effect_error(
                            EffectDiagnosticCode::RawBodyMultipleActive,
                            "multiple active raw bodies in one function",
                            w.span,
                        ));
                        return selected;
                    }
                    selected = Some(HirBody::Wasm(w.clone()));
                }
                Stmt::LlvmIr(l) => {
                    if selected.is_some() {
                        self.diagnostics.push(effect_error(
                            EffectDiagnosticCode::RawBodyMultipleActive,
                            "multiple active raw bodies in one function",
                            l.span,
                        ));
                        return selected;
                    }
                    selected = Some(HirBody::LlvmIr(l.clone()));
                }
                Stmt::Directive(_) => {}
                _ => return None,
            }
        }
        selected
    }
}
