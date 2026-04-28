use alloc::vec::Vec;

use crate::ast::Effect;
use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::HirExprKind;
use crate::types::{TypeId, TypeKind};

use super::control_apply::SpecialApplyResult;
use super::indirect_apply::apply_indirect_function_call;
use super::trait_call_apply::TraitMethodApplyResult;
use super::{BlockChecker, StackEntry};

macro_rules! function_apply_log {
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

impl<'a> BlockChecker<'a> {
    pub(super) fn apply_function(
        &mut self,
        func: StackEntry,
        params: Vec<TypeId>,
        result: TypeId,
        effect: Effect,
        mut args: Vec<StackEntry>,
        type_args: Vec<TypeId>,
        expected_ret: Option<TypeId>,
    ) -> Option<StackEntry> {
        if params.is_empty() && args.len() == 1 && matches!(args[0].expr.kind, HirExprKind::Unit) {
            args.clear();
        }

        if matches!(self.current_effect, Effect::Pure) && matches!(effect, Effect::Impure) {
            self.diagnostics.push(
                Diagnostic::error("pure context cannot call impure function", func.expr.span)
                    .with_id(DiagnosticId::TypePureCallsImpureFunction),
            );
            return None;
        }

        if let Some(assign) = func.assign {
            return self.apply_assignment_function(func, args, assign);
        }

        match self.apply_control_special_function(&func, &args) {
            SpecialApplyResult::Handled(result) => return result,
            SpecialApplyResult::NotHandled => {}
        }

        // General call or let/set
        if let HirExprKind::Var(name) | HirExprKind::FnValue(name) = &func.expr.kind {
            if crate::log::is_verbose() && name.contains("Result") {
                function_apply_log!(
                    "apply_function debug: callee={} type={} args=[{}] explicit_type_args=[{}]",
                    name,
                    self.ctx.type_to_string(func.ty),
                    args.iter()
                        .map(|arg| self.ctx.type_to_string(arg.ty))
                        .collect::<Vec<_>>()
                        .join(", "),
                    type_args
                        .iter()
                        .map(|ty| self.ctx.type_to_string(*ty))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            let symbol_resolved = matches!(&func.expr.kind, HirExprKind::FnValue(_));
            let callable_lookup =
                self.lookup_callable_apply_bindings(name, symbol_resolved, func.expr.span);
            if !callable_lookup.bindings.is_empty() && !callable_lookup.has_function_value_binding {
                {
                    let explicit_type_args = type_args.clone();
                    let binding = self.select_overload_candidate(
                        name,
                        &callable_lookup.bindings,
                        &args,
                        &explicit_type_args,
                        expected_ret,
                        func.expr.span,
                    )?;
                    return self.apply_selected_callable_function(
                        name,
                        binding,
                        func.expr.span,
                        args,
                        type_args,
                        expected_ret,
                        result,
                    );
                }
            }
        }

        if let HirExprKind::Var(name) = &func.expr.kind {
            if self.env.lookup_all_callables(name).is_empty() {
                match self.apply_unbound_trait_method_call(
                    name,
                    &params,
                    result,
                    effect,
                    &args,
                    &type_args,
                    expected_ret,
                    func.expr.span,
                ) {
                    TraitMethodApplyResult::Handled(result) => return result,
                    TraitMethodApplyResult::NotHandled => {}
                }
            } else if self.env.lookup_value(name).is_some() {
                if !matches!(self.ctx.get(func.ty), TypeKind::Function { .. }) {
                    self.diagnostics.push(
                        Diagnostic::error("variable is not callable", func.expr.span)
                            .with_id(DiagnosticId::TypeVariableNotCallable),
                    );
                    return None;
                }
            }
        }

        apply_indirect_function_call(self, func, args, result, expected_ret)
    }
}
