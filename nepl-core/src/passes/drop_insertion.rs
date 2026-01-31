#![no_std]
extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::{HirBlock, HirExpr, HirExprKind, HirLine, HirModule};
use crate::types::TypeId;

/// Insert automatic `drop` calls at end of scopes and on early returns.
///
/// This pass walks the HIR and inserts `Drop` expressions to deallocate
/// heap-owned values at scope boundaries. The pass operates lexically:
/// - At the end of a block, drops are inserted for all live bindings.
/// - On early returns (If, Match arms), drops are inserted before exit.
/// - Variable scope is tracked through Let/Set statements.
pub fn insert_drops(module: &mut HirModule, unit_ty: TypeId) {
    for func in &mut module.functions {
        if let crate::hir::HirBody::Block(ref mut block) = func.body {
            let mut ctx = DropInsertionContext::new(unit_ty);
            insert_drops_in_block(block, &mut ctx);
        }
    }
}

struct DropInsertionContext {
    /// Unit type id to use for inserted Drop expressions.
    unit_ty: TypeId,
    /// Variables in scope at the current depth (names).
    live_vars: Vec<String>,
    /// Stack of variable scopes (for nested blocks).
    scopes: Vec<BTreeSet<String>>,
}

impl DropInsertionContext {
    fn new(unit_ty: TypeId) -> Self {
        Self {
            unit_ty,
            live_vars: Vec::new(),
            scopes: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(BTreeSet::new());
    }

    fn pop_scope(&mut self) -> BTreeSet<String> {
        self.scopes.pop().unwrap_or_default()
    }

    fn declare_var(&mut self, name: String) {
        self.live_vars.push(name.clone());
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name);
        }
    }

    fn get_scope_vars(&self) -> Vec<String> {
        if let Some(scope) = self.scopes.last() {
            scope.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }
}

fn insert_drops_in_block(block: &mut HirBlock, ctx: &mut DropInsertionContext) {
    ctx.push_scope();

    // Process each line, tracking variable bindings.
    for i in 0..block.lines.len() {
        let line = &mut block.lines[i];
        insert_drops_in_expr(&mut line.expr, ctx);

        // Track variables declared in Let expressions.
        if let HirExprKind::Let { name, .. } = &line.expr.kind {
            ctx.declare_var(name.clone());
        }
    }

    // At the end of the block, insert drops for all variables in this scope
    // (in reverse order of declaration, LIFO).
    let scope_vars = ctx.get_scope_vars();
    for var in scope_vars.iter().rev() {
        let drop_expr = HirExpr {
            ty: ctx.unit_ty,
            kind: HirExprKind::Drop { name: var.clone() },
            span: block.span,
        };
        block.lines.push(HirLine {
            expr: drop_expr,
            drop_result: true,
        });
    }

    ctx.pop_scope();
}

fn insert_drops_in_expr(expr: &mut HirExpr, ctx: &mut DropInsertionContext) {
    match &mut expr.kind {
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            // Process condition.
            insert_drops_in_expr(cond, ctx);

            // Process both branches with separate scopes.
            // (Each branch can have its own locals that need dropping.)
            let old_len = ctx.scopes.len();
            insert_drops_in_expr(then_branch, ctx);
            while ctx.scopes.len() > old_len {
                ctx.pop_scope();
            }

            let old_len = ctx.scopes.len();
            insert_drops_in_expr(else_branch, ctx);
            while ctx.scopes.len() > old_len {
                ctx.pop_scope();
            }
        }
        HirExprKind::While { cond, body } => {
            insert_drops_in_expr(cond, ctx);
            insert_drops_in_expr(body, ctx);
        }
        HirExprKind::Match { scrutinee, arms } => {
            insert_drops_in_expr(scrutinee, ctx);
            for arm in arms {
                // Each arm is a lexical scope: push, declare bind, recurse,
                // then pop and insert drops for that arm specifically.
                ctx.push_scope();
                if let Some(ref bind) = arm.bind_local {
                    ctx.declare_var(bind.clone());
                }
                insert_drops_in_expr(&mut arm.body, ctx);

                // Collect vars from this arm's scope and append drop
                // expressions to the arm body. If the arm body is not a
                // block, wrap it into a block so we can append lines.
                let scope_vars = ctx.pop_scope();
                if !scope_vars.is_empty() {
                    match &mut arm.body.kind {
                        HirExprKind::Block(ref mut b) => {
                            for var in scope_vars.iter().rev() {
                                let drop_expr = HirExpr {
                                    ty: ctx.unit_ty,
                                    kind: HirExprKind::Drop { name: var.clone() },
                                    span: arm.body.span,
                                };
                                b.lines.push(HirLine {
                                    expr: drop_expr,
                                    drop_result: true,
                                });
                            }
                        }
                        _ => {
                            // Replace the arm body with a block containing the
                            // original expression followed by the drops.
                            let original = arm.body.clone();
                            let mut new_block = HirBlock {
                                lines: Vec::new(),
                                ty: ctx.unit_ty,
                                span: original.span,
                            };
                            new_block.lines.push(HirLine {
                                expr: original,
                                drop_result: false,
                            });
                            for var in scope_vars.iter().rev() {
                                let drop_expr = HirExpr {
                                    ty: ctx.unit_ty,
                                    kind: HirExprKind::Drop { name: var.clone() },
                                    span: arm.body.span,
                                };
                                new_block.lines.push(HirLine {
                                    expr: drop_expr,
                                    drop_result: true,
                                });
                            }
                            arm.body.kind = HirExprKind::Block(new_block);
                        }
                    }
                }
            }
        }
        HirExprKind::Block(ref mut block) => {
            insert_drops_in_block(block, ctx);
        }
        HirExprKind::Let { value, .. } => {
            insert_drops_in_expr(value, ctx);
        }
        HirExprKind::Set { value, .. } => {
            insert_drops_in_expr(value, ctx);
        }
        HirExprKind::Call { args, .. } => {
            for arg in args {
                insert_drops_in_expr(arg, ctx);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(ref mut p) = payload {
                insert_drops_in_expr(p, ctx);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for field in fields {
                insert_drops_in_expr(field, ctx);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            for item in items {
                insert_drops_in_expr(item, ctx);
            }
        }
        _ => {}
    }
}
