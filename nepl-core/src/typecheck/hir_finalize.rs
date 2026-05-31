use alloc::format;
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::TypeDiagnosticCode;
use crate::hir::{FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction};
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::diagnostics::type_error;

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
                        application,
                        self_ty,
                        ..
                    } => {
                        for ty in &mut application.args {
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
                effect: _,
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
            | HirExprKind::MemoizedFunctionValue(_)
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

pub(super) fn unresolved_generic_call_type_arg_diagnostics(
    ctx: &TypeCtx,
    function: &HirFunction,
) -> Vec<Diagnostic> {
    let allowed = function_type_params(ctx, function.func_ty);
    let mut diagnostics = Vec::new();
    if let HirBody::Block(block) = &function.body {
        unresolved_generic_call_type_arg_diagnostics_in_block(
            ctx,
            block,
            &allowed,
            &mut diagnostics,
        );
    }
    diagnostics
}

fn function_type_params(ctx: &TypeCtx, func_ty: TypeId) -> Vec<TypeId> {
    match ctx.get(func_ty) {
        TypeKind::Function { type_params, .. } => type_params
            .into_iter()
            .map(|ty| ctx.resolve_id(ty))
            .collect(),
        _ => Vec::new(),
    }
}

fn unresolved_generic_call_type_arg_diagnostics_in_block(
    ctx: &TypeCtx,
    block: &HirBlock,
    allowed: &[TypeId],
    diagnostics: &mut Vec<Diagnostic>,
) {
    for line in &block.lines {
        unresolved_generic_call_type_arg_diagnostics_in_expr(ctx, &line.expr, allowed, diagnostics);
    }
}

fn unresolved_generic_call_type_arg_diagnostics_in_expr(
    ctx: &TypeCtx,
    expr: &HirExpr,
    allowed: &[TypeId],
    diagnostics: &mut Vec<Diagnostic>,
) {
    match &expr.kind {
        HirExprKind::Call { callee, args } => {
            if let FuncRef::User(name, type_args, _) = callee {
                if let Some(ty) = type_args
                    .iter()
                    .copied()
                    .find(|ty| type_contains_unbound_var_outside(ctx, *ty, allowed))
                {
                    diagnostics.push(type_error(
                        TypeDiagnosticCode::GenericTypeArgsUnresolved,
                        format!(
                            "generic call '{}' has unresolved type argument '{}'",
                            name,
                            ctx.type_to_string(ty)
                        ),
                        expr.span,
                    ));
                }
            }
            for arg in args {
                unresolved_generic_call_type_arg_diagnostics_in_expr(
                    ctx,
                    arg,
                    allowed,
                    diagnostics,
                );
            }
        }
        HirExprKind::CallIndirect {
            callee,
            args,
            params: _,
            result: _,
            effect: _,
        } => {
            unresolved_generic_call_type_arg_diagnostics_in_expr(ctx, callee, allowed, diagnostics);
            for arg in args {
                unresolved_generic_call_type_arg_diagnostics_in_expr(
                    ctx,
                    arg,
                    allowed,
                    diagnostics,
                );
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            unresolved_generic_call_type_arg_diagnostics_in_expr(ctx, cond, allowed, diagnostics);
            unresolved_generic_call_type_arg_diagnostics_in_expr(
                ctx,
                then_branch,
                allowed,
                diagnostics,
            );
            unresolved_generic_call_type_arg_diagnostics_in_expr(
                ctx,
                else_branch,
                allowed,
                diagnostics,
            );
        }
        HirExprKind::While { cond, body } => {
            unresolved_generic_call_type_arg_diagnostics_in_expr(ctx, cond, allowed, diagnostics);
            unresolved_generic_call_type_arg_diagnostics_in_expr(ctx, body, allowed, diagnostics);
        }
        HirExprKind::Match { scrutinee, arms } => {
            unresolved_generic_call_type_arg_diagnostics_in_expr(
                ctx,
                scrutinee,
                allowed,
                diagnostics,
            );
            for arm in arms {
                unresolved_generic_call_type_arg_diagnostics_in_expr(
                    ctx,
                    &arm.body,
                    allowed,
                    diagnostics,
                );
            }
        }
        HirExprKind::Block(block) => {
            unresolved_generic_call_type_arg_diagnostics_in_block(ctx, block, allowed, diagnostics);
        }
        HirExprKind::Let { value, .. }
        | HirExprKind::Set { value, .. }
        | HirExprKind::AddrOf(value)
        | HirExprKind::Deref(value) => {
            unresolved_generic_call_type_arg_diagnostics_in_expr(ctx, value, allowed, diagnostics);
        }
        HirExprKind::TupleConstruct { items } | HirExprKind::Intrinsic { args: items, .. } => {
            for item in items {
                unresolved_generic_call_type_arg_diagnostics_in_expr(
                    ctx,
                    item,
                    allowed,
                    diagnostics,
                );
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            if let Some(payload) = payload {
                unresolved_generic_call_type_arg_diagnostics_in_expr(
                    ctx,
                    payload,
                    allowed,
                    diagnostics,
                );
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            for field in fields {
                unresolved_generic_call_type_arg_diagnostics_in_expr(
                    ctx,
                    field,
                    allowed,
                    diagnostics,
                );
            }
        }
        HirExprKind::FnValue(_)
        | HirExprKind::MemoizedFunctionValue(_)
        | HirExprKind::Var(_)
        | HirExprKind::Unit
        | HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Drop { .. } => {}
    }
}

fn type_contains_unbound_var_outside(ctx: &TypeCtx, ty: TypeId, allowed: &[TypeId]) -> bool {
    let ty = ctx.resolve_id(ty);
    if allowed.iter().any(|allowed| ctx.resolve_id(*allowed) == ty) {
        return false;
    }
    match ctx.get(ty) {
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Named(_) => false,
        TypeKind::Var(tv) => tv.binding.is_none(),
        TypeKind::Enum { type_params, .. } | TypeKind::Struct { type_params, .. } => type_params
            .iter()
            .any(|ty| type_contains_unbound_var_outside(ctx, *ty, allowed)),
        TypeKind::Function {
            type_params,
            params,
            result,
            ..
        } => {
            type_params
                .iter()
                .any(|ty| type_contains_unbound_var_outside(ctx, *ty, allowed))
                || params
                    .iter()
                    .any(|ty| type_contains_unbound_var_outside(ctx, *ty, allowed))
                || type_contains_unbound_var_outside(ctx, result, allowed)
        }
        TypeKind::Tuple { items } => items
            .iter()
            .any(|ty| type_contains_unbound_var_outside(ctx, *ty, allowed)),
        TypeKind::Apply { base: _, args } => args
            .iter()
            .any(|ty| type_contains_unbound_var_outside(ctx, *ty, allowed)),
        TypeKind::Box(inner) | TypeKind::Reference(inner, _) => {
            type_contains_unbound_var_outside(ctx, inner, allowed)
        }
    }
}
