use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::DiagnosticCode;
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::binding_rules::function_user_param_specificity;
use super::env::{Binding, BindingKind};
use super::signature::{function_signature_string, type_contains_unbound_var};
use super::traits::insert_substitution_mapping;
use super::{BlockChecker, FieldAccessorKind, StackEntry};

macro_rules! overload_selection_log {
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

#[derive(Clone, Copy)]
struct OverloadCandidate<'b> {
    binding: &'b Binding,
    type_param_count: usize,
    instantiated_specificity: usize,
    declared_specificity: usize,
    field_accessor: Option<FieldAccessorKind>,
}

impl<'a> BlockChecker<'a> {
    pub(super) fn select_overload_candidate(
        &mut self,
        name: &str,
        bindings: &[Binding],
        args: &[StackEntry],
        explicit_type_args: &[TypeId],
        expected_ret: Option<TypeId>,
        span: Span,
    ) -> Option<Binding> {
        let use_expected = expected_ret.is_some() && bindings.len() > 1;
        if crate::log::is_verbose() && use_expected {
            if let Some(expected) = expected_ret {
                overload_selection_log!(
                    "overload debug: '{}' using expected_ret={}",
                    name,
                    self.ctx.type_to_string(expected)
                );
            }
        }

        let mut candidates: Vec<OverloadCandidate<'_>> = Vec::new();
        let mut mismatch_count = false;
        for binding in bindings {
            if crate::log::is_verbose() {
                overload_selection_log!(
                    "overload debug: consider '{}' candidate {}",
                    name,
                    function_signature_string(self.ctx, binding.ty)
                );
            }
            let capture_len = match &binding.kind {
                BindingKind::Func { captures, .. } => captures.len(),
                _ => 0,
            };
            let checkpoint = self.ctx.checkpoint();
            let inst_ty = if !explicit_type_args.is_empty() {
                let func_data = if let TypeKind::Function {
                    type_params,
                    params,
                    result,
                    effect,
                } = self.ctx.get(binding.ty)
                {
                    Some((type_params, params, result, effect))
                } else {
                    None
                };
                let Some((type_params, params, result, effect)) = func_data else {
                    if crate::log::is_verbose() {
                        overload_selection_log!(
                            "overload debug: skip '{}' candidate {} reason=not_function_after_type_args",
                            name,
                            function_signature_string(self.ctx, binding.ty)
                        );
                    }
                    self.ctx.rollback(checkpoint);
                    continue;
                };
                if type_params.len() != explicit_type_args.len() {
                    mismatch_count = true;
                    self.ctx.rollback(checkpoint);
                    continue;
                }
                let mut mapping = BTreeMap::new();
                for (p, a) in type_params.iter().zip(explicit_type_args.iter()) {
                    insert_substitution_mapping(self.ctx, &mut mapping, *p, *a);
                }
                let substituted_params = params
                    .iter()
                    .map(|p| self.ctx.substitute(*p, &mapping))
                    .collect::<Vec<_>>();
                let substituted_result = self.ctx.substitute(result, &mapping);
                self.ctx
                    .function(Vec::new(), substituted_params, substituted_result, effect)
            } else {
                let (inst_ty, _args, _mapping) = self.ctx.instantiate(binding.ty);
                inst_ty
            };

            let func_ty = self.ctx.get(inst_ty);
            let (c_params, c_result, _c_effect) = match func_ty {
                TypeKind::Function {
                    params,
                    result,
                    effect,
                    ..
                } => (params, result, effect),
                _ => {
                    if crate::log::is_verbose() {
                        overload_selection_log!(
                            "overload debug: skip '{}' candidate {} reason=not_function_instantiated",
                            name,
                            function_signature_string(self.ctx, binding.ty)
                        );
                    }
                    self.ctx.rollback(checkpoint);
                    continue;
                }
            };
            if c_params.len() < capture_len {
                if crate::log::is_verbose() {
                    overload_selection_log!(
                        "overload debug: skip '{}' candidate {} reason=capture_len params={} capture={}",
                        name,
                        function_signature_string(self.ctx, binding.ty),
                        c_params.len(),
                        capture_len
                    );
                }
                self.ctx.rollback(checkpoint);
                continue;
            }
            let user_params = &c_params[capture_len..];
            if user_params.len() != args.len() {
                if crate::log::is_verbose() {
                    overload_selection_log!(
                        "overload debug: skip '{}' candidate {} reason=arity user_params={} args={}",
                        name,
                        function_signature_string(self.ctx, binding.ty),
                        user_params.len(),
                        args.len()
                    );
                }
                self.ctx.rollback(checkpoint);
                continue;
            }
            let mut ok = true;
            for (arg, pty) in args.iter().zip(user_params.iter()) {
                if !self.char_literal_matches_context(arg, *pty)
                    && self.ctx.unify(arg.ty, *pty).is_err()
                {
                    if crate::log::is_verbose() {
                        overload_selection_log!(
                            "overload debug: skip '{}' candidate {} reason=unify arg={} param={}",
                            name,
                            function_signature_string(self.ctx, binding.ty),
                            self.ctx.type_to_string(arg.ty),
                            self.ctx.type_to_string(*pty)
                        );
                    }
                    ok = false;
                    break;
                }
            }
            if ok && use_expected {
                if let Some(expected) = expected_ret {
                    if self.ctx.unify(c_result, expected).is_err() {
                        if crate::log::is_verbose() {
                            overload_selection_log!(
                                "overload debug: skip '{}' candidate {} reason=expected_ret result={} expected={}",
                                name,
                                function_signature_string(self.ctx, binding.ty),
                                self.ctx.type_to_string(c_result),
                                self.ctx.type_to_string(expected)
                            );
                        }
                        ok = false;
                    }
                }
            }
            if ok {
                if crate::log::is_verbose() {
                    overload_selection_log!(
                        "overload debug: accept '{}' candidate {}",
                        name,
                        function_signature_string(self.ctx, binding.ty)
                    );
                }
                let type_param_count = match self.ctx.get(self.ctx.resolve_id(binding.ty)) {
                    TypeKind::Function { type_params, .. } => type_params.len(),
                    _ => 0,
                };
                let instantiated_specificity =
                    function_user_param_specificity(self.ctx, inst_ty, args.len());
                let declared_specificity =
                    function_user_param_specificity(self.ctx, binding.ty, args.len());
                candidates.push(OverloadCandidate {
                    binding,
                    type_param_count,
                    instantiated_specificity,
                    declared_specificity,
                    field_accessor: match &binding.kind {
                        BindingKind::Func { field_accessor, .. } => *field_accessor,
                        _ => None,
                    },
                });
            }
            self.ctx.rollback(checkpoint);
        }

        // In a pure context, if both pure and impure candidates match,
        // prefer pure ones to avoid false pure-call diagnostics from name collisions
        // between different modules' overloads of the same function.
        if candidates.len() > 1 && matches!(self.current_effect, Effect::Pure) {
            let pure_only: Vec<OverloadCandidate<'_>> = candidates
                .iter()
                .filter(|c| {
                    matches!(
                        self.ctx.get(c.binding.ty),
                        TypeKind::Function {
                            effect: Effect::Pure,
                            ..
                        }
                    )
                })
                .cloned()
                .collect();
            if !pure_only.is_empty() {
                candidates = pure_only;
            }
        }

        if candidates.is_empty() {
            if crate::log::is_verbose() {
                let arg_tys = args
                    .iter()
                    .map(|a| self.ctx.type_to_string(a.ty))
                    .collect::<Vec<_>>()
                    .join(", ");
                let all = bindings
                    .iter()
                    .map(|b| format!("{}:{}", b.name, function_signature_string(self.ctx, b.ty)))
                    .collect::<Vec<_>>()
                    .join(" | ");
                overload_selection_log!(
                    "overload debug: no candidate for '{}' args=[{}] candidates=[{}]",
                    name,
                    arg_tys,
                    all
                );
            }
            if mismatch_count {
                self.diagnostics.push(
                    Diagnostic::error("type arguments do not match any overload", span).with_code(
                        DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::OverloadTypeArgsMismatch,
                        ),
                    ),
                );
            } else {
                self.diagnostics.push(
                    Diagnostic::error("no matching overload found", span).with_code(
                        DiagnosticCode::Type(
                            crate::diagnostic_codes::TypeDiagnosticCode::OverloadNoMatch,
                        ),
                    ),
                );
            }
            return None;
        }
        if candidates.len() > 1 {
            let mut sig_seen: BTreeSet<String> = BTreeSet::new();
            let mut dedup: Vec<OverloadCandidate<'_>> = Vec::new();
            for c in candidates {
                let sig = function_signature_string(self.ctx, c.binding.ty);
                if sig_seen.insert(sig) {
                    dedup.push(c);
                }
            }
            candidates = dedup;
        }
        if candidates.len() > 1 {
            let ordinary: Vec<OverloadCandidate<'_>> = candidates
                .iter()
                .filter(|b| b.field_accessor.is_none())
                .cloned()
                .collect();
            if !ordinary.is_empty() {
                candidates = ordinary;
            }
        }
        if candidates.len() > 1 {
            let concrete: Vec<OverloadCandidate<'_>> = candidates
                .iter()
                .filter(|b| !type_contains_unbound_var(self.ctx, b.binding.ty))
                .cloned()
                .collect();
            if !concrete.is_empty() {
                candidates = concrete;
            }
        }
        if candidates.len() > 1 {
            let min_type_params = candidates
                .iter()
                .map(|b| b.type_param_count)
                .min()
                .unwrap_or(0);
            let narrowed: Vec<OverloadCandidate<'_>> = candidates
                .into_iter()
                .filter(|b| b.type_param_count == min_type_params)
                .collect();
            candidates = narrowed;
        }
        if candidates.len() > 1 {
            if crate::log::is_verbose() {
                for candidate in &candidates {
                    overload_selection_log!(
                        "overload debug: specificity '{}' candidate {} score={}",
                        name,
                        function_signature_string(self.ctx, candidate.binding.ty),
                        candidate.instantiated_specificity
                    );
                }
            }
            let max_specificity = candidates
                .iter()
                .map(|b| b.instantiated_specificity)
                .max()
                .unwrap_or(0);
            let narrowed: Vec<OverloadCandidate<'_>> = candidates
                .into_iter()
                .filter(|b| b.instantiated_specificity == max_specificity)
                .collect();
            candidates = narrowed;
        }
        if candidates.len() > 1 {
            let max_declared_specificity = candidates
                .iter()
                .map(|b| b.declared_specificity)
                .max()
                .unwrap_or(0);
            let narrowed: Vec<OverloadCandidate<'_>> = candidates
                .into_iter()
                .filter(|b| b.declared_specificity == max_declared_specificity)
                .collect();
            candidates = narrowed;
        }
        if candidates.len() > 1 {
            self.diagnostics
                .push(Diagnostic::error("ambiguous overload", span).with_code(
                    DiagnosticCode::Type(
                        crate::diagnostic_codes::TypeDiagnosticCode::OverloadAmbiguous,
                    ),
                ));
            return None;
        }

        Some(candidates[0].binding.clone())
    }
}
