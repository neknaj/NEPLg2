use alloc::collections::BTreeMap;
use alloc::format;
use alloc::vec::Vec;

use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::binding_rules::function_user_param_specificity;
use super::diagnostics::type_error;
use super::env::{Binding, BindingKind};
use super::generic_call_constraints::{
    resolve_generic_type_args_from_constraints, GenericCallConstraint,
};
use super::overload_candidate::{
    OverloadCandidate, OverloadCandidateRejection, OverloadCandidateStats,
};
use super::overload_narrowing::narrow_overload_candidates;
use super::signature::function_signature_string;
use super::traits::insert_substitution_mapping;
use super::type_expectation::TypeExpectation;
use super::{BlockChecker, StackEntry};

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

fn result_may_satisfy_expectation(
    ctx: &mut TypeCtx,
    declared_result: TypeId,
    expectation: TypeExpectation,
) -> bool {
    let expected = expectation.target();
    // This is only a pre-materialization filter. Rollback-scoped unification
    // accepts cases where the declared generic result and the outer expected
    // result contain unknowns in different positions, while preserving the
    // actual TypeCtx state for the real candidate check below.
    let checkpoint = ctx.checkpoint();
    let may_satisfy = ctx.unify(declared_result, expected).is_ok()
        || ctx.type_pattern_matches(declared_result, expected)
        || ctx.type_pattern_matches(expected, declared_result);
    ctx.rollback(checkpoint);
    may_satisfy
}

impl<'a> BlockChecker<'a> {
    pub(super) fn select_overload_candidate(
        &mut self,
        name: &str,
        bindings: &[Binding],
        args: &[StackEntry],
        explicit_type_args: &[TypeId],
        expected_ret: Option<TypeExpectation>,
        span: Span,
    ) -> Option<Binding> {
        let use_expected = expected_ret.is_some() && bindings.len() > 1;
        if crate::log::is_verbose() && use_expected {
            if let Some(expectation) = expected_ret {
                overload_selection_log!(
                    "overload debug: '{}' using expected_ret={}",
                    name,
                    self.ctx.type_to_string(expectation.target())
                );
            }
        }

        let mut candidates: Vec<OverloadCandidate<'_>> = Vec::new();
        let mut mismatch_count = false;
        let mut first_generic_conflict = None;
        let mut stats = OverloadCandidateStats::default();
        for binding in bindings {
            stats.record_considered();
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
            let func_data = match self.ctx.get(binding.ty) {
                TypeKind::Function {
                    type_params,
                    params,
                    result,
                    effect,
                } => (type_params, params, result, effect),
                _ => {
                    if crate::log::is_verbose() {
                        overload_selection_log!(
                            "overload debug: skip '{}' candidate {} reason=not_function",
                            name,
                            function_signature_string(self.ctx, binding.ty)
                        );
                    }
                    stats.record_rejection(OverloadCandidateRejection::NotFunction);
                    continue;
                }
            };
            let (type_params, params, result, effect) = func_data;
            if !explicit_type_args.is_empty() && type_params.len() != explicit_type_args.len() {
                mismatch_count = true;
                stats.record_rejection(OverloadCandidateRejection::TypeArgumentCount);
                continue;
            }
            if params.len() < capture_len {
                if crate::log::is_verbose() {
                    overload_selection_log!(
                        "overload debug: skip '{}' candidate {} reason=capture_len params={} capture={}",
                        name,
                        function_signature_string(self.ctx, binding.ty),
                        params.len(),
                        capture_len
                    );
                }
                stats.record_rejection(OverloadCandidateRejection::CaptureArity);
                continue;
            }
            let declared_user_param_count = params.len() - capture_len;
            if declared_user_param_count != args.len() {
                if crate::log::is_verbose() {
                    overload_selection_log!(
                        "overload debug: skip '{}' candidate {} reason=arity user_params={} args={}",
                        name,
                        function_signature_string(self.ctx, binding.ty),
                        declared_user_param_count,
                        args.len()
                    );
                }
                stats.record_rejection(OverloadCandidateRejection::UserArity);
                continue;
            }
            if use_expected && explicit_type_args.is_empty() {
                if let Some(expectation) = expected_ret {
                    if !result_may_satisfy_expectation(self.ctx, result, expectation) {
                        if crate::log::is_verbose() {
                            overload_selection_log!(
                                "overload debug: skip '{}' candidate {} reason=declared_expected_ret result={} expected={}",
                                name,
                                function_signature_string(self.ctx, binding.ty),
                                self.ctx.type_to_string(result),
                                self.ctx.type_to_string(expectation.target())
                            );
                        }
                        stats.record_rejection(OverloadCandidateRejection::DeclaredExpectedResult);
                        continue;
                    }
                }
            }

            let checkpoint = self.ctx.checkpoint();
            stats.record_materialized();
            let (inst_ty, instantiated_type_args, type_arg_mapping) = if !explicit_type_args
                .is_empty()
            {
                let mut mapping = BTreeMap::new();
                for (p, a) in type_params.iter().zip(explicit_type_args.iter()) {
                    insert_substitution_mapping(self.ctx, &mut mapping, *p, *a);
                }
                let substituted_params = params
                    .iter()
                    .map(|p| self.ctx.substitute(*p, &mapping))
                    .collect::<Vec<_>>();
                let substituted_result = self.ctx.substitute(result, &mapping);
                (
                    self.ctx
                        .function(Vec::new(), substituted_params, substituted_result, effect),
                    Vec::new(),
                    mapping,
                )
            } else {
                self.ctx.instantiate(binding.ty)
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
                    stats.record_rejection(OverloadCandidateRejection::InstantiatedNotFunction);
                    self.ctx.rollback(checkpoint);
                    continue;
                }
            };
            let user_params = &c_params[capture_len..];
            let mut ok = true;
            let mut generic_constraints = Vec::new();
            if use_expected {
                if let Some(expectation) = expected_ret {
                    let constraint =
                        GenericCallConstraint::expected_result(result, c_result, expectation, span);
                    generic_constraints.push(constraint);
                    if let Err(_) = constraint.check(self.ctx) {
                        if crate::log::is_verbose() {
                            overload_selection_log!(
                                "overload debug: skip '{}' candidate {} reason=expected_ret result={} expected={}",
                                name,
                                function_signature_string(self.ctx, binding.ty),
                                self.ctx.type_to_string(c_result),
                                self.ctx.type_to_string(expectation.target())
                            );
                        }
                        stats.record_rejection(OverloadCandidateRejection::ExpectedResult);
                        ok = false;
                    }
                }
            }
            if ok {
                for (idx, (arg, pty)) in args.iter().zip(user_params.iter()).enumerate() {
                    let actual = match self.char_literal_context_type(arg, *pty) {
                        Some(Ok(resolved)) => resolved,
                        Some(Err(())) => {
                            if crate::log::is_verbose() {
                                overload_selection_log!(
                                    "overload debug: skip '{}' candidate {} reason=char_context arg={} param={}",
                                    name,
                                    function_signature_string(self.ctx, binding.ty),
                                    self.ctx.type_to_string(arg.ty),
                                    self.ctx.type_to_string(*pty)
                                );
                            }
                            stats.record_rejection(OverloadCandidateRejection::ArgumentType);
                            ok = false;
                            break;
                        }
                        None => arg.ty,
                    };
                    let declared_param_ty = params[capture_len + idx];
                    let constraint = GenericCallConstraint::argument(
                        idx,
                        declared_param_ty,
                        *pty,
                        actual,
                        arg.expr.span,
                    );
                    generic_constraints.push(constraint);
                    if let Err(_) = constraint.check(self.ctx) {
                        if crate::log::is_verbose() {
                            overload_selection_log!(
                                "overload debug: skip '{}' candidate {} reason=unify arg={} param={}",
                                name,
                                function_signature_string(self.ctx, binding.ty),
                                self.ctx.type_to_string(actual),
                                self.ctx.type_to_string(*pty)
                            );
                        }
                        stats.record_rejection(OverloadCandidateRejection::ArgumentType);
                        ok = false;
                        break;
                    }
                }
            }
            if explicit_type_args.is_empty() {
                let resolution = resolve_generic_type_args_from_constraints(
                    self.ctx,
                    &type_params,
                    instantiated_type_args,
                    &generic_constraints,
                );
                if let Some(conflict) = resolution.conflicts.first().copied() {
                    if crate::log::is_verbose() {
                        overload_selection_log!(
                            "overload debug: skip '{}' candidate {} reason=generic_constraint_conflict message={}",
                            name,
                            function_signature_string(self.ctx, binding.ty),
                            conflict.diagnostic_message(self.ctx)
                        );
                    }
                    if first_generic_conflict.is_none() {
                        first_generic_conflict = Some(conflict);
                    }
                    stats.record_rejection(OverloadCandidateRejection::GenericConstraintConflict);
                    ok = false;
                }
            }
            if ok && bindings.len() > 1 {
                let type_param_bounds = match &binding.kind {
                    BindingKind::Func {
                        type_param_bounds, ..
                    } => type_param_bounds,
                    _ => unreachable!("callable binding must be a function"),
                };
                if !self.selected_function_trait_bounds_may_satisfy(
                    binding.ty,
                    inst_ty,
                    type_param_bounds,
                    &type_arg_mapping,
                ) {
                    stats.record_rejection(OverloadCandidateRejection::TraitBoundUnsatisfied);
                    ok = false;
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
                stats.record_accepted();
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
        stats.assert_materialization_guard();
        if crate::log::is_verbose() {
            overload_selection_log!(
                "overload debug: '{}' stats considered={} materialized={} accepted={} pre_materialized_rejected={}",
                name,
                stats.considered,
                stats.materialized,
                stats.accepted,
                stats.pre_materialized_rejections()
            );
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
            if let Some(conflict) = first_generic_conflict {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::GenericConstraintConflict,
                    conflict.diagnostic_message(self.ctx),
                    span,
                ));
            } else if mismatch_count {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::OverloadTypeArgsMismatch,
                    "type arguments do not match any overload",
                    span,
                ));
            } else if stats.rejected_only_by_trait_bounds_after_materialization() {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::TraitBoundUnsatisfied,
                    "no overload candidate satisfies the required trait bounds",
                    span,
                ));
            } else {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::OverloadNoMatch,
                    "no matching overload found",
                    span,
                ));
            }
            return None;
        }

        match narrow_overload_candidates(self.ctx, self.current_effect, name, candidates) {
            Ok(candidate) => Some(candidate.binding.clone()),
            Err(ambiguity) => {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::OverloadAmbiguous,
                    ambiguity.diagnostic_message(),
                    span,
                ));
                None
            }
        }
    }
}
