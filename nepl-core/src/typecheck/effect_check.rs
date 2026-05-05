use alloc::format;
use alloc::string::String;

use crate::ast::{Block, Effect, Stmt};
use crate::diagnostic_codes::EffectDiagnosticCode;
use crate::effects::{
    intrinsic_effect, intrinsic_is_raw_memory_effect, raw_body_direct_callees,
    raw_body_memory_operations, raw_callee_is_raw_memory_effect,
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
            if !memory_ops.is_empty() && !self.raw_body_memory_operations_allowed(span) {
                let op = memory_ops
                    .first()
                    .map(String::as_str)
                    .unwrap_or("raw memory operation");
                self.diagnostics.push(effect_error(
                    EffectDiagnosticCode::PureCallsImpure,
                    format!(
                        "pure raw body cannot access raw memory instruction '{}'",
                        op
                    ),
                    span,
                ));
                return false;
            }
            for callee in raw_body_direct_callees(body) {
                if raw_callee_is_raw_memory_effect(&callee) {
                    if !self.raw_body_memory_operations_allowed(span) {
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

    pub(super) fn raw_body_memory_operations_allowed(&self, span: Span) -> bool {
        let Some(source_map) = self.source_map else {
            return false;
        };
        source_map.raw_memory_operations_allowed(span.file_id)
    }

    pub(super) fn raw_memory_intrinsic_allowed(&self, name: &str, span: Span) -> bool {
        intrinsic_is_raw_memory_effect(name) && self.raw_body_memory_operations_allowed(span)
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
