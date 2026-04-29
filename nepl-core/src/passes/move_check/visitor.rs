use alloc::collections::{BTreeMap, BTreeSet};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::diagnostic::Diagnostic;
use crate::diagnostic_codes::{
    DiagnosticCode, ResourceBorrowDiagnosticCode, ResourceDiagnosticCode,
    ResourceMoveDiagnosticCode,
};
use crate::hir::{FuncRef, HirBlock, HirExpr, HirExprKind};
use crate::layout::storage_size_bytes;
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::branch_merge::{
    changed_state_names, merge_continuing_branch_states, snapshot_top_state, BranchStateSnapshot,
};
use super::provenance::{
    field_move_path_from_addr, field_move_path_from_selector, field_reference_path_from_addr,
    i32_const_from_value, is_field_get_name, raw_addr_alias_from_value,
    raw_aggregate_field_projection_from_get_call, raw_aggregate_field_projection_from_get_field,
    raw_memory_place_key, RawAggregateFieldProjection,
};
use super::raw_memory::{raw_memory_call_kind, RawMemoryCallKind};
use super::raw_memory_args::{
    raw_bulk_copy_size_arg_bytes, raw_byte_write_size_arg_bytes, raw_dealloc_place_key,
    raw_dealloc_size_arg_bytes, raw_store_write_size_bytes,
};
use super::state::{BorrowBinding, BorrowKind, ExprBorrow, ResourceStateSnapshot, VarState};
use super::summary::RawMemoryEffectSummary;
use super::{
    aggregate_field_function_aliases_from_value, aggregate_field_raw_aliases_from_value,
    enum_payload_aggregate_field_function_aliases_from_value,
    enum_payload_aggregate_field_raw_aliases_from_value, enum_payload_function_aliases_from_value,
    enum_payload_raw_aliases_from_value, expression_function_value_aliases,
    instantiate_function_raw_alias_summary, instantiate_known_function_raw_memory_effects,
    match_bind_aggregate_field_function_aliases, match_bind_aggregate_field_raw_aliases,
    match_bind_function_value_aliases, match_bind_raw_addr_alias, value_alias_summary_from_value,
    MoveCheckContext,
};
// Logic to traverse HIR
fn collect_var_uses_block(block: &HirBlock) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    let mut stack = Vec::new();
    for line in block.lines.iter().rev() {
        stack.push(&line.expr);
    }
    while let Some(expr) = stack.pop() {
        match &expr.kind {
            HirExprKind::Var(name) => {
                *counts.entry(name.clone()).or_insert(0) += 1;
            }
            HirExprKind::Call { args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::CallIndirect { callee, args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
                stack.push(callee);
            }
            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                stack.push(else_branch);
                stack.push(then_branch);
                stack.push(cond);
            }
            HirExprKind::While { cond, body } => {
                stack.push(body);
                stack.push(cond);
            }
            HirExprKind::Match { scrutinee, arms } => {
                for arm in arms.iter().rev() {
                    stack.push(&arm.body);
                }
                stack.push(scrutinee);
            }
            HirExprKind::Block(block) => {
                for line in block.lines.iter().rev() {
                    stack.push(&line.expr);
                }
            }
            HirExprKind::Let { value, .. } | HirExprKind::Set { value, .. } => {
                stack.push(value);
            }
            HirExprKind::StructConstruct { fields, .. } => {
                for field in fields.iter().rev() {
                    stack.push(field);
                }
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                if let Some(payload) = payload {
                    stack.push(payload);
                }
            }
            HirExprKind::TupleConstruct { items } | HirExprKind::Intrinsic { args: items, .. } => {
                for item in items.iter().rev() {
                    stack.push(item);
                }
            }
            HirExprKind::AddrOf(inner) | HirExprKind::Deref(inner) => {
                stack.push(inner);
            }
            HirExprKind::FnValue(_)
            | HirExprKind::Unit
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Drop { .. } => {}
        }
    }
    counts
}

fn borrow_source_name(expr: &HirExpr) -> Option<String> {
    match &expr.kind {
        HirExprKind::Var(name) => Some(name.clone()),
        HirExprKind::Deref(inner) => borrow_source_name(inner),
        _ => None,
    }
}

fn borrow_binding(expr: &HirExpr, kind: BorrowKind) -> Option<BorrowBinding> {
    borrow_source_name(expr).map(|source| BorrowBinding { source, kind })
}

fn type_contains_reference(tctx: &crate::types::TypeCtx, ty: TypeId) -> bool {
    fn inner(tctx: &crate::types::TypeCtx, ty: TypeId, visiting: &mut BTreeSet<TypeId>) -> bool {
        let resolved = tctx.resolve_id(ty);
        if !visiting.insert(resolved) {
            return false;
        }
        let contains = match tctx.get_ref(resolved) {
            TypeKind::Reference(_, _) => true,
            TypeKind::Tuple { items } => items.iter().any(|item| inner(tctx, *item, visiting)),
            TypeKind::Struct { fields, .. } => {
                fields.iter().any(|field| inner(tctx, *field, visiting))
            }
            TypeKind::Enum { variants, .. } => variants
                .iter()
                .filter_map(|variant| variant.payload)
                .any(|payload| inner(tctx, payload, visiting)),
            TypeKind::Apply { base, args } => {
                inner(tctx, *base, visiting) || args.iter().any(|arg| inner(tctx, *arg, visiting))
            }
            TypeKind::Box(inner_ty) => inner(tctx, *inner_ty, visiting),
            TypeKind::Var(var) => var
                .binding
                .map(|binding| inner(tctx, binding, visiting))
                .unwrap_or(false),
            TypeKind::Unit
            | TypeKind::I32
            | TypeKind::U8
            | TypeKind::F32
            | TypeKind::Bool
            | TypeKind::Char
            | TypeKind::Str
            | TypeKind::Never
            | TypeKind::Named(_)
            | TypeKind::Function { .. } => false,
        };
        visiting.remove(&resolved);
        contains
    }

    inner(tctx, ty, &mut BTreeSet::new())
}

pub(super) fn is_never_type(tctx: &crate::types::TypeCtx, ty: TypeId) -> bool {
    matches!(tctx.get_ref(tctx.resolve_id(ty)), TypeKind::Never)
}

fn borrow_bindings_from_place(expr: &HirExpr, ctx: &MoveCheckContext) -> Vec<BorrowBinding> {
    match &expr.kind {
        HirExprKind::Var(name) => ctx.borrow_bindings(name),
        HirExprKind::Deref(inner) => borrow_bindings_from_place(inner, ctx),
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && !args.is_empty() => {
            borrow_bindings_from_place(&args[0], ctx)
        }
        _ => Vec::new(),
    }
}

fn addr_of_borrow_kind(tctx: &crate::types::TypeCtx, ty: TypeId) -> BorrowKind {
    reference_borrow_kind(tctx, ty).unwrap_or(BorrowKind::Shared)
}

fn borrow_bindings_from_reference_arg(
    arg: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Vec<BorrowBinding> {
    match &arg.kind {
        HirExprKind::AddrOf(inner) => borrow_binding(inner, addr_of_borrow_kind(tctx, arg.ty))
            .into_iter()
            .collect(),
        _ => borrow_bindings_from_place(arg, ctx),
    }
}

fn reference_source_name(expr: &HirExpr) -> Option<String> {
    match &expr.kind {
        HirExprKind::AddrOf(inner) => borrow_source_name(inner),
        HirExprKind::Deref(inner) => reference_source_name(inner),
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && !args.is_empty() => {
            reference_source_name(&args[0])
        }
        HirExprKind::Var(name) => Some(name.clone()),
        _ => None,
    }
}

fn resource_move_error(
    code: ResourceMoveDiagnosticCode,
    message: impl Into<String>,
    span: Span,
) -> Diagnostic {
    Diagnostic::error_with_code(
        DiagnosticCode::Resource(ResourceDiagnosticCode::Move(code)),
        message,
        span,
    )
}

fn resource_borrow_error(
    code: ResourceBorrowDiagnosticCode,
    message: impl Into<String>,
    span: Span,
) -> Diagnostic {
    Diagnostic::error_with_code(
        DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(code)),
        message,
        span,
    )
}

fn report_loop_possibly_moved(
    ctx: &mut MoveCheckContext,
    saved: &ResourceStateSnapshot,
    body_state: &ResourceStateSnapshot,
    span: Span,
) {
    for name in changed_state_names(saved, body_state) {
        let start_state = snapshot_top_state(saved, name.as_str()).unwrap_or(VarState::Valid);
        let end_state = snapshot_top_state(body_state, name.as_str()).unwrap_or(start_state);
        let merged = MoveCheckContext::merge_state_pair(start_state, end_state);
        if matches!(merged, VarState::PossiblyMoved)
            && matches!(
                start_state,
                VarState::Valid | VarState::BorrowedShared | VarState::BorrowedUnique
            )
            && matches!(end_state, VarState::Moved | VarState::PossiblyMoved)
        {
            ctx.diagnostics.push(resource_move_error(
                ResourceMoveDiagnosticCode::LoopPossiblyMoved,
                alloc::format!("potentially moved value: `{}`", name),
                span,
            ));
        }
    }
}

fn check_non_copy_deref(
    expr: &HirExpr,
    inner: &HirExpr,
    result_borrows: &[ExprBorrow],
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) {
    if tctx.is_copy(expr.ty) {
        return;
    }
    let source = result_borrows
        .first()
        .map(|borrow| borrow.binding.source.clone())
        .or_else(|| reference_source_name(inner));
    let message = if let Some(source) = source {
        alloc::format!("cannot move out of shared borrowed value: `{}`", source)
    } else {
        "cannot move non-Copy value out of shared reference".to_string()
    };
    ctx.diagnostics.push(resource_borrow_error(
        ResourceBorrowDiagnosticCode::MoveFromShared,
        message,
        expr.span,
    ));
}

pub(super) fn visit_block_with_escape(
    block: &HirBlock,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    escape_depth: Option<usize>,
) -> Vec<ExprBorrow> {
    ctx.push_scope();
    ctx.push_use_counts(collect_var_uses_block(block));
    let mut result_borrows = Vec::new();
    let last_index = block.lines.len().saturating_sub(1);
    for (idx, line) in block.lines.iter().enumerate() {
        let line_escape = if idx == last_index && !line.drop_result {
            escape_depth
        } else {
            None
        };
        let line_borrows = visit_expr_with_escape(&line.expr, ctx, tctx, line_escape);
        if idx == last_index && !line.drop_result {
            result_borrows = line_borrows;
            if let Some(depth) = escape_depth {
                ctx.check_expr_borrows_escape(&result_borrows, line.expr.span, depth);
            }
        }
    }
    ctx.pop_use_counts();
    ctx.pop_scope();
    result_borrows
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
) -> Vec<ExprBorrow> {
    let arg_escape_depth = ctx.current_scope_depth();
    let result_borrows = borrow_bindings_from_reference_arg(arg, ctx, tctx)
        .into_iter()
        .map(ExprBorrow::needs_retain)
        .collect();
    match &arg.kind {
        HirExprKind::AddrOf(inner) => visit_temporary_borrow(inner, ctx, kind),
        _ if field_reference_path_from_addr(arg, tctx).is_some() => {
            if let Some(path) = field_reference_path_from_addr(arg, tctx) {
                ctx.check_field_temporary_borrow(&path, arg.span, kind);
            }
        }
        _ => {
            visit_expr_with_escape(arg, ctx, tctx, Some(arg_escape_depth));
        }
    }
    result_borrows
}

fn visit_call_args_with_params(
    args: &[HirExpr],
    params: Option<&[TypeId]>,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Vec<ExprBorrow> {
    let mut result_borrows = Vec::new();
    let mut call_borrows = Vec::new();
    for (i, arg) in args.iter().enumerate() {
        let arg_escape_depth = ctx.current_scope_depth();
        let param_ty = params.and_then(|p| p.get(i)).copied();
        let arg_borrows =
            if let Some(kind) = param_ty.and_then(|ty| reference_borrow_kind(tctx, ty)) {
                visit_reference_call_arg(arg, kind, ctx, tctx)
            } else {
                visit_expr_with_escape(arg, ctx, tctx, Some(arg_escape_depth))
            };
        call_borrows.extend(ctx.retain_expr_borrows(arg_borrows.clone()));
        result_borrows.extend(arg_borrows);
    }
    ctx.release_borrow_bindings(&call_borrows);
    result_borrows
}

fn visit_aggregate_items_with_escape(
    items: &[HirExpr],
    aggregate: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    escape_depth: Option<usize>,
) -> Vec<ExprBorrow> {
    let mut result_borrows = Vec::new();
    let mut construction_borrows = Vec::new();
    for item in items {
        let item_borrows = visit_expr_with_escape(item, ctx, tctx, escape_depth);
        construction_borrows.extend(ctx.retain_expr_borrows(item_borrows.clone()));
        result_borrows.extend(item_borrows);
    }
    if let Some(depth) = escape_depth {
        ctx.check_expr_borrows_escape(&result_borrows, aggregate.span, depth);
    }
    ctx.release_borrow_bindings(&construction_borrows);
    result_borrows
}

fn can_visit_expr_iteratively(
    expr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> bool {
    let mut stack = Vec::new();
    stack.push(expr);
    while let Some(expr) = stack.pop() {
        match &expr.kind {
            HirExprKind::Var(_)
            | HirExprKind::FnValue(_)
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Unit
            | HirExprKind::Drop { .. } => {}
            HirExprKind::Call { callee, args } => {
                if raw_memory_call_kind(callee, args, expr.ty, tctx).is_some() {
                    return false;
                }
                match callee {
                    FuncRef::Builtin(name) | FuncRef::User(name, _, _)
                        if is_field_get_name(name) || name == "if" || name == "while" =>
                    {
                        return false;
                    }
                    _ => {}
                }
                let params = match callee {
                    FuncRef::User(name, _, _) => {
                        if ctx
                            .function_raw_alias_summaries
                            .get(name)
                            .is_some_and(|summary| !summary.raw_memory_effects.is_empty())
                        {
                            return false;
                        }
                        ctx.function_params.get(name).map(Vec::as_slice)
                    }
                    _ => None,
                };
                for (i, arg) in args.iter().enumerate().rev() {
                    let param_ty = params.and_then(|p| p.get(i)).copied();
                    if param_ty
                        .and_then(|ty| reference_borrow_kind(tctx, ty))
                        .is_some()
                    {
                        return false;
                    }
                    stack.push(arg);
                }
            }
            HirExprKind::CallIndirect {
                callee,
                params,
                args,
                ..
            } => {
                if expression_function_value_aliases(callee, ctx, tctx)
                    .iter()
                    .filter_map(|callee_alias| {
                        ctx.function_raw_alias_summaries.get(callee_alias.as_str())
                    })
                    .any(|summary| !summary.raw_memory_effects.is_empty())
                {
                    return false;
                }
                for (i, arg) in args.iter().enumerate().rev() {
                    if params
                        .get(i)
                        .copied()
                        .and_then(|ty| reference_borrow_kind(tctx, ty))
                        .is_some()
                    {
                        return false;
                    }
                    stack.push(arg);
                }
                stack.push(callee);
            }
            HirExprKind::StructConstruct { fields, .. } => {
                for field in fields.iter().rev() {
                    stack.push(field);
                }
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                if let Some(payload) = payload {
                    stack.push(payload);
                }
            }
            HirExprKind::TupleConstruct { items } => {
                for item in items.iter().rev() {
                    stack.push(item);
                }
            }
            HirExprKind::Intrinsic { name, args, .. } => {
                if matches!(name.as_str(), "get_field" | "load" | "store") {
                    return false;
                }
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::If { .. }
            | HirExprKind::While { .. }
            | HirExprKind::Match { .. }
            | HirExprKind::Block(_)
            | HirExprKind::Let { .. }
            | HirExprKind::Set { .. }
            | HirExprKind::AddrOf(_)
            | HirExprKind::Deref(_) => return false,
        }
    }
    true
}

fn visit_expr_iteratively(
    expr: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) {
    let mut stack = Vec::new();
    stack.push(expr);
    while let Some(expr) = stack.pop() {
        let is_copy = tctx.is_copy(expr.ty);
        match &expr.kind {
            HirExprKind::Var(name) => {
                ctx.check_use(name, expr.span, is_copy);
                ctx.note_var_use(name);
            }
            HirExprKind::Drop { name } => ctx.check_drop(name, expr.span),
            HirExprKind::Call { args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
            }
            HirExprKind::CallIndirect { callee, args, .. } => {
                for arg in args.iter().rev() {
                    stack.push(arg);
                }
                stack.push(callee);
            }
            HirExprKind::StructConstruct { fields, .. } => {
                for field in fields.iter().rev() {
                    stack.push(field);
                }
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                if let Some(payload) = payload {
                    stack.push(payload);
                }
            }
            HirExprKind::TupleConstruct { items } | HirExprKind::Intrinsic { args: items, .. } => {
                for item in items.iter().rev() {
                    stack.push(item);
                }
            }
            HirExprKind::FnValue(_)
            | HirExprKind::LiteralI32(_)
            | HirExprKind::LiteralF32(_)
            | HirExprKind::LiteralBool(_)
            | HirExprKind::LiteralStr(_)
            | HirExprKind::Unit => {}
            HirExprKind::If { .. }
            | HirExprKind::While { .. }
            | HirExprKind::Match { .. }
            | HirExprKind::Block(_)
            | HirExprKind::Let { .. }
            | HirExprKind::Set { .. }
            | HirExprKind::AddrOf(_)
            | HirExprKind::Deref(_) => unreachable!("iterative move check precheck failed"),
        }
    }
}

fn visit_expr(
    expr: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Vec<ExprBorrow> {
    visit_expr_with_escape(expr, ctx, tctx, None)
}

fn visit_raw_memory_call(
    kind: RawMemoryCallKind,
    callee: &FuncRef,
    expr: &HirExpr,
    args: &[HirExpr],
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) {
    match kind {
        RawMemoryCallKind::Load => {
            if let Some(addr) = args.get(0) {
                visit_expr(addr, ctx, tctx);
                if let Some(path) = field_move_path_from_addr(addr, expr.ty, tctx) {
                    ctx.check_field_move(&path, expr.span);
                } else if let Some(place) = raw_memory_place_key(addr, ctx, tctx) {
                    ctx.check_raw_non_copy_load(
                        place.as_str(),
                        storage_size_bytes(tctx, expr.ty),
                        expr.span,
                    );
                }
            }
        }
        RawMemoryCallKind::Store => {
            if let Some(addr) = args.get(0) {
                visit_expr(addr, ctx, tctx);
                if let Some(value) = args.get(1) {
                    visit_expr(value, ctx, tctx);
                }
                if let Some(place) = raw_dealloc_place_key(addr, ctx, tctx) {
                    if args.get(1).is_some_and(|value| !tctx.is_copy(value.ty)) {
                        let size = args
                            .get(1)
                            .map(|value| storage_size_bytes(tctx, value.ty))
                            .unwrap_or(0);
                        ctx.check_raw_non_copy_store(place.as_str(), size, expr.span);
                    } else {
                        ctx.check_raw_non_copy_byte_write(
                            place.as_str(),
                            raw_store_write_size_bytes(callee, args.get(1), tctx),
                            expr.span,
                        );
                    }
                }
            }
        }
        RawMemoryCallKind::Dealloc => {
            for arg in args {
                visit_expr(arg, ctx, tctx);
            }
            if let Some(addr) = args.get(0) {
                if let Some(place) = raw_dealloc_place_key(addr, ctx, tctx) {
                    ctx.check_raw_non_copy_dealloc(
                        place.as_str(),
                        raw_dealloc_size_arg_bytes(args.get(1), tctx),
                        expr.span,
                    );
                }
            }
        }
        RawMemoryCallKind::Realloc => {
            for arg in args {
                visit_expr(arg, ctx, tctx);
            }
            if let Some(addr) = args.get(0) {
                if let Some(place) = raw_dealloc_place_key(addr, ctx, tctx) {
                    ctx.check_raw_non_copy_realloc(
                        place.as_str(),
                        raw_dealloc_size_arg_bytes(args.get(1), tctx),
                        expr.span,
                    );
                }
            }
        }
        RawMemoryCallKind::BulkCopy => {
            for arg in args {
                visit_expr(arg, ctx, tctx);
            }
            if let (Some(dst), Some(src)) = (args.get(0), args.get(1)) {
                if let (Some(dst_place), Some(src_place)) = (
                    raw_dealloc_place_key(dst, ctx, tctx),
                    raw_dealloc_place_key(src, ctx, tctx),
                ) {
                    ctx.check_raw_non_copy_bulk_copy(
                        dst_place.as_str(),
                        src_place.as_str(),
                        raw_bulk_copy_size_arg_bytes(args, tctx),
                        expr.span,
                    );
                }
            }
        }
        RawMemoryCallKind::ByteWrite => {
            for arg in args {
                visit_expr(arg, ctx, tctx);
            }
            if let Some(addr) = args.get(0) {
                if let Some(place) = raw_dealloc_place_key(addr, ctx, tctx) {
                    ctx.check_raw_non_copy_byte_write(
                        place.as_str(),
                        raw_byte_write_size_arg_bytes(callee, args, tctx),
                        expr.span,
                    );
                }
            }
        }
    }
}

fn visit_raw_aggregate_field_projection(
    projection: RawAggregateFieldProjection<'_>,
    expr: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Vec<ExprBorrow> {
    if tctx.is_copy(projection.field_ty) {
        visit_temporary_borrow(projection.addr, ctx, BorrowKind::Shared);
    } else {
        visit_expr(projection.addr, ctx, tctx);
        ctx.check_raw_non_copy_load(projection.place.as_str(), projection.size, expr.span);
    }
    Vec::new()
}

fn apply_raw_memory_effect_summary(
    effect: &RawMemoryEffectSummary,
    span: Span,
    ctx: &mut MoveCheckContext,
) {
    match effect {
        RawMemoryEffectSummary::Load { place, size } => {
            ctx.check_raw_non_copy_load(place.as_str(), *size, span);
        }
        RawMemoryEffectSummary::Store { place, size } => {
            ctx.check_raw_non_copy_store(place.as_str(), *size, span);
        }
        RawMemoryEffectSummary::Dealloc { place, size } => {
            ctx.check_raw_non_copy_dealloc(place.as_str(), *size, span);
        }
        RawMemoryEffectSummary::Realloc { place, size } => {
            ctx.check_raw_non_copy_realloc(place.as_str(), *size, span);
        }
        RawMemoryEffectSummary::BulkCopy { dst, src, size } => {
            ctx.check_raw_non_copy_bulk_copy(dst.as_str(), src.as_str(), *size, span);
        }
        RawMemoryEffectSummary::ByteWrite { place, size } => {
            ctx.check_raw_non_copy_byte_write(place.as_str(), *size, span);
        }
        RawMemoryEffectSummary::IndirectCall { .. } => {}
    }
}

fn apply_function_raw_memory_effects(
    callee: &FuncRef,
    args: &[HirExpr],
    span: Span,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) {
    let FuncRef::User(name, _, _) = callee else {
        return;
    };
    let Some(summary) = ctx.function_raw_alias_summaries.get(name).cloned() else {
        return;
    };
    let instantiated = instantiate_function_raw_alias_summary(&summary, args, ctx, tctx);
    for effect in &instantiated.raw_memory_effects {
        apply_raw_memory_effect_summary(effect, span, ctx);
    }
}

fn apply_indirect_function_raw_memory_effects(
    callee: &HirExpr,
    args: &[HirExpr],
    span: Span,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) {
    let callee_aliases = expression_function_value_aliases(callee, ctx, tctx);
    if callee_aliases.is_empty() {
        return;
    }
    let arg_summaries = args
        .iter()
        .map(|arg| value_alias_summary_from_value(arg, ctx, tctx))
        .collect::<Vec<_>>();
    for callee_alias in callee_aliases {
        let effects = instantiate_known_function_raw_memory_effects(
            callee_alias.as_str(),
            &arg_summaries,
            ctx,
            tctx,
            ctx.function_raw_alias_summaries.len().saturating_add(1),
        );
        for effect in &effects {
            apply_raw_memory_effect_summary(effect, span, ctx);
        }
    }
}

fn visit_expr_with_escape(
    expr: &HirExpr,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
    escape_depth: Option<usize>,
) -> Vec<ExprBorrow> {
    if !type_contains_reference(tctx, expr.ty) && can_visit_expr_iteratively(expr, ctx, tctx) {
        visit_expr_iteratively(expr, ctx, tctx);
        return Vec::new();
    }

    let is_copy = tctx.is_copy(expr.ty);
    // ctx.diagnostics.push(Diagnostic::warning(alloc::format!("DEBUG: visiting kind {:?}", expr.kind), expr.span));

    match &expr.kind {
        HirExprKind::Var(name) => {
            let result_borrows = ctx
                .borrow_bindings(name)
                .into_iter()
                .map(ExprBorrow::needs_retain)
                .collect();
            ctx.check_use(name, expr.span, is_copy);
            if let Some(depth) = escape_depth {
                ctx.check_var_escape(name, expr.span, depth);
            }
            ctx.note_var_use(name);
            result_borrows
        }
        HirExprKind::FnValue(_) => Vec::new(),
        HirExprKind::Call { callee, args } => {
            if let Some(kind) = raw_memory_call_kind(callee, args, expr.ty, tctx) {
                visit_raw_memory_call(kind, callee, expr, args, ctx, tctx);
                return Vec::new();
            }
            if let Some(projection) =
                raw_aggregate_field_projection_from_get_call(callee, args, expr.ty, ctx, tctx)
            {
                return visit_raw_aggregate_field_projection(projection, expr, ctx, tctx);
            }
            match callee {
                FuncRef::Builtin(name) | FuncRef::User(name, _, _) if is_field_get_name(name) => {
                    let result_borrows = if type_contains_reference(tctx, expr.ty) {
                        args.first()
                            .map(|base| {
                                borrow_bindings_from_place(base, ctx)
                                    .into_iter()
                                    .map(ExprBorrow::needs_retain)
                                    .collect()
                            })
                            .unwrap_or_default()
                    } else {
                        Vec::new()
                    };
                    if let Some(base) = args.get(0) {
                        if tctx.is_copy(expr.ty) {
                            visit_temporary_borrow(base, ctx, BorrowKind::Shared);
                        } else if !args.get(1).is_some_and(|selector| {
                            visit_field_get_move_source(base, selector, expr.ty, ctx, tctx)
                        }) && !visit_field_move_source(base, expr.ty, ctx, tctx)
                        {
                            visit_expr(base, ctx, tctx);
                        }
                    }
                    for arg in args.iter().skip(1) {
                        visit_expr(arg, ctx, tctx);
                    }
                    if let Some(depth) = escape_depth {
                        ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                    }
                    result_borrows
                }
                FuncRef::Builtin(name) | FuncRef::User(name, _, _) if name == "if" => {
                    if args.len() == 3 {
                        visit_expr(&args[0], ctx, tctx);

                        let saved = ctx.snapshot_resource_state();
                        let then_borrows =
                            visit_expr_with_escape(&args[1], ctx, tctx, escape_depth);
                        let then_state = ctx.snapshot_resource_state();
                        ctx.restore_resource_state(&saved);

                        let else_borrows =
                            visit_expr_with_escape(&args[2], ctx, tctx, escape_depth);
                        let else_state = ctx.snapshot_resource_state();
                        ctx.restore_resource_state(&saved);

                        let then_continues = !is_never_type(tctx, args[1].ty);
                        let else_continues = !is_never_type(tctx, args[2].ty);
                        let branches = [
                            BranchStateSnapshot {
                                continues: then_continues,
                                state: then_state,
                            },
                            BranchStateSnapshot {
                                continues: else_continues,
                                state: else_state,
                            },
                        ];
                        merge_continuing_branch_states(ctx, &saved, &branches);
                        let mut result_borrows = Vec::new();
                        if then_continues {
                            result_borrows.extend(then_borrows);
                        }
                        if else_continues {
                            result_borrows.extend(else_borrows);
                        }
                        result_borrows
                    } else {
                        Vec::new()
                    }
                }
                FuncRef::Builtin(name) | FuncRef::User(name, _, _) if name == "while" => {
                    if args.len() == 2 {
                        visit_expr(&args[0], ctx, tctx);

                        let saved = ctx.snapshot_resource_state();
                        visit_expr(&args[1], ctx, tctx);
                        let body_state = ctx.snapshot_resource_state();

                        report_loop_possibly_moved(ctx, &saved, &body_state, args[1].span);
                        let branches = [
                            BranchStateSnapshot {
                                continues: true,
                                state: saved.clone(),
                            },
                            BranchStateSnapshot {
                                continues: true,
                                state: body_state,
                            },
                        ];
                        merge_continuing_branch_states(ctx, &saved, &branches);
                        visit_expr(&args[0], ctx, tctx);
                    }
                    Vec::new()
                }
                _ => {
                    let params = match callee {
                        FuncRef::User(name, _, _) => ctx.function_params.get(name).cloned(),
                        _ => None,
                    };
                    let result_borrows =
                        visit_call_args_with_params(args, params.as_deref(), ctx, tctx);
                    apply_function_raw_memory_effects(callee, args, expr.span, ctx, tctx);
                    if type_contains_reference(tctx, expr.ty) {
                        if let Some(depth) = escape_depth {
                            ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                        }
                        result_borrows
                    } else {
                        Vec::new()
                    }
                }
            }
        }
        HirExprKind::CallIndirect {
            callee,
            params,
            args,
            ..
        } => {
            visit_expr(callee, ctx, tctx);
            let result_borrows =
                visit_call_args_with_params(args, Some(params.as_slice()), ctx, tctx);
            apply_indirect_function_raw_memory_effects(callee, args, expr.span, ctx, tctx);
            if type_contains_reference(tctx, expr.ty) {
                if let Some(depth) = escape_depth {
                    ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                }
                result_borrows
            } else {
                Vec::new()
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            visit_expr(cond, ctx, tctx);

            let saved = ctx.snapshot_resource_state();
            let then_borrows = visit_expr_with_escape(then_branch, ctx, tctx, escape_depth);
            let then_state = ctx.snapshot_resource_state();
            ctx.restore_resource_state(&saved);

            let else_borrows = visit_expr_with_escape(else_branch, ctx, tctx, escape_depth);
            let else_state = ctx.snapshot_resource_state();
            ctx.restore_resource_state(&saved);

            let then_continues = !is_never_type(tctx, then_branch.ty);
            let else_continues = !is_never_type(tctx, else_branch.ty);
            let branches = [
                BranchStateSnapshot {
                    continues: then_continues,
                    state: then_state,
                },
                BranchStateSnapshot {
                    continues: else_continues,
                    state: else_state,
                },
            ];
            merge_continuing_branch_states(ctx, &saved, &branches);
            let mut result_borrows = Vec::new();
            if then_continues {
                result_borrows.extend(then_borrows);
            }
            if else_continues {
                result_borrows.extend(else_borrows);
            }
            result_borrows
        }
        HirExprKind::While { cond, body } => {
            visit_expr(cond, ctx, tctx);
            let saved = ctx.snapshot_resource_state();
            visit_expr(body, ctx, tctx);
            let body_state = ctx.snapshot_resource_state();

            report_loop_possibly_moved(ctx, &saved, &body_state, expr.span);
            let branches = [
                BranchStateSnapshot {
                    continues: true,
                    state: saved.clone(),
                },
                BranchStateSnapshot {
                    continues: true,
                    state: body_state,
                },
            ];
            merge_continuing_branch_states(ctx, &saved, &branches);
            visit_expr(cond, ctx, tctx);
            Vec::new()
        }
        HirExprKind::Match { scrutinee, arms } => {
            let scrutinee_borrows = visit_expr(scrutinee, ctx, tctx);

            let mut branch_states = Vec::new();
            let mut result_borrows = Vec::new();
            let saved = ctx.snapshot_resource_state();

            for arm in arms {
                ctx.restore_resource_state(&saved);
                ctx.push_scope();
                if let Some(bind) = &arm.bind_local {
                    let retained_borrows = ctx.retain_expr_borrows(scrutinee_borrows.clone());
                    let raw_addr_alias = match_bind_raw_addr_alias(scrutinee, arm, ctx, tctx);
                    let aggregate_field_raw_aliases =
                        match_bind_aggregate_field_raw_aliases(scrutinee, arm, ctx, tctx);
                    let aggregate_field_function_aliases =
                        match_bind_aggregate_field_function_aliases(scrutinee, arm, ctx, tctx);
                    let function_value_aliases =
                        match_bind_function_value_aliases(scrutinee, arm, ctx, tctx);
                    ctx.declare_var_with_borrows(bind.clone(), retained_borrows);
                    ctx.set_raw_addr_alias(bind, raw_addr_alias);
                    ctx.set_aggregate_field_raw_aliases(bind, aggregate_field_raw_aliases);
                    ctx.set_aggregate_field_function_aliases(
                        bind,
                        aggregate_field_function_aliases,
                    );
                    ctx.set_function_value_aliases(bind, function_value_aliases);
                }
                let arm_borrows = visit_expr_with_escape(&arm.body, ctx, tctx, escape_depth);
                ctx.pop_scope();
                let arm_state = ctx.snapshot_resource_state();
                let continues = !is_never_type(tctx, arm.body.ty);
                if continues {
                    result_borrows.extend(arm_borrows);
                }
                branch_states.push(BranchStateSnapshot {
                    continues,
                    state: arm_state,
                });
            }
            ctx.restore_resource_state(&saved);

            merge_continuing_branch_states(ctx, &saved, &branch_states);
            result_borrows
        }
        HirExprKind::Block(b) => visit_block_with_escape(b, ctx, tctx, escape_depth),
        // HirExprKind::Let { name, value, .. } => {
        //     visit_expr(value, ctx, tctx);
        //     ctx.declare_var(name.clone());
        // }
        HirExprKind::Set { value, name } => {
            let target_depth = ctx
                .scope_depth_of(name)
                .unwrap_or_else(|| ctx.current_scope_depth());
            let raw_addr_alias = raw_addr_alias_from_value(value, ctx, tctx);
            let i32_const_alias = i32_const_from_value(value, ctx, tctx);
            let enum_payload_raw_aliases = enum_payload_raw_aliases_from_value(value, ctx, tctx);
            let aggregate_field_raw_aliases =
                aggregate_field_raw_aliases_from_value(value, ctx, tctx);
            let aggregate_field_function_aliases =
                aggregate_field_function_aliases_from_value(value, ctx, tctx);
            let enum_payload_aggregate_field_raw_aliases =
                enum_payload_aggregate_field_raw_aliases_from_value(value, ctx, tctx);
            let enum_payload_aggregate_field_function_aliases =
                enum_payload_aggregate_field_function_aliases_from_value(value, ctx, tctx);
            let enum_payload_function_aliases =
                enum_payload_function_aliases_from_value(value, ctx, tctx);
            let function_value_aliases = expression_function_value_aliases(value, ctx, tctx);
            let value_borrows = visit_expr_with_escape(value, ctx, tctx, Some(target_depth));
            ctx.check_assign(name, expr.span);
            let retained_borrows = ctx.retain_expr_borrows(value_borrows);
            ctx.set_borrow_bindings(name, retained_borrows);
            ctx.set_raw_addr_alias(name, raw_addr_alias);
            ctx.set_i32_const_alias(name, i32_const_alias);
            ctx.set_enum_payload_raw_aliases(name, enum_payload_raw_aliases);
            ctx.set_aggregate_field_raw_aliases(name, aggregate_field_raw_aliases);
            ctx.set_aggregate_field_function_aliases(name, aggregate_field_function_aliases);
            ctx.set_enum_payload_aggregate_field_raw_aliases(
                name,
                enum_payload_aggregate_field_raw_aliases,
            );
            ctx.set_enum_payload_aggregate_field_function_aliases(
                name,
                enum_payload_aggregate_field_function_aliases,
            );
            ctx.set_enum_payload_function_aliases(name, enum_payload_function_aliases);
            ctx.set_function_value_aliases(name, function_value_aliases);
            Vec::new()
        }
        HirExprKind::Let { name, value, .. } => {
            let storage_depth = ctx.current_scope_depth();
            let raw_addr_alias = raw_addr_alias_from_value(value, ctx, tctx);
            let i32_const_alias = i32_const_from_value(value, ctx, tctx);
            let enum_payload_raw_aliases = enum_payload_raw_aliases_from_value(value, ctx, tctx);
            let aggregate_field_raw_aliases =
                aggregate_field_raw_aliases_from_value(value, ctx, tctx);
            let aggregate_field_function_aliases =
                aggregate_field_function_aliases_from_value(value, ctx, tctx);
            let enum_payload_aggregate_field_raw_aliases =
                enum_payload_aggregate_field_raw_aliases_from_value(value, ctx, tctx);
            let enum_payload_aggregate_field_function_aliases =
                enum_payload_aggregate_field_function_aliases_from_value(value, ctx, tctx);
            let enum_payload_function_aliases =
                enum_payload_function_aliases_from_value(value, ctx, tctx);
            let function_value_aliases = expression_function_value_aliases(value, ctx, tctx);
            let value_borrows = visit_expr_with_escape(value, ctx, tctx, Some(storage_depth));
            let retained_borrows = ctx.retain_expr_borrows(value_borrows);
            ctx.declare_var_with_borrows(name.clone(), retained_borrows);
            ctx.set_raw_addr_alias(name, raw_addr_alias);
            ctx.set_i32_const_alias(name, i32_const_alias);
            ctx.set_enum_payload_raw_aliases(name, enum_payload_raw_aliases);
            ctx.set_aggregate_field_raw_aliases(name, aggregate_field_raw_aliases);
            ctx.set_aggregate_field_function_aliases(name, aggregate_field_function_aliases);
            ctx.set_enum_payload_aggregate_field_raw_aliases(
                name,
                enum_payload_aggregate_field_raw_aliases,
            );
            ctx.set_enum_payload_aggregate_field_function_aliases(
                name,
                enum_payload_aggregate_field_function_aliases,
            );
            ctx.set_enum_payload_function_aliases(name, enum_payload_function_aliases);
            ctx.set_function_value_aliases(name, function_value_aliases);
            ctx.set_state(name, VarState::Valid);
            if ctx.remaining_uses(name) == 0 {
                ctx.release_borrow_binding(name);
            }
            Vec::new()
        }
        HirExprKind::StructConstruct { fields, .. } => {
            visit_aggregate_items_with_escape(fields, expr, ctx, tctx, escape_depth)
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            let mut result_borrows = Vec::new();
            if let Some(p) = payload {
                result_borrows.extend(visit_expr_with_escape(p, ctx, tctx, escape_depth));
            }
            if let Some(depth) = escape_depth {
                ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
            }
            result_borrows
        }
        HirExprKind::TupleConstruct { items } => {
            visit_aggregate_items_with_escape(items, expr, ctx, tctx, escape_depth)
        }
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => match name.as_str() {
            "load" => {
                let is_copy_load = type_args
                    .get(0)
                    .map(|ty| tctx.is_copy(*ty))
                    .unwrap_or(false);
                if let Some(addr) = args.get(0) {
                    if is_copy_load {
                        visit_temporary_borrow(addr, ctx, BorrowKind::Shared);
                    } else if !visit_field_move_source(addr, expr.ty, ctx, tctx) {
                        visit_expr(addr, ctx, tctx);
                        if let Some(place) = raw_memory_place_key(addr, ctx, tctx) {
                            ctx.check_raw_non_copy_load(
                                place.as_str(),
                                storage_size_bytes(tctx, expr.ty),
                                expr.span,
                            );
                        }
                    }
                }
                if is_copy_load && type_contains_reference(tctx, expr.ty) {
                    let result_borrows = args
                        .first()
                        .map(|addr| {
                            borrow_bindings_from_place(addr, ctx)
                                .into_iter()
                                .map(ExprBorrow::needs_retain)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    if let Some(depth) = escape_depth {
                        ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                    }
                    result_borrows
                } else {
                    Vec::new()
                }
            }
            "store" => {
                if let Some(addr) = args.get(0) {
                    visit_expr(addr, ctx, tctx);
                }
                if let Some(val) = args.get(1) {
                    visit_expr(val, ctx, tctx);
                }
                if let (Some(addr), Some(val)) = (args.get(0), args.get(1)) {
                    if let Some(place) = raw_memory_place_key(addr, ctx, tctx) {
                        if !tctx.is_copy(val.ty) {
                            ctx.check_raw_non_copy_store(
                                place.as_str(),
                                storage_size_bytes(tctx, val.ty),
                                expr.span,
                            );
                        } else {
                            ctx.check_raw_non_copy_byte_write(
                                place.as_str(),
                                Some(storage_size_bytes(tctx, val.ty)),
                                expr.span,
                            );
                        }
                    }
                }
                Vec::new()
            }
            "get_field" => {
                if let Some(projection) =
                    raw_aggregate_field_projection_from_get_field(expr, ctx, tctx)
                {
                    visit_raw_aggregate_field_projection(projection, expr, ctx, tctx)
                } else {
                    let result_borrows = if type_contains_reference(tctx, expr.ty) {
                        args.first()
                            .map(|base| {
                                borrow_bindings_from_place(base, ctx)
                                    .into_iter()
                                    .map(ExprBorrow::needs_retain)
                                    .collect()
                            })
                            .unwrap_or_default()
                    } else {
                        Vec::new()
                    };
                    if let Some(base) = args.get(0) {
                        if tctx.is_copy(expr.ty) {
                            visit_temporary_borrow(base, ctx, BorrowKind::Shared);
                        } else if !args.get(1).is_some_and(|selector| {
                            visit_field_get_move_source(base, selector, expr.ty, ctx, tctx)
                        }) && !visit_field_move_source(base, expr.ty, ctx, tctx)
                        {
                            visit_expr(base, ctx, tctx);
                        }
                    }
                    for arg in args.iter().skip(1) {
                        visit_expr(arg, ctx, tctx);
                    }
                    if type_contains_reference(tctx, expr.ty) {
                        if let Some(depth) = escape_depth {
                            ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                        }
                        result_borrows
                    } else {
                        Vec::new()
                    }
                }
            }
            _ => {
                let mut result_borrows = Vec::new();
                for arg in args {
                    result_borrows.extend(visit_expr(arg, ctx, tctx));
                }
                if type_contains_reference(tctx, expr.ty) {
                    if let Some(depth) = escape_depth {
                        ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                    }
                    result_borrows
                } else {
                    Vec::new()
                }
            }
        },
        HirExprKind::AddrOf(inner) => {
            let kind = addr_of_borrow_kind(tctx, expr.ty);
            let binding = borrow_binding(inner, kind);
            if let (Some(depth), Some(binding)) = (escape_depth, binding.as_ref()) {
                ctx.check_binding_escape(binding, expr.span, depth);
            }
            visit_temporary_borrow(inner, ctx, kind);
            binding.map(ExprBorrow::needs_retain).into_iter().collect()
        }
        HirExprKind::Deref(inner) => {
            let result_borrows = visit_expr(inner, ctx, tctx);
            check_non_copy_deref(expr, inner, &result_borrows, ctx, tctx);
            if type_contains_reference(tctx, expr.ty) {
                if let Some(depth) = escape_depth {
                    ctx.check_expr_borrows_escape(&result_borrows, expr.span, depth);
                }
                result_borrows
            } else {
                Vec::new()
            }
        }
        HirExprKind::Drop { name } => {
            ctx.check_drop(name, expr.span);
            Vec::new()
        }
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit => Vec::new(),
    }
}

fn visit_temporary_borrow(expr: &HirExpr, ctx: &mut MoveCheckContext, kind: BorrowKind) {
    match &expr.kind {
        HirExprKind::Var(name) => {
            ctx.check_temporary_borrow(name, expr.span, kind);
        }
        HirExprKind::Deref(inner) => {
            visit_temporary_borrow(inner, ctx, kind);
        }
        HirExprKind::Intrinsic { args, .. } => {
            for arg in args {
                visit_temporary_borrow(arg, ctx, kind);
            }
        }
        _ => {}
    }
}

fn visit_field_move_source(
    expr: &HirExpr,
    field_ty: TypeId,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> bool {
    if let Some(path) = field_move_path_from_addr(expr, field_ty, tctx) {
        ctx.check_field_move(&path, expr.span);
        return true;
    }
    match &expr.kind {
        HirExprKind::Var(name) => {
            if !tctx.is_copy(expr.ty) {
                ctx.check_use(name, expr.span, false);
                return true;
            }
            false
        }
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && !args.is_empty() => {
            visit_field_move_source(&args[0], field_ty, ctx, tctx)
        }
        _ => false,
    }
}

fn visit_field_get_move_source(
    owner: &HirExpr,
    selector: &HirExpr,
    field_ty: TypeId,
    ctx: &mut MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> bool {
    if let Some(path) = field_move_path_from_selector(owner, selector, field_ty, ctx, tctx) {
        ctx.check_field_move(&path, owner.span);
        true
    } else {
        false
    }
}
