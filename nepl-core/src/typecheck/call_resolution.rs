use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{
    Block, Effect, FnBody, FnDef, MatchPattern, PrefixExpr, PrefixItem, Stmt, Symbol,
};
use crate::hir::{HirExpr, HirExprKind};
use crate::types::{TypeId, TypeKind};

use super::env::BindingKind;
use super::traits::insert_substitution_mapping;
use super::{BlockChecker, StackEntry};

impl<'a> BlockChecker<'a> {
    pub(super) fn user_visible_arity(&self, func_expr: &HirExpr, params: &[TypeId]) -> usize {
        let total_param_len = params.len();
        if let HirExprKind::Var(name) = &func_expr.kind {
            let bindings = self.env.lookup_all_callables(name);
            if !bindings.is_empty() {
                let mut arity: Option<usize> = None;
                for b in bindings {
                    if let BindingKind::Func { arity: current, .. } = &b.kind {
                        match arity {
                            Some(prev) if prev != *current => return total_param_len,
                            Some(_) => {}
                            None => arity = Some(*current),
                        }
                    }
                }
                if let Some(arity) = arity {
                    return arity;
                }
            }
        }
        total_param_len
    }

    pub(super) fn collect_bound_names_from_prefix(expr: &PrefixExpr, out: &mut BTreeSet<String>) {
        for item in &expr.items {
            match item {
                PrefixItem::Symbol(Symbol::Let { name, .. }) => {
                    out.insert(name.name.clone());
                }
                PrefixItem::Block(b, _) => {
                    Self::collect_bound_names_from_block(b, out);
                }
                PrefixItem::Match(m, _) => {
                    for arm in &m.arms {
                        if let MatchPattern::Variant { bind: Some(b), .. } = &arm.pattern {
                            out.insert(b.name.clone());
                        }
                        Self::collect_bound_names_from_block(&arm.body, out);
                    }
                }
                _ => {}
            }
        }
    }

    pub(super) fn collect_bound_names_from_block(block: &Block, out: &mut BTreeSet<String>) {
        for stmt in &block.items {
            match stmt {
                Stmt::Expr(e) | Stmt::ExprSemi(e, _) => {
                    Self::collect_bound_names_from_prefix(e, out);
                }
                Stmt::FnDef(f) => {
                    out.insert(f.name.name.clone());
                }
                _ => {}
            }
        }
    }

    pub(super) fn collect_ref_names_from_prefix(expr: &PrefixExpr, out: &mut BTreeSet<String>) {
        for item in &expr.items {
            match item {
                PrefixItem::Symbol(Symbol::Ident(id, _, _)) => {
                    out.insert(id.name.clone());
                }
                PrefixItem::Block(b, _) => {
                    Self::collect_ref_names_from_block(b, out);
                }
                PrefixItem::Match(m, _) => {
                    Self::collect_ref_names_from_prefix(&m.scrutinee, out);
                    for arm in &m.arms {
                        Self::collect_ref_names_from_block(&arm.body, out);
                    }
                }
                PrefixItem::Tuple(items, _) => {
                    for it in items {
                        Self::collect_ref_names_from_prefix(it, out);
                    }
                }
                PrefixItem::Group(inner, _) => {
                    Self::collect_ref_names_from_prefix(inner, out);
                }
                _ => {}
            }
        }
    }

    pub(super) fn collect_ref_names_from_block(block: &Block, out: &mut BTreeSet<String>) {
        for stmt in &block.items {
            match stmt {
                Stmt::Expr(e) | Stmt::ExprSemi(e, _) => {
                    Self::collect_ref_names_from_prefix(e, out);
                }
                Stmt::FnDef(_) => {}
                _ => {}
            }
        }
    }

    pub(super) fn collect_nested_fn_captures(&self, f: &FnDef) -> Vec<(String, TypeId)> {
        let FnBody::Parsed(body) = &f.body else {
            return Vec::new();
        };
        let mut refs = BTreeSet::new();
        let mut bounds = BTreeSet::new();
        for p in &f.params {
            bounds.insert(p.name.clone());
        }
        Self::collect_bound_names_from_block(body, &mut bounds);
        Self::collect_ref_names_from_block(body, &mut refs);
        let mut captures = Vec::new();
        for name in refs {
            if bounds.contains(&name) || name == f.name.name {
                continue;
            }
            if let Some(b) = self.env.lookup_any(&name) {
                if matches!(b.kind, BindingKind::Var) {
                    captures.push((name, b.ty));
                }
            }
        }
        captures
    }

    pub(super) fn find_outer_function_consumer(
        &mut self,
        stack: &[StackEntry],
        inner_pos: usize,
        min_func_pos: usize,
    ) -> Option<usize> {
        for j in (min_func_pos..inner_pos).rev() {
            if self.is_unresolved_overloaded_callable_entry(&stack[j]) {
                continue;
            }
            if !stack[j].auto_call {
                continue;
            }
            let Some((params, _result, _effect)) = self.function_signature_for_entry(&stack[j])
            else {
                continue;
            };
            let total_arity = params.len();
            let arity = self.user_visible_arity(&stack[j].expr, &params);
            if stack.len() < j + 1 + arity {
                continue;
            }
            if inner_pos < j + 1 {
                continue;
            }
            let user_arg_idx = inner_pos - (j + 1);
            if user_arg_idx >= arity {
                continue;
            }
            let capture_len = total_arity.saturating_sub(arity);
            let arg_idx = capture_len + user_arg_idx;
            if arg_idx >= total_arity {
                continue;
            }
            let pty = self.ctx.resolve_id(params[arg_idx]);
            if matches!(self.ctx.get(pty), TypeKind::Function { .. }) {
                return Some(j);
            }
        }
        None
    }

    pub(super) fn infer_expected_from_outer_consumer(
        &mut self,
        stack: &[StackEntry],
        inner_pos: usize,
        min_func_pos: usize,
    ) -> Option<TypeId> {
        for j in (min_func_pos..inner_pos).rev() {
            if self.is_unresolved_overloaded_callable_entry(&stack[j]) {
                continue;
            }
            if !stack[j].auto_call {
                continue;
            }
            let Some((params, _result, _effect)) = self.function_signature_for_entry(&stack[j])
            else {
                continue;
            };
            let total_arity = params.len();
            let arity = self.user_visible_arity(&stack[j].expr, &params);
            if stack.len() < j + 1 + arity {
                continue;
            }
            if inner_pos < j + 1 {
                continue;
            }
            let user_arg_idx = inner_pos - (j + 1);
            if user_arg_idx >= arity {
                continue;
            }
            if self.has_unresolved_callable_between(stack, j + 1, inner_pos) {
                continue;
            }
            let capture_len = total_arity.saturating_sub(arity);
            let arg_idx = capture_len + user_arg_idx;
            if arg_idx >= total_arity {
                continue;
            }
            // Slots after the current argument may still be arguments to the
            // nested callable being reduced, not siblings of the outer call.
            // Only earlier outer arguments are known to be complete here.
            for k in 0..user_arg_idx {
                let outer_arg_pos = j + 1 + k;
                if outer_arg_pos >= stack.len() {
                    continue;
                }
                let pidx = capture_len + k;
                if pidx >= total_arity {
                    continue;
                }
                let pty = params[pidx];
                let aty = stack[outer_arg_pos].ty;
                let _ = self.ctx.unify(aty, pty);
            }
            return Some(self.ctx.resolve_id(params[arg_idx]));
        }
        None
    }

    pub(super) fn infer_expected_from_outer_consumer_next_arg(
        &mut self,
        stack: &[StackEntry],
        inner_pos: usize,
        min_func_pos: usize,
    ) -> Option<TypeId> {
        for j in (min_func_pos..inner_pos).rev() {
            if self.is_unresolved_overloaded_callable_entry(&stack[j]) {
                continue;
            }
            if !stack[j].auto_call {
                continue;
            }
            let Some((params, _result, _effect)) = self.function_signature_for_entry(&stack[j])
            else {
                continue;
            };
            let total_arity = params.len();
            let arity = self.user_visible_arity(&stack[j].expr, &params);
            if inner_pos < j + 1 {
                continue;
            }
            let provided_user_args = inner_pos - (j + 1);
            if provided_user_args >= arity {
                continue;
            }
            if self.has_unresolved_callable_between(stack, j + 1, inner_pos) {
                continue;
            }
            let user_arg_idx = provided_user_args;
            let capture_len = total_arity.saturating_sub(arity);
            let arg_idx = capture_len + user_arg_idx;
            if arg_idx >= total_arity {
                continue;
            }
            for k in 0..provided_user_args {
                let outer_arg_pos = j + 1 + k;
                if outer_arg_pos >= stack.len() {
                    continue;
                }
                let pidx = capture_len + k;
                if pidx >= total_arity {
                    continue;
                }
                let pty = params[pidx];
                let aty = stack[outer_arg_pos].ty;
                let _ = self.ctx.unify(aty, pty);
            }
            return Some(self.ctx.resolve_id(params[arg_idx]));
        }
        None
    }

    pub(super) fn is_unresolved_overloaded_callable_entry(&self, entry: &StackEntry) -> bool {
        let HirExprKind::Var(name) = &entry.expr.kind else {
            return false;
        };
        if !entry.type_args.is_empty() {
            return false;
        }
        self.env.lookup_all_callables(name).len() > 1
    }

    pub(super) fn function_signature_for_entry(
        &mut self,
        entry: &StackEntry,
    ) -> Option<(Vec<TypeId>, TypeId, Effect)> {
        let rty = self.ctx.resolve_id(entry.ty);
        let TypeKind::Function {
            type_params,
            params,
            result,
            effect,
        } = self.ctx.get(rty)
        else {
            return None;
        };
        if entry.type_args.is_empty() {
            if !type_params.is_empty() {
                let (inst_ty, _fresh_args, _mapping) = self.ctx.instantiate(rty);
                if let TypeKind::Function {
                    params,
                    result,
                    effect,
                    ..
                } = self.ctx.get(inst_ty)
                {
                    return Some((params, result, effect));
                }
                return None;
            }
            return Some((params, result, effect));
        }
        if type_params.len() != entry.type_args.len() {
            // The entry.ty is likely a fresh placeholder (0 type_params) created when
            // the callable was pushed with explicit type args. Look up the actual binding
            // type by name so we can apply the type args correctly.
            if let HirExprKind::Var(name) = &entry.expr.kind {
                let name = name.clone();
                let type_args = entry.type_args.clone();
                let binding_tys: Vec<TypeId> = self
                    .env
                    .lookup_all_callables(&name)
                    .into_iter()
                    .map(|b| b.ty)
                    .collect();
                for binding_ty in binding_tys {
                    let func_data = if let TypeKind::Function {
                        type_params: tps,
                        params: ps,
                        result: r,
                        effect: e,
                    } = self.ctx.get(binding_ty)
                    {
                        Some((tps, ps, r, e))
                    } else {
                        None
                    };
                    let Some((tps, ps, r, e)) = func_data else {
                        continue;
                    };
                    if tps.len() != type_args.len() {
                        continue;
                    }
                    let mut mapping = BTreeMap::new();
                    for (tp, ta) in tps.iter().zip(type_args.iter()) {
                        insert_substitution_mapping(self.ctx, &mut mapping, *tp, *ta);
                    }
                    let sub_params = ps
                        .iter()
                        .map(|p| self.ctx.substitute(*p, &mapping))
                        .collect::<Vec<_>>();
                    let sub_result = self.ctx.substitute(r, &mapping);
                    return Some((sub_params, sub_result, e));
                }
            }
            return None;
        }
        let mut mapping = BTreeMap::new();
        for (p, a) in type_params.iter().zip(entry.type_args.iter()) {
            insert_substitution_mapping(self.ctx, &mut mapping, *p, *a);
        }
        let substituted_params = params
            .iter()
            .map(|p| self.ctx.substitute(*p, &mapping))
            .collect::<Vec<_>>();
        let substituted_result = self.ctx.substitute(result, &mapping);
        Some((substituted_params, substituted_result, effect))
    }

    pub(super) fn pipe_target_input_type(&mut self, entry: &StackEntry) -> Option<TypeId> {
        let Some((params, _result, _effect)) = self.function_signature_for_entry(entry) else {
            return None;
        };
        let total_arity = params.len();
        let arity = self.user_visible_arity(&entry.expr, &params);
        if arity == 0 {
            return None;
        }
        let capture_len = total_arity.saturating_sub(arity);
        let arg_idx = capture_len;
        if arg_idx >= total_arity {
            return None;
        }
        Some(self.ctx.resolve_id(params[arg_idx]))
    }

    pub(super) fn reduce_pipe_pending_segment_with_target(
        &mut self,
        mut pending: Vec<StackEntry>,
        target: &StackEntry,
        fallback_expected: Option<TypeId>,
    ) -> Option<StackEntry> {
        if pending.is_empty() {
            return None;
        }
        let expected_input = self
            .pipe_target_input_type(target)
            .filter(|t| self.is_concrete_type(*t))
            .or(fallback_expected.map(|t| self.ctx.resolve_id(t)));
        let mut open_calls = Vec::new();
        for (i, entry) in pending.iter().enumerate() {
            let rty = self.ctx.resolve_id(entry.ty);
            if entry.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. }) {
                open_calls.push(i);
            }
        }
        self.reduce_calls(
            &mut pending,
            &mut open_calls,
            expected_input.map(|t| (t, 0)),
        );
        if pending.len() == 1 {
            pending.pop()
        } else {
            None
        }
    }

    pub(super) fn pipe_pending_base(
        &mut self,
        stack: &[StackEntry],
        open_calls: &[usize],
        default_base: usize,
    ) -> usize {
        if stack.len() <= default_base + 1 {
            return default_base;
        }
        let top_idx = stack.len() - 1;
        let Some(_) = open_calls
            .iter()
            .rev()
            .copied()
            .find(|&idx| idx >= default_base && idx < top_idx)
        else {
            return default_base;
        };
        if self.pipe_segment_reduces_to_single_value(stack, default_base) {
            return default_base;
        }
        for idx in open_calls.iter().copied() {
            if idx < default_base || idx >= top_idx {
                continue;
            }
            if self.pipe_segment_reduces_to_single_value(stack, idx) {
                return idx;
            }
        }
        top_idx
    }

    pub(super) fn pipe_segment_reduces_to_single_value(
        &mut self,
        stack: &[StackEntry],
        segment_base: usize,
    ) -> bool {
        if segment_base >= stack.len() {
            return false;
        }
        let checkpoint = self.ctx.checkpoint();
        let diagnostics_len = self.diagnostics.len();
        let trait_checks_len = self.pending_trait_bound_checks.len();
        let mut segment = stack[segment_base..].to_vec();
        let mut open_calls = segment
            .iter()
            .enumerate()
            .filter_map(|(idx, entry)| {
                let rty = self.ctx.resolve_id(entry.ty);
                if entry.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. }) {
                    Some(idx)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        self.reduce_calls(&mut segment, &mut open_calls, None);
        let reduced = segment.len() == 1;
        self.pending_trait_bound_checks.truncate(trait_checks_len);
        self.diagnostics.truncate(diagnostics_len);
        self.ctx.rollback(checkpoint);
        reduced
    }

    pub(super) fn has_unresolved_callable_between(
        &self,
        stack: &[StackEntry],
        start: usize,
        end_exclusive: usize,
    ) -> bool {
        if start >= end_exclusive || start >= stack.len() {
            return false;
        }
        let end = end_exclusive.min(stack.len());
        for i in start..end {
            if self.is_unresolved_overloaded_callable_entry(&stack[i]) {
                return true;
            }
        }
        false
    }

    pub(super) fn unresolved_overloaded_entry_has_larger_arity(
        &mut self,
        stack: &[StackEntry],
        pos: usize,
    ) -> bool {
        if pos >= stack.len() {
            return false;
        }
        let entry = &stack[pos];
        if !self.is_unresolved_overloaded_callable_entry(entry) {
            return false;
        }
        let available_args = stack.len().saturating_sub(pos + 1);
        match &entry.expr.kind {
            HirExprKind::Var(name) => self.env.lookup_all_callables(name).iter().any(|b| match &b
                .kind
            {
                BindingKind::Func {
                    arity, captures, ..
                } => arity.saturating_sub(captures.len()) > available_args,
                _ => false,
            }),
            _ => false,
        }
    }

    pub(super) fn should_defer_overloaded_nullary_entry(
        &mut self,
        stack: &[StackEntry],
        pos: usize,
    ) -> bool {
        if pos >= stack.len() {
            return false;
        }
        let entry = &stack[pos];
        if !self.is_unresolved_overloaded_callable_entry(entry) {
            return false;
        }
        let has_nullary_overload = match &entry.expr.kind {
            HirExprKind::Var(name) => self.env.lookup_all_callables(name).iter().any(|b| match &b
                .kind
            {
                BindingKind::Func {
                    arity, captures, ..
                } => arity.saturating_sub(captures.len()) == 0,
                _ => false,
            }),
            _ => false,
        };
        if !has_nullary_overload {
            return false;
        }
        stack.iter().skip(pos + 1).any(|entry| {
            let rty = self.ctx.resolve_id(entry.ty);
            entry.auto_call && matches!(self.ctx.get(rty), TypeKind::Function { .. })
        })
    }

    pub(super) fn choose_callable_type_by_available_arity(
        &mut self,
        name: &str,
        available_args: usize,
    ) -> Option<(usize, TypeId)> {
        let callables = self.env.lookup_all_callables(name);
        if callables.len() <= 1 {
            return None;
        }
        let mut has_mixed_arity = false;
        let mut first_arity: Option<usize> = None;
        for b in &callables {
            if let BindingKind::Func { arity, .. } = b.kind {
                if first_arity.is_none() {
                    first_arity = Some(arity);
                } else if first_arity != Some(arity) {
                    has_mixed_arity = true;
                }
            }
        }
        // In a pure context, prefer pure overloads over impure ones.
        // When selecting by arity, a pure lower-arity overload beats an impure
        // higher-arity one to prevent false D3025 errors from name collisions
        // across modules (e.g. math::add vs fenwick::add in a pure fold).
        let in_pure_context = matches!(self.current_effect, Effect::Pure);

        // Also proceed when arities are uniform but purity is mixed — e.g.
        // vec::with_capacity (pure) vs ringbuffer::with_capacity (impure) both
        // have arity 1.  In a pure context we must pick the pure variant to
        // avoid a spurious D3025 before full overload resolution runs.
        let has_mixed_purity_among_applicable = in_pure_context && {
            let mut has_pure = false;
            let mut has_impure = false;
            for b in &callables {
                if let BindingKind::Func { arity, .. } = b.kind {
                    if arity <= available_args {
                        if matches!(
                            self.ctx.get(self.ctx.resolve_id(b.ty)),
                            TypeKind::Function {
                                effect: Effect::Pure,
                                ..
                            }
                        ) {
                            has_pure = true;
                        } else {
                            has_impure = true;
                        }
                    }
                }
            }
            has_pure && has_impure
        };
        if !has_mixed_arity && !has_mixed_purity_among_applicable {
            return None;
        }

        let mut best: Option<(usize, TypeId, bool)> = None; // (arity, ty, is_pure)
        for b in callables {
            if let BindingKind::Func { arity, .. } = b.kind {
                if arity > available_args {
                    continue;
                }
                let is_pure = matches!(
                    self.ctx.get(self.ctx.resolve_id(b.ty)),
                    TypeKind::Function {
                        effect: Effect::Pure,
                        ..
                    }
                );
                let should_replace = match &best {
                    None => true,
                    Some((_best_arity, _, best_is_pure)) if in_pure_context => {
                        // Pure candidate always beats impure; among same purity prefer higher arity
                        (is_pure && !best_is_pure)
                            || (is_pure == *best_is_pure && arity > *_best_arity)
                    }
                    Some((best_arity, _, _)) => arity > *best_arity,
                };
                if should_replace {
                    best = Some((arity, b.ty, is_pure));
                }
            }
        }
        best.map(|(arity, ty, _)| (arity, ty))
    }
}
