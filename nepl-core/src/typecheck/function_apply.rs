use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{Effect, Ident};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::types::{TypeId, TypeKind};

use super::constructor_apply::ConstructorApplyResult;
use super::control_apply::SpecialApplyResult;
use super::env::BindingKind;
use super::field_apply::FieldAccessorApplyResult;
use super::indirect_apply::apply_indirect_function_call;
use super::signature::type_contains_unbound_var;
use super::trait_call_apply::TraitMethodApplyResult;
use super::traits::{infer_instantiated_type_arg, insert_substitution_mapping};
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
            let qualified_call = if symbol_resolved {
                None
            } else {
                self.lookup_qualified_bindings(&Ident {
                    name: name.clone(),
                    span: func.expr.span,
                })
            };
            let bindings = if symbol_resolved {
                self.env
                    .lookup_all_callables_by_symbol(name)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
            } else if let Some((_, qualified)) = &qualified_call {
                qualified.clone()
            } else {
                self.env
                    .lookup_all_callables(name)
                    .into_iter()
                    .cloned()
                    .collect::<Vec<_>>()
            };
            let has_function_value_binding = if symbol_resolved {
                false
            } else if qualified_call.is_some() {
                false
            } else {
                self.env
                    .lookup_value(name)
                    .map(|b| {
                        let rty = self.ctx.resolve_id(b.ty);
                        matches!(self.ctx.get(rty), TypeKind::Function { .. })
                    })
                    .unwrap_or(false)
            };
            if !bindings.is_empty() && !has_function_value_binding {
                {
                    let explicit_type_args = type_args.clone();
                    let binding = self.select_overload_candidate(
                        name,
                        &bindings,
                        &args,
                        &explicit_type_args,
                        expected_ret,
                        func.expr.span,
                    )?;
                    let selected_field_accessor = match &binding.kind {
                        BindingKind::Func { field_accessor, .. } => *field_accessor,
                        _ => None,
                    };
                    let (selected_symbol, selected_builtin) = match &binding.kind {
                        BindingKind::Func {
                            symbol, builtin, ..
                        } => (symbol.clone(), *builtin),
                        _ => (name.clone(), None),
                    };
                    let selected_def_id = match &binding.kind {
                        BindingKind::Func { def_id, .. } => *def_id,
                        _ => None,
                    };
                    let selected_type_param_bounds = match &binding.kind {
                        BindingKind::Func {
                            type_param_bounds, ..
                        } => type_param_bounds.clone(),
                        _ => BTreeMap::new(),
                    };
                    let selected_type_snapshot = (!explicit_type_args.is_empty())
                        .then(|| self.ctx.snapshot_type_var_bindings(binding.ty));
                    let (inst_ty, mut resolved_args, type_arg_mapping) =
                        if !explicit_type_args.is_empty() {
                            let func_data = if let TypeKind::Function {
                                type_params,
                                params,
                                result,
                                effect,
                            } = self.ctx.get(binding.ty)
                            {
                                Some((type_params.clone(), params.clone(), result, effect))
                            } else {
                                None
                            };
                            let Some((type_params, params, result, effect)) = func_data else {
                                return None;
                            };
                            if type_params.len() != explicit_type_args.len() {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "type arguments do not match overload",
                                        func.expr.span,
                                    )
                                    .with_id(DiagnosticId::TypeOverloadTypeArgsMismatch),
                                );
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
                                self.ctx.function(
                                    Vec::new(),
                                    substituted_params,
                                    substituted_result,
                                    effect,
                                ),
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
                        self.diagnostics.push(Diagnostic::error(
                            "internal error: capture arity mismatch",
                            func.expr.span,
                        ));
                        return None;
                    }
                    let user_params = &c_params[captures.len()..];
                    if user_params.len() != args.len() {
                        self.diagnostics.push(
                            Diagnostic::error("argument count mismatch", func.expr.span)
                                .with_id(DiagnosticId::TypeArgumentArityMismatch),
                        );
                        return None;
                    }
                    for (arg, param_ty) in args.iter_mut().zip(user_params.iter()) {
                        match self.char_literal_context_type(arg, *param_ty) {
                            Some(Ok(resolved)) => {
                                arg.ty = resolved;
                                arg.expr.ty = resolved;
                                continue;
                            }
                            Some(Err(())) => {
                                self.diagnostics.push(
                                    Diagnostic::error("argument type mismatch", arg.expr.span)
                                        .with_id(DiagnosticId::TypeArgumentTypeMismatch),
                                );
                                continue;
                            }
                            None => {}
                        }
                        if self.ctx.unify(arg.ty, *param_ty).is_err() {
                            self.diagnostics.push(
                                Diagnostic::error("argument type mismatch", arg.expr.span)
                                    .with_id(DiagnosticId::TypeArgumentTypeMismatch),
                            );
                        }
                    }
                    if matches!(self.current_effect, Effect::Pure)
                        && matches!(c_effect, Effect::Impure)
                    {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "pure context cannot call impure function",
                                func.expr.span,
                            )
                            .with_id(DiagnosticId::TypePureCallsImpureFunction),
                        );
                        return None;
                    }

                    if explicit_type_args.is_empty() {
                        resolved_args = resolved_args
                            .into_iter()
                            .map(|t| self.ctx.resolve_id(t))
                            .collect();
                        if let TypeKind::Function { type_params, .. } = self.ctx.get(binding.ty) {
                            if type_params.len() == resolved_args.len() {
                                for (idx, tp) in type_params.iter().enumerate() {
                                    if let Some(inferred) = infer_instantiated_type_arg(
                                        self.ctx, binding.ty, inst_ty, *tp,
                                    ) {
                                        resolved_args[idx] = self.ctx.resolve_id(inferred);
                                    }
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
                            func.expr.span,
                        );
                    }

                    if let Some(field_accessor) = selected_field_accessor {
                        match self.apply_field_accessor_function(
                            field_accessor,
                            &args,
                            func.expr.span,
                        ) {
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
                        func.expr.span,
                    ) {
                        ConstructorApplyResult::Handled(result) => return result,
                        ConstructorApplyResult::NotHandled => {}
                    }

                    let trait_callee = self.infer_selected_trait_method_callee(
                        name,
                        &args,
                        user_params,
                        &type_args,
                        result,
                        expected_ret,
                    );
                    let callee = if selected_builtin.is_some() {
                        FuncRef::Builtin(selected_symbol.clone())
                    } else if let Some(tc) = trait_callee {
                        tc
                    } else {
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
                            span: func.expr.span,
                        });
                    }
                    for (arg, param_ty) in args.into_iter().zip(user_params.iter()) {
                        let arg_ty = arg.ty;
                        let mut arg_expr = arg.expr;
                        if let HirExprKind::Var(var_name) = &arg_expr.kind {
                            if self.env.lookup_value(var_name).is_none() {
                                let callables = self.env.lookup_all_callables(var_name);
                                if !callables.is_empty() {
                                    let mut matched_symbol: Option<String> = None;
                                    let mut ambiguous = false;
                                    for cb in callables {
                                        let (symbol, captures_len) = match &cb.kind {
                                            BindingKind::Func {
                                                symbol, captures, ..
                                            } => (symbol.clone(), captures.len()),
                                            _ => continue,
                                        };
                                        if captures_len != 0 {
                                            continue;
                                        }
                                        let checkpoint = self.ctx.checkpoint();
                                        let (cand_ty, _fresh, _mapping) =
                                            self.ctx.instantiate(cb.ty);
                                        let matched = self.ctx.unify(cand_ty, *param_ty).is_ok();
                                        self.ctx.rollback(checkpoint);
                                        if matched {
                                            if matched_symbol.is_some() {
                                                ambiguous = true;
                                                break;
                                            }
                                            matched_symbol = Some(symbol);
                                        }
                                    }
                                    if ambiguous {
                                        self.diagnostics.push(
                                            Diagnostic::error("ambiguous overload", arg_expr.span)
                                                .with_id(DiagnosticId::TypeAmbiguousOverload),
                                        );
                                        return None;
                                    }
                                    if let Some(symbol) = matched_symbol {
                                        arg_expr = HirExpr {
                                            ty: arg_ty,
                                            kind: HirExprKind::FnValue(symbol),
                                            span: arg_expr.span,
                                        };
                                    }
                                }
                            }
                        }
                        final_args.push(arg_expr);
                    }
                    let resolved_result = self.ctx.resolve_id(c_result);
                    return Some(StackEntry {
                        ty: resolved_result,
                        expr: HirExpr {
                            ty: resolved_result,
                            kind: HirExprKind::Call {
                                callee,
                                args: final_args,
                            },
                            span: func.expr.span,
                        },
                        type_args: Vec::new(),
                        assign: None,
                        auto_call: true,
                    });
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
