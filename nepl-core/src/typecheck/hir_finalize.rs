use alloc::vec::Vec;

use crate::hir::{FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction};
use crate::types::TypeCtx;

pub(super) fn resolve_type_ids_in_function(ctx: &TypeCtx, function: &mut HirFunction) {
    function.func_ty = ctx.resolve_id(function.func_ty);
    function.result = ctx.resolve_id(function.result);
    for param in &mut function.params {
        param.ty = ctx.resolve_id(param.ty);
    }
    match &mut function.body {
        HirBody::Block(block) => resolve_type_ids_in_block(ctx, block),
        HirBody::Wasm(_) | HirBody::LlvmIr(_) => {}
    }
}

pub(super) fn resolve_type_ids_in_block(ctx: &TypeCtx, block: &mut HirBlock) {
    block.ty = ctx.resolve_id(block.ty);
    for line in &mut block.lines {
        resolve_type_ids_in_expr(ctx, &mut line.expr);
    }
}

pub(super) fn resolve_type_ids_in_expr(ctx: &TypeCtx, expr: &mut HirExpr) {
    let mut pending = Vec::new();
    pending.push(expr);
    while let Some(expr) = pending.pop() {
        expr.ty = ctx.resolve_id(expr.ty);
        match &mut expr.kind {
            HirExprKind::Call { callee, args } => {
                match callee {
                    FuncRef::User(_, type_args, _) => {
                        for ty in type_args {
                            *ty = ctx.resolve_id(*ty);
                        }
                    }
                    FuncRef::Trait {
                        trait_args,
                        self_ty,
                        ..
                    } => {
                        for ty in trait_args {
                            *ty = ctx.resolve_id(*ty);
                        }
                        *self_ty = ctx.resolve_id(*self_ty);
                    }
                    FuncRef::Builtin(_) => {}
                }
                for arg in args {
                    pending.push(arg);
                }
            }
            HirExprKind::CallIndirect {
                callee,
                params,
                result,
                args,
            } => {
                pending.push(callee);
                for ty in params {
                    *ty = ctx.resolve_id(*ty);
                }
                *result = ctx.resolve_id(*result);
                for arg in args {
                    pending.push(arg);
                }
            }
            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                pending.push(cond);
                pending.push(then_branch);
                pending.push(else_branch);
            }
            HirExprKind::While { cond, body } => {
                pending.push(cond);
                pending.push(body);
            }
            HirExprKind::Match { scrutinee, arms } => {
                pending.push(scrutinee);
                for arm in arms {
                    pending.push(&mut arm.body);
                }
            }
            HirExprKind::Block(block) => {
                block.ty = ctx.resolve_id(block.ty);
                for line in &mut block.lines {
                    pending.push(&mut line.expr);
                }
            }
            HirExprKind::Let { value, .. }
            | HirExprKind::Set { value, .. }
            | HirExprKind::AddrOf(value)
            | HirExprKind::Deref(value) => pending.push(value),
            HirExprKind::TupleConstruct { items } | HirExprKind::Intrinsic { args: items, .. } => {
                for item in items {
                    pending.push(item);
                }
            }
            HirExprKind::EnumConstruct {
                type_args, payload, ..
            } => {
                for ty in type_args {
                    *ty = ctx.resolve_id(*ty);
                }
                if let Some(payload) = payload {
                    pending.push(payload);
                }
            }
            HirExprKind::StructConstruct {
                type_args, fields, ..
            } => {
                for ty in type_args {
                    *ty = ctx.resolve_id(*ty);
                }
                for field in fields {
                    pending.push(field);
                }
            }
            HirExprKind::FnValue(_)
            | HirExprKind::Var(_)
            | HirExprKind::Unit
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Drop { .. } => {}
        }
    }
}
