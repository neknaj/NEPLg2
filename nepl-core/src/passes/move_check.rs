#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::hir::{FuncRef, HirBlock, HirExpr, HirExprKind, HirLine, HirModule};
use crate::span::Span;
use crate::types::TypeId;

/// Tracks ownership state of variables.
/// Currently simple: either Valid (Initialized) or Moved.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum VarState {
    Valid,
    Moved,
    PossiblyMoved,
}

struct MoveCheckContext {
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
            Some(VarState::Moved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("use of moved value: `{}`", name),
                    span,
                ));
            }
            Some(VarState::PossiblyMoved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("use of potentially moved value: `{}`", name),
                    span,
                ));
            }
            None => {}
        }
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

fn visit_expr(expr: &HirExpr, ctx: &mut MoveCheckContext, tctx: &crate::types::TypeCtx) {
    let is_copy = tctx.is_copy(expr.ty);
    // ctx.diagnostics.push(Diagnostic::warning(alloc::format!("DEBUG: visiting kind {:?}", expr.kind), expr.span));

    match &expr.kind {
        HirExprKind::Var(name) => {
            ctx.check_use(name, expr.span, is_copy);
        }
        HirExprKind::Call { callee, args } => match callee {
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

                        let merged = match (then_state, else_state) {
                            (VarState::Valid, VarState::Valid) => VarState::Valid,
                            (VarState::Moved, VarState::Moved) => VarState::Moved,
                            _ => VarState::PossiblyMoved,
                        };
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
                        if end_state != start_state && start_state == VarState::Valid {
                            ctx.set_state(&name, VarState::PossiblyMoved);
                            ctx.diagnostics.push(Diagnostic::error(
                                alloc::format!("potentially moved value: `{}`", name),
                                args[1].span,
                            ));
                        }
                    }
                    visit_expr(&args[0], ctx, tctx);
                }
            }
            _ => {
                for arg in args {
                    visit_expr(arg, ctx, tctx);
                }
            }
        },
        HirExprKind::CallIndirect { callee, args, .. } => {
            visit_expr(callee, ctx, tctx);
            for arg in args {
                visit_expr(arg, ctx, tctx);
            }
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

                let merged = match (then_state, else_state) {
                    (VarState::Valid, VarState::Valid) => VarState::Valid,
                    (VarState::Moved, VarState::Moved) => VarState::Moved,
                    _ => VarState::PossiblyMoved,
                };
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
                if end_state != start_state && start_state == VarState::Valid {
                    ctx.set_state(&name, VarState::PossiblyMoved);
                    ctx.diagnostics.push(Diagnostic::error(
                        alloc::format!("potentially moved value: `{}`", name),
                        expr.span,
                    ));
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
                let mut start_state = VarState::Valid;
                for diff in &all_branch_diffs {
                    if let Some(s) = diff.get(&name) {
                        start_state = *s;
                        break;
                    }
                }

                let mut all_valid = true;
                let mut all_moved = true;

                for branch_final in &all_branch_finals {
                    let state = branch_final.get(&name).copied().unwrap_or(start_state);
                    match state {
                        VarState::Valid => all_moved = false,
                        VarState::Moved => all_valid = false,
                        _ => {
                            all_valid = false;
                            all_moved = false;
                        }
                    }
                }

                let merged = if all_valid {
                    VarState::Valid
                } else if all_moved {
                    VarState::Moved
                } else {
                    VarState::PossiblyMoved
                };
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
            ctx.set_state(name, VarState::Valid);
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
                            visit_borrow(addr, ctx, tctx);
                        } else {
                            visit_expr(addr, ctx, tctx);
                        }
                    }
                }
                "store" => {
                    if let Some(addr) = args.get(0) {
                        visit_borrow(addr, ctx, tctx);
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
            visit_borrow(inner, ctx, tctx);
        }
        HirExprKind::Deref(inner) => {
            visit_expr(inner, ctx, tctx);
        }
        HirExprKind::Drop { .. } => {}
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit => {}
    }
}

fn visit_borrow(expr: &HirExpr, ctx: &mut MoveCheckContext, tctx: &crate::types::TypeCtx) {
    match &expr.kind {
        HirExprKind::Var(name) => match ctx.get_state(name) {
            Some(VarState::Valid) => {
                // Borrow is OK, value stays Valid.
            }
            Some(VarState::Moved) => {
                ctx.diagnostics.push(Diagnostic::error(
                    alloc::format!("borrow of moved value: `{}`", name),
                    expr.span,
                ));
            }
            Some(VarState::PossiblyMoved) => {
                ctx.diagnostics.push(Diagnostic::error(
                    alloc::format!("borrow of potentially moved value: `{}`", name),
                    expr.span,
                ));
            }
            None => {}
        },
        HirExprKind::Deref(inner) => {
            // Re-borrowing a dereference. Still a borrow.
            visit_borrow(inner, ctx, tctx);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                visit_borrow(arg, ctx, tctx);
            }
        }
        _ => visit_expr(expr, ctx, tctx),
    }
}

fn get_top(map: &BTreeMap<String, Vec<VarState>>, name: &str) -> Option<VarState> {
    map.get(name).and_then(|s| s.last().copied())
}

pub fn run(module: &HirModule, types: &crate::types::TypeCtx) -> Vec<Diagnostic> {
    let mut ctx = MoveCheckContext::new();

    for func in &module.functions {
        let mut f_ctx = MoveCheckContext::new();
        for param in &func.params {
            f_ctx.declare_param(param.name.clone());
        }

        match &func.body {
            crate::hir::HirBody::Block(b) => visit_block(b, &mut f_ctx, types),
            _ => {}
        }

        ctx.diagnostics.extend(f_ctx.diagnostics);
    }

    ctx.diagnostics
}
