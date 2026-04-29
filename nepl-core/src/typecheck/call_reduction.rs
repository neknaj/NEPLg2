use alloc::collections::BTreeSet;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::{Ident, MatchPattern};
use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::hir::{HirExpr, HirExprKind};
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::diagnostics::type_error;
use super::env::BindingKind;
use super::{BlockChecker, FieldIdx, StackEntry};

fn call_reduction_dump_enabled() -> bool {
    #[cfg(target_os = "none")]
    {
        false
    }
    #[cfg(not(target_os = "none"))]
    {
        std::env::var("NEPL_DUMP_HIR").is_ok()
    }
}

macro_rules! call_reduction_log {
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

macro_rules! call_reduction_dump {
    ($($arg:tt)*) => {
        if call_reduction_dump_enabled() {
            call_reduction_log!($($arg)*);
        }
    };
}

impl<'a> BlockChecker<'a> {
    pub(super) fn stack_entry_is_open_call(&mut self, entry: &StackEntry) -> bool {
        let rty = self.ctx.resolve(entry.ty);
        entry.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. })
    }

    pub(super) fn rebuild_open_calls(
        &mut self,
        stack: &[StackEntry],
        open_calls: &mut Vec<usize>,
        min_func_pos: usize,
    ) {
        open_calls.clear();
        for i in min_func_pos..stack.len() {
            if self.stack_entry_is_open_call(&stack[i]) {
                open_calls.push(i);
            }
        }
    }

    pub(super) fn next_reducible_call_pos(
        &mut self,
        stack: &[StackEntry],
        open_calls: &mut Vec<usize>,
        min_func_pos: usize,
    ) -> Option<usize> {
        if open_calls.is_empty() {
            self.rebuild_open_calls(stack, open_calls, min_func_pos);
        }
        let mut cursor = open_calls.len();
        while cursor > 0 {
            cursor -= 1;
            let i = open_calls[cursor];
            if i < min_func_pos || i >= stack.len() || !self.stack_entry_is_open_call(&stack[i]) {
                open_calls.remove(cursor);
                continue;
            }
            if self.should_defer_overloaded_nullary_entry(stack, i) {
                continue;
            }
            return Some(i);
        }
        None
    }

    pub(super) fn update_open_calls_after_reduction(
        &mut self,
        stack: &[StackEntry],
        open_calls: &mut Vec<usize>,
        func_pos: usize,
        args_to_take: usize,
    ) {
        let removed_end = func_pos + 1 + args_to_take;
        let first_removed = open_calls.partition_point(|&i| i < func_pos);
        let first_after_removed = open_calls.partition_point(|&i| i < removed_end);
        open_calls.drain(first_removed..first_after_removed);
        for i in &mut open_calls[first_removed..] {
            *i = i.saturating_sub(args_to_take);
        }
        if func_pos < stack.len() && self.stack_entry_is_open_call(&stack[func_pos]) {
            open_calls.insert(first_removed, func_pos);
        }
        open_calls.dedup();
    }

    pub(super) fn call_reduction_state_key(&self, stack: &[StackEntry]) -> String {
        let mut out = String::new();
        for entry in stack {
            out.push_str(&self.ctx.type_to_string(entry.ty));
            out.push(':');
            match &entry.expr.kind {
                HirExprKind::Var(name) => {
                    out.push_str("var:");
                    out.push_str(name);
                }
                HirExprKind::FnValue(name) => {
                    out.push_str("fn:");
                    out.push_str(name);
                }
                HirExprKind::Call { callee, args } => {
                    out.push_str("call:");
                    out.push_str(&format!("{:?}/{}", callee, args.len()));
                }
                HirExprKind::CallIndirect { args, .. } => {
                    out.push_str("call_indirect:");
                    out.push_str(&args.len().to_string());
                }
                _ => out.push_str("expr"),
            }
            out.push('|');
        }
        out
    }

    pub(super) fn reduce_calls_from(
        &mut self,
        stack: &mut Vec<StackEntry>,
        open_calls: &mut Vec<usize>,
        min_func_pos: usize,
        expected: Option<(TypeId, usize)>,
        label: &str,
    ) {
        let mut no_progress_states = BTreeSet::new();
        loop {
            call_reduction_dump!(
                "{}: stack=[{}]",
                label,
                stack
                    .iter()
                    .map(|e| match &e.expr.kind {
                        HirExprKind::Var(n) => n.clone(),
                        _ => "<expr>".to_string(),
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            );

            let Some(mut func_pos) = self.next_reducible_call_pos(stack, open_calls, min_func_pos)
            else {
                break;
            };
            if let Some(outer) = self.find_outer_function_consumer(stack, func_pos, min_func_pos) {
                func_pos = outer;
            }

            let available_args = stack.len().saturating_sub(func_pos + 1);
            let chosen_callable = match &stack[func_pos].expr.kind {
                HirExprKind::Var(name)
                    if stack[func_pos].type_args.is_empty()
                        && self.env.lookup_all_callables(name).len() > 1 =>
                {
                    self.choose_callable_type_by_available_arity(name, available_args)
                }
                HirExprKind::Var(name) if self.env.lookup_value(name).is_none() => {
                    self.choose_callable_type_by_available_arity(name, available_args)
                }
                _ => None,
            };
            let ty_for_infer = chosen_callable
                .map(|(_, ty)| ty)
                .unwrap_or(stack[func_pos].ty);
            let (inst_ty, _fresh_args) = if !stack[func_pos].type_args.is_empty() {
                (ty_for_infer, stack[func_pos].type_args.clone())
            } else {
                let (inst_ty, fresh_args, _mapping) = self.ctx.instantiate(ty_for_infer);
                (inst_ty, fresh_args)
            };
            let func_ty = self.ctx.get(inst_ty);
            let (params, result, effect) = match func_ty {
                TypeKind::Function {
                    params,
                    result,
                    effect,
                    ..
                } => (params, result, effect),
                _ => {
                    self.diagnostics.push(type_error(
                        TypeDiagnosticCode::CallReductionLimitExceeded,
                        "call reduction found non-function after instantiation",
                        stack[func_pos].expr.span,
                    ));
                    break;
                }
            };
            let needed_args = chosen_callable
                .map(|(arity, _)| arity)
                .unwrap_or_else(|| self.user_visible_arity(&stack[func_pos].expr, &params));
            let consume_unit_sugar = needed_args == 0
                && stack
                    .get(func_pos + 1)
                    .map(|e| matches!(e.expr.kind, HirExprKind::Unit))
                    .unwrap_or(false);
            let args_to_take = needed_args + if consume_unit_sugar { 1 } else { 0 };
            if stack.len() < func_pos + 1 + args_to_take {
                break;
            }
            let expected_ret = expected.and_then(|(target, base_len)| {
                let new_len = stack.len().saturating_sub(args_to_take);
                if new_len == base_len + 1 {
                    Some(target)
                } else {
                    None
                }
            });
            let outer_expected =
                self.infer_expected_from_outer_consumer(stack, func_pos, min_func_pos);
            let expected_ret = expected_ret.or(outer_expected);

            let before_len = stack.len();
            let drained = stack
                .drain(func_pos..func_pos + 1 + args_to_take)
                .collect::<Vec<_>>();
            let mut drained = drained.into_iter();
            let Some(mut func_entry) = drained.next() else {
                break;
            };
            let args = drained.collect::<Vec<_>>();
            func_entry.ty = inst_ty;
            func_entry.expr.ty = inst_ty;
            let explicit_type_args = func_entry.type_args.clone();
            let debug_name = match &func_entry.expr.kind {
                HirExprKind::Var(name) => Some(name.clone()),
                _ => None,
            };
            if crate::log::is_verbose() {
                call_reduction_log!(
                    "    Reducing {}: {} at pos {} with {} args, assign={:?}",
                    label,
                    self.ctx.type_to_string(inst_ty),
                    func_pos,
                    params.len(),
                    func_entry.assign
                );
                if label == "reduce_calls_guarded"
                    && matches!(
                        debug_name.as_deref(),
                        Some(
                            "get"
                                | "is_none"
                                | "must_hm"
                                | "make_hm"
                                | "new"
                                | "DefaultHash32"
                                | "A"
                                | "use_a"
                        )
                    )
                {
                    let before = stack
                        .iter()
                        .map(|e| self.ctx.type_to_string(e.ty))
                        .collect::<Vec<_>>()
                        .join(", ");
                    call_reduction_log!("      stack before guarded apply [{}]", before);
                }
            }
            let applied = self.apply_function(
                func_entry,
                params,
                result,
                effect,
                args,
                explicit_type_args,
                expected_ret,
            );

            if let Some(val) = applied {
                if crate::log::is_verbose()
                    && label == "reduce_calls_guarded"
                    && matches!(
                        debug_name.as_deref(),
                        Some(
                            "get"
                                | "is_none"
                                | "must_hm"
                                | "make_hm"
                                | "new"
                                | "DefaultHash32"
                                | "A"
                                | "use_a"
                        )
                    )
                {
                    call_reduction_log!("      guarded result {}", self.ctx.type_to_string(val.ty));
                }
                stack.insert(func_pos, val);
                self.update_open_calls_after_reduction(stack, open_calls, func_pos, args_to_take);
                if stack.len() >= before_len {
                    let state_key = self.call_reduction_state_key(stack);
                    if !no_progress_states.insert(state_key) {
                        let span = stack
                            .get(func_pos)
                            .map(|entry| entry.expr.span)
                            .unwrap_or_else(Span::dummy);
                        self.diagnostics.push(type_error(
                            TypeDiagnosticCode::CallReductionLimitExceeded,
                            "call reduction made no progress",
                            span,
                        ));
                        break;
                    }
                } else {
                    no_progress_states.clear();
                }
            } else {
                break;
            }
        }
    }

    pub(super) fn reduce_calls(
        &mut self,
        stack: &mut Vec<StackEntry>,
        open_calls: &mut Vec<usize>,
        expected: Option<(TypeId, usize)>,
    ) {
        self.reduce_calls_from(stack, open_calls, 0, expected, "reduce_calls");
    }

    pub(super) fn resolve_dotted_field_symbol(
        &mut self,
        id: &Ident,
        forced_value: bool,
    ) -> Option<StackEntry> {
        if !id.name.contains('.') || id.name.contains("::") {
            return None;
        }

        let mut parts = id.name.split('.');
        let base_name = parts.next()?;
        let base_binding = self.env.lookup_value(base_name)?;
        if !matches!(base_binding.kind, BindingKind::Var) {
            return None;
        }

        let mut current = HirExpr {
            ty: base_binding.ty,
            kind: HirExprKind::Var(base_name.to_string()),
            span: id.span,
        };
        let mut current_ty = base_binding.ty;

        for field_name in parts {
            let (field_ty, offset) = self.resolve_field_access(
                current_ty,
                FieldIdx::Name(field_name.to_string()),
                id.span,
            )?;
            let addr_expr = if offset == 0 {
                current
            } else {
                HirExpr {
                    ty: self.ctx.i32(),
                    kind: HirExprKind::Intrinsic {
                        name: "add".to_string(),
                        type_args: vec![self.ctx.i32()],
                        args: vec![
                            current,
                            HirExpr {
                                ty: self.ctx.i32(),
                                kind: HirExprKind::LiteralI32(offset as i32),
                                span: id.span,
                            },
                        ],
                    },
                    span: id.span,
                }
            };
            current = HirExpr {
                ty: field_ty,
                kind: HirExprKind::Intrinsic {
                    name: "load".to_string(),
                    type_args: vec![field_ty],
                    args: vec![addr_expr],
                },
                span: id.span,
            };
            current_ty = field_ty;
        }

        Some(StackEntry {
            ty: current_ty,
            expr: current,
            type_args: Vec::new(),
            assign: None,
            auto_call: !forced_value,
        })
    }

    pub(super) fn reduce_calls_guarded(
        &mut self,
        stack: &mut Vec<StackEntry>,
        open_calls: &mut Vec<usize>,
        min_func_pos: usize,
        expected: Option<(TypeId, usize)>,
    ) {
        self.reduce_calls_from(
            stack,
            open_calls,
            min_func_pos,
            expected,
            "reduce_calls_guarded",
        );
    }

    /// マッチアームのバリアント名からスクルーティニーの期待型を推論する。
    /// 例: `Result::Ok`, `Result::Err` → `Result<fresh_A, fresh_B>` を返す。
    /// これにより `match with_capacity<.T> n:` のような式でオーバーロードが
    /// 解決できるようになる（スクルーティニーに期待型が伝播される）。
    pub(super) fn infer_expected_type_from_match_arms(
        &mut self,
        arms: &[crate::ast::MatchArm],
    ) -> Option<TypeId> {
        for arm in arms {
            let variant_name = match &arm.pattern {
                MatchPattern::Variant { name, .. } => &name.name,
                _ => continue,
            };
            // "EnumName::VariantName" → "EnumName"
            let enum_name = if let Some(idx) = variant_name.rfind("::") {
                &variant_name[..idx]
            } else {
                continue;
            };
            let enum_info = self.enums.get(enum_name)?;
            let enum_ty = enum_info.ty;
            let type_params = enum_info.type_params.clone();
            return if type_params.is_empty() {
                Some(enum_ty)
            } else {
                let fresh_vars: Vec<TypeId> = type_params
                    .iter()
                    .map(|_| self.ctx.fresh_var(None))
                    .collect();
                Some(self.ctx.apply(enum_ty, fresh_vars))
            };
        }
        None
    }
}
