use alloc::format;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::diagnostic_codes::{EffectDiagnosticCode, TypeDiagnosticCode};
use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::span::Span;
use crate::types::TypeId;

use super::diagnostics::{effect_error, type_error};
use super::syntax_helpers::split_qualified_name;
use super::{BlockChecker, StackEntry};

pub(super) enum TraitMethodApplyResult {
    Handled(Option<StackEntry>),
    NotHandled,
}

pub(super) enum TraitMethodResolution {
    NotTraitMethod,
    Resolved(TraitMethodCall),
    MissingSelfType,
    UnsatisfiedBound {
        applied_trait_name: alloc::string::String,
    },
    PureCallsImpure,
}

pub(super) struct TraitMethodCall {
    trait_name: alloc::string::String,
    trait_args: Vec<TypeId>,
    method: alloc::string::String,
    self_ty: TypeId,
}

impl TraitMethodCall {
    pub(super) fn into_func_ref(self) -> FuncRef {
        FuncRef::Trait {
            trait_name: self.trait_name,
            trait_args: self.trait_args,
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
        expected_ret: Option<TypeId>,
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
        expected_ret: Option<TypeId>,
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

        let applied_trait_name =
            self.infer_trait_application_name(trait_name, trait_info, sig, args, expected_ret);
        let applied_trait_args =
            self.infer_trait_application_args(trait_info, sig, args, expected_ret);
        let mut inferred_self_ty = None;
        if let (Some(self_hint), Some(first_param), Some(arg)) = (
            type_args.first().copied(),
            params.first().copied(),
            args.first(),
        ) {
            if self.ctx.same_type(first_param, self_hint) {
                let candidate = self.ctx.resolve_id(arg.ty);
                if self.type_satisfies_trait_application(candidate, trait_name, &applied_trait_args)
                {
                    inferred_self_ty = Some(candidate);
                }
            }
        }
        if inferred_self_ty.is_none() {
            if let Some(self_hint) = type_args.first().copied() {
                if let Some(expected) = expected_ret {
                    let _ = self.ctx.unify(result, expected);
                }
                let resolved_hint = self.ctx.resolve_id(self_hint);
                inferred_self_ty = self
                    .infer_unique_type_param_for_trait_ref(trait_name, &applied_trait_args)
                    .or_else(|| {
                        if self.type_param_has_trait_application_bound(
                            resolved_hint,
                            trait_name,
                            &applied_trait_args,
                        ) {
                            Some(resolved_hint)
                        } else {
                            None
                        }
                    })
                    .or(Some(resolved_hint));
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

        if !self.type_satisfies_trait_application(self_ty, trait_name, &applied_trait_args) {
            return TraitMethodResolution::UnsatisfiedBound { applied_trait_name };
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
            trait_name: trait_name.to_string(),
            trait_args: applied_trait_args,
            method: method_name.to_string(),
            self_ty,
        })
    }

    pub(super) fn apply_unbound_trait_method_call(
        &mut self,
        name: &str,
        params: &[TypeId],
        result: TypeId,
        effect: Effect,
        args: &[StackEntry],
        type_args: &[TypeId],
        expected_ret: Option<TypeId>,
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
            TraitMethodResolution::UnsatisfiedBound { applied_trait_name } => {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::TraitBoundUnsatisfied,
                    format!("type does not satisfy trait bound '{}'", applied_trait_name),
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
        let resolved_result = self.ctx.resolve_id(result);
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
            auto_call: true,
        }))
    }

    fn type_satisfies_trait_application(
        &self,
        candidate: TypeId,
        trait_name: &str,
        applied_trait_args: &[TypeId],
    ) -> bool {
        self.type_param_has_trait_application_bound(candidate, trait_name, applied_trait_args)
            || self.impls.iter().any(|imp| {
                imp.matches_trait_application(self.ctx, trait_name, applied_trait_args)
                    && self.ctx.type_pattern_matches(imp.target_ty, candidate)
            })
    }
}
