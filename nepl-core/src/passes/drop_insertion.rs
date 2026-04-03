#![no_std]
extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::TraitCapability;
use crate::hir::{FuncRef, HirBlock, HirExpr, HirExprKind, HirLine, HirMatchArm, HirModule};
use crate::types::{TypeCtx, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VarState {
    Valid,
    Moved,
    PossiblyMoved,
}

#[derive(Debug, Clone, Copy)]
struct VarInfo {
    ty: TypeId,
    state: VarState,
}

struct DropPlan {
    trait_name: String,
    method_name: String,
    unit_ty: TypeId,
}

struct DropInsertionContext<'a> {
    types: &'a mut TypeCtx,
    plan: &'a DropPlan,
    var_stacks: BTreeMap<String, Vec<VarInfo>>,
    scopes: Vec<Vec<String>>,
}

impl<'a> DropInsertionContext<'a> {
    fn new(types: &'a mut TypeCtx, plan: &'a DropPlan) -> Self {
        Self {
            types,
            plan,
            var_stacks: BTreeMap::new(),
            scopes: Vec::new(),
        }
    }

    fn push_scope(&mut self) {
        self.scopes.push(Vec::new());
    }

    fn pop_scope(&mut self) {
        let names = self.scopes.pop().unwrap_or_default();
        for name in names {
            if let Some(stack) = self.var_stacks.get_mut(&name) {
                stack.pop();
                if stack.is_empty() {
                    self.var_stacks.remove(&name);
                }
            }
        }
    }

    fn declare_var(&mut self, name: String, ty: TypeId) {
        self.var_stacks
            .entry(name.clone())
            .or_default()
            .push(VarInfo {
                ty,
                state: VarState::Valid,
            });
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(name);
        }
    }

    fn get_var(&self, name: &str) -> Option<VarInfo> {
        self.var_stacks.get(name).and_then(|stack| stack.last().copied())
    }

    fn set_state(&mut self, name: &str, state: VarState) {
        if let Some(stack) = self.var_stacks.get_mut(name) {
            if let Some(last) = stack.last_mut() {
                last.state = state;
            }
        }
    }

    fn merge_state(a: VarState, b: VarState) -> VarState {
        match (a, b) {
            (VarState::Valid, VarState::Valid) => VarState::Valid,
            (VarState::Moved, VarState::Moved) => VarState::Moved,
            (VarState::PossiblyMoved, _) | (_, VarState::PossiblyMoved) => VarState::PossiblyMoved,
            (VarState::Moved, _) | (_, VarState::Moved) => VarState::PossiblyMoved,
        }
    }

    fn scope_drop_lines(&mut self, span: crate::span::Span) -> Vec<HirLine> {
        let mut out = Vec::new();
        let Some(scope) = self.scopes.last() else {
            return out;
        };
        for name in scope.iter().rev() {
            let Some(info) = self.get_var(name) else {
                continue;
            };
            if info.state != VarState::Valid {
                continue;
            }
            if !self.types.has_drop(info.ty) {
                continue;
            }
            out.push(HirLine {
                expr: drop_call_expr(self.types, self.plan, name.clone(), info.ty, span),
                drop_result: true,
            });
        }
        out
    }
}

pub fn insert_drops(module: &mut HirModule, types: &mut TypeCtx) {
    let Some(plan) = find_drop_plan(module, types.unit()) else {
        return;
    };
    for func in &mut module.functions {
        if let crate::hir::HirBody::Block(ref mut block) = func.body {
            let mut ctx = DropInsertionContext::new(types, &plan);
            ctx.push_scope();
            for param in &func.params {
                ctx.declare_var(param.name.clone(), param.ty);
            }
            insert_drops_in_block(block, &mut ctx);
            ctx.pop_scope();
        }
    }
}

fn find_drop_plan(module: &HirModule, unit_ty: TypeId) -> Option<DropPlan> {
    for tr in &module.traits {
        if !tr.capabilities.iter().any(|cap| *cap == TraitCapability::Drop) {
            continue;
        }
        let method_name = if tr.methods.contains_key("drop") {
            String::from("drop")
        } else {
            tr.methods.keys().next().cloned()?
        };
        return Some(DropPlan {
            trait_name: tr.name.clone(),
            method_name,
            unit_ty,
        });
    }
    None
}

fn drop_call_expr(
    types: &mut TypeCtx,
    plan: &DropPlan,
    name: String,
    ty: TypeId,
    span: crate::span::Span,
) -> HirExpr {
    HirExpr {
        ty: plan.unit_ty,
        kind: HirExprKind::Call {
            callee: FuncRef::Trait {
                trait_name: plan.trait_name.clone(),
                trait_args: Vec::new(),
                method: plan.method_name.clone(),
                self_ty: ty,
            },
            args: vec![HirExpr {
                ty: types.reference(ty, false),
                kind: HirExprKind::AddrOf(Box::new(HirExpr {
                    ty,
                    kind: HirExprKind::Var(name),
                    span,
                })),
                span,
            }],
        },
        span,
    }
}

fn insert_drops_in_block(block: &mut HirBlock, ctx: &mut DropInsertionContext<'_>) {
    ctx.push_scope();
    for line in &mut block.lines {
        insert_drops_in_expr(&mut line.expr, ctx);
        if let HirExprKind::Let { name, value, .. } = &line.expr.kind {
            ctx.declare_var(name.clone(), value.ty);
        }
    }
    let drops = ctx.scope_drop_lines(block.span);
    block.lines.extend(drops);
    ctx.pop_scope();
}

fn insert_drops_in_expr(expr: &mut HirExpr, ctx: &mut DropInsertionContext<'_>) {
    match &mut expr.kind {
        HirExprKind::Var(name) => {
            if !ctx.types.is_copy(expr.ty) {
                ctx.set_state(name, VarState::Moved);
            }
        }
        HirExprKind::FnValue(_)
        | HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit => {}
        HirExprKind::Call { callee, args } => match callee {
            FuncRef::Builtin(name) | FuncRef::User(name, _) if name == "get" => {
                if let Some(base) = args.get_mut(0) {
                    if !ctx.types.is_copy(expr.ty) {
                        insert_drops_in_expr(base, ctx);
                    }
                }
                for arg in args.iter_mut().skip(1) {
                    insert_drops_in_expr(arg, ctx);
                }
            }
            _ => {
                for arg in args {
                    insert_drops_in_expr(arg, ctx);
                }
            }
        },
        HirExprKind::CallIndirect { callee, args, .. } => {
            insert_drops_in_expr(callee, ctx);
            for arg in args {
                insert_drops_in_expr(arg, ctx);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            insert_drops_in_expr(cond, ctx);
            let saved = ctx.var_stacks.clone();
            insert_drops_in_expr(then_branch, ctx);
            let then_state = ctx.var_stacks.clone();
            ctx.var_stacks = saved.clone();
            insert_drops_in_expr(else_branch, ctx);
            let else_state = ctx.var_stacks.clone();
            ctx.var_stacks = saved.clone();
            merge_outer_states(ctx, &saved, &then_state, &else_state);
        }
        HirExprKind::While { cond, body } => {
            insert_drops_in_expr(cond, ctx);
            let saved = ctx.var_stacks.clone();
            insert_drops_in_expr(body, ctx);
            let body_state = ctx.var_stacks.clone();
            ctx.var_stacks = saved.clone();
            merge_outer_states(ctx, &saved, &saved, &body_state);
        }
        HirExprKind::Match { scrutinee, arms } => {
            insert_drops_in_expr(scrutinee, ctx);
            let saved = ctx.var_stacks.clone();
            let mut arm_states = Vec::new();
            for arm in arms {
                ctx.var_stacks = saved.clone();
                process_match_arm(arm, ctx);
                arm_states.push(ctx.var_stacks.clone());
            }
            ctx.var_stacks = saved.clone();
            merge_many_outer_states(ctx, &saved, &arm_states);
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                insert_drops_in_expr(payload, ctx);
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
        HirExprKind::Block(block) => insert_drops_in_block(block, ctx),
        HirExprKind::Let { value, .. } => {
            insert_drops_in_expr(value, ctx);
        }
        HirExprKind::Set { name, value } => {
            insert_drops_in_expr(value, ctx);
            ctx.set_state(name, VarState::Valid);
        }
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => match name.as_str() {
            "load" => {
                let is_copy_load = type_args
                    .get(0)
                    .map(|ty| ctx.types.is_copy(*ty))
                    .unwrap_or(false);
                if !is_copy_load {
                    if let Some(addr) = args.get_mut(0) {
                        insert_drops_in_expr(addr, ctx);
                    }
                }
            }
            "store" => {
                if let Some(val) = args.get_mut(1) {
                    insert_drops_in_expr(val, ctx);
                }
            }
            _ => {
                for arg in args {
                    insert_drops_in_expr(arg, ctx);
                }
            }
        },
        HirExprKind::AddrOf(_) => {}
        HirExprKind::Deref(inner) => {
            insert_drops_in_expr(inner, ctx);
        }
        HirExprKind::Drop { name } => {
            ctx.set_state(name, VarState::Moved);
        }
    }
}

fn process_match_arm(arm: &mut HirMatchArm, ctx: &mut DropInsertionContext<'_>) {
    ctx.push_scope();
    if let Some(bind) = &arm.bind_local {
        let ty = match &arm.body.kind {
            HirExprKind::EnumConstruct {
                payload: Some(payload),
                ..
            } => payload.ty,
            _ => arm.body.ty,
        };
        ctx.declare_var(bind.clone(), ty);
    }
    insert_drops_in_expr(&mut arm.body, ctx);
    let drops = ctx.scope_drop_lines(arm.body.span);
    append_drop_lines_to_expr(&mut arm.body, drops);
    ctx.pop_scope();
}

fn append_drop_lines_to_expr(expr: &mut HirExpr, drops: Vec<HirLine>) {
    if drops.is_empty() {
        return;
    }
    match &mut expr.kind {
        HirExprKind::Block(block) => {
            block.lines.extend(drops);
        }
        _ => {
            let original = expr.clone();
            expr.kind = HirExprKind::Block(HirBlock {
                lines: {
                    let mut lines = Vec::new();
                    lines.push(HirLine {
                        expr: original,
                        drop_result: false,
                    });
                    lines.extend(drops);
                    lines
                },
                ty: expr.ty,
                span: expr.span,
            });
        }
    }
}

fn merge_outer_states(
    ctx: &mut DropInsertionContext<'_>,
    saved: &BTreeMap<String, Vec<VarInfo>>,
    then_state: &BTreeMap<String, Vec<VarInfo>>,
    else_state: &BTreeMap<String, Vec<VarInfo>>,
) {
    for (name, saved_stack) in saved {
        let Some(saved_top) = saved_stack.last().copied() else {
            continue;
        };
        let then_top = then_state
            .get(name)
            .and_then(|stack| stack.last().copied())
            .unwrap_or(saved_top);
        let else_top = else_state
            .get(name)
            .and_then(|stack| stack.last().copied())
            .unwrap_or(saved_top);
        let merged = DropInsertionContext::merge_state(then_top.state, else_top.state);
        if let Some(stack) = ctx.var_stacks.get_mut(name) {
            if let Some(last) = stack.last_mut() {
                last.state = merged;
            }
        }
    }
}

fn merge_many_outer_states(
    ctx: &mut DropInsertionContext<'_>,
    saved: &BTreeMap<String, Vec<VarInfo>>,
    arm_states: &[BTreeMap<String, Vec<VarInfo>>],
) {
    for (name, saved_stack) in saved {
        let Some(saved_top) = saved_stack.last().copied() else {
            continue;
        };
        let mut merged = saved_top.state;
        for arm_state in arm_states {
            let state = arm_state
                .get(name)
                .and_then(|stack| stack.last().copied())
                .unwrap_or(saved_top)
                .state;
            merged = DropInsertionContext::merge_state(merged, state);
        }
        if let Some(stack) = ctx.var_stacks.get_mut(name) {
            if let Some(last) = stack.last_mut() {
                last.state = merged;
            }
        }
    }
}
