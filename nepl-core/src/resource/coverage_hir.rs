extern crate alloc;

use alloc::string::String;

use crate::hir::{HirBlock, HirBody, HirExpr, HirExprKind};
use crate::types::TypeCtx;

use super::coverage::ResourceCoverageCounts;
use super::coverage_hir_projection::{
    callee_projects_reference_address, compiler_field_load_base_and_offset, field_get_call_owner,
    get_field_intrinsic_owner, get_field_ref_intrinsic_owner, intrinsic_projects_reference_address,
    raw_load_address_expr,
};
use super::coverage_hir_raw::should_count_raw_memory_call;
use super::lower_raw_memory::{raw_memory_op_from_callee, raw_memory_op_from_intrinsic};

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
            if callee_projects_reference_address(callee, args, types) {
                counts.deref_projections += 1;
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
                get_field_ref_intrinsic_owner(name, args, expr.ty, types, string_literals)
            {
                counts.borrows += 1;
                counts.deref_projections += 1;
                hir_place_expr_coverage(owner, counts, types, string_literals);
                return;
            }
            if let Some(owner) =
                get_field_intrinsic_owner(name, args, expr.ty, types, string_literals)
            {
                counts.reads += 1;
                hir_field_projection_source_coverage(owner, counts, types, string_literals);
                return;
            }
            if intrinsic_projects_reference_address(name, args, types) {
                counts.deref_projections += 1;
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
