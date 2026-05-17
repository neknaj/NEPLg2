extern crate alloc;

use alloc::string::String;

use crate::hir::{HirExpr, HirExprKind};
use crate::types::TypeCtx;

use super::address_projection::intrinsic_is_address_projection;
use super::coverage::ResourceCoverageCounts;
use super::coverage_hir::hir_expr_coverage;
use super::coverage_hir_projection::{intrinsic_projects_reference_field, raw_load_address_expr};

pub(super) fn hir_place_expr_coverage(
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
        HirExprKind::Intrinsic { name, args, .. }
            if intrinsic_is_address_projection(name) && !args.is_empty() =>
        {
            if intrinsic_projects_reference_field(name, args, expr.ty, types) {
                counts.borrows += 1;
                counts.deref_projections += 1;
            }
            hir_place_expr_coverage(&args[0], counts, types, string_literals);
            for arg in args.iter().skip(1) {
                hir_expr_coverage(arg, counts, types, string_literals);
            }
        }
        _ => hir_expr_coverage(expr, counts, types, string_literals),
    }
}

pub(super) fn hir_field_projection_source_coverage(
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

pub(super) fn hir_reference_owner_source_coverage(
    expr: &HirExpr,
    counts: &mut ResourceCoverageCounts,
    types: &TypeCtx,
    string_literals: &[String],
) {
    if let HirExprKind::AddrOf(inner) = &expr.kind {
        hir_place_expr_coverage(inner, counts, types, string_literals);
    } else {
        hir_place_expr_coverage(expr, counts, types, string_literals);
    }
}
