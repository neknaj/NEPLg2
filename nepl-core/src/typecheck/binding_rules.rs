use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::ast::{FnBody, FnDef, PrefixItem, Stmt};
use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::ResolveDiagnosticCode;
use crate::resolve::ImportResolution;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::diagnostics::resolve_warning;
use super::env::{Binding, BindingKind, Env};
use super::signature::same_function_signature;
use super::FieldAccessorKind;

pub(super) fn is_important_shadow_symbol(name: &str) -> bool {
    matches!(
        name,
        "print"
            | "println"
            | "print_i32"
            | "println_i32"
            | "add"
            | "sub"
            | "mul"
            | "div"
            | "eq"
            | "lt"
            | "le"
            | "gt"
            | "ge"
    )
}

pub(super) fn emit_shadow_warning(
    diagnostics: &mut Vec<Diagnostic>,
    env: &Env,
    name: &str,
    span: Span,
    kind: &str,
) {
    if let Some(shadowed) = env.lookup_outer_defined(name) {
        if !is_important_shadow_symbol(name) {
            return;
        }
        let message = format!("important symbol '{}' is shadowed by local {}", name, kind);
        let mut diag = resolve_warning(ResolveDiagnosticCode::ShadowImportantSymbol, message, span);
        diag = diag.with_secondary_label(
            shadowed.span,
            Some(String::from("shadowed definition is here")),
        );
        diagnostics.push(diag);
    } else if is_important_shadow_symbol(name) {
        diagnostics.push(resolve_warning(
            ResolveDiagnosticCode::ShadowImportantSymbol,
            format!(
                "definition '{}' may shadow important stdlib symbol ({})",
                name, kind
            ),
            span,
        ));
    }
}

pub(super) fn shadow_blocked_by_nonshadow<'a>(env: &'a Env, name: &str) -> Option<&'a Binding> {
    env.lookup_any(name).and_then(|b| {
        if b.no_shadow && b.defined {
            Some(b)
        } else {
            None
        }
    })
}

pub(super) fn is_callable_binding(binding: &Binding) -> bool {
    matches!(binding.kind, BindingKind::Func { .. })
}

pub(super) fn find_same_signature_func_in_file<'a>(
    env: &'a Env,
    name: &str,
    ty: TypeId,
    span: Span,
    ctx: &TypeCtx,
) -> Option<&'a Binding> {
    env.lookup_all_callables(name).into_iter().find(|b| {
        b.span != span
            && b.span.file_id == span.file_id
            && matches!(b.kind, BindingKind::Func { .. })
            && same_function_signature(ctx, b.ty, ty)
    })
}

pub(super) fn find_visible_same_signature_func<'a>(
    env: &'a Env,
    import_resolution: &ImportResolution,
    name: &str,
    ty: TypeId,
    span: Span,
    ctx: &TypeCtx,
) -> Option<&'a Binding> {
    env.lookup_all_callables(name).into_iter().find(|b| {
        b.span != span
            && import_resolution.binding_is_visible_unqualified(
                span.file_id.0,
                name,
                b.span.file_id.0,
                &b.name,
            )
            && matches!(b.kind, BindingKind::Func { .. })
            && same_function_signature(ctx, b.ty, ty)
    })
}

pub(super) fn find_visible_nonshadow_same_signature_func<'a>(
    env: &'a Env,
    import_resolution: &ImportResolution,
    name: &str,
    ty: TypeId,
    span: Span,
    ctx: &TypeCtx,
) -> Option<&'a Binding> {
    find_visible_same_signature_func(env, import_resolution, name, ty, span, ctx)
        .filter(|b| b.no_shadow && b.defined)
}

pub(super) fn find_invalid_same_file_overload<'a>(
    env: &'a Env,
    name: &str,
    arity: usize,
    span: Span,
) -> Option<&'a Binding> {
    if span.file_id.0 != 0 {
        return None;
    }
    env.lookup_all_callables(name).into_iter().find(|b| {
        if b.span.file_id != span.file_id {
            return false;
        }
        let BindingKind::Func {
            arity: existing_arity,
            ..
        } = b.kind
        else {
            return false;
        };
        existing_arity != arity
    })
}

pub(super) fn type_shape_specificity(ctx: &TypeCtx, ty: TypeId) -> usize {
    match ctx.get(ctx.resolve_id(ty)) {
        TypeKind::Var(_) => 0,
        TypeKind::Unit
        | TypeKind::I32
        | TypeKind::U8
        | TypeKind::F32
        | TypeKind::Bool
        | TypeKind::Char
        | TypeKind::Str
        | TypeKind::Never
        | TypeKind::Named(_) => 2,
        TypeKind::Enum { .. } | TypeKind::Struct { .. } => 3,
        TypeKind::Apply { base, args } => {
            4 + type_shape_specificity(ctx, base)
                + args
                    .iter()
                    .map(|arg| type_shape_specificity(ctx, *arg))
                    .sum::<usize>()
        }
        TypeKind::Tuple { items } => {
            1 + items
                .iter()
                .map(|item| type_shape_specificity(ctx, *item))
                .sum::<usize>()
        }
        TypeKind::Function { params, result, .. } => {
            1 + params
                .iter()
                .map(|param| type_shape_specificity(ctx, *param))
                .sum::<usize>()
                + type_shape_specificity(ctx, result)
        }
        TypeKind::Box(inner) | TypeKind::Reference(inner, _) => {
            1 + type_shape_specificity(ctx, inner)
        }
    }
}

pub(super) fn function_user_param_specificity(
    ctx: &TypeCtx,
    ty: TypeId,
    user_arity: usize,
) -> usize {
    match ctx.get(ctx.resolve_id(ty)) {
        TypeKind::Function { params, result, .. } => {
            let capture_len = params.len().saturating_sub(user_arity);
            let user_params = &params[capture_len..];
            user_params
                .iter()
                .map(|param| type_shape_specificity(ctx, *param))
                .sum::<usize>()
                + type_shape_specificity(ctx, result)
        }
        _ => 0,
    }
}

pub(super) fn detect_field_accessor_fn(def: &FnDef) -> Option<FieldAccessorKind> {
    let FnBody::Parsed(block) = &def.body else {
        return None;
    };
    if block.items.len() != 1 {
        return None;
    }
    let expr = match &block.items[0] {
        Stmt::Expr(expr) | Stmt::ExprSemi(expr, _) => expr,
        _ => return None,
    };
    match expr.items.as_slice() {
        [PrefixItem::Intrinsic(intrin, _)] if intrin.name == "get_field" => {
            Some(FieldAccessorKind::Get)
        }
        [PrefixItem::Intrinsic(intrin, _)] if intrin.name == "get_field_ref" => {
            Some(FieldAccessorKind::GetRef)
        }
        [PrefixItem::Intrinsic(intrin, _)] if intrin.name == "set_field" => {
            Some(FieldAccessorKind::Put)
        }
        _ => None,
    }
}
