use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::binding_rules::function_user_param_specificity;
use super::diagnostics::type_error;
use super::env::{Binding, BindingKind};
use super::generic_call_constraints::{
    resolve_generic_type_args_from_constraints, GenericCallConstraint,
};
use super::signature::{function_signature_string, type_contains_unbound_var};
use super::traits::insert_substitution_mapping;
use super::type_expectation::TypeExpectation;
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

#[derive(Clone, Copy)]
enum OverloadCandidateRejection {
    NotFunction,
    TypeArgumentCount,
    CaptureArity,
    UserArity,
    DeclaredExpectedResult,
    InstantiatedNotFunction,
    ArgumentType,
    ExpectedResult,
    GenericConstraintConflict,
}

#[derive(Clone, Copy)]
enum OverloadCandidateMaterializationPhase {
    BeforeInstantiation,
    AfterInstantiation,
}

impl OverloadCandidateRejection {
    fn materialization_phase(self) -> OverloadCandidateMaterializationPhase {
        match self {
            OverloadCandidateRejection::NotFunction
            | OverloadCandidateRejection::TypeArgumentCount
            | OverloadCandidateRejection::CaptureArity
            | OverloadCandidateRejection::UserArity
            | OverloadCandidateRejection::DeclaredExpectedResult => {
                OverloadCandidateMaterializationPhase::BeforeInstantiation
            }
            OverloadCandidateRejection::InstantiatedNotFunction
            | OverloadCandidateRejection::ArgumentType
            | OverloadCandidateRejection::ExpectedResult
            | OverloadCandidateRejection::GenericConstraintConflict => {
                OverloadCandidateMaterializationPhase::AfterInstantiation
            }
        }
    }
}

#[derive(Default)]
struct OverloadCandidateStats {
    considered: usize,
    materialized: usize,
    accepted: usize,
    rejected_before_materialization: usize,
    rejected_after_materialization: usize,
    not_function: usize,
    type_argument_count: usize,
    capture_arity: usize,
    user_arity: usize,
    declared_expected_result: usize,
    instantiated_not_function: usize,
    argument_type: usize,
    expected_result: usize,
    generic_constraint_conflict: usize,
}

impl OverloadCandidateStats {
    fn record_considered(&mut self) {
        self.considered += 1;
    }

    fn record_materialized(&mut self) {
        self.materialized += 1;
    }

    fn record_accepted(&mut self) {
        self.accepted += 1;
    }

    fn record_rejection(&mut self, reason: OverloadCandidateRejection) {
        match reason.materialization_phase() {
            OverloadCandidateMaterializationPhase::BeforeInstantiation => {
                self.rejected_before_materialization += 1
            }
            OverloadCandidateMaterializationPhase::AfterInstantiation => {
                self.rejected_after_materialization += 1
            }
        }
        match reason {
            OverloadCandidateRejection::NotFunction => self.not_function += 1,
            OverloadCandidateRejection::TypeArgumentCount => self.type_argument_count += 1,
            OverloadCandidateRejection::CaptureArity => self.capture_arity += 1,
            OverloadCandidateRejection::UserArity => self.user_arity += 1,
            OverloadCandidateRejection::DeclaredExpectedResult => {
                self.declared_expected_result += 1
            }
            OverloadCandidateRejection::InstantiatedNotFunction => {
                self.instantiated_not_function += 1
            }
            OverloadCandidateRejection::ArgumentType => self.argument_type += 1,
            OverloadCandidateRejection::ExpectedResult => self.expected_result += 1,
            OverloadCandidateRejection::GenericConstraintConflict => {
                self.generic_constraint_conflict += 1
            }
        }
    }

    fn pre_materialized_rejections(&self) -> usize {
        self.rejected_before_materialization
    }

    fn assert_materialization_guard(&self) {
        debug_assert!(self.materialized + self.pre_materialized_rejections() <= self.considered);
    }
}

#[derive(Clone, Copy)]
enum OverloadCandidateNarrowingStage {
    InitialCandidates,
    PreferPureFunction,
    SignatureDedup,
    PreferOrdinaryFunction,
    PreferConcreteSignature,
    PreferFewerTypeParameters,
    PreferInstantiatedSpecificity,
    PreferDeclaredSpecificity,
}

impl OverloadCandidateNarrowingStage {
    fn diagnostic_label(self) -> &'static str {
        match self {
            OverloadCandidateNarrowingStage::InitialCandidates => "initial candidate filtering",
            OverloadCandidateNarrowingStage::PreferPureFunction => "pure function preference",
            OverloadCandidateNarrowingStage::SignatureDedup => "signature deduplication",
            OverloadCandidateNarrowingStage::PreferOrdinaryFunction => {
                "ordinary function preference"
            }
            OverloadCandidateNarrowingStage::PreferConcreteSignature => {
                "concrete signature preference"
            }
            OverloadCandidateNarrowingStage::PreferFewerTypeParameters => {
                "type parameter count preference"
            }
            OverloadCandidateNarrowingStage::PreferInstantiatedSpecificity => {
                "instantiated specificity preference"
            }
            OverloadCandidateNarrowingStage::PreferDeclaredSpecificity => {
                "declared specificity preference"
            }
        }
    }
}

#[derive(Clone, Copy)]
struct OverloadAmbiguityReason {
    after_stage: OverloadCandidateNarrowingStage,
    remaining_candidates: usize,
}

impl OverloadAmbiguityReason {
    fn after_stage(
        after_stage: OverloadCandidateNarrowingStage,
        remaining_candidates: usize,
    ) -> Self {
        Self {
            after_stage,
            remaining_candidates,
        }
    }

    fn diagnostic_message(self) -> String {
        format!(
            "ambiguous overload after {} ({} candidates remain)",
            self.after_stage.diagnostic_label(),
            self.remaining_candidates
        )
    }
}

fn result_may_satisfy_expectation(
    ctx: &TypeCtx,
    declared_result: TypeId,
    expectation: TypeExpectation,
) -> bool {
    let expected = expectation.target();
    ctx.type_pattern_matches(declared_result, expected)
        || ctx.type_pattern_matches(expected, declared_result)
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
            let (inst_ty, instantiated_type_args) = if !explicit_type_args.is_empty() {
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
                )
            } else {
                let (inst_ty, args, _mapping) = self.ctx.instantiate(binding.ty);
                (inst_ty, args)
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

        // In a pure context, if both pure and impure candidates match,
        // prefer pure ones to avoid false pure-call diagnostics from name collisions
        // between different modules' overloads of the same function.
        let mut last_narrowing_stage = OverloadCandidateNarrowingStage::InitialCandidates;
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
            last_narrowing_stage = OverloadCandidateNarrowingStage::PreferPureFunction;
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
            } else {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::OverloadNoMatch,
                    "no matching overload found",
                    span,
                ));
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
            last_narrowing_stage = OverloadCandidateNarrowingStage::SignatureDedup;
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
            last_narrowing_stage = OverloadCandidateNarrowingStage::PreferOrdinaryFunction;
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
            last_narrowing_stage = OverloadCandidateNarrowingStage::PreferConcreteSignature;
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
            last_narrowing_stage = OverloadCandidateNarrowingStage::PreferFewerTypeParameters;
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
            last_narrowing_stage = OverloadCandidateNarrowingStage::PreferInstantiatedSpecificity;
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
            last_narrowing_stage = OverloadCandidateNarrowingStage::PreferDeclaredSpecificity;
        }
        if candidates.len() > 1 {
            let ambiguity =
                OverloadAmbiguityReason::after_stage(last_narrowing_stage, candidates.len());
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::OverloadAmbiguous,
                ambiguity.diagnostic_message(),
                span,
            ));
            return None;
        }

        Some(candidates[0].binding.clone())
    }
}
