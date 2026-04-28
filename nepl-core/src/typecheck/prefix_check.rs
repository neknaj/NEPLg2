use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::{Effect, Ident, Literal, PrefixExpr, PrefixItem, Symbol};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::effects::intrinsic_effect;
use crate::hir::{HirExpr, HirExprKind};
use crate::types::{TypeId, TypeKind};

use super::binding_rules::{emit_shadow_warning, shadow_blocked_by_nonshadow};
use super::env::{Binding, BindingKind};
use super::syntax_helpers::{parse_i32_literal, parse_variant_name};
use super::type_expr::type_from_expr;
use super::{AssignKind, BlockChecker, FieldIdx, StackEntry};

fn prefix_check_dump_enabled() -> bool {
    #[cfg(target_os = "none")]
    {
        false
    }
    #[cfg(not(target_os = "none"))]
    {
        std::env::var("NEPL_DUMP_HIR").is_ok()
    }
}

macro_rules! prefix_check_log {
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

macro_rules! prefix_check_dump {
    ($($arg:tt)*) => {
        if prefix_check_dump_enabled() {
            prefix_check_log!($($arg)*);
        }
    };
}

impl<'a> BlockChecker<'a> {
    pub(super) fn check_prefix(
        &mut self,
        expr: &PrefixExpr,
        base_depth: usize,
        stack: &mut Vec<StackEntry>,
        expected_last_ty: Option<TypeId>,
    ) -> Option<(HirExpr, bool)> {
        // Track indices of functions on the stack to avoid linear scanning in reduce_calls.
        // This makes reduction O(1) amortized instead of O(N^2).
        let mut open_calls: Vec<usize> = Vec::new();
        // Initialize open_calls from existing stack (if any)
        for (i, entry) in stack.iter().enumerate() {
            let rty = self.ctx.resolve(entry.ty);
            if entry.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. }) {
                open_calls.push(i);
            }
        }

        let mut dropped = false;
        let mut last_expr: Option<HirExpr> = None;
        let mut pipe_pending: Option<Vec<StackEntry>> = None;
        let mut seen_pipe = false;
        // (target_type, stack_depth_when_annotation_appeared)
        let mut pending_ascription: Option<(TypeId, usize)> =
            expected_last_ty.map(|t| (t, base_depth));

        // Try to apply a pending ascription when the next expression is complete.
        fn try_apply_pending_ascription(
            this: &mut BlockChecker,
            stack: &mut Vec<StackEntry>,
            pending: &mut Option<(TypeId, usize)>,
        ) {
            let Some((target_ty, base_len)) = *pending else {
                return;
            };
            // The next expression is complete exactly when the stack returns to base_len + 1
            if stack.len() == base_len + 1 {
                let top = stack.last().unwrap();
                // Do not apply to functions
                if !matches!(this.ctx.get(top.ty), TypeKind::Function { .. }) {
                    let sp = top.expr.span;
                    this.apply_ascription(stack.as_mut_slice(), target_ty, sp);
                    *pending = None;
                }
            }
        }

        // pipe 直前でも、現在の引数式だけに付いた型注釈は次の pipe へ持ち越さない。
        fn is_local_pending_ascription(
            stack: &[StackEntry],
            pending: Option<(TypeId, usize)>,
            base_depth: usize,
        ) -> bool {
            pending
                .map(|(_, base)| {
                    base > base_depth
                        && stack
                            .get(base.saturating_sub(1))
                            .map(|entry| entry.assign.is_none())
                            .unwrap_or(true)
                })
                .unwrap_or(false)
        }

        for (idx, item) in expr.items.iter().enumerate() {
            // std::eprintln!("  Item: {:?}", item);
            let next_is_pipe = matches!(expr.items.get(idx + 1), Some(PrefixItem::Pipe(_)));
            match item {
                PrefixItem::Literal(lit, span) => {
                    let (ty, hir) = match lit {
                        Literal::Int(text) => {
                            let v = match parse_i32_literal(text) {
                                Some(v) => v,
                                None => {
                                    self.diagnostics
                                        .push(Diagnostic::error("invalid integer literal", *span));
                                    0
                                }
                            };
                            (self.ctx.i32(), HirExprKind::LiteralI32(v))
                        }
                        Literal::Float(text) => {
                            let v = text.parse::<f32>().unwrap_or(0.0);
                            (self.ctx.f32(), HirExprKind::LiteralF32(v))
                        }
                        Literal::Bool(b) => (self.ctx.bool(), HirExprKind::LiteralBool(*b)),
                        Literal::Char(c) => {
                            let value = if *c <= i32::MAX as u32 {
                                *c as i32
                            } else {
                                self.diagnostics.push(Diagnostic::error(
                                    "char literal is outside current i32-backed codegen range",
                                    *span,
                                ));
                                0
                            };
                            (self.ctx.char(), HirExprKind::LiteralI32(value))
                        }
                        Literal::Str(s) => {
                            let id = self.string_table.intern(s.clone());
                            (self.ctx.str(), HirExprKind::LiteralStr(id))
                        }
                        Literal::Unit => (self.ctx.unit(), HirExprKind::Unit),
                    };
                    stack.push(StackEntry {
                        ty,
                        expr: HirExpr {
                            ty,
                            kind: hir,
                            span: *span,
                        },
                        type_args: Vec::new(),
                        assign: None,
                        auto_call: true,
                    });
                    last_expr = Some(stack.last().unwrap().expr.clone());
                }
                PrefixItem::Symbol(sym) => match sym {
                    Symbol::Ident(id, type_args, forced_value) => {
                        let qualified_bindings = self.lookup_qualified_bindings(id);
                        let in_let_self_init = stack.iter().rev().any(|e| {
                            matches!(e.assign, Some(AssignKind::Let))
                                && matches!(&e.expr.kind, HirExprKind::Var(n) if n == &id.name)
                        });
                        if let Some(entry) = self.resolve_dotted_field_symbol(id, *forced_value) {
                            stack.push(entry);
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        } else {
                            let selected_from_qualified = qualified_bindings.is_some();
                            let selected_binding = if let Some((_, qualified)) = &qualified_bindings
                            {
                                if qualified.len() == 1 {
                                    Some((qualified[0].clone(), false))
                                } else {
                                    None
                                }
                            } else {
                                let explicit_callable_candidate =
                                    if !*forced_value && !type_args.is_empty() {
                                        let mut matching = self
                                            .lookup_all_unqualified_callables(id)
                                            .into_iter()
                                            .filter(|binding| {
                                                let ty = self.ctx.resolve_id(binding.ty);
                                                match self.ctx.get(ty) {
                                                    TypeKind::Function { type_params, .. } => {
                                                        type_params.len() == type_args.len()
                                                    }
                                                    _ => false,
                                                }
                                            })
                                            .cloned()
                                            .collect::<Vec<_>>();
                                        if matching.len() == 1 {
                                            Some(matching.remove(0))
                                        } else {
                                            None
                                        }
                                    } else {
                                        None
                                    };
                                let expected_function_from_outer = self
                                    .infer_expected_from_outer_consumer_next_arg(
                                        &stack,
                                        stack.len(),
                                        0,
                                    )
                                    .map(|t| {
                                        let resolved = self.ctx.resolve(t);
                                        matches!(self.ctx.get(resolved), TypeKind::Function { .. })
                                    })
                                    .unwrap_or(false);
                                let expected_function_from_ascription = pending_ascription
                                    .and_then(|(target_ty, base_len)| {
                                        if stack.len() == base_len {
                                            let resolved = self.ctx.resolve(target_ty);
                                            match self.ctx.get(resolved) {
                                                TypeKind::Function { .. } => Some(true),
                                                _ => Some(false),
                                            }
                                        } else {
                                            None
                                        }
                                    })
                                    .unwrap_or(false);
                                let expected_function_from_outer = expected_function_from_outer
                                    || expected_function_from_ascription;
                                let value_candidate =
                                    self.lookup_unqualified_value_for_read(id, !in_let_self_init);
                                let has_any_value = self.lookup_unqualified_value_any(id).is_some();
                                let value_is_function = value_candidate
                                    .map(|b| {
                                        let rty = self.ctx.resolve_id(b.ty);
                                        matches!(self.ctx.get(rty), TypeKind::Function { .. })
                                    })
                                    .unwrap_or(false);
                                let preferred_callable = if !*forced_value
                                    && stack.is_empty()
                                    && expr.items.get(idx + 1).is_some()
                                    && (!has_any_value || !value_is_function)
                                {
                                    self.lookup_unqualified_callable_any(id)
                                } else {
                                    None
                                };
                                let selected = explicit_callable_candidate
                                    .as_ref()
                                    .or(preferred_callable)
                                    .or(value_candidate)
                                    .or_else(|| {
                                        if !has_any_value {
                                            self.lookup_unqualified_callable_any(id)
                                        } else {
                                            None
                                        }
                                    })
                                    .and_then(|binding| {
                                        let should_delay_overload_resolution = !*forced_value
                                            && !expected_function_from_outer
                                            && matches!(binding.kind, BindingKind::Func { .. })
                                            && type_args.is_empty()
                                            && self.lookup_all_unqualified_callables(id).len() > 1
                                            && expr.items.get(idx + 1).is_some();
                                        if should_delay_overload_resolution {
                                            None
                                        } else {
                                            Some(binding)
                                        }
                                    });
                                selected
                                    .cloned()
                                    .map(|binding| (binding, expected_function_from_outer))
                            };
                            if let Some((binding, expected_function_from_outer)) = selected_binding
                            {
                                if *forced_value {
                                    match &binding.kind {
                                        BindingKind::Func { captures, .. } => {
                                            if !captures.is_empty() {
                                                self.diagnostics.push(Diagnostic::error(
                                                    "capturing function cannot be used as a function value yet",
                                                    id.span,
                                                ).with_id(DiagnosticId::TypeCapturingFunctionValueUnsupported));
                                                return None;
                                            }
                                        }
                                        _ => {
                                            self.diagnostics.push(Diagnostic::error(
                                                "only callable symbols can be referenced with '@'",
                                                id.span,
                                            ).with_id(DiagnosticId::TypeAtRequiresCallable));
                                            return None;
                                        }
                                    }
                                }
                                let ty = binding.ty;
                                let auto_call = match &binding.kind {
                                    BindingKind::Func { .. } => {
                                        !*forced_value && !expected_function_from_outer
                                    }
                                    _ => !*forced_value,
                                };
                                let hir_kind = match &binding.kind {
                                    BindingKind::Func { symbol, .. }
                                        if *forced_value
                                            || expected_function_from_outer
                                            || selected_from_qualified
                                            || binding.name != id.name =>
                                    {
                                        HirExprKind::FnValue(symbol.clone())
                                    }
                                    _ => HirExprKind::Var(binding.name.clone()),
                                };
                                let explicit_args = match binding.kind {
                                    BindingKind::Func { .. } => {
                                        let mut args = Vec::new();
                                        for arg_expr in type_args {
                                            args.push(type_from_expr(
                                                self.ctx,
                                                self.labels,
                                                arg_expr,
                                            ));
                                        }
                                        args
                                    }
                                    _ => {
                                        if !type_args.is_empty() {
                                            self.diagnostics.push(
                                                Diagnostic::error(
                                                    "type arguments are not allowed for variables",
                                                    id.span,
                                                )
                                                .with_id(
                                                    DiagnosticId::TypeVariableTypeArgsNotAllowed,
                                                ),
                                            );
                                        }
                                        Vec::new()
                                    }
                                };
                                stack.push(StackEntry {
                                    ty,
                                    expr: HirExpr {
                                        ty,
                                        kind: hir_kind,
                                        span: id.span,
                                    },
                                    type_args: explicit_args,
                                    assign: None,
                                    auto_call,
                                });
                                last_expr = Some(stack.last().unwrap().expr.clone());
                            } else {
                                let mut lookup_name = id.name.clone();
                                let outer_expected_callable_arity = self
                                    .infer_expected_from_outer_consumer_next_arg(
                                        &stack,
                                        stack.len(),
                                        0,
                                    )
                                    .and_then(|expected_ty| {
                                        let resolved_target = self.ctx.resolve(expected_ty);
                                        match self.ctx.get(resolved_target) {
                                            TypeKind::Function { params, .. } => Some(
                                                if params.len() == 1
                                                    && matches!(
                                                        self.ctx
                                                            .get(self.ctx.resolve_id(params[0])),
                                                        TypeKind::Unit
                                                    )
                                                {
                                                    0
                                                } else {
                                                    params.len()
                                                },
                                            ),
                                            _ => None,
                                        }
                                    });
                                let mut bindings =
                                    if let Some((member, qualified)) = &qualified_bindings {
                                        lookup_name = member.clone();
                                        qualified.clone()
                                    } else {
                                        self.lookup_all_unqualified_any_defined(id)
                                            .into_iter()
                                            .cloned()
                                            .collect()
                                    };
                                if bindings.is_empty()
                                    && qualified_bindings.is_none()
                                    && !self.import_resolution.has_qualified_targets()
                                {
                                    if let Some((ns, member)) = parse_variant_name(&id.name) {
                                        if !self.enums.contains_key(ns)
                                            && !self.traits.contains_key(ns)
                                        {
                                            let member_id = Ident {
                                                name: member.to_string(),
                                                span: id.span,
                                            };
                                            let alt = self
                                                .lookup_all_unqualified_any_defined(&member_id)
                                                .into_iter()
                                                .cloned()
                                                .collect::<Vec<_>>();
                                            if !alt.is_empty() {
                                                lookup_name = member.to_string();
                                                bindings = alt;
                                            }
                                        }
                                    }
                                }
                                if !type_args.is_empty() {
                                    bindings.retain(|binding| {
                                        let ty = self.ctx.resolve_id(binding.ty);
                                        match self.ctx.get(ty) {
                                            TypeKind::Function { type_params, .. } => {
                                                type_params.len() == type_args.len()
                                            }
                                            _ => false,
                                        }
                                    });
                                }
                                if qualified_bindings.is_some() && bindings.len() == 1 {
                                    let binding = bindings.remove(0);
                                    let mut explicit_args = Vec::new();
                                    if matches!(binding.kind, BindingKind::Func { .. }) {
                                        for arg_expr in type_args {
                                            explicit_args.push(type_from_expr(
                                                self.ctx,
                                                self.labels,
                                                arg_expr,
                                            ));
                                        }
                                    } else if !type_args.is_empty() {
                                        self.diagnostics.push(
                                            Diagnostic::error(
                                                "type arguments are not allowed for variables",
                                                id.span,
                                            )
                                            .with_id(DiagnosticId::TypeVariableTypeArgsNotAllowed),
                                        );
                                    }
                                    let ty = binding.ty;
                                    let hir_kind = match &binding.kind {
                                        BindingKind::Func { symbol, .. } => {
                                            HirExprKind::FnValue(symbol.clone())
                                        }
                                        _ => HirExprKind::Var(lookup_name.clone()),
                                    };
                                    stack.push(StackEntry {
                                        ty,
                                        expr: HirExpr {
                                            ty,
                                            kind: hir_kind,
                                            span: id.span,
                                        },
                                        type_args: explicit_args,
                                        assign: None,
                                        auto_call: !*forced_value,
                                    });
                                    last_expr = Some(stack.last().unwrap().expr.clone());
                                    continue;
                                }
                                if !bindings.is_empty() {
                                    let call_name = if qualified_bindings.is_some() {
                                        id.name.clone()
                                    } else {
                                        lookup_name.clone()
                                    };
                                    let callable_overload_count = bindings
                                        .iter()
                                        .filter(|b| matches!(b.kind, BindingKind::Func { .. }))
                                        .count();
                                    let overloaded_callable_only = callable_overload_count > 1
                                        && bindings
                                            .iter()
                                            .all(|b| matches!(b.kind, BindingKind::Func { .. }));
                                    if overloaded_callable_only
                                        && !*forced_value
                                        && type_args.is_empty()
                                    {
                                        let remaining_items =
                                            expr.items.len().saturating_sub(idx + 1);
                                        let mut arities: Vec<usize> = bindings
                                            .iter()
                                            .filter_map(|b| match b.kind {
                                                BindingKind::Func { arity, .. } => Some(arity),
                                                _ => None,
                                            })
                                            .collect();
                                        arities.sort_unstable();
                                        arities.dedup();
                                        let inferred_arity = outer_expected_callable_arity
                                            .filter(|a| arities.contains(a))
                                            .or_else(|| {
                                                if matches!(
                                                    expr.items.get(idx + 1),
                                                    Some(PrefixItem::Pipe(_))
                                                ) && arities.contains(&0)
                                                {
                                                    return Some(0);
                                                }
                                                arities
                                                    .iter()
                                                    .copied()
                                                    .filter(|a| *a <= remaining_items)
                                                    .max()
                                            })
                                            .or_else(|| arities.first().copied())
                                            .unwrap_or(0);
                                        let mut params = Vec::new();
                                        for _ in 0..inferred_arity {
                                            params.push(self.ctx.fresh_var(None));
                                        }
                                        let result = self.ctx.fresh_var(None);
                                        let ty = self.ctx.function(
                                            Vec::new(),
                                            params,
                                            result,
                                            Effect::Pure,
                                        );
                                        stack.push(StackEntry {
                                            ty,
                                            expr: HirExpr {
                                                ty,
                                                kind: HirExprKind::Var(call_name.clone()),
                                                span: id.span,
                                            },
                                            type_args: Vec::new(),
                                            assign: None,
                                            auto_call: true,
                                        });
                                        last_expr = Some(stack.last().unwrap().expr.clone());
                                    } else if let Some(binding) = bindings
                                        .iter()
                                        .cloned()
                                        .find(|b| matches!(b.kind, BindingKind::Var))
                                    {
                                        if *forced_value {
                                            self.diagnostics.push(Diagnostic::error(
                                            "only callable symbols can be referenced with '@'",
                                            id.span,
                                        ).with_id(DiagnosticId::TypeAtRequiresCallable));
                                            return None;
                                        }
                                        if !type_args.is_empty() {
                                            self.diagnostics.push(
                                                Diagnostic::error(
                                                    "type arguments are not allowed for variables",
                                                    id.span,
                                                )
                                                .with_id(
                                                    DiagnosticId::TypeVariableTypeArgsNotAllowed,
                                                ),
                                            );
                                        }
                                        let ty = binding.ty;
                                        stack.push(StackEntry {
                                            ty,
                                            expr: HirExpr {
                                                ty,
                                                kind: HirExprKind::Var(lookup_name.clone()),
                                                span: id.span,
                                            },
                                            type_args: Vec::new(),
                                            assign: None,
                                            auto_call: !*forced_value,
                                        });
                                        if crate::log::is_verbose()
                                            && matches!(
                                                lookup_name.as_str(),
                                                "A" | "use_a" | "DefaultHash32" | "new" | "must_hm"
                                            )
                                        {
                                            prefix_check_log!(
                                                "push value {} ty={} auto_call={}",
                                                lookup_name,
                                                self.ctx.type_to_string(ty),
                                                !*forced_value
                                            );
                                        }
                                        last_expr = Some(stack.last().unwrap().expr.clone());
                                    } else {
                                        let expected_callable_arity = pending_ascription
                                            .and_then(|(target_ty, base_len)| {
                                                if stack.len() == base_len {
                                                    let resolved_target =
                                                        self.ctx.resolve(target_ty);
                                                    match self.ctx.get(resolved_target) {
                                                        TypeKind::Function { params, .. } => Some(
                                                            if params.len() == 1
                                                                && matches!(
                                                                    self.ctx.get(
                                                                        self.ctx
                                                                            .resolve_id(params[0])
                                                                    ),
                                                                    TypeKind::Unit
                                                                )
                                                            {
                                                                0
                                                            } else {
                                                                params.len()
                                                            },
                                                        ),
                                                        _ => None,
                                                    }
                                                } else {
                                                    None
                                                }
                                            })
                                            .or(outer_expected_callable_arity);
                                        if let Some(exp_arity) = expected_callable_arity {
                                            let allow_fnvalue_selection =
                                                *forced_value || idx + 1 >= expr.items.len();
                                            if allow_fnvalue_selection {
                                                let mut arity_candidates: Vec<&Binding> = bindings
                                            .iter()
                                            .filter(|b| {
                                                matches!(
                                                    b.kind,
                                                    BindingKind::Func { arity, .. } if arity == exp_arity
                                                )
                                            })
                                            .collect();
                                                if arity_candidates.len() == 1 {
                                                    let binding = arity_candidates.remove(0);
                                                    if *forced_value {
                                                        if let BindingKind::Func {
                                                            captures, ..
                                                        } = &binding.kind
                                                        {
                                                            if !captures.is_empty() {
                                                                self.diagnostics.push(Diagnostic::error(
                                                            "capturing function cannot be used as a function value yet",
                                                            id.span,
                                                        ).with_id(DiagnosticId::TypeCapturingFunctionValueUnsupported));
                                                                return None;
                                                            }
                                                        }
                                                    }
                                                    let mut explicit_args = Vec::new();
                                                    if !type_args.is_empty() {
                                                        for arg_expr in type_args {
                                                            explicit_args.push(type_from_expr(
                                                                self.ctx,
                                                                self.labels,
                                                                arg_expr,
                                                            ));
                                                        }
                                                    }
                                                    let ty = binding.ty;
                                                    let fn_symbol = match &binding.kind {
                                                        BindingKind::Func { symbol, .. } => {
                                                            symbol.clone()
                                                        }
                                                        _ => lookup_name.clone(),
                                                    };
                                                    stack.push(StackEntry {
                                                        ty,
                                                        expr: HirExpr {
                                                            ty,
                                                            // 期待関数型で一意に選べた過負荷関数は
                                                            // ここで関数値として確定させる。
                                                            kind: HirExprKind::FnValue(fn_symbol),
                                                            span: id.span,
                                                        },
                                                        type_args: explicit_args,
                                                        assign: None,
                                                        auto_call: false,
                                                    });
                                                    last_expr =
                                                        Some(stack.last().unwrap().expr.clone());
                                                    continue;
                                                } else if arity_candidates.len() > 1 {
                                                    self.diagnostics.push(
                                                        Diagnostic::error(
                                                            "ambiguous overload",
                                                            id.span,
                                                        )
                                                        .with_id(
                                                            DiagnosticId::TypeAmbiguousOverload,
                                                        ),
                                                    );
                                                    return None;
                                                }
                                            }
                                        }
                                        let mut has_pure = false;
                                        let mut has_impure = false;
                                        let mut arity = None;
                                        for b in &bindings {
                                            if let BindingKind::Func {
                                                effect: e,
                                                arity: a,
                                                ..
                                            } = b.kind
                                            {
                                                match e {
                                                    Effect::Pure => has_pure = true,
                                                    Effect::Impure => has_impure = true,
                                                }
                                                if arity.is_none() {
                                                    arity = Some(a);
                                                }
                                            }
                                        }
                                        let arity = arity.unwrap_or(0);
                                        let effect = if has_pure || !has_impure {
                                            Effect::Pure
                                        } else {
                                            Effect::Impure
                                        };
                                        let has_captures = bindings.iter().any(|b| {
                                        matches!(
                                            &b.kind,
                                            BindingKind::Func { captures, .. } if !captures.is_empty()
                                        )
                                    });
                                        if *forced_value && has_captures {
                                            self.diagnostics.push(Diagnostic::error(
                                            "capturing function cannot be used as a function value yet",
                                            id.span,
                                        ).with_id(DiagnosticId::TypeCapturingFunctionValueUnsupported));
                                            return None;
                                        }
                                        let mut explicit_args = Vec::new();
                                        if !type_args.is_empty() {
                                            for arg_expr in type_args {
                                                explicit_args.push(type_from_expr(
                                                    self.ctx,
                                                    self.labels,
                                                    arg_expr,
                                                ));
                                            }
                                        }
                                        let mut params = Vec::new();
                                        for _ in 0..arity {
                                            params.push(self.ctx.fresh_var(None));
                                        }
                                        let result = self.ctx.fresh_var(None);
                                        let ty =
                                            self.ctx.function(Vec::new(), params, result, effect);
                                        stack.push(StackEntry {
                                            ty,
                                            expr: HirExpr {
                                                ty,
                                                kind: if *forced_value {
                                                    HirExprKind::FnValue(call_name.clone())
                                                } else {
                                                    HirExprKind::Var(call_name.clone())
                                                },
                                                span: id.span,
                                            },
                                            type_args: explicit_args,
                                            assign: None,
                                            auto_call: true,
                                        });
                                        if crate::log::is_verbose()
                                            && matches!(
                                                lookup_name.as_str(),
                                                "A" | "use_a" | "DefaultHash32" | "new" | "must_hm"
                                            )
                                        {
                                            prefix_check_log!(
                                                "push callable {} ty={} auto_call=true",
                                                lookup_name,
                                                self.ctx.type_to_string(ty)
                                            );
                                        }
                                        last_expr = Some(stack.last().unwrap().expr.clone());
                                    }
                                } else if let Some((trait_name, method_name)) =
                                    parse_variant_name(&id.name)
                                {
                                    if let Some(trait_info) = self.traits.get(trait_name) {
                                        if !type_args.is_empty() {
                                            self.diagnostics.push(Diagnostic::error(
                                                "type arguments are not supported for trait methods yet",
                                                id.span,
                                            ).with_id(DiagnosticId::TypeTraitMethodTypeArgsNotSupported));
                                            return None;
                                        }
                                        if let Some(sig) = trait_info.methods.get(method_name) {
                                            let applied_trait_name = self
                                                .infer_trait_application_name(
                                                    trait_name,
                                                    trait_info,
                                                    *sig,
                                                    &[],
                                                    None,
                                                );
                                            let method_self = self
                                                .infer_unique_type_param_for_trait(
                                                    &applied_trait_name,
                                                )
                                                .unwrap_or_else(|| {
                                                    self.ctx.fresh_var(Some(String::from("Self")))
                                                });
                                            let mut mapping = BTreeMap::new();
                                            mapping.insert(
                                                self.ctx.resolve_id(trait_info.self_ty),
                                                method_self,
                                            );
                                            let inst_ty = self.ctx.substitute(*sig, &mapping);
                                            stack.push(StackEntry {
                                                ty: inst_ty,
                                                expr: HirExpr {
                                                    ty: inst_ty,
                                                    kind: if *forced_value {
                                                        HirExprKind::FnValue(id.name.clone())
                                                    } else {
                                                        HirExprKind::Var(id.name.clone())
                                                    },
                                                    span: id.span,
                                                },
                                                type_args: vec![method_self],
                                                assign: None,
                                                auto_call: !*forced_value,
                                            });
                                            last_expr = Some(stack.last().unwrap().expr.clone());
                                        } else {
                                            self.diagnostics.push(
                                                Diagnostic::error(
                                                    format!(
                                                        "unknown method '{}' for trait '{}'",
                                                        method_name, trait_name
                                                    ),
                                                    id.span,
                                                )
                                                .with_id(DiagnosticId::TypeTraitMethodNotFound),
                                            );
                                            return None;
                                        }
                                    } else {
                                        self.diagnostics.push(
                                            Diagnostic::error("undefined identifier", id.span)
                                                .with_id(DiagnosticId::TypeUndefinedIdentifier),
                                        );
                                    }
                                } else {
                                    self.diagnostics.push(
                                        Diagnostic::error("undefined identifier", id.span)
                                            .with_id(DiagnosticId::TypeUndefinedIdentifier),
                                    );
                                }
                            }
                        }
                    }
                    Symbol::Let {
                        name,
                        mutable,
                        no_shadow,
                    } => {
                        // Use current-scope lookup so `let` always creates a local binding
                        // (shadowing outer bindings) rather than reusing an outer binding.
                        let ty = if let Some(b) = self.env.lookup_current_value(&name.name) {
                            if b.no_shadow && b.span != name.span {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "cannot shadow non-shadowable symbol '{}'",
                                            name.name
                                        ),
                                        name.span,
                                    )
                                    .with_id(DiagnosticId::TypeNoShadowViolation),
                                );
                                self.diagnostics.push(
                                    Diagnostic::error("non-shadowable declaration is here", b.span)
                                        .with_id(DiagnosticId::TypeNoShadowViolation)
                                        .with_secondary_label(
                                            name.span,
                                            Some("shadow attempt".into()),
                                        ),
                                );
                                return None;
                            }
                            b.ty
                        } else {
                            if let Some(blocked) = shadow_blocked_by_nonshadow(self.env, &name.name)
                            {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "cannot shadow non-shadowable symbol '{}'",
                                            name.name
                                        ),
                                        name.span,
                                    )
                                    .with_id(DiagnosticId::TypeNoShadowViolation),
                                );
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "non-shadowable declaration is here",
                                        blocked.span,
                                    )
                                    .with_id(DiagnosticId::TypeNoShadowViolation)
                                    .with_secondary_label(name.span, Some("shadow attempt".into())),
                                );
                                return None;
                            }
                            if *no_shadow && self.env.lookup_any(&name.name).is_some() {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                        "noshadow declaration '{}' conflicts with existing symbol",
                                        name.name
                                    ),
                                        name.span,
                                    )
                                    .with_id(DiagnosticId::TypeNoShadowConflict),
                                );
                                return None;
                            }
                            let t = self.ctx.fresh_var(None);
                            emit_shadow_warning(
                                &mut self.diagnostics,
                                self.env,
                                &name.name,
                                name.span,
                                if *mutable { "let mut" } else { "let" },
                            );
                            let _ = self.env.insert_local(Binding {
                                name: name.name.clone(),
                                ty: t,
                                mutable: *mutable,
                                no_shadow: *no_shadow,
                                defined: false,
                                span: name.span,
                                kind: BindingKind::Var,
                            });
                            prefix_check_dump!("typecheck: inserted local binding {}", name.name);
                            t
                        };
                        let func_ty =
                            self.ctx
                                .function(Vec::new(), vec![ty], self.ctx.unit(), Effect::Pure);
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var(name.name.clone()),
                                span: name.span,
                            },
                            type_args: Vec::new(),
                            assign: Some(AssignKind::Let),
                            auto_call: false,
                        });
                        // defer applying ascription until the expression is complete
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                    Symbol::Set { name } => {
                        if let Some((binding, scope_index)) =
                            self.env.lookup_value_with_scope(&name.name)
                        {
                            if !binding.mutable {
                                self.diagnostics.push(
                                    Diagnostic::error("cannot set immutable variable", name.span)
                                        .with_id(DiagnosticId::TypeImmutableMutation),
                                );
                            }
                            let effect = if scope_index == 0 {
                                Effect::Impure
                            } else {
                                Effect::Pure
                            };
                            let func_ty = self.ctx.function(
                                Vec::new(),
                                vec![binding.ty],
                                self.ctx.unit(),
                                effect,
                            );
                            stack.push(StackEntry {
                                ty: func_ty,
                                expr: HirExpr {
                                    ty: func_ty,
                                    kind: HirExprKind::Var(name.name.clone()),
                                    span: name.span,
                                },
                                type_args: Vec::new(),
                                assign: Some(AssignKind::Set),
                                auto_call: true,
                            });
                            // defer applying ascription until the expression is complete
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        } else {
                            self.diagnostics.push(
                                Diagnostic::error("undefined variable", name.span)
                                    .with_id(DiagnosticId::TypeUndefinedVariable),
                            );
                        }
                    }
                    Symbol::AddrOf { span, mutable } => {
                        if crate::log::is_verbose() {
                            prefix_check_log!("check_prefix: pushing AddrOf to stack");
                        }
                        let a = self.ctx.fresh_var(None);
                        let ref_a = self.ctx.reference(a, *mutable);
                        let func_ty = self.ctx.function(Vec::new(), vec![a], ref_a, Effect::Pure);
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var("&&addr_of".to_string()),
                                span: *span,
                            },
                            type_args: Vec::new(),
                            assign: Some(AssignKind::AddrOf(*mutable)),
                            auto_call: true,
                        });
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                    Symbol::Deref(span) => {
                        let a = self.ctx.fresh_var(None);
                        let ref_a = self.ctx.reference(a, false);
                        let func_ty = self.ctx.function(Vec::new(), vec![ref_a], a, Effect::Pure);
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var("&&deref".to_string()),
                                span: *span,
                            },
                            type_args: Vec::new(),
                            assign: Some(AssignKind::Deref),
                            auto_call: true,
                        });
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                    Symbol::If(sp) => {
                        let t_cond = self.ctx.bool();
                        let t_branch = self.ctx.fresh_var(None);
                        let func_ty = self.ctx.function(
                            Vec::new(),
                            vec![t_cond, t_branch, t_branch],
                            t_branch,
                            Effect::Pure,
                        );
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var("if".to_string()),
                                span: *sp,
                            },
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: true,
                        });
                        // defer applying ascription until the expression is complete
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                    Symbol::While(sp) => {
                        let t_cond = self.ctx.bool();
                        let func_ty = self.ctx.function(
                            Vec::new(),
                            vec![t_cond, self.ctx.unit()],
                            self.ctx.unit(),
                            Effect::Pure,
                        );
                        stack.push(StackEntry {
                            ty: func_ty,
                            expr: HirExpr {
                                ty: func_ty,
                                kind: HirExprKind::Var("while".to_string()),
                                span: *sp,
                            },
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: true,
                        });
                        // defer applying ascription until the expression is complete
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                },
                PrefixItem::Intrinsic(intrin, sp) => {
                    let intrin_effect = intrinsic_effect(&intrin.name);
                    if matches!(self.current_effect, Effect::Pure)
                        && matches!(intrin_effect, Effect::Impure)
                        && !self.raw_memory_intrinsic_allowed(&intrin.name, *sp)
                    {
                        self.diagnostics.push(
                            Diagnostic::error("pure context cannot call impure function", *sp)
                                .with_id(DiagnosticId::TypePureCallsImpureFunction),
                        );
                        return None;
                    }

                    let mut type_args = Vec::new();
                    for t in &intrin.type_args {
                        type_args.push(type_from_expr(self.ctx, self.labels, t));
                    }

                    let mut args = Vec::new();
                    for arg in &intrin.args {
                        let mut arg_stack = Vec::new();
                        if let Some((hexpr, _)) = self.check_prefix(arg, 0, &mut arg_stack, None) {
                            args.push(hexpr);
                        } else {
                            return None;
                        }
                    }

                    let ty = if intrin.name == "size_of" || intrin.name == "align_of" {
                        self.ctx.i32()
                    } else if intrin.name == "load" {
                        if type_args.len() == 1 {
                            type_args[0]
                        } else {
                            self.ctx.unit()
                        }
                    } else if intrin.name == "store" {
                        self.ctx.unit()
                    } else if intrin.name == "callsite_span" {
                        if type_args.len() == 1 {
                            type_args[0]
                        } else {
                            self.diagnostics.push(
                                Diagnostic::error("callsite_span expects 1 type arg", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicTypeArgArityMismatch),
                            );
                            self.ctx.unit()
                        }
                    } else if intrin.name == "set_field" {
                        self.ctx.unit() // temporary, will continue below
                    } else if intrin.name == "unreachable" {
                        self.ctx.never()
                    } else if intrin.name == "i32_to_f32" {
                        self.ctx.f32()
                    } else if intrin.name == "i32_to_u8" {
                        self.ctx.u8()
                    } else if intrin.name == "i32_to_u32" {
                        self.ctx.lookup_named("u32").unwrap_or_else(|| {
                            self.ctx.register_named(
                                "u32".to_string(),
                                TypeKind::Named("u32".to_string()),
                            )
                        })
                    } else if intrin.name == "f32_to_i32" {
                        self.ctx.i32()
                    } else if intrin.name == "u8_to_i32" {
                        self.ctx.i32()
                    } else if intrin.name == "char_to_i32" {
                        self.ctx.i32()
                    } else if intrin.name == "i32_to_char" {
                        self.ctx.char()
                    } else if intrin.name == "u32_to_i32" {
                        self.ctx.i32()
                    } else if intrin.name == "i64_to_u64" {
                        self.ctx.lookup_named("u64").unwrap_or_else(|| {
                            self.ctx.register_named(
                                "u64".to_string(),
                                TypeKind::Named("u64".to_string()),
                            )
                        })
                    } else if intrin.name == "u64_to_i64" {
                        self.ctx.lookup_named("i64").unwrap_or_else(|| {
                            self.ctx.register_named(
                                "i64".to_string(),
                                TypeKind::Named("i64".to_string()),
                            )
                        })
                    } else if intrin.name == "reinterpret_i32_f32" {
                        self.ctx.f32()
                    } else if intrin.name == "reinterpret_f32_i32" {
                        self.ctx.i32()
                    } else if intrin.name == "str_addr" {
                        self.ctx.i32()
                    } else if intrin.name == "str_from_addr_unchecked" {
                        self.ctx.str()
                    } else if intrin.name == "get_field" {
                        self.ctx.fresh_var(None)
                    } else if intrin.name == "get_field_ref" {
                        self.ctx.fresh_var(None)
                    } else if intrin.name == "set_field" {
                        self.ctx.unit()
                    } else {
                        self.diagnostics.push(
                            Diagnostic::error("unknown intrinsic", *sp)
                                .with_id(DiagnosticId::TypeUnknownIntrinsic),
                        );
                        self.ctx.unit()
                    };

                    if intrin.name == "get_field" {
                        let obj = args[0].clone();
                        let idx = &args[1];
                        let res = match &idx.kind {
                            HirExprKind::LiteralI32(val) => self.resolve_field_access(
                                obj.ty,
                                FieldIdx::Index(*val as usize),
                                *sp,
                            ),
                            HirExprKind::LiteralStr(sid) => {
                                let name = self.string_table.get(*sid).unwrap().clone();
                                self.resolve_field_access(obj.ty, FieldIdx::Name(name), *sp)
                            }
                            _ => None,
                        };
                        if let Some((f_ty, offset)) = res {
                            // Unify our determined ty (fresh var) with the actual field type
                            let _ = self.ctx.unify(ty, f_ty);

                            // Lower to load(add(obj, offset))
                            let addr_expr = if offset == 0 {
                                obj
                            } else {
                                HirExpr {
                                    ty: self.ctx.i32(),
                                    kind: HirExprKind::Intrinsic {
                                        name: "add".to_string(),
                                        type_args: vec![self.ctx.i32()],
                                        args: vec![
                                            obj,
                                            HirExpr {
                                                ty: self.ctx.i32(),
                                                kind: HirExprKind::LiteralI32(offset as i32),
                                                span: idx.span,
                                            },
                                        ],
                                    },
                                    span: *sp,
                                }
                            };
                            let hexpr = HirExpr {
                                ty: f_ty,
                                kind: HirExprKind::Intrinsic {
                                    name: "load".to_string(),
                                    type_args: vec![f_ty],
                                    args: vec![addr_expr],
                                },
                                span: *sp,
                            };
                            stack.push(StackEntry {
                                ty: f_ty,
                                expr: hexpr.clone(),
                                type_args: Vec::new(),
                                assign: None,
                                auto_call: true,
                            });
                            last_expr = Some(hexpr);
                            continue;
                        }
                        // which pushes HirExprKind::Intrinsic and uses the fresh variable 'ty'.
                    } else if intrin.name == "get_field_ref" {
                        let obj = args[0].clone();
                        let idx = &args[1];
                        let resolved_obj_ty = self.ctx.resolve(obj.ty);
                        let base_ty = match self.ctx.get(resolved_obj_ty) {
                            TypeKind::Reference(inner, _) => inner,
                            _ => {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "get_field_ref expects a reference to a composite value",
                                        obj.span,
                                    )
                                    .with_id(DiagnosticId::TypeInvalidFieldAccess),
                                );
                                self.ctx.never()
                            }
                        };
                        let res = match &idx.kind {
                            HirExprKind::LiteralI32(val) => self.resolve_field_access(
                                base_ty,
                                FieldIdx::Index(*val as usize),
                                *sp,
                            ),
                            HirExprKind::LiteralStr(sid) => {
                                let name = self.string_table.get(*sid).unwrap().clone();
                                self.resolve_field_access(base_ty, FieldIdx::Name(name), *sp)
                            }
                            _ => None,
                        };
                        if let Some((f_ty, offset)) = res {
                            let ref_ty = self.ctx.reference(f_ty, false);
                            let _ = self.ctx.unify(ty, ref_ty);
                            let addr_expr = if offset == 0 {
                                obj
                            } else {
                                HirExpr {
                                    ty: ref_ty,
                                    kind: HirExprKind::Intrinsic {
                                        name: "add".to_string(),
                                        type_args: vec![self.ctx.i32()],
                                        args: vec![
                                            obj,
                                            HirExpr {
                                                ty: self.ctx.i32(),
                                                kind: HirExprKind::LiteralI32(offset as i32),
                                                span: idx.span,
                                            },
                                        ],
                                    },
                                    span: *sp,
                                }
                            };
                            let hexpr = HirExpr {
                                ty: ref_ty,
                                kind: addr_expr.kind,
                                span: *sp,
                            };
                            stack.push(StackEntry {
                                ty: ref_ty,
                                expr: hexpr.clone(),
                                type_args: Vec::new(),
                                assign: None,
                                auto_call: true,
                            });
                            last_expr = Some(hexpr);
                            continue;
                        }
                    } else if intrin.name == "set_field" {
                        let obj = args[0].clone();
                        let idx = &args[1];
                        let val = args[2].clone();
                        let res = match &idx.kind {
                            HirExprKind::LiteralI32(v) => {
                                self.resolve_field_access(obj.ty, FieldIdx::Index(*v as usize), *sp)
                            }
                            HirExprKind::LiteralStr(sid) => {
                                let name = self.string_table.get(*sid).unwrap().clone();
                                self.resolve_field_access(obj.ty, FieldIdx::Name(name), *sp)
                            }
                            _ => None,
                        };
                        if let Some((f_ty, offset)) = res {
                            // Unify value type with field type
                            if let Err(_) = self.ctx.unify(val.ty, f_ty) {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "type mismatch in set_field: expected {}, found {}",
                                            self.ctx.type_to_string(f_ty),
                                            self.ctx.type_to_string(val.ty)
                                        ),
                                        *sp,
                                    )
                                    .with_id(DiagnosticId::TypeAssignmentTypeMismatch),
                                );
                            }

                            // Lower to store(add(obj, offset), val)
                            let addr_expr = if offset == 0 {
                                obj
                            } else {
                                HirExpr {
                                    ty: self.ctx.i32(),
                                    kind: HirExprKind::Intrinsic {
                                        name: "add".to_string(),
                                        type_args: vec![self.ctx.i32()],
                                        args: vec![
                                            obj,
                                            HirExpr {
                                                ty: self.ctx.i32(),
                                                kind: HirExprKind::LiteralI32(offset as i32),
                                                span: idx.span,
                                            },
                                        ],
                                    },
                                    span: *sp,
                                }
                            };
                            let hexpr = HirExpr {
                                ty: self.ctx.unit(),
                                kind: HirExprKind::Intrinsic {
                                    name: "store".to_string(),
                                    type_args: vec![f_ty],
                                    args: vec![addr_expr, val],
                                },
                                span: *sp,
                            };
                            stack.push(StackEntry {
                                ty: self.ctx.unit(),
                                expr: hexpr.clone(),
                                type_args: Vec::new(),
                                assign: None,
                                auto_call: true,
                            });
                            last_expr = Some(hexpr);
                            continue;
                        }
                    }

                    // Validate intrinsic argument types for known cast/bitcast intrinsics
                    if intrin.name == "i32_to_f32"
                        || intrin.name == "reinterpret_i32_f32"
                        || intrin.name == "i32_to_u8"
                        || intrin.name == "i32_to_u32"
                        || intrin.name == "i32_to_char"
                    {
                        if args.len() != 1 {
                            self.diagnostics.push(
                                Diagnostic::error("intrinsic expects 1 argument", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicArgArityMismatch),
                            );
                        } else if let Err(_) = self.ctx.unify(args[0].ty, self.ctx.i32()) {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "intrinsic argument type mismatch (expected i32)",
                                    *sp,
                                )
                                .with_id(DiagnosticId::TypeIntrinsicArgTypeMismatch),
                            );
                        }
                    } else if intrin.name == "char_to_i32" {
                        if args.len() != 1 {
                            self.diagnostics.push(
                                Diagnostic::error("intrinsic expects 1 argument", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicArgArityMismatch),
                            );
                        } else if let Err(_) = self.ctx.unify(args[0].ty, self.ctx.char()) {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "intrinsic argument type mismatch (expected char)",
                                    *sp,
                                )
                                .with_id(DiagnosticId::TypeIntrinsicArgTypeMismatch),
                            );
                        }
                    } else if intrin.name == "f32_to_i32" || intrin.name == "reinterpret_f32_i32" {
                        if args.len() != 1 {
                            self.diagnostics.push(
                                Diagnostic::error("intrinsic expects 1 argument", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicArgArityMismatch),
                            );
                        } else if let Err(_) = self.ctx.unify(args[0].ty, self.ctx.f32()) {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "intrinsic argument type mismatch (expected f32)",
                                    *sp,
                                )
                                .with_id(DiagnosticId::TypeIntrinsicArgTypeMismatch),
                            );
                        }
                    } else if intrin.name == "u8_to_i32" || intrin.name == "u32_to_i32" {
                        if args.len() != 1 {
                            self.diagnostics.push(
                                Diagnostic::error("intrinsic expects 1 argument", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicArgArityMismatch),
                            );
                        } else {
                            let expected = if intrin.name == "u8_to_i32" {
                                self.ctx.u8()
                            } else {
                                self.ctx.lookup_named("u32").unwrap_or_else(|| {
                                    self.ctx.register_named(
                                        "u32".to_string(),
                                        TypeKind::Named("u32".to_string()),
                                    )
                                })
                            };
                            if let Err(_) = self.ctx.unify(args[0].ty, expected) {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "intrinsic argument type mismatch (expected {})",
                                            self.ctx.type_to_string(expected)
                                        ),
                                        *sp,
                                    )
                                    .with_id(DiagnosticId::TypeIntrinsicArgTypeMismatch),
                                );
                            }
                        }
                    } else if intrin.name == "i64_to_u64" || intrin.name == "u64_to_i64" {
                        if args.len() != 1 {
                            self.diagnostics.push(
                                Diagnostic::error("intrinsic expects 1 argument", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicArgArityMismatch),
                            );
                        } else {
                            let expected = if intrin.name == "i64_to_u64" {
                                self.ctx.lookup_named("i64").unwrap_or_else(|| {
                                    self.ctx.register_named(
                                        "i64".to_string(),
                                        TypeKind::Named("i64".to_string()),
                                    )
                                })
                            } else {
                                self.ctx.lookup_named("u64").unwrap_or_else(|| {
                                    self.ctx.register_named(
                                        "u64".to_string(),
                                        TypeKind::Named("u64".to_string()),
                                    )
                                })
                            };
                            if let Err(_) = self.ctx.unify(args[0].ty, expected) {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "intrinsic argument type mismatch (expected {})",
                                            self.ctx.type_to_string(expected)
                                        ),
                                        *sp,
                                    )
                                    .with_id(DiagnosticId::TypeIntrinsicArgTypeMismatch),
                                );
                            }
                        }
                    } else if intrin.name == "str_addr" || intrin.name == "str_from_addr_unchecked"
                    {
                        if args.len() != 1 {
                            self.diagnostics.push(
                                Diagnostic::error("intrinsic expects 1 argument", *sp)
                                    .with_id(DiagnosticId::TypeIntrinsicArgArityMismatch),
                            );
                        } else {
                            let expected = if intrin.name == "str_addr" {
                                self.ctx.str()
                            } else {
                                self.ctx.i32()
                            };
                            if let Err(_) = self.ctx.unify(args[0].ty, expected) {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        format!(
                                            "intrinsic argument type mismatch (expected {})",
                                            self.ctx.type_to_string(expected)
                                        ),
                                        *sp,
                                    )
                                    .with_id(DiagnosticId::TypeIntrinsicArgTypeMismatch),
                                );
                            }
                        }
                    }

                    stack.push(StackEntry {
                        ty,
                        expr: HirExpr {
                            ty,
                            kind: HirExprKind::Intrinsic {
                                name: intrin.name.clone(),
                                type_args,
                                args,
                            },
                            span: *sp,
                        },
                        type_args: Vec::new(),
                        assign: None,
                        auto_call: true,
                    });
                    last_expr = Some(stack.last().unwrap().expr.clone());
                }
                PrefixItem::TypeAnnotation(ty_expr, _span) => {
                    let ty = type_from_expr(self.ctx, self.labels, ty_expr);
                    // record target type and current stack depth; do NOT treat as an expression
                    pending_ascription = Some((ty, stack.len()));
                }
                PrefixItem::Match(mexpr, _sp) => {
                    if let Some((hexpr, ty)) = self.check_match_expr(mexpr) {
                        stack.push(StackEntry {
                            ty,
                            expr: hexpr,
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: true,
                        });
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    }
                }
                PrefixItem::Tuple(items, sp) => {
                    let mut elems = Vec::new();
                    let mut elem_tys = Vec::new();
                    for elem in items {
                        let mut elem_stack = Vec::new();
                        if let Some((hexpr, _)) = self.check_prefix(elem, 0, &mut elem_stack, None)
                        {
                            elem_tys.push(hexpr.ty);
                            elems.push(hexpr);
                        } else {
                            return None;
                        }
                    }
                    let ty = self.ctx.tuple(elem_tys);
                    stack.push(StackEntry {
                        ty,
                        expr: HirExpr {
                            ty,
                            kind: HirExprKind::TupleConstruct { items: elems },
                            span: *sp,
                        },
                        type_args: Vec::new(),
                        assign: None,
                        auto_call: true,
                    });
                    last_expr = Some(stack.last().unwrap().expr.clone());
                }
                PrefixItem::Group(inner, _sp) => {
                    let mut group_stack = Vec::new();
                    if let Some((hexpr, _)) = self.check_prefix(inner, 0, &mut group_stack, None) {
                        stack.push(StackEntry {
                            ty: hexpr.ty,
                            expr: hexpr,
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: true,
                        });
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    } else {
                        return None;
                    }
                }
                PrefixItem::Block(b, sp) => {
                    // Treat blocks uniformly; parser now desugars `if:`/`if <cond>:`
                    // layout forms into ordinary prefix items, so the checker
                    // should not special-case `if` here.
                    let (blk, val_ty) = self.check_block(b, 0, true, None)?;
                    if let Some(ty) = val_ty {
                        stack.push(StackEntry {
                            ty,
                            expr: HirExpr {
                                ty,
                                kind: HirExprKind::Block(blk),
                                span: *sp,
                            },
                            type_args: Vec::new(),
                            assign: None,
                            auto_call: false,
                        });
                        // defer applying ascription until the expression is complete
                        last_expr = Some(stack.last().unwrap().expr.clone());
                    } else {
                        last_expr = Some(HirExpr {
                            ty: self.ctx.unit(),
                            kind: HirExprKind::Block(blk),
                            span: *sp,
                        });
                    }
                }
                PrefixItem::Pipe(sp) => {
                    if pipe_pending.is_some() {
                        self.diagnostics.push(
                            Diagnostic::error(
                                "pipe already pending; consecutive |> not allowed",
                                *sp,
                            )
                            .with_id(DiagnosticId::TypePipeError),
                        );
                        continue;
                    }
                    // Don't drain past any let/set binding on the stack.
                    let default_pipe_base = stack
                        .iter()
                        .rposition(|e| e.assign.is_some())
                        .map(|p| base_depth.max(p + 1))
                        .unwrap_or(base_depth);
                    let pipe_base =
                        self.pipe_pending_base(stack.as_slice(), &open_calls, default_pipe_base);
                    if stack.len() == pipe_base {
                        self.diagnostics.push(
                            Diagnostic::error("pipe requires a value on the stack", *sp)
                                .with_id(DiagnosticId::TypePipeError),
                        );
                        continue;
                    }
                    let pending = stack.drain(pipe_base..).collect::<Vec<_>>();
                    last_expr = pending.last().map(|se| se.expr.clone());
                    pipe_pending = Some(pending);
                    seen_pipe = true;
                }
            }

            if !matches!(item, PrefixItem::Pipe(_) | PrefixItem::TypeAnnotation(_, _)) {
                if let Some(pending) = pipe_pending.take() {
                    // The last pushed element should be a callable (function type)
                    if let Some(top) = stack.last() {
                        if top.auto_call
                            && matches!(self.ctx.get(top.ty), TypeKind::Function { .. })
                        {
                            let Some(lowered_val) = self.reduce_pipe_pending_segment_with_target(
                                pending,
                                top,
                                pending_ascription.map(|(target, _)| target),
                            ) else {
                                self.diagnostics.push(
                                    Diagnostic::error(
                                        "pipe left-hand side did not reduce to a single value",
                                        expr.span,
                                    )
                                    .with_id(DiagnosticId::TypePipeError),
                                );
                                continue;
                            };
                            // pipe では「関数を積んだ直後に引数を注入」するため、
                            // 通常の末尾関数追跡だけでは open_calls に載らない。
                            let func_idx = stack.len() - 1;
                            if !open_calls.iter().any(|&i| i == func_idx) {
                                open_calls.push(func_idx);
                            }
                            stack.push(lowered_val);
                            last_expr = Some(stack.last().unwrap().expr.clone());
                        } else {
                            self.diagnostics.push(
                                Diagnostic::error(
                                    "pipe target must be a callable expression",
                                    expr.span,
                                )
                                .with_id(DiagnosticId::TypePipeError),
                            );
                            stack.extend(pending);
                        }
                    } else {
                        self.diagnostics.push(
                            Diagnostic::error("pipe target missing", expr.span)
                                .with_id(DiagnosticId::TypePipeError),
                        );
                        stack.extend(pending);
                    }
                }
            }

            // Maintain open_calls stack
            open_calls.retain(|&i| i < stack.len());
            if let Some(top) = stack.last() {
                let idx = stack.len() - 1;
                let rty = self.ctx.resolve(top.ty);
                if top.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. }) {
                    if open_calls.last() != Some(&idx) {
                        open_calls.push(idx);
                    }
                }
            }

            // Try applying pending ascription before call reduction.
            // If the next token is `|>`, defer ascription until pipe injection
            // and subsequent call reduction are completed.
            if !next_is_pipe
                || is_local_pending_ascription(stack.as_slice(), pending_ascription, base_depth)
            {
                try_apply_pending_ascription(self, stack, &mut pending_ascription);
            }

            let has_more_items = idx + 1 < expr.items.len();
            let defer_unresolved_overload = pipe_pending.is_none()
                && has_more_items
                && (0..stack.len()).rev().any(|i| {
                    let entry = &stack[i];
                    let rty = self.ctx.resolve_id(entry.ty);
                    entry.auto_call
                        && matches!(self.ctx.get(rty), TypeKind::Function { .. })
                        && self.unresolved_overloaded_entry_has_larger_arity(stack, i)
                });
            let delay_overloaded_nullary = pipe_pending.is_none()
                && stack.len() == base_depth + 1
                && stack
                    .last()
                    .map(|entry| {
                        if !self.is_unresolved_overloaded_callable_entry(entry) {
                            return false;
                        }
                        let rty = self.ctx.resolve_id(entry.ty);
                        let TypeKind::Function { params, .. } = self.ctx.get(rty) else {
                            return false;
                        };
                        self.user_visible_arity(&entry.expr, &params) == 0
                            && (next_is_pipe || has_more_items)
                    })
                    .unwrap_or(false);
            if !delay_overloaded_nullary && !defer_unresolved_overload {
                let mut pending_base = pending_ascription.map(|(_, base)| base);
                let mut pipe_guard = false;
                let reduction_expected = if next_is_pipe && seen_pipe {
                    pending_ascription.filter(|pending| {
                        is_local_pending_ascription(stack.as_slice(), Some(*pending), base_depth)
                    })
                } else {
                    pending_ascription
                };
                if next_is_pipe {
                    if let Some(assign_pos) = stack.iter().rposition(|e| e.assign.is_some()) {
                        let guard_pos = assign_pos + 1;
                        pending_base =
                            Some(pending_base.map_or(guard_pos, |base| base.max(guard_pos)));
                        pipe_guard = true;
                    }
                }
                if let Some(base_len) = pending_base {
                    self.reduce_calls_guarded(stack, &mut open_calls, base_len, reduction_expected);
                } else {
                    self.reduce_calls(stack, &mut open_calls, reduction_expected);
                }
                // std::eprintln!("  Stack after reduce: {:?}", stack.iter().map(|e| self.ctx.type_to_string(e.ty)).collect::<Vec<_>>());

                // Try applying pending ascription after call reduction.
                if !next_is_pipe
                    || is_local_pending_ascription(stack.as_slice(), pending_ascription, base_depth)
                {
                    try_apply_pending_ascription(self, stack, &mut pending_ascription);
                }

                if pending_base.is_some() && pending_ascription.is_none() && !pipe_guard {
                    self.reduce_calls(stack, &mut open_calls, reduction_expected);
                }
            }
        }

        if pipe_pending.is_some() {
            self.diagnostics.push(
                Diagnostic::error("pipe has no target", expr.span)
                    .with_id(DiagnosticId::TypePipeError),
            );
        }

        let leading_let = matches!(
            expr.items.first(),
            Some(PrefixItem::Symbol(Symbol::Let { .. }))
        );
        if leading_let {
            let mut pending_base = pending_ascription.map(|(_, base)| base);
            let mut open_calls: Vec<usize> = Vec::new();
            if let Some(base_len) = pending_base {
                self.reduce_calls_guarded(stack, &mut open_calls, base_len, pending_ascription);
            } else {
                self.reduce_calls(stack, &mut open_calls, pending_ascription);
            }
            try_apply_pending_ascription(self, stack, &mut pending_ascription);
            pending_base = pending_ascription.map(|(_, base)| base);
            if let Some(base_len) = pending_base {
                self.reduce_calls_guarded(stack, &mut open_calls, base_len, pending_ascription);
            } else {
                self.reduce_calls(stack, &mut open_calls, pending_ascription);
            }
        }
        // Validate final stack depth. `let` is special-cased because its RHS
        // expression remains on stack until we lower to `HirExprKind::Let`.
        if leading_let {
            if stack.len() > base_depth + 2 {
                let extras = stack.len() - (base_depth + 2);
                for _ in 0..extras {
                    stack.pop();
                }
                dropped = true;
            }
        } else if stack.len() > base_depth + 1 {
            let extras = stack.len() - (base_depth + 1);
            if crate::log::is_verbose() {
                let tys = stack
                    .iter()
                    .map(|e| self.ctx.type_to_string(e.ty))
                    .collect::<Vec<_>>()
                    .join(", ");
                prefix_check_log!("prefix final extras before trim [{}]", tys);
            }
            for _ in 0..extras {
                stack.pop();
            }
            dropped = true;
        }

        let result_expr = if leading_let && stack.len() >= base_depth + 2 {
            stack[base_depth + 1].expr.clone()
        } else if stack.len() == base_depth + 1 {
            if leading_let {
                stack.last().unwrap().expr.clone()
            } else {
                let top = stack.last_mut().unwrap();
                let placeholder = HirExpr {
                    ty: top.expr.ty,
                    kind: HirExprKind::Unit,
                    span: top.expr.span,
                };
                core::mem::replace(&mut top.expr, placeholder)
            }
        } else if let Some(ref e) = last_expr {
            e.clone()
        } else {
            HirExpr {
                ty: self.ctx.unit(),
                kind: HirExprKind::Unit,
                span: expr.span,
            }
        };

        // If this prefix began with a `let` symbol but reduction did not
        // produce a `Let` HIR node (e.g. for layout/colon forms), lower it
        // here: update the hoisted binding, mark it defined, and return a
        // `Let` expression so downstream codegen sees a stable binding.
        if let Some(PrefixItem::Symbol(Symbol::Let { name, mutable, .. })) = expr.items.first() {
            if !matches!(result_expr.kind, HirExprKind::Let { .. }) {
                // If RHS remains on stack (auto_call disabled for `let`), use it as
                // the binding value directly.
                let value_expr = if stack.len() >= base_depth + 2 {
                    stack[base_depth + 1].expr.clone()
                } else {
                    match &result_expr.kind {
                        HirExprKind::Var(n) if n == &name.name => {
                            if let Some(le) = last_expr.clone() {
                                le
                            } else {
                                result_expr.clone()
                            }
                        }
                        HirExprKind::Block(blk) => {
                            // Detect `if:` layout: block with exactly 3 lines and the
                            // original prefix contained an `if` symbol. In that case
                            // synthesize an `If` node from the three lines.
                            if blk.lines.len() == 3
                                && expr
                                    .items
                                    .iter()
                                    .any(|it| matches!(it, PrefixItem::Symbol(Symbol::If(_))))
                            {
                                let cond = blk.lines[0].expr.clone();
                                let then_branch = blk.lines[1].expr.clone();
                                let else_branch = blk.lines[2].expr.clone();
                                HirExpr {
                                    ty: then_branch.ty,
                                    kind: HirExprKind::If {
                                        cond: Box::new(cond),
                                        then_branch: Box::new(then_branch),
                                        else_branch: Box::new(else_branch),
                                    },
                                    span: result_expr.span,
                                }
                            } else {
                                result_expr.clone()
                            }
                        }
                        _ => result_expr.clone(),
                    }
                };

                if let Some(b) = self.env.lookup_mut(&name.name) {
                    b.defined = true;
                    b.ty = value_expr.ty;
                }
                let let_expr = HirExpr {
                    ty: self.ctx.unit(),
                    kind: HirExprKind::Let {
                        name: name.name.clone(),
                        mutable: *mutable,
                        value: Box::new(value_expr),
                    },
                    span: expr.span,
                };
                while stack.len() > base_depth {
                    let _ = stack.pop();
                }
                stack.push(StackEntry {
                    ty: self.ctx.unit(),
                    expr: let_expr.clone(),
                    type_args: Vec::new(),
                    assign: None,
                    auto_call: true,
                });
                return Some((let_expr, dropped));
            }
        }

        Some((result_expr, dropped))
    }
}
