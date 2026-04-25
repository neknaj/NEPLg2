#![no_std]
extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_ids::DiagnosticId;
use crate::hir::{FuncRef, HirBlock, HirExpr, HirExprKind, HirModule};
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

/// Tracks ownership state of variables.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum VarState {
    Valid,
    BorrowedShared,
    BorrowedUnique,
    Moved,
    PossiblyMoved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BorrowKind {
    Shared,
    Unique,
}

struct MoveCheckContext {
    /// Function parameter types after monomorphization.
    function_params: BTreeMap<String, Vec<TypeId>>,
    /// State of all variables currently in scope.
    /// Stack of variable states (for shadowing support).
    var_stacks: BTreeMap<String, Vec<VarState>>,
    /// Diagnostics (errors) collected.
    diagnostics: Vec<Diagnostic>,
    /// Scopes for variable cleanup
    scopes: Vec<BTreeSet<String>>,
    /// History of changes for undoing/merging branches
    history: Vec<BTreeMap<String, VarState>>,
}

impl MoveCheckContext {
    fn new() -> Self {
        Self {
            function_params: BTreeMap::new(),
            var_stacks: BTreeMap::new(),
            diagnostics: Vec::new(),
            scopes: Vec::new(),
            history: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(BTreeSet::new());
    }

    fn pop_scope(&mut self) {
        let vars_to_pop = self.scopes.pop().unwrap_or_default();
        for name in vars_to_pop {
            if let Some(stack) = self.var_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.var_stacks.remove(&name);
                }
            }
        }
    }

    fn declare_var(&mut self, name: String) {
        self.var_stacks
            .entry(name.clone())
            .or_default()
            .push(VarState::Valid);
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name);
        }
    }

    // For function params
    fn declare_param(&mut self, name: String) {
        self.declare_var(name);
    }

    fn get_state(&self, name: &str) -> Option<VarState> {
        self.var_stacks.get(name).and_then(|s| s.last().copied())
    }

    fn set_state(&mut self, name: &str, state: VarState) {
        if let Some(stack) = self.var_stacks.get_mut(name) {
            if let Some(last) = stack.last_mut() {
                if *last == state {
                    return;
                }
                if let Some(h) = self.history.last_mut() {
                    h.entry(name.to_string()).or_insert(*last);
                }
                *last = state;
            }
        }
    }

    fn push_history(&mut self) {
        self.history.push(BTreeMap::new());
    }

    fn pop_history(&mut self) -> BTreeMap<String, VarState> {
        self.history.pop().unwrap_or_default()
    }

    fn apply_history(&mut self, history: BTreeMap<String, VarState>) {
        for (name, old_state) in history {
            self.set_state(&name, old_state);
        }
    }

    fn undo_history(&mut self, history: &BTreeMap<String, VarState>) {
        // To undo, we set the state back to the original values recorded in history
        for (name, old_state) in history {
            if let Some(stack) = self.var_stacks.get_mut(name) {
                if let Some(last) = stack.last_mut() {
                    *last = *old_state;
                }
            }
        }
    }

    fn check_use(&mut self, name: &str, span: Span, is_copy: bool) {
        // NOTE: reserved words should not be treated as variables
        if matches!(name, "if" | "while" | "let" | "set") {
            return;
        }

        match self.get_state(name) {
            Some(VarState::Valid) => {
                if !is_copy {
                    // Moving a non-Copy value is OK: just mark it as moved.
                    self.set_state(name, VarState::Moved);
                }
            }
            Some(VarState::BorrowedShared) => {
                if !is_copy {
                    self.diagnostics.push(Diagnostic::error(
                        alloc::format!("cannot move out of shared borrowed value: `{}`", name),
                        span,
                    ).with_id(DiagnosticId::TypeMoveFromSharedBorrowedValue));
                }
            }
            Some(VarState::BorrowedUnique) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("use of uniquely borrowed value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeUseUniquelyBorrowedValue));
            }
            Some(VarState::Moved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("use of moved value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeUseMovedValue));
            }
            Some(VarState::PossiblyMoved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("use of potentially moved value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeUsePossiblyMovedValue));
            }
            None => {}
        }
    }

    fn with_function_params(function_params: BTreeMap<String, Vec<TypeId>>) -> Self {
        let mut ctx = Self::new();
        ctx.function_params = function_params;
        ctx
    }

    fn check_assign(&mut self, name: &str, span: Span) {
        match self.get_state(name) {
            Some(VarState::BorrowedShared) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("cannot assign to shared borrowed value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeAssignSharedBorrowedValue));
            }
            Some(VarState::BorrowedUnique) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("cannot assign to uniquely borrowed value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeAssignUniquelyBorrowedValue));
            }
            _ => {
                self.set_state(name, VarState::Valid);
            }
        }
    }

    fn check_drop(&mut self, name: &str, span: Span) {
        match self.get_state(name) {
            Some(VarState::Valid) => self.set_state(name, VarState::Moved),
            Some(VarState::BorrowedShared) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("cannot drop shared borrowed value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeDropSharedBorrowedValue));
            }
            Some(VarState::BorrowedUnique) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("cannot drop uniquely borrowed value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeDropUniquelyBorrowedValue));
            }
            Some(VarState::Moved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("drop of moved value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeDropMovedValue));
            }
            Some(VarState::PossiblyMoved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("drop of potentially moved value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeDropPossiblyMovedValue));
            }
            None => {}
        }
    }

    fn check_borrow(&mut self, name: &str, span: Span, kind: BorrowKind, is_copy: bool) {
        if is_copy {
            return;
        }
        match self.get_state(name) {
            Some(VarState::Valid) => {
                let next = match kind {
                    BorrowKind::Shared => VarState::BorrowedShared,
                    BorrowKind::Unique => VarState::BorrowedUnique,
                };
                self.set_state(name, next);
            }
            Some(VarState::BorrowedShared) => match kind {
                BorrowKind::Shared => {}
                BorrowKind::Unique => {
                    self.diagnostics.push(Diagnostic::error(
                        alloc::format!(
                            "cannot uniquely borrow shared borrowed value: `{}`",
                            name
                        ),
                        span,
                    ).with_id(DiagnosticId::TypeUniqueBorrowSharedBorrowedValue));
                }
            },
            Some(VarState::BorrowedUnique) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("cannot borrow uniquely borrowed value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeBorrowUniquelyBorrowedValue));
            }
            Some(VarState::Moved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("borrow of moved value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeBorrowMovedValue));
            }
            Some(VarState::PossiblyMoved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("borrow of potentially moved value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeBorrowPossiblyMovedValue));
            }
            None => {}
        }
    }

    fn check_temporary_borrow(&mut self, name: &str, span: Span, kind: BorrowKind, is_copy: bool) {
        if is_copy {
            return;
        }
        match self.get_state(name) {
            Some(VarState::Valid) => {}
            Some(VarState::BorrowedShared) => {
                if matches!(kind, BorrowKind::Unique) {
                    self.diagnostics.push(Diagnostic::error(
                        alloc::format!(
                            "cannot uniquely borrow shared borrowed value: `{}`",
                            name
                        ),
                        span,
                    ).with_id(DiagnosticId::TypeUniqueBorrowSharedBorrowedValue));
                }
            }
            Some(VarState::BorrowedUnique) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("cannot borrow uniquely borrowed value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeBorrowUniquelyBorrowedValue));
            }
            Some(VarState::Moved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("borrow of moved value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeBorrowMovedValue));
            }
            Some(VarState::PossiblyMoved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("borrow of potentially moved value: `{}`", name),
                    span,
                ).with_id(DiagnosticId::TypeBorrowPossiblyMovedValue));
            }
            None => {}
        }
    }

    fn merge_state_pair(a: VarState, b: VarState) -> VarState {
        use VarState::*;
        match (a, b) {
            (Valid, Valid) => Valid,
            (BorrowedShared, BorrowedShared) => BorrowedShared,
            (BorrowedUnique, BorrowedUnique) => BorrowedUnique,
            (Moved, Moved) => Moved,
            (PossiblyMoved, _) | (_, PossiblyMoved) => PossiblyMoved,
            (Moved, _) | (_, Moved) => PossiblyMoved,
            (BorrowedUnique, BorrowedShared) | (BorrowedShared, BorrowedUnique) => BorrowedShared,
            (BorrowedShared, Valid) | (Valid, BorrowedShared) => BorrowedShared,
            (BorrowedUnique, Valid) | (Valid, BorrowedUnique) => BorrowedShared,
        }
    }

    fn merge_states(states: &[VarState]) -> VarState {
        let mut it = states.iter().copied();
        let first = it.next().unwrap_or(VarState::Valid);
        it.fold(first, Self::merge_state_pair)
    }
}

// Logic to traverse HIR
fn visit_block(block: &HirBlock, ctx: &mut MoveCheckContext, tctx: &crate::types::TypeCtx) {
    ctx.push_scope();
    for line in &block.lines {
        visit_expr(&line.expr, ctx, tctx);
    }
    ctx.pop_scope();
}

fn reference_borrow_kind(tctx: &crate::types::TypeCtx, ty: TypeId) -> Option<BorrowKind> {
    match tctx.get_ref(tctx.resolve_id(ty)) {
        TypeKind::Reference(_, true) => Some(BorrowKind::Unique),
        TypeKind::Reference(_, false) => Some(BorrowKind::Shared),
        _ => None,
    }
}

fn visit_reference_call_arg(
    arg: &HirExpr,
    kind: BorrowKind,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) {
    match &arg.kind {
        HirExprKind::AddrOf(inner) => visit_temporary_borrow(inner, ctx, tctx, kind),
        _ => visit_expr(arg, ctx, tctx),
    }
}

fn visit_call_args_with_params(
    args: &[HirExpr],
    params: Option<&[TypeId]>,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) {
    for (i, arg) in args.iter().enumerate() {
        let param_ty = params.and_then(|p| p.get(i)).copied();
        if let Some(kind) = param_ty.and_then(|ty| reference_borrow_kind(tctx, ty)) {
            visit_reference_call_arg(arg, kind, ctx, tctx);
        } else {
            visit_expr(arg, ctx, tctx);
        }
    }
}

fn visit_expr(expr: &HirExpr, ctx: &mut MoveCheckContext, tctx: &crate::types::TypeCtx) {
    let is_copy = tctx.is_copy(expr.ty);
    // ctx.diagnostics.push(Diagnostic::warning(alloc::format!("DEBUG: visiting kind {:?}", expr.kind), expr.span));

    match &expr.kind {
        HirExprKind::Var(name) => {
            ctx.check_use(name, expr.span, is_copy);
        }
        HirExprKind::FnValue(_) => {}
        HirExprKind::Call { callee, args } => match callee {
            FuncRef::Builtin(name) | FuncRef::User(name, _) if name == "get" => {
                if let Some(base) = args.get(0) {
                    if tctx.is_copy(expr.ty) {
                        visit_temporary_borrow(base, ctx, tctx, BorrowKind::Shared);
                    } else if !visit_field_move_source(base, ctx, tctx) {
                        visit_expr(base, ctx, tctx);
                    }
                }
                for arg in args.iter().skip(1) {
                    visit_expr(arg, ctx, tctx);
                }
            }
            FuncRef::Builtin(name) | FuncRef::User(name, _) if name == "if" => {
                if args.len() == 3 {
                    visit_expr(&args[0], ctx, tctx);

                    ctx.push_history();
                    visit_expr(&args[1], ctx, tctx);
                    let then_diff = ctx.pop_history();
                    let mut then_final = BTreeMap::new();
                    for name in then_diff.keys() {
                        let fallback = *then_diff.get(name).unwrap_or(&VarState::Valid);
                        then_final.insert(name.clone(), ctx.get_state(name).unwrap_or(fallback));
                    }
                    ctx.undo_history(&then_diff);

                    ctx.push_history();
                    visit_expr(&args[2], ctx, tctx);
                    let else_diff = ctx.pop_history();
                    let mut else_final = BTreeMap::new();
                    for name in else_diff.keys() {
                        let fallback = *else_diff.get(name).unwrap_or(&VarState::Valid);
                        else_final.insert(name.clone(), ctx.get_state(name).unwrap_or(fallback));
                    }
                    ctx.undo_history(&else_diff);

                    let mut all_modified: BTreeSet<String> =
                        then_diff.keys().cloned().collect();
                    all_modified.extend(else_diff.keys().cloned());

                    for name in all_modified {
                        let start_state = then_diff
                            .get(&name)
                            .or_else(|| else_diff.get(&name))
                            .copied()
                            .unwrap_or_else(|| ctx.get_state(&name).unwrap_or(VarState::Valid));

                        let then_state = then_final.get(&name).copied().unwrap_or(start_state);
                        let else_state = else_final.get(&name).copied().unwrap_or(start_state);

                        let merged = MoveCheckContext::merge_state_pair(then_state, else_state);
                        ctx.set_state(&name, merged);
                    }
                }
            }
            FuncRef::Builtin(name) | FuncRef::User(name, _) if name == "while" => {
                if args.len() == 2 {
                    visit_expr(&args[0], ctx, tctx);

                    ctx.push_history();
                    visit_expr(&args[1], ctx, tctx);
                    let body_diff = ctx.pop_history();

                    for (name, start_state) in body_diff {
                        let end_state = ctx.get_state(&name).unwrap_or(start_state);
                        let merged = MoveCheckContext::merge_state_pair(start_state, end_state);
                        ctx.set_state(&name, merged);
                        if matches!(merged, VarState::PossiblyMoved)
                            && matches!(start_state, VarState::Valid | VarState::BorrowedShared | VarState::BorrowedUnique)
                            && matches!(end_state, VarState::Moved | VarState::PossiblyMoved)
                        {
                            ctx.diagnostics.push(Diagnostic::error(
                                alloc::format!("potentially moved value: `{}`", name),
                                args[1].span,
                            ).with_id(DiagnosticId::TypeLoopPotentiallyMovedValue));
                        }
                    }
                    visit_expr(&args[0], ctx, tctx);
                }
            }
            _ => {
                let params = match callee {
                    FuncRef::User(name, _) => ctx.function_params.get(name).cloned(),
                    _ => None,
                };
                visit_call_args_with_params(args, params.as_deref(), ctx, tctx);
            }
        },
        HirExprKind::CallIndirect {
            callee,
            params,
            args,
            ..
        } => {
            visit_expr(callee, ctx, tctx);
            visit_call_args_with_params(args, Some(params.as_slice()), ctx, tctx);
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            visit_expr(cond, ctx, tctx);

            ctx.push_history();
            visit_expr(then_branch, ctx, tctx);
            let then_diff = ctx.pop_history();
            let mut then_final = BTreeMap::new();
            for name in then_diff.keys() {
                let fallback = *then_diff.get(name).unwrap_or(&VarState::Valid);
                then_final.insert(name.clone(), ctx.get_state(name).unwrap_or(fallback));
            }
            ctx.undo_history(&then_diff);

            ctx.push_history();
            visit_expr(else_branch, ctx, tctx);
            let else_diff = ctx.pop_history();
            let mut else_final = BTreeMap::new();
            for name in else_diff.keys() {
                let fallback = *else_diff.get(name).unwrap_or(&VarState::Valid);
                else_final.insert(name.clone(), ctx.get_state(name).unwrap_or(fallback));
            }
            ctx.undo_history(&else_diff);

            let mut all_modified: BTreeSet<String> = then_diff.keys().cloned().collect();
            all_modified.extend(else_diff.keys().cloned());

            for name in all_modified {
                let start_state = then_diff
                    .get(&name)
                    .or_else(|| else_diff.get(&name))
                    .copied()
                    .unwrap_or_else(|| ctx.get_state(&name).unwrap_or(VarState::Valid));

                let then_state = then_final.get(&name).copied().unwrap_or(start_state);
                let else_state = else_final.get(&name).copied().unwrap_or(start_state);

                let merged = MoveCheckContext::merge_state_pair(then_state, else_state);
                ctx.set_state(&name, merged);
            }
        }
        HirExprKind::While { cond, body } => {
            visit_expr(cond, ctx, tctx);
            ctx.push_history();
            visit_expr(body, ctx, tctx);
            let body_diff = ctx.pop_history();

            for (name, start_state) in body_diff {
                let end_state = ctx.get_state(&name).unwrap_or(start_state);
                let merged = MoveCheckContext::merge_state_pair(start_state, end_state);
                ctx.set_state(&name, merged);
                if matches!(merged, VarState::PossiblyMoved)
                    && matches!(start_state, VarState::Valid | VarState::BorrowedShared | VarState::BorrowedUnique)
                    && matches!(end_state, VarState::Moved | VarState::PossiblyMoved)
                {
                    ctx.diagnostics.push(Diagnostic::error(
                        alloc::format!("potentially moved value: `{}`", name),
                        expr.span,
                    ).with_id(DiagnosticId::TypeLoopPotentiallyMovedValue));
                }
            }
            visit_expr(cond, ctx, tctx);
        }
        HirExprKind::Match { scrutinee, arms } => {
            visit_expr(scrutinee, ctx, tctx);

            let mut all_branch_diffs = Vec::new();
            let mut all_branch_finals = Vec::new();

            for arm in arms {
                ctx.push_history();
                ctx.push_scope();
                if let Some(bind) = &arm.bind_local {
                    ctx.declare_var(bind.clone());
                }
                visit_expr(&arm.body, ctx, tctx);
                ctx.pop_scope();
                let diff = ctx.pop_history();
                let mut final_states = BTreeMap::new();
                for name in diff.keys() {
                    let fallback = *diff.get(name).unwrap_or(&VarState::Valid);
                    final_states.insert(name.clone(), ctx.get_state(name).unwrap_or(fallback));
                }
                ctx.undo_history(&diff);
                all_branch_diffs.push(diff);
                all_branch_finals.push(final_states);
            }

            let mut all_modified = BTreeSet::new();
            for diff in &all_branch_diffs {
                for name in diff.keys() {
                    all_modified.insert(name.clone());
                }
            }

            for name in all_modified {
                let start_state = all_branch_diffs
                    .iter()
                    .find_map(|diff| diff.get(&name).copied())
                    .unwrap_or(VarState::Valid);
                let mut states = Vec::with_capacity(all_branch_finals.len());
                for branch_final in &all_branch_finals {
                    states.push(branch_final.get(&name).copied().unwrap_or(start_state));
                }
                let merged = MoveCheckContext::merge_states(&states);
                ctx.set_state(&name, merged);
            }
        }
        HirExprKind::Block(b) => visit_block(b, ctx, tctx),
        // HirExprKind::Let { name, value, .. } => {
        //     visit_expr(value, ctx, tctx);
        //     ctx.declare_var(name.clone());
        // }
        HirExprKind::Set { value, name } => {
            visit_expr(value, ctx, tctx);
            ctx.check_assign(name, expr.span);
        }
        HirExprKind::Let { name, value, .. } => {
            visit_expr(value, ctx, tctx);

            // A new binding starts as Valid.
            ctx.declare_var(name.clone());
            ctx.set_state(name, VarState::Valid);
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for f in fields {
                visit_expr(f, ctx, tctx);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(p) = payload {
                visit_expr(p, ctx, tctx);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                visit_expr(item, ctx, tctx);
            }
        }
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => {
            match name.as_str() {
                "load" => {
                    let is_copy_load = type_args
                        .get(0)
                        .map(|ty| tctx.is_copy(*ty))
                        .unwrap_or(false);
                    if let Some(addr) = args.get(0) {
                        if is_copy_load {
                            visit_temporary_borrow(addr, ctx, tctx, BorrowKind::Shared);
                        } else if !visit_field_move_source(addr, ctx, tctx) {
                            visit_temporary_borrow(addr, ctx, tctx, BorrowKind::Unique);
                        }
                    }
                }
                "store" => {
                    if let Some(addr) = args.get(0) {
                        visit_temporary_borrow(addr, ctx, tctx, BorrowKind::Unique);
                    }
                    if let Some(val) = args.get(1) {
                        visit_expr(val, ctx, tctx);
                    }
                }
                _ => {
                    for arg in args {
                        visit_expr(arg, ctx, tctx);
                    }
                }
            }
        }
        HirExprKind::AddrOf(inner) => {
            visit_borrow(inner, ctx, tctx, BorrowKind::Shared);
        }
        HirExprKind::Deref(inner) => {
            visit_expr(inner, ctx, tctx);
        }
        HirExprKind::Drop { name } => {
            ctx.check_drop(name, expr.span);
        }
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit => {}
    }
}

fn visit_borrow(
    expr: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    kind: BorrowKind,
) {
    match &expr.kind {
        HirExprKind::Var(name) => {
            let is_copy = tctx.is_copy(expr.ty);
            ctx.check_borrow(name, expr.span, kind, is_copy);
        }
        HirExprKind::Deref(inner) => {
            // Re-borrowing a dereference. Still a borrow.
            visit_borrow(inner, ctx, tctx, kind);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                visit_borrow(arg, ctx, tctx, kind);
            }
        }
        _ => visit_expr(expr, ctx, tctx),
    }
}

fn visit_temporary_borrow(
    expr: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    kind: BorrowKind,
) {
    match &expr.kind {
        HirExprKind::Var(name) => {
            let is_copy = tctx.is_copy(expr.ty);
            ctx.check_temporary_borrow(name, expr.span, kind, is_copy);
        }
        HirExprKind::Deref(inner) => {
            visit_temporary_borrow(inner, ctx, tctx, kind);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                visit_temporary_borrow(arg, ctx, tctx, kind);
            }
        }
        _ => {}
    }
}

fn visit_field_move_source(
    expr: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> bool {
    match &expr.kind {
        HirExprKind::Var(name) => {
            if !tctx.is_copy(expr.ty) {
                ctx.check_use(name, expr.span, false);
                return true;
            }
            false
        }
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && !args.is_empty() => {
            visit_field_move_source(&args[0], ctx, tctx)
        }
        _ => false,
    }
}

fn get_top(map: &BTreeMap<String, Vec<VarState>>, name: &str) -> Option<VarState> {
    map.get(name).and_then(|s| s.last().copied())
}

pub fn run(module: &HirModule, types: &crate::types::TypeCtx) -> Vec<Diagnostic> {
    let function_params: BTreeMap<String, Vec<TypeId>> = module
        .functions
        .iter()
        .map(|func| {
            (
                func.name.clone(),
                func.params.iter().map(|param| param.ty).collect(),
            )
        })
        .collect();
    let mut diagnostics = Vec::new();

    for func in &module.functions {
        let mut f_ctx = MoveCheckContext::with_function_params(function_params.clone());
        for param in &func.params {
            f_ctx.declare_param(param.name.clone());
        }

        match &func.body {
            crate::hir::HirBody::Block(b) => visit_block(b, &mut f_ctx, types),
            _ => {}
        }

        diagnostics.extend(f_ctx.diagnostics);
    }

    diagnostics
}
