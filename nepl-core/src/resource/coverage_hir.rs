extern crate alloc;

use alloc::string::String;

use crate::hir::{FuncRef, HirBlock, HirBody, HirExpr, HirExprKind};
use crate::layout::aggregate_fields_with_offsets;
use crate::runtime_helpers::helper_base_name;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::coverage::ResourceCoverageCounts;
use super::lower_raw_address::is_named_struct_type;
use super::lower_raw_memory::{raw_memory_op_from_callee, raw_memory_op_from_intrinsic};
use super::model::RawMemoryOp;
use super::type_pattern::field_type_matches_result;

pub(super) fn hir_body_coverage(
    body: &HirBody,
    types: &TypeCtx,
    string_literals: &[String],
) -> ResourceCoverageCounts {
    let mut counts = ResourceCoverageCounts::default();
    if let HirBody::Block(block) = body {
        hir_block_coverage(block, &mut counts, types, string_literals);
    }
    counts
}

fn hir_block_coverage(
    block: &HirBlock,
    counts: &mut ResourceCoverageCounts,
    types: &TypeCtx,
    string_literals: &[String],
) {
    for line in &block.lines {
        hir_expr_coverage(&line.expr, counts, types, string_literals);
    }
}

fn hir_expr_coverage(
    expr: &HirExpr,
    counts: &mut ResourceCoverageCounts,
    types: &TypeCtx,
    string_literals: &[String],
) {
    match &expr.kind {
        HirExprKind::LiteralI32(_)
        | HirExprKind::LiteralF32(_)
        | HirExprKind::LiteralBool(_)
        | HirExprKind::LiteralStr(_)
        | HirExprKind::Unit => {}
        HirExprKind::Var(_) => {
            counts.reads += 1;
        }
        HirExprKind::Drop { .. } => {
            counts.drops += 1;
        }
        HirExprKind::FnValue(_) => {
            counts.function_values += 1;
        }
        HirExprKind::Call { callee, args } => {
            if let Some(owner) = field_get_call_owner(callee, args, expr.ty, types, string_literals)
            {
                counts.reads += 1;
                hir_field_projection_source_coverage(owner, counts, types, string_literals);
                return;
            }
            counts.direct_calls += 1;
            if raw_memory_op_from_callee(callee)
                .filter(|operation| should_count_raw_memory_call(operation, args, types))
                .is_some()
            {
                counts.raw_memory_ops += 1;
            }
            for arg in args {
                hir_expr_coverage(arg, counts, types, string_literals);
            }
        }
        HirExprKind::CallIndirect { callee, args, .. } => {
            counts.indirect_calls += 1;
            hir_expr_coverage(callee, counts, types, string_literals);
            for arg in args {
                hir_expr_coverage(arg, counts, types, string_literals);
            }
        }
        HirExprKind::If {
            cond,
            then_branch,
            else_branch,
        } => {
            hir_expr_coverage(cond, counts, types, string_literals);
            hir_expr_coverage(then_branch, counts, types, string_literals);
            hir_expr_coverage(else_branch, counts, types, string_literals);
        }
        HirExprKind::While { cond, body } => {
            hir_expr_coverage(cond, counts, types, string_literals);
            hir_expr_coverage(body, counts, types, string_literals);
        }
        HirExprKind::Match { scrutinee, arms } => {
            hir_expr_coverage(scrutinee, counts, types, string_literals);
            for arm in arms {
                hir_expr_coverage(&arm.body, counts, types, string_literals);
            }
        }
        HirExprKind::EnumConstruct { payload, .. } => {
            counts.constructs += 1;
            if let Some(payload) = payload {
                hir_expr_coverage(payload, counts, types, string_literals);
            }
        }
        HirExprKind::StructConstruct { fields, .. } => {
            counts.constructs += 1;
            for field in fields {
                hir_expr_coverage(field, counts, types, string_literals);
            }
        }
        HirExprKind::TupleConstruct { items } => {
            counts.constructs += 1;
            for item in items {
                hir_expr_coverage(item, counts, types, string_literals);
            }
        }
        HirExprKind::Block(block) => hir_block_coverage(block, counts, types, string_literals),
        HirExprKind::Let { value, .. } => {
            counts.declares += 1;
            hir_expr_coverage(value, counts, types, string_literals);
        }
        HirExprKind::Set { value, .. } => {
            counts.assigns += 1;
            hir_expr_coverage(value, counts, types, string_literals);
        }
        HirExprKind::Intrinsic { name, args, .. } => {
            if let Some(owner) =
                get_field_intrinsic_owner(name, args, expr.ty, types, string_literals)
            {
                counts.reads += 1;
                hir_field_projection_source_coverage(owner, counts, types, string_literals);
                return;
            }
            if let Some((base, _)) = compiler_field_load_base_and_offset(name, args, expr.ty, types)
            {
                counts.reads += 1;
                hir_field_projection_source_coverage(base, counts, types, string_literals);
                return;
            }
            if raw_memory_op_from_intrinsic(name)
                .filter(|operation| should_count_raw_memory_call(operation, args, types))
                .is_some()
            {
                counts.raw_memory_ops += 1;
            }
            for arg in args {
                hir_expr_coverage(arg, counts, types, string_literals);
            }
        }
        HirExprKind::AddrOf(inner) => {
            counts.borrows += 1;
            hir_place_expr_coverage(inner, counts, types, string_literals);
        }
        HirExprKind::Deref(inner) => {
            counts.reads += 1;
            counts.deref_projections += 1;
            hir_place_expr_coverage(inner, counts, types, string_literals);
        }
    }
}

fn hir_place_expr_coverage(
    expr: &HirExpr,
    counts: &mut ResourceCoverageCounts,
    types: &TypeCtx,
    string_literals: &[String],
) {
    match &expr.kind {
        HirExprKind::Var(_) => {}
        HirExprKind::Deref(inner) => {
            counts.deref_projections += 1;
            hir_place_expr_coverage(inner, counts, types, string_literals);
        }
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && !args.is_empty() => {
            hir_place_expr_coverage(&args[0], counts, types, string_literals);
            for arg in args.iter().skip(1) {
                hir_expr_coverage(arg, counts, types, string_literals);
            }
        }
        _ => hir_expr_coverage(expr, counts, types, string_literals),
    }
}

fn hir_field_projection_source_coverage(
    expr: &HirExpr,
    counts: &mut ResourceCoverageCounts,
    types: &TypeCtx,
    string_literals: &[String],
) {
    if let Some(address) = raw_load_address_expr(expr) {
        counts.deref_projections += 1;
        hir_place_expr_coverage(address, counts, types, string_literals);
    } else {
        hir_place_expr_coverage(expr, counts, types, string_literals);
    }
}

fn field_get_call_owner<'a>(
    callee: &FuncRef,
    args: &'a [HirExpr],
    field_ty: TypeId,
    types: &TypeCtx,
    string_literals: &[String],
) -> Option<&'a HirExpr> {
    let name = match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => helper_base_name(name),
        FuncRef::Trait { .. } => return None,
    };
    if name != "get" {
        return None;
    }
    let owner = args.first()?;
    let field_name = literal_field_name(string_literals, args.get(1)?)?;
    aggregate_field_exists_by_name(types, owner.ty, field_name, field_ty).then_some(owner)
}

fn get_field_intrinsic_owner<'a>(
    name: &str,
    args: &'a [HirExpr],
    field_ty: TypeId,
    types: &TypeCtx,
    string_literals: &[String],
) -> Option<&'a HirExpr> {
    if helper_base_name(name) != "get_field" {
        return None;
    }
    let owner = args.first()?;
    let field_name = literal_field_name(string_literals, args.get(1)?)?;
    aggregate_field_exists_by_name(types, owner.ty, field_name, field_ty).then_some(owner)
}

fn raw_load_address_expr(expr: &HirExpr) -> Option<&HirExpr> {
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. } if name == "load" => args.first(),
        HirExprKind::Call { callee, args } if callee_is_raw_load(callee) => args.first(),
        _ => None,
    }
}

fn literal_field_name<'a>(string_literals: &'a [String], expr: &HirExpr) -> Option<&'a str> {
    match &expr.kind {
        HirExprKind::LiteralStr(index) => string_literals.get(*index as usize).map(String::as_str),
        _ => None,
    }
}

fn compiler_field_load_base_and_offset<'a>(
    name: &str,
    args: &'a [HirExpr],
    field_ty: TypeId,
    types: &TypeCtx,
) -> Option<(&'a HirExpr, usize)> {
    if name != "load" {
        return None;
    }
    let address = args.first()?;
    let (base, offset) = compiler_field_address_base_and_offset(address)?;
    aggregate_field_exists(types, base.ty, offset, field_ty).then_some((base, offset))
}

fn compiler_field_address_base_and_offset(expr: &HirExpr) -> Option<(&HirExpr, usize)> {
    match &expr.kind {
        HirExprKind::Intrinsic { name, args, .. } if name == "add" && args.len() == 2 => {
            let offset = match args[1].kind {
                HirExprKind::LiteralI32(value) if value >= 0 => value as usize,
                _ => return None,
            };
            Some((&args[0], offset))
        }
        HirExprKind::Call { callee, args }
            if callee_base_name(callee).is_some_and(|name| name == "add") && args.len() == 2 =>
        {
            let offset = match args[1].kind {
                HirExprKind::LiteralI32(value) if value >= 0 => value as usize,
                _ => return None,
            };
            Some((&args[0], offset))
        }
        _ => Some((expr, 0)),
    }
}

fn aggregate_field_exists(
    types: &TypeCtx,
    owner_ty: TypeId,
    offset: usize,
    field_ty: TypeId,
) -> bool {
    if !is_aggregate_projection_owner(types, owner_ty) {
        return false;
    }
    aggregate_fields_with_offsets(types, owner_ty)
        .iter()
        .any(|field| field.offset == offset && field_type_matches_result(types, field.ty, field_ty))
}

fn aggregate_field_exists_by_name(
    types: &TypeCtx,
    owner_ty: TypeId,
    field_name: &str,
    field_ty: TypeId,
) -> bool {
    let Some(index) = aggregate_field_index(types, owner_ty, field_name) else {
        return false;
    };
    aggregate_fields_with_offsets(types, owner_ty)
        .get(index)
        .is_some_and(|field| field_type_matches_result(types, field.ty, field_ty))
}

fn aggregate_field_index(types: &TypeCtx, owner_ty: TypeId, field_name: &str) -> Option<usize> {
    let resolved = types.resolve_named_type_id(types.resolve_id(owner_ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { field_names, .. } => {
            field_names.iter().position(|name| name == field_name)
        }
        TypeKind::Tuple { .. } => field_name.parse::<usize>().ok(),
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            match types.get_ref(base) {
                TypeKind::Struct { field_names, .. } => {
                    field_names.iter().position(|name| name == field_name)
                }
                TypeKind::Tuple { .. } => field_name.parse::<usize>().ok(),
                _ => None,
            }
        }
        _ => None,
    }
}

fn is_aggregate_projection_owner(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Struct { .. } | TypeKind::Tuple { .. } => true,
        TypeKind::Apply { base, .. } => {
            let base = types.resolve_named_type_id(*base);
            matches!(
                types.get_ref(base),
                TypeKind::Struct { .. } | TypeKind::Tuple { .. }
            )
        }
        _ => false,
    }
}

fn should_count_raw_memory_call(
    operation: &RawMemoryOp,
    args: &[HirExpr],
    types: &TypeCtx,
) -> bool {
    match operation {
        RawMemoryOp::Load
        | RawMemoryOp::Store
        | RawMemoryOp::Dealloc
        | RawMemoryOp::Realloc
        | RawMemoryOp::Fill { .. }
        | RawMemoryOp::BulkCopy
        | RawMemoryOp::BulkMove => args
            .first()
            .map(|arg| !is_named_struct_type(types, arg.ty, "MemPtr"))
            .unwrap_or(true),
        RawMemoryOp::Alloc
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow
        | RawMemoryOp::Other { .. } => true,
    }
}

fn callee_base_name(callee: &FuncRef) -> Option<&str> {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Some(helper_base_name(name)),
        FuncRef::Trait { .. } => None,
    }
}

fn callee_is_raw_load(callee: &FuncRef) -> bool {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => {
            let base = helper_base_name(name);
            base == "load" || base.starts_with("load_")
        }
        FuncRef::Trait { .. } => false,
    }
}
