extern crate alloc;

use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::TraitCapability;
use crate::hir::{FuncRef, HirBlock, HirExpr, HirExprKind, HirLine, HirMatchArm, HirModule};
use crate::layout::aggregate_fields_with_offsets;
use crate::types::{TypeCtx, TypeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VarState {
    Valid,
    Moved,
    PossiblyMoved,
}

#[derive(Debug, Clone)]
struct VarInfo {
    ty: TypeId,
    state: VarState,
    moved_fields: BTreeMap<usize, TypeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FieldMovePath {
    owner: String,
    offset: usize,
    field_ty: TypeId,
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
    next_temp_id: usize,
}

impl<'a> DropInsertionContext<'a> {
    fn new(types: &'a mut TypeCtx, plan: &'a DropPlan) -> Self {
        Self {
            types,
            plan,
            var_stacks: BTreeMap::new(),
            scopes: Vec::new(),
            next_temp_id: 0,
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
                moved_fields: BTreeMap::new(),
            });
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(name);
        }
    }

    fn get_var(&self, name: &str) -> Option<VarInfo> {
        self.var_stacks
            .get(name)
            .and_then(|stack| stack.last().cloned())
    }

    fn set_state(&mut self, name: &str, state: VarState) {
        if let Some(stack) = self.var_stacks.get_mut(name) {
            if let Some(last) = stack.last_mut() {
                last.state = state;
            }
        }
    }

    fn reset_var_to_valid(&mut self, name: &str) {
        if let Some(stack) = self.var_stacks.get_mut(name) {
            if let Some(last) = stack.last_mut() {
                last.state = VarState::Valid;
                last.moved_fields.clear();
            }
        }
    }

    fn mark_field_moved(&mut self, path: &FieldMovePath) {
        if let Some(stack) = self.var_stacks.get_mut(path.owner.as_str()) {
            if let Some(last) = stack.last_mut() {
                last.moved_fields.insert(path.offset, path.field_ty);
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
        let scope = self.scopes.last().cloned().unwrap_or_default();
        for name in scope.iter().rev() {
            let Some(info) = self.get_var(name) else {
                continue;
            };
            out.extend(self.drop_lines_for_info(name, &info, span));
        }
        out
    }

    fn drop_lines_for_info(
        &mut self,
        name: &str,
        info: &VarInfo,
        span: crate::span::Span,
    ) -> Vec<HirLine> {
        if info.state != VarState::Valid {
            return Vec::new();
        }
        if info.moved_fields.is_empty() {
            if self.types.has_drop_impl_target(info.ty) {
                return vec![HirLine {
                    expr: drop_call_expr(self.types, self.plan, name.to_string(), info.ty, span),
                    drop_result: true,
                }];
            }
            return self.structural_field_drop_lines(name, info.ty, info.ty, 0, span);
        }

        let fields = aggregate_fields_with_offsets(self.types, info.ty);
        let mut out = Vec::new();
        for field in fields {
            if info.moved_fields.contains_key(&field.offset) {
                continue;
            }
            out.extend(self.structural_field_drop_lines(
                name,
                info.ty,
                field.ty,
                field.offset,
                span,
            ));
        }
        out
    }

    fn structural_field_drop_lines(
        &mut self,
        name: &str,
        owner_ty: TypeId,
        ty: TypeId,
        base_offset: usize,
        span: crate::span::Span,
    ) -> Vec<HirLine> {
        let drop_fields = structural_drop_fields(self.types, ty, base_offset);
        drop_fields
            .into_iter()
            .map(|(offset, field_ty)| HirLine {
                expr: drop_field_call_expr(
                    self.types,
                    self.plan,
                    name.to_string(),
                    owner_ty,
                    field_ty,
                    offset,
                    span,
                ),
                drop_result: true,
            })
            .collect()
    }

    fn fresh_assignment_temp(&mut self) -> String {
        loop {
            let name = format!("__nepl_drop_assign_tmp_{}", self.next_temp_id);
            self.next_temp_id += 1;
            if !self.var_stacks.contains_key(name.as_str()) {
                return name;
            }
        }
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
        if !tr
            .capabilities
            .iter()
            .any(|cap| *cap == TraitCapability::Drop)
        {
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

fn drop_field_call_expr(
    types: &mut TypeCtx,
    plan: &DropPlan,
    owner_name: String,
    owner_ty: TypeId,
    field_ty: TypeId,
    offset: usize,
    span: crate::span::Span,
) -> HirExpr {
    let ref_ty = types.reference(field_ty, false);
    let arg = if offset == 0 {
        HirExpr {
            ty: ref_ty,
            kind: HirExprKind::Var(owner_name),
            span,
        }
    } else {
        HirExpr {
            ty: ref_ty,
            kind: HirExprKind::Intrinsic {
                name: "add".to_string(),
                type_args: vec![types.i32()],
                args: vec![
                    HirExpr {
                        ty: owner_ty,
                        kind: HirExprKind::Var(owner_name),
                        span,
                    },
                    HirExpr {
                        ty: types.i32(),
                        kind: HirExprKind::LiteralI32(offset as i32),
                        span,
                    },
                ],
            },
            span,
        }
    };
    HirExpr {
        ty: plan.unit_ty,
        kind: HirExprKind::Call {
            callee: FuncRef::Trait {
                trait_name: plan.trait_name.clone(),
                trait_args: Vec::new(),
                method: plan.method_name.clone(),
                self_ty: field_ty,
            },
            args: vec![arg],
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
            FuncRef::Builtin(name) | FuncRef::User(name, _, _) if name == "get" => {
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
            let target_name = name.clone();
            let old_info = ctx.get_var(target_name.as_str());
            insert_drops_in_expr(value, ctx);
            let should_drop_old = old_info
                .filter(|info| info.state == VarState::Valid)
                .filter(|_info| {
                    ctx.get_var(target_name.as_str())
                        .map(|current| current.state == VarState::Valid)
                        .unwrap_or(false)
                });
            if let Some(info) = should_drop_old {
                let drop_lines = ctx.drop_lines_for_info(target_name.as_str(), &info, expr.span);
                if drop_lines.is_empty() {
                    ctx.reset_var_to_valid(target_name.as_str());
                    return;
                }
                let temp_name = ctx.fresh_assignment_temp();
                let temp_ty = value.ty;
                let temp_span = value.span;
                let unit_ty = ctx.types.unit();
                let original_value = core::mem::replace(
                    value,
                    Box::new(HirExpr {
                        ty: unit_ty,
                        kind: HirExprKind::Unit,
                        span: expr.span,
                    }),
                );
                expr.kind = HirExprKind::Block(HirBlock {
                    lines: {
                        let mut lines = vec![HirLine {
                            expr: HirExpr {
                                ty: unit_ty,
                                kind: HirExprKind::Let {
                                    name: temp_name.clone(),
                                    mutable: false,
                                    value: original_value,
                                },
                                span: expr.span,
                            },
                            drop_result: false,
                        }];
                        lines.extend(drop_lines);
                        lines.push(HirLine {
                            expr: HirExpr {
                                ty: unit_ty,
                                kind: HirExprKind::Set {
                                    name: target_name.clone(),
                                    value: Box::new(HirExpr {
                                        ty: temp_ty,
                                        kind: HirExprKind::Var(temp_name),
                                        span: temp_span,
                                    }),
                                },
                                span: expr.span,
                            },
                            drop_result: false,
                        });
                        lines
                    },
                    ty: unit_ty,
                    span: expr.span,
                });
            }
            ctx.reset_var_to_valid(target_name.as_str());
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
                    if let (Some(field_ty), Some(addr)) =
                        (type_args.first().copied(), args.get_mut(0))
                    {
                        if let Some(path) = field_move_path_from_addr(addr, field_ty, ctx.types) {
                            ctx.mark_field_moved(&path);
                        } else {
                            insert_drops_in_expr(addr, ctx);
                        }
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
    if let (Some(bind), Some(ty)) = (&arm.bind_local, arm.bind_ty) {
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

fn structural_drop_fields(types: &TypeCtx, ty: TypeId, base_offset: usize) -> Vec<(usize, TypeId)> {
    fn inner(
        types: &TypeCtx,
        ty: TypeId,
        base_offset: usize,
        visiting: &mut BTreeSet<TypeId>,
    ) -> Vec<(usize, TypeId)> {
        let resolved = types.resolve_id(ty);
        if types.has_drop_impl_target(resolved) {
            return vec![(base_offset, resolved)];
        }
        if !visiting.insert(resolved) {
            return Vec::new();
        }
        let mut out = Vec::new();
        for field in aggregate_fields_with_offsets(types, resolved) {
            out.extend(inner(types, field.ty, base_offset + field.offset, visiting));
        }
        visiting.remove(&resolved);
        out
    }

    inner(types, ty, base_offset, &mut BTreeSet::new())
}

fn field_move_path_from_addr(
    addr: &HirExpr,
    field_ty: TypeId,
    types: &TypeCtx,
) -> Option<FieldMovePath> {
    fn base_owner(expr: &HirExpr) -> Option<(&str, TypeId, usize)> {
        match &expr.kind {
            HirExprKind::Var(name) => Some((name.as_str(), expr.ty, 0)),
            HirExprKind::Intrinsic { name, args, .. } if name == "add" && args.len() >= 2 => {
                let (owner, owner_ty, base_offset) = base_owner(&args[0])?;
                let offset = match &args[1].kind {
                    HirExprKind::LiteralI32(value) if *value >= 0 => *value as usize,
                    _ => return None,
                };
                Some((owner, owner_ty, base_offset + offset))
            }
            _ => None,
        }
    }

    let (owner, owner_ty, offset) = base_owner(addr)?;
    let field_ty = types.resolve_id(field_ty);
    let is_declared_field = aggregate_fields_with_offsets(types, owner_ty)
        .into_iter()
        .any(|field| field.offset == offset && types.resolve_id(field.ty) == field_ty);
    if is_declared_field {
        Some(FieldMovePath {
            owner: owner.to_string(),
            offset,
            field_ty,
        })
    } else {
        None
    }
}

fn merge_info(saved: &VarInfo, branch_infos: &[VarInfo]) -> VarInfo {
    let mut state = branch_infos
        .iter()
        .map(|info| info.state)
        .reduce(DropInsertionContext::merge_state)
        .unwrap_or(saved.state);
    let first_fields = branch_infos
        .first()
        .map(|info| info.moved_fields.clone())
        .unwrap_or_else(|| saved.moved_fields.clone());
    let fields_match = branch_infos
        .iter()
        .all(|info| info.moved_fields == first_fields);
    let moved_fields = if fields_match {
        first_fields
    } else {
        state = VarState::PossiblyMoved;
        BTreeMap::new()
    };
    VarInfo {
        ty: saved.ty,
        state,
        moved_fields,
    }
}

fn merge_outer_states(
    ctx: &mut DropInsertionContext<'_>,
    saved: &BTreeMap<String, Vec<VarInfo>>,
    then_state: &BTreeMap<String, Vec<VarInfo>>,
    else_state: &BTreeMap<String, Vec<VarInfo>>,
) {
    for (name, saved_stack) in saved {
        let Some(saved_top) = saved_stack.last().cloned() else {
            continue;
        };
        let then_top = then_state
            .get(name)
            .and_then(|stack| stack.last().cloned())
            .unwrap_or_else(|| saved_top.clone());
        let else_top = else_state
            .get(name)
            .and_then(|stack| stack.last().cloned())
            .unwrap_or_else(|| saved_top.clone());
        let merged = merge_info(&saved_top, &[then_top, else_top]);
        if let Some(stack) = ctx.var_stacks.get_mut(name) {
            if let Some(last) = stack.last_mut() {
                *last = merged;
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
        let Some(saved_top) = saved_stack.last().cloned() else {
            continue;
        };
        let mut infos = Vec::new();
        for arm_state in arm_states {
            let info = arm_state
                .get(name)
                .and_then(|stack| stack.last().cloned())
                .unwrap_or_else(|| saved_top.clone());
            infos.push(info);
        }
        let merged = merge_info(&saved_top, &infos);
        if let Some(stack) = ctx.var_stacks.get_mut(name) {
            if let Some(last) = stack.last_mut() {
                *last = merged;
            }
        }
    }
}
