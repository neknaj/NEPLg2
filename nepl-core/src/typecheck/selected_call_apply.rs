use alloc::collections::BTreeMap;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::diagnostic_codes::{EffectDiagnosticCode, TypeDiagnosticCode};
use crate::function_identity::FunctionValueIdentity;
use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::constructor_apply::ConstructorApplyResult;
use super::diagnostics::{effect_error, type_error};
use super::env::{
    resolved_function_value_identity, Binding, BindingKind, FunctionValueIdentityReject,
};
use super::field_apply::FieldAccessorApplyResult;
use super::generic_call_constraints::{
    resolve_generic_type_args_from_constraints, GenericCallConstraint,
};
use super::memo_call::compiler_known_primitive_for_callable;
use super::signature::type_contains_unbound_var;
use super::trait_call_apply::TraitMethodResolution;
use super::traits::{insert_substitution_mapping, BoundEnv};
use super::type_expectation::TypeExpectation;
use super::{BlockChecker, StackEntry};

impl<'a> BlockChecker<'a> {
    pub(super) fn apply_selected_callable_function(
        &mut self,
        name: &str,
        binding: Binding,
        span: Span,
        mut args: Vec<StackEntry>,
        type_args: Vec<TypeId>,
        expected_ret: Option<TypeExpectation>,
        result: TypeId,
    ) -> Option<StackEntry> {
        let explicit_type_args = type_args.clone();
        let compiler_known_primitive =
            compiler_known_primitive_for_callable(self.source_map, name, &binding);
        let selected_field_accessor = match &binding.kind {
            BindingKind::Func { field_accessor, .. } => *field_accessor,
            _ => None,
        };
        let (selected_symbol, selected_builtin) = match &binding.kind {
            BindingKind::Func {
                symbol, builtin, ..
            } => (symbol.clone(), *builtin),
            _ => (name.to_string(), None),
        };
        let selected_def_id = match &binding.kind {
            BindingKind::Func { def_id, .. } => *def_id,
            _ => None,
        };
        let selected_type_param_bounds = match &binding.kind {
            BindingKind::Func {
                type_param_bounds, ..
            } => type_param_bounds.clone(),
            _ => BoundEnv::new(),
        };
        let declared_func_data = match self.ctx.get(binding.ty) {
            TypeKind::Function {
                type_params,
                params,
                result,
                effect,
            } => Some((type_params, params, result, effect)),
            _ => None,
        };
        let selected_type_snapshot = (!explicit_type_args.is_empty())
            .then(|| self.ctx.snapshot_type_var_bindings(binding.ty));
        let (inst_ty, mut resolved_args, type_arg_mapping) = if !explicit_type_args.is_empty() {
            let Some((type_params, params, result, effect)) = declared_func_data.clone() else {
                return None;
            };
            if type_params.len() != explicit_type_args.len() {
                self.diagnostics.push(type_error(
                    TypeDiagnosticCode::OverloadTypeArgsMismatch,
                    "type arguments do not match overload",
                    span,
                ));
                return None;
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
            (
                self.ctx
                    .function(Vec::new(), substituted_params, substituted_result, effect),
                explicit_type_args.clone(),
                mapping,
            )
        } else {
            self.ctx.instantiate(binding.ty)
        };

        let (c_params, c_result, c_effect) = match self.ctx.get(inst_ty) {
            TypeKind::Function {
                params,
                result,
                effect,
                ..
            } => (params, result, effect),
            _ => return None,
        };
        let captures = match &binding.kind {
            BindingKind::Func { captures, .. } => captures.clone(),
            _ => Vec::new(),
        };
        if c_params.len() < captures.len() {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::CallCaptureArityMismatch,
                "internal error: capture arity mismatch",
                span,
            ));
            return None;
        }
        let user_params = &c_params[captures.len()..];
        if user_params.len() != args.len() {
            self.diagnostics.push(type_error(
                TypeDiagnosticCode::ArgumentArityMismatch,
                "argument count mismatch",
                span,
            ));
            return None;
        }
        let declared_result = declared_func_data
            .as_ref()
            .map(|(_, _, result, _)| *result)
            .unwrap_or(c_result);
        let mut generic_constraints = Vec::new();
        if let Some(expectation) = expected_ret {
            let constraint = GenericCallConstraint::expected_result(
                declared_result,
                c_result,
                expectation,
                span,
            );
            if let Err(violation) = constraint.check(self.ctx) {
                self.diagnostics.push(type_error(
                    violation.diagnostic_code(),
                    violation.message(),
                    violation.span(),
                ));
                return None;
            }
            generic_constraints.push(constraint);
        }
        for (idx, (arg, param_ty)) in args.iter_mut().zip(user_params.iter()).enumerate() {
            match self.char_literal_context_type(arg, *param_ty) {
                Some(Ok(resolved)) => {
                    arg.ty = resolved;
                    arg.expr.ty = resolved;
                    continue;
                }
                Some(Err(())) => {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::ArgumentMismatch,
                        "argument type mismatch",
                        arg.expr.span,
                    ));
                    continue;
                }
                None => {}
            }
            let declared_param_ty = declared_func_data
                .as_ref()
                .and_then(|(_, params, _, _)| params.get(captures.len() + idx).copied())
                .unwrap_or(*param_ty);
            let constraint = GenericCallConstraint::argument(
                idx,
                declared_param_ty,
                *param_ty,
                arg.ty,
                arg.expr.span,
            );
            if let Err(violation) = constraint.check(self.ctx) {
                self.diagnostics.push(type_error(
                    violation.diagnostic_code(),
                    violation.message(),
                    violation.span(),
                ));
            }
            generic_constraints.push(constraint);
        }
        if matches!(self.current_effect, Effect::Pure) && matches!(c_effect, Effect::Impure) {
            self.diagnostics.push(effect_error(
                EffectDiagnosticCode::PureCallsImpure,
                "pure context cannot call impure function",
                span,
            ));
            return None;
        }

        let mut resolved_result = self.ctx.resolve_id(c_result);
        if explicit_type_args.is_empty() {
            if let Some((type_params, _, _, _)) = declared_func_data.as_ref() {
                let resolution = resolve_generic_type_args_from_constraints(
                    self.ctx,
                    type_params,
                    resolved_args,
                    &generic_constraints,
                );
                resolved_args = resolution.resolved_args;
                let has_conflicts = !resolution.conflicts.is_empty();
                for conflict in resolution.conflicts {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::GenericConstraintConflict,
                        conflict.diagnostic_message(self.ctx),
                        span,
                    ));
                }
                if !has_conflicts && type_params.len() == resolved_args.len() {
                    if let Some((type_params, _, declared_result, _)) = declared_func_data.as_ref()
                    {
                        let mut mapping = BTreeMap::new();
                        for (param, arg) in type_params.iter().zip(resolved_args.iter()) {
                            insert_substitution_mapping(self.ctx, &mut mapping, *param, *arg);
                        }
                        let substituted_result = self.ctx.substitute(*declared_result, &mapping);
                        resolved_result = self.ctx.resolve_id(substituted_result);
                    }
                }
            }
        }

        if let Some(snapshot) = &selected_type_snapshot {
            self.ctx.restore_type_var_bindings(snapshot);
        }

        if !selected_type_param_bounds.is_empty() {
            self.check_selected_function_trait_bounds(
                name,
                binding.ty,
                inst_ty,
                &selected_type_param_bounds,
                &type_arg_mapping,
                span,
            );
        }

        if let Some(field_accessor) = selected_field_accessor {
            match self.apply_field_accessor_function(field_accessor, &args, span) {
                FieldAccessorApplyResult::Handled(result) => return result,
                FieldAccessorApplyResult::NotHandled => {}
            }
        }

        match self.apply_constructor_function(
            name,
            &args,
            &c_params,
            &resolved_args,
            user_params,
            c_result,
            span,
        ) {
            ConstructorApplyResult::Handled(result) => return result,
            ConstructorApplyResult::NotHandled => {}
        }

        if let Some(primitive) = compiler_known_primitive {
            return self.apply_compiler_known_primitive(
                primitive,
                &args,
                resolved_result,
                span,
                expected_ret,
            );
        }

        let trait_resolution = self.resolve_selected_trait_method_call(
            name,
            &args,
            user_params,
            &type_args,
            result,
            expected_ret,
        );
        let callee = if selected_builtin.is_some() {
            FuncRef::Builtin(selected_symbol.clone())
        } else {
            match trait_resolution {
                TraitMethodResolution::Resolved(call) => call.into_func_ref(),
                TraitMethodResolution::NotTraitMethod
                | TraitMethodResolution::MissingSelfType
                | TraitMethodResolution::UnsatisfiedBound { .. }
                | TraitMethodResolution::ConstraintConflict { .. }
                | TraitMethodResolution::SelfTypeAmbiguous { .. }
                | TraitMethodResolution::PureCallsImpure => {
                    if !resolved_args.is_empty()
                        && resolved_args
                            .iter()
                            .all(|t| !type_contains_unbound_var(self.ctx, *t))
                    {
                        self.instantiations
                            .entry(selected_symbol.clone())
                            .or_insert_with(Vec::new)
                            .push(resolved_args.clone());
                    }
                    FuncRef::User(
                        selected_symbol.clone(),
                        resolved_args.clone(),
                        selected_def_id,
                    )
                }
            }
        };
        let mut final_args: Vec<HirExpr> = Vec::new();
        for (cap_name, cap_ty) in captures.iter() {
            let resolved_cap_ty = self
                .env
                .lookup_value(cap_name)
                .map(|b| self.ctx.resolve_id(b.ty))
                .unwrap_or(*cap_ty);
            final_args.push(HirExpr {
                ty: resolved_cap_ty,
                kind: HirExprKind::Var(cap_name.clone()),
                span,
            });
        }
        for (arg, param_ty) in args.into_iter().zip(user_params.iter()) {
            let arg_ty = arg.ty;
            let mut arg_expr = arg.expr;
            if let HirExprKind::Var(var_name) = &arg_expr.kind {
                if self.env.lookup_value(var_name).is_none() {
                    let callables = self.env.lookup_all_callables(var_name);
                    if !callables.is_empty() {
                        let mut matched_identity: Option<FunctionValueIdentity> = None;
                        let mut unresolved_identity = false;
                        let mut ambiguous = false;
                        for cb in callables {
                            let captures_len = match &cb.kind {
                                BindingKind::Func {
                                    captures,
                                    ..
                                } => captures.len(),
                                _ => continue,
                            };
                            if captures_len != 0 {
                                continue;
                            }
                            let checkpoint = self.ctx.checkpoint();
                            let (cand_ty, _fresh, _mapping) = self.ctx.instantiate(cb.ty);
                            let matched = self.ctx.unify(cand_ty, *param_ty).is_ok();
                            self.ctx.rollback(checkpoint);
                            if matched {
                                let identity = match resolved_function_value_identity(
                                    cb,
                                    arg_ty,
                                    Vec::new(),
                                ) {
                                    Ok(identity) => identity,
                                    Err(FunctionValueIdentityReject::UnresolvedIdentity) => {
                                        unresolved_identity = true;
                                        continue;
                                    }
                                    Err(FunctionValueIdentityReject::CapturingUnsupported) => {
                                        continue;
                                    }
                                    Err(FunctionValueIdentityReject::NotCallable) => continue,
                                };
                                if matched_identity.is_some() {
                                    ambiguous = true;
                                    break;
                                }
                                matched_identity = Some(identity);
                            }
                        }
                        if ambiguous {
                            self.diagnostics.push(type_error(
                                TypeDiagnosticCode::OverloadAmbiguous,
                                "ambiguous overload",
                                arg_expr.span,
                            ));
                            return None;
                        }
                        if unresolved_identity && matched_identity.is_none() {
                            self.diagnostics.push(type_error(
                                TypeDiagnosticCode::FunctionValueUnresolvedIdentity,
                                "function value requires a resolved named function identity",
                                arg_expr.span,
                            ));
                            return None;
                        }
                        if let Some(identity) = matched_identity {
                            arg_expr = HirExpr {
                                ty: arg_ty,
                                kind: HirExprKind::FnValue(identity),
                                span: arg_expr.span,
                            };
                        }
                    }
                }
            }
            final_args.push(arg_expr);
        }
        Some(StackEntry {
            ty: resolved_result,
            expr: HirExpr {
                ty: resolved_result,
                kind: HirExprKind::Call {
                    callee,
                    args: final_args,
                },
                span,
            },
            type_args: Vec::new(),
            assign: None,
            explicit_function_ref: false,
            auto_call: true,
        })
    }
}
