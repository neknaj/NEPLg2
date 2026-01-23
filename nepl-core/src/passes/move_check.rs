#![no_std]
extern crate alloc;

use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;

use crate::hir::{HirBlock, HirExpr, HirExprKind, HirModule};
use crate::span::Span;
use crate::diagnostic::Diagnostic;
use crate::types::TypeId;

/// Tracks ownership state of variables.
/// Currently simple: either Valid (Initialized) or Moved.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
enum VarState {
    Valid,
    Moved,
}

struct MoveCheckContext {
    /// State of all variables currently in scope.
    /// Stack of variable states (for shadowing support).
    var_stacks: BTreeMap<String, Vec<VarState>>,
    /// Diagnostics (errors) collected.
    diagnostics: Vec<Diagnostic>,
    /// Scopes for variable cleanup
    scopes: Vec<BTreeSet<String>>,
}

impl MoveCheckContext {
    fn new() -> Self {
        Self {
            var_stacks: BTreeMap::new(),
            diagnostics: Vec::new(),
            scopes: Vec::new(),
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
        self.var_stacks.entry(name.clone()).or_default().push(VarState::Valid);
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
                *last = state;
            }
        }
    }

    fn check_use(&mut self, name: &str, span: Span, is_copy: bool) {
        // Ignore builtins
        if matches!(name, "if" | "while" | "let" | "set" | "print_i32" | "print_str") {
            return;
        }

        match self.get_state(name) {
            Some(VarState::Valid) => {
                if !is_copy {
                    self.set_state(name, VarState::Moved);
                }
            }
            Some(VarState::Moved) => {
                self.diagnostics.push(Diagnostic::error(
                    alloc::format!("use of moved value: `{}` (is_copy={})", name, is_copy),
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
    let is_copy = match tctx.get(expr.ty) {
        crate::types::TypeKind::I32 | crate::types::TypeKind::F32 | 
        crate::types::TypeKind::Bool | crate::types::TypeKind::Unit => true,
        crate::types::TypeKind::Reference(_, _) => true,
        crate::types::TypeKind::Box(_) => false,
        _ => false,
    };

    match &expr.kind {
        HirExprKind::Var(name) => {
             ctx.check_use(name, expr.span, is_copy);
        }
        HirExprKind::If { cond, then_branch, else_branch } => {
            visit_expr(cond, ctx, tctx);
            
            let start_vars = ctx.var_stacks.clone();
            visit_expr(then_branch, ctx, tctx);
            let then_vars = ctx.var_stacks.clone();
            
            ctx.var_stacks = start_vars.clone();
            visit_expr(else_branch, ctx, tctx);
            let else_vars = ctx.var_stacks.clone();
            
            for (name, stack) in start_vars {
                if let Some(start_state) = stack.last() {
                    let then_state = get_top(&then_vars, &name).unwrap_or(*start_state);
                    let else_state = get_top(&else_vars, &name).unwrap_or(*start_state);
                    
                    if then_state == VarState::Moved || else_state == VarState::Moved {
                         ctx.set_state(&name, VarState::Moved);
                    }
                }
            }
        }
        HirExprKind::While { cond, body } => {
            visit_expr(cond, ctx, tctx);
            
            // Run body once to track moves
            visit_expr(body, ctx, tctx);
            
            // For loops, we additionally need to check if things moved in 1st iteration
            // are used in 2nd iteration.
            // We can do this by visiting body AGAIN with the post-body state.
            visit_expr(cond, ctx, tctx);
            visit_expr(body, ctx, tctx);
        }
        HirExprKind::Match { scrutinee, arms } => {
            visit_expr(scrutinee, ctx, tctx);
            let start_vars = ctx.var_stacks.clone();
            let mut branch_states = Vec::new();
            
            for arm in arms {
                 ctx.var_stacks = start_vars.clone();
                 ctx.push_scope();
                 if let Some(bind) = &arm.bind_local {
                     ctx.declare_var(bind.clone());
                 }
                 visit_expr(&arm.body, ctx, tctx);
                 ctx.pop_scope();
                 branch_states.push(ctx.var_stacks.clone());
            }
            
             for (name, stack) in start_vars {
                if let Some(start_state) = stack.last() {
                    let mut moved = false;
                    for branch in &branch_states {
                         if let Some(s) = get_top(branch, &name) {
                             if s == VarState::Moved { moved = true; break; }
                         }
                    }
                    if moved {
                         ctx.set_state(&name, VarState::Moved);
                    }
                }
            }
        }
        HirExprKind::Block(b) => visit_block(b, ctx, tctx),
        HirExprKind::Let { name, value, .. } => {
            visit_expr(value, ctx, tctx);
            ctx.declare_var(name.clone());
        },
        HirExprKind::Set { value, name } => {
             visit_expr(value, ctx, tctx);
             ctx.set_state(name, VarState::Valid);
        },
        HirExprKind::Call { args, .. } => {
            for arg in args { visit_expr(arg, ctx, tctx); }
        },
        HirExprKind::StructConstruct { fields, .. } => {
            for f in fields { visit_expr(f, ctx, tctx); }
        },
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(p) = payload { visit_expr(p, ctx, tctx); }
        },
        _ => {}
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
    
    ctx.diagnostics.push(Diagnostic::error("DEBUG: move_check finished", Span::dummy()));
    ctx.diagnostics
}
