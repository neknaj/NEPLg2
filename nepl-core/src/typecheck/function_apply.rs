use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::ast::{Effect, Ident};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::types::{TypeId, TypeKind};

use super::binding_rules::function_user_param_specificity;
use super::constructor_apply::ConstructorApplyResult;
use super::control_apply::SpecialApplyResult;
use super::env::{Binding, BindingKind};
use super::field_apply::FieldAccessorApplyResult;
use super::indirect_apply::apply_indirect_function_call;
use super::signature::{function_signature_string, type_contains_unbound_var};
use super::syntax_helpers::parse_variant_name;
use super::traits::{
    format_trait_ref_name, infer_instantiated_type_arg, insert_substitution_mapping,
    trait_application_matches, TraitBoundRef,
};
use super::{BlockChecker, FieldAccessorKind, StackEntry};

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
                self.env.lookup_all_callables_by_symbol(name)
            } else if let Some((_, qualified)) = &qualified_call {
                qualified.iter().collect()
            } else {
                self.env.lookup_all_callables(name)
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
                    let use_expected = expected_ret.is_some() && bindings.len() > 1;
                    if crate::log::is_verbose() && use_expected {
                        if let Some(expected) = expected_ret {
                            function_apply_log!(
                                "overload debug: '{}' using expected_ret={}",
                                name,
                                self.ctx.type_to_string(expected)
                            );
                        }
                    }
                    #[derive(Clone, Copy)]
                    struct OverloadCandidate<'b> {
                        binding: &'b Binding,
                        type_param_count: usize,
                        instantiated_specificity: usize,
                        declared_specificity: usize,
                        field_accessor: Option<FieldAccessorKind>,
                    }

                    let mut candidates: Vec<OverloadCandidate<'_>> = Vec::new();
                    let mut mismatch_count = false;
                    for binding in &bindings {
                        if crate::log::is_verbose() {
                            function_apply_log!(
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
                                    function_apply_log!(
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
                            self.ctx.function(
                                Vec::new(),
                                substituted_params,
                                substituted_result,
                                effect,
                            )
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
                                    function_apply_log!(
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
                                function_apply_log!(
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
                                function_apply_log!(
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
                                    function_apply_log!(
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
                                        function_apply_log!(
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
                                function_apply_log!(
                                    "overload debug: accept '{}' candidate {}",
                                    name,
                                    function_signature_string(self.ctx, binding.ty)
                                );
                            }
                            let type_param_count =
                                match self.ctx.get(self.ctx.resolve_id(binding.ty)) {
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
                    // prefer pure ones to avoid false D3025 from name collisions
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
                                .map(|b| {
                                    format!(
                                        "{}:{}",
                                        b.name,
                                        function_signature_string(self.ctx, b.ty)
                                    )
                                })
                                .collect::<Vec<_>>()
                                .join(" | ");
                            function_apply_log!(
                                "overload debug: no candidate for '{}' args=[{}] candidates=[{}]",
                                name,
                                arg_tys,
                                all
                            );
                        }
                        if mismatch_count {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "type arguments do not match any overload",
                                    func.expr.span,
                                )
                                .with_id(DiagnosticId::TypeOverloadTypeArgsMismatch),
                            );
                        } else {
                            self.diagnostics.push(
                                Diagnostic::error("no matching overload found", func.expr.span)
                                    .with_id(DiagnosticId::TypeNoMatchingOverload),
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
                                function_apply_log!(
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
                        self.diagnostics.push(
                            Diagnostic::error("ambiguous overload", func.expr.span)
                                .with_id(DiagnosticId::TypeAmbiguousOverload),
                        );
                        return None;
                    }

                    let binding = candidates[0].binding;
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

                    if let BindingKind::Func {
                        type_param_bounds, ..
                    } = &binding.kind
                    {
                        if !type_param_bounds.is_empty() {
                            for (tp, bounds) in type_param_bounds.iter() {
                                let Some(raw_arg) = type_arg_mapping.get(tp) else {
                                    continue;
                                };
                                let resolved_arg = self.ctx.resolve_id(*raw_arg);
                                for b in bounds {
                                    let substituted_trait_args = b
                                        .trait_args
                                        .iter()
                                        .map(|arg| self.ctx.substitute(*arg, &type_arg_mapping))
                                        .collect::<Vec<_>>();
                                    let substituted_bound = TraitBoundRef {
                                        name: format_trait_ref_name(
                                            &b.trait_base_name,
                                            &substituted_trait_args,
                                            self.ctx,
                                        ),
                                        trait_base_name: b.trait_base_name.clone(),
                                        trait_args: substituted_trait_args,
                                        trait_self_ty: self
                                            .ctx
                                            .substitute(b.trait_self_ty, &type_arg_mapping),
                                    };
                                    if crate::log::is_verbose() {
                                        function_apply_log!(
                                            "trait-bound debug: callee='{}' tp={} raw_arg={} resolved_arg={} bound={} current_bounds={}",
                                            name,
                                            self.ctx.type_to_string(*tp),
                                            self.ctx.type_to_string(*raw_arg),
                                            self.ctx.type_to_string(resolved_arg),
                                            substituted_bound.name,
                                            self.type_param_bounds
                                                .iter()
                                                .map(|(bound_tp, bs)| {
                                                    format!(
                                                        "{}:[{}]",
                                                        self.ctx.type_to_string(*bound_tp),
                                                        bs.iter()
                                                            .map(|bb| bb.name.clone())
                                                            .collect::<Vec<_>>()
                                                            .join("|")
                                                    )
                                                })
                                                .collect::<Vec<_>>()
                                                .join(", ")
                                        );
                                    }
                                    if self
                                        .trait_bound_satisfied_by_ref(&substituted_bound, *raw_arg)
                                        || self.trait_bound_satisfied_by_ref(
                                            &substituted_bound,
                                            resolved_arg,
                                        )
                                    {
                                        continue;
                                    }
                                    let inferred_arg = infer_instantiated_type_arg(
                                        self.ctx, binding.ty, inst_ty, *tp,
                                    )
                                    .unwrap_or(resolved_arg);
                                    if self.trait_bound_satisfied_by_ref(
                                        &substituted_bound,
                                        inferred_arg,
                                    ) {
                                        continue;
                                    }
                                    if self.is_concrete_type(inferred_arg) {
                                        self.diagnostics.push(
                                            Diagnostic::error(
                                                format!(
                                                    "type does not satisfy trait bound '{}'",
                                                    substituted_bound.name
                                                ),
                                                func.expr.span,
                                            )
                                            .with_id(DiagnosticId::TypeTraitBoundUnsatisfied),
                                        );
                                    } else {
                                        self.pending_trait_bound_checks.push((
                                            substituted_bound,
                                            inferred_arg,
                                            func.expr.span,
                                        ));
                                    }
                                }
                            }
                        }
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

                    let mut trait_callee: Option<FuncRef> = None;
                    if let Some((trait_name, method_name)) = parse_variant_name(name) {
                        if let Some(trait_info) = self.traits.get(trait_name) {
                            if let Some(sig) = trait_info.methods.get(method_name) {
                                let applied_trait_args = self.infer_trait_application_args(
                                    trait_info,
                                    *sig,
                                    &args,
                                    expected_ret,
                                );
                                let mut inferred_self_ty = None;
                                if let (Some(self_hint), Some(first_param), Some(arg)) = (
                                    type_args.first().copied(),
                                    user_params.first().copied(),
                                    args.first(),
                                ) {
                                    if self.ctx.same_type(first_param, self_hint) {
                                        let candidate = self.ctx.resolve_id(arg.ty);
                                        let candidate_ok = self.type_param_has_bound_ref(
                                            candidate,
                                            trait_name,
                                            &applied_trait_args,
                                        ) || self.impls.iter().any(|imp| {
                                            imp.trait_base_name.as_deref() == Some(trait_name)
                                                && imp.trait_args.len() == applied_trait_args.len()
                                                && trait_application_matches(
                                                    self.ctx,
                                                    trait_name,
                                                    &applied_trait_args,
                                                    trait_name,
                                                    &imp.trait_args,
                                                )
                                                && self
                                                    .ctx
                                                    .type_pattern_matches(imp.target_ty, candidate)
                                        });
                                        if candidate_ok {
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
                                            .infer_unique_type_param_for_trait_ref(
                                                trait_name,
                                                &applied_trait_args,
                                            )
                                            .or_else(|| {
                                                if self.type_param_has_bound_ref(
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
                                if let Some(self_ty) = inferred_self_ty {
                                    trait_callee = Some(FuncRef::Trait {
                                        trait_name: trait_name.to_string(),
                                        trait_args: applied_trait_args,
                                        method: method_name.to_string(),
                                        self_ty,
                                    });
                                }
                            }
                        }
                    }
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
                if let Some((trait_name, method_name)) = parse_variant_name(name) {
                    if let Some(trait_info) = self.traits.get(trait_name) {
                        if let Some(sig) = trait_info.methods.get(method_name) {
                            let applied_trait_name = self.infer_trait_application_name(
                                trait_name,
                                trait_info,
                                *sig,
                                &args,
                                expected_ret,
                            );
                            let applied_trait_args = self.infer_trait_application_args(
                                trait_info,
                                *sig,
                                &args,
                                expected_ret,
                            );
                            let mut inferred_self_ty = None;
                            if let (Some(self_hint), Some(first_param), Some(arg)) = (
                                type_args.first().copied(),
                                params.first().copied(),
                                args.first(),
                            ) {
                                if self.ctx.same_type(first_param, self_hint) {
                                    let candidate = self.ctx.resolve_id(arg.ty);
                                    let candidate_ok = self.type_param_has_bound_ref(
                                        candidate,
                                        trait_name,
                                        &applied_trait_args,
                                    ) || self.impls.iter().any(|imp| {
                                        imp.trait_base_name.as_deref() == Some(trait_name)
                                            && imp.trait_args.len() == applied_trait_args.len()
                                            && trait_application_matches(
                                                self.ctx,
                                                trait_name,
                                                &applied_trait_args,
                                                trait_name,
                                                &imp.trait_args,
                                            )
                                            && self
                                                .ctx
                                                .type_pattern_matches(imp.target_ty, candidate)
                                    });
                                    if candidate_ok {
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
                                        .infer_unique_type_param_for_trait_ref(
                                            trait_name,
                                            &applied_trait_args,
                                        )
                                        .or_else(|| {
                                            if self.type_param_has_bound_ref(
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
                            let Some(self_ty) = inferred_self_ty else {
                                self.diagnostics.push(Diagnostic::error(
                                    "trait method call requires receiver argument or expected self type",
                                    func.expr.span,
                                ).with_id(DiagnosticId::TypeTraitBoundUnsatisfied));
                                return None;
                            };
                            let trait_ok = self.type_param_has_bound_ref(
                                self_ty,
                                trait_name,
                                &applied_trait_args,
                            ) || self.impls.iter().any(|imp| {
                                imp.trait_base_name.as_deref() == Some(trait_name)
                                    && imp.trait_args.len() == applied_trait_args.len()
                                    && trait_application_matches(
                                        self.ctx,
                                        trait_name,
                                        &applied_trait_args,
                                        trait_name,
                                        &imp.trait_args,
                                    )
                                    && self.ctx.type_pattern_matches(imp.target_ty, self_ty)
                            });
                            if !trait_ok {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "type does not satisfy trait bound '{}'",
                                            applied_trait_name
                                        ),
                                        func.expr.span,
                                    )
                                    .with_id(DiagnosticId::TypeTraitBoundUnsatisfied),
                                );
                                return None;
                            }
                            if matches!(self.current_effect, Effect::Pure)
                                && matches!(effect, Effect::Impure)
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
                            let resolved_result = self.ctx.resolve_id(result);
                            return Some(StackEntry {
                                ty: resolved_result,
                                expr: HirExpr {
                                    ty: resolved_result,
                                    kind: HirExprKind::Call {
                                        callee: FuncRef::Trait {
                                            trait_name: trait_name.to_string(),
                                            trait_args: applied_trait_args,
                                            method: method_name.to_string(),
                                            self_ty,
                                        },
                                        args: args.into_iter().map(|a| a.expr).collect(),
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
