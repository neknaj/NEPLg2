extern crate alloc;

use alloc::string::String;

use crate::hir::HirExpr;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::coverage::ResourceCoverageCounts;
use super::coverage_hir_place::hir_place_expr_coverage;
use super::coverage_hir_scope::HirCoverageContext;

pub(super) fn hir_match_scrutinee_coverage(
    context: &mut HirCoverageContext,
    scrutinee: &HirExpr,
    counts: &mut ResourceCoverageCounts,
    types: &TypeCtx,
    string_literals: &[String],
) {
    if type_is_reference_to_enum(types, scrutinee.ty) {
        hir_place_expr_coverage(scrutinee, counts, types, string_literals);
    } else {
        context.hir_expr_coverage(scrutinee, counts, types, string_literals);
    }
}

fn type_is_reference_to_enum(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    let TypeKind::Reference(target, _) = types.get_ref(resolved) else {
        return false;
    };
    type_is_enum_like(types, *target)
}

fn type_is_enum_like(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Enum { .. } => true,
        TypeKind::Apply { base, .. } => {
            matches!(
                types.get_ref(types.resolve_named_type_id(*base)),
                TypeKind::Enum { .. }
            )
        }
        _ => false,
    }
}
