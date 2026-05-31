use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::diagnostic_codes::{EffectDiagnosticCode, TypeDiagnosticCode};
use crate::hir::{FuncRef, HirExpr, HirExprKind, HirTraitApplication, HirTraitMethodId};
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::diagnostics::{effect_error, type_error};
use super::syntax_helpers::split_qualified_name;
use super::trait_check::{TraitSelfTypeAmbiguity, TraitSelfTypeInference};
use super::traits::{infer_type_param_from_instantiated_pair, TraitApplication, TraitId};
use super::type_argument_inference::TypeArgumentConflict;
use super::type_expectation::TypeExpectation;
use super::{BlockChecker, StackEntry};

pub(super) enum TraitMethodApplyResult {
    Handled(Option<StackEntry>),
    NotHandled,
}

pub(super) enum TraitMethodResolution {
    NotTraitMethod,
    Resolved(TraitMethodCall),
    MissingSelfType,
    UnsatisfiedBound { application: TraitApplication },
    ConstraintConflict { conflict: TypeArgumentConflict },
    SelfTypeAmbiguous { ambiguity: TraitSelfTypeAmbiguity },
    PureCallsImpure,
}

pub(super) struct TraitMethodCall {
    application: TraitApplication,
    method: HirTraitMethodId,
    self_ty: TypeId,
}

impl TraitMethodCall {
    pub(super) fn into_func_ref(self) -> FuncRef {
        let TraitApplication { trait_id, args } = self.application;
        FuncRef::Trait {
            application: HirTraitApplication::new(trait_id.as_str().to_string(), args),
            method: self.method,
            self_ty: self.self_ty,
        }
    }
}

impl<'a> BlockChecker<'a> {
    pub(super) fn resolve_selected_trait_method_call(
        &mut self,
        callee_name: &str,
        args: &[StackEntry],
        user_params: &[TypeId],
        type_args: &[TypeId],
        result: TypeId,
        expected_ret: Option<TypeExpectation>,
    ) -> TraitMethodResolution {
        self.resolve_trait_method_call(
            callee_name,
            user_params,
            result,
            None,
            args,
            type_args,
            expected_ret,
            false,
        )
    }

    fn resolve_trait_method_call(
        &mut self,
        name: &str,
        params: &[TypeId],
        result: TypeId,
        effect: Option<Effect>,
        args: &[StackEntry],
        type_args: &[TypeId],
        expected_ret: Option<TypeExpectation>,
        require_self_type: bool,
    ) -> TraitMethodResolution {
        let Some((trait_name, method_name)) = split_qualified_name(name) else {
            return TraitMethodResolution::NotTraitMethod;
        };
        let Some(trait_info) = self.traits.get(trait_name) else {
            return TraitMethodResolution::NotTraitMethod;
        };
        let Some(sig) = trait_info.methods.get(method_name).copied() else {
            return TraitMethodResolution::NotTraitMethod;
        };

        let application_args =
            self.resolve_trait_application_args(trait_info, sig, args, expected_ret);
        if let Some(conflict) = application_args.conflicts.first().copied() {
            return TraitMethodResolution::ConstraintConflict { conflict };
        }

        let application = TraitApplication {
            trait_id: TraitId::from_name(trait_name),
            args: application_args.resolved_args,
        };
        let self_inference_target = type_args.first().copied().unwrap_or(trait_info.self_ty);
        let mut inferred_self_ty =
            self.infer_trait_method_self_ty_from_args(self_inference_target, params, args);
        if inferred_self_ty.is_none() {
            if let Some(self_hint) = type_args.first().copied() {
                if let Some(expectation) = expected_ret {
                    let _ = self.ctx.unify(result, expectation.target());
                }
                let resolved_hint = self.ctx.resolve_id(self_hint);
                inferred_self_ty = match self
                    .resolve_self_type_param_for_trait_ref(&application.trait_id, &application.args)
                {
                    TraitSelfTypeInference::NoEvidence => {
                        if self.is_unbound_type_var(resolved_hint) {
                            None
                        } else {
                            Some(resolved_hint)
                        }
                    }
                    TraitSelfTypeInference::Unique(self_ty) => Some(self_ty),
                    TraitSelfTypeInference::Ambiguous(ambiguity) => {
                        return TraitMethodResolution::SelfTypeAmbiguous { ambiguity };
                    }
                };
            }
        }
        if inferred_self_ty.is_none() {
            if let Some(first) = args.first() {
                inferred_self_ty = Some(self.ctx.resolve_id(first.ty));
            }
        }
        let Some(self_ty) = inferred_self_ty else {
            return if require_self_type {
                TraitMethodResolution::MissingSelfType
            } else {
                TraitMethodResolution::NotTraitMethod
            };
        };

        if !self.type_satisfies_trait_application(self_ty, &application) {
            return TraitMethodResolution::UnsatisfiedBound { application };
        }

        if effect
            .map(|effect| {
                matches!(self.current_effect, Effect::Pure) && matches!(effect, Effect::Impure)
            })
            .unwrap_or(false)
        {
            return TraitMethodResolution::PureCallsImpure;
        }

        TraitMethodResolution::Resolved(TraitMethodCall {
            application,
            method: HirTraitMethodId::from_name(method_name.to_string()),
            self_ty,
        })
    }

    fn infer_trait_method_self_ty_from_args(
        &self,
        self_target: TypeId,
        params: &[TypeId],
        args: &[StackEntry],
    ) -> Option<TypeId> {
        let resolved_target = self.ctx.resolve_id(self_target);
        let target_label = match self.ctx.get(resolved_target) {
            TypeKind::Var(var) => var.label,
            _ => None,
        };
        let mut inferred = None;
        for (param, arg) in params.iter().zip(args.iter()) {
            let Some(candidate) = infer_type_param_from_instantiated_pair(
                self.ctx,
                *param,
                arg.ty,
                resolved_target,
                target_label.as_deref(),
            ) else {
                continue;
            };
            let candidate = self.ctx.resolve_id(candidate);
            inferred = match inferred {
                None => Some(candidate),
                Some(current) if self.ctx.same_type(current, candidate) => Some(current),
                Some(_) => return None,
            };
        }
        inferred
    }

    fn is_unbound_type_var(&self, ty: TypeId) -> bool {
        matches!(
            self.ctx.get(self.ctx.resolve_id(ty)),
            TypeKind::Var(var) if var.binding.is_none()
        )
    }

    pub(super) fn apply_unbound_trait_method_call(
        &mut self,
        name: &str,
        params: &[TypeId],
        result: TypeId,
        effect: Effect,
        args: &[StackEntry],
        type_args: &[TypeId],
        expected_ret: Option<TypeExpectation>,
        span: Span,
    ) -> TraitMethodApplyResult {
        let call = match self.resolve_trait_method_call(
            name,
            params,
            result,
            Some(effect),
            args,
            type_args,
            expected_ret,
            true,
        ) {
            TraitMethodResolution::NotTraitMethod => return TraitMethodApplyResult::NotHandled,
            TraitMethodResolution::Resolved(call) => call,
            TraitMethodResolution::MissingSelfType => {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::TraitBoundUnsatisfied,
                    "trait method call requires receiver argument or expected self type",
                    span,
                ));
                return TraitMethodApplyResult::Handled(None);
            }
            TraitMethodResolution::UnsatisfiedBound { application } => {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::TraitBoundUnsatisfied,
                    format!(
                        "type does not satisfy trait bound '{}'",
                        application.display_name(self.ctx)
                    ),
                    span,
                ));
                return TraitMethodApplyResult::Handled(None);
            }
            TraitMethodResolution::ConstraintConflict { conflict } => {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::TraitConstraintConflict,
                    conflict.diagnostic_message(self.ctx),
                    span,
                ));
                return TraitMethodApplyResult::Handled(None);
            }
            TraitMethodResolution::SelfTypeAmbiguous { ambiguity } => {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::TraitSelfTypeAmbiguous,
                    ambiguity.diagnostic_message(self),
                    span,
                ));
                return TraitMethodApplyResult::Handled(None);
            }
            TraitMethodResolution::PureCallsImpure => {
                self.diagnostics.push(effect_error(
                    EffectDiagnosticCode::PureCallsImpure,
                    "pure context cannot call impure function",
                    span,
                ));
                return TraitMethodApplyResult::Handled(None);
            }
        };
        let mut resolved_result = self.ctx.resolve_id(result);
        if let Some(expectation) = expected_ret {
            if self
                .ctx
                .unify(resolved_result, expectation.target())
                .is_ok()
            {
                resolved_result = self.ctx.resolve_id(expectation.target());
            } else {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::AnnotationMismatch,
                    "call result does not match expected type",
                    expectation.diagnostic_span(span),
                ));
                return TraitMethodApplyResult::Handled(None);
            }
        }
        TraitMethodApplyResult::Handled(Some(StackEntry {
            ty: resolved_result,
            expr: HirExpr {
                ty: resolved_result,
                kind: HirExprKind::Call {
                    callee: call.into_func_ref(),
                    args: args.iter().cloned().map(|a| a.expr).collect(),
                },
                span,
            },
            type_args: Vec::new(),
            assign: None,
            explicit_function_ref: false,
            auto_call: true,
        }))
    }

    fn type_satisfies_trait_application(
        &self,
        candidate: TypeId,
        application: &TraitApplication,
    ) -> bool {
        self.type_param_has_trait_application_bound(
            candidate,
            &application.trait_id,
            &application.args,
        ) || self.impls.iter().any(|imp| {
            imp.matches_trait_application(self.ctx, &application.trait_id, &application.args)
                && self.ctx.type_pattern_matches(imp.target_ty, candidate)
        })
    }
}
