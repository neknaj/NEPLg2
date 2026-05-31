extern crate alloc;

use alloc::string::String;

use crate::hir::{HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirModule};
use crate::resource_primitives::{CollectionSlotBorrowPrimitive, CollectionSlotLifecyclePrimitive};
use crate::types::TypeCtx;

use super::coverage::ResourceCoverageCounts;
use super::coverage_hir_match::hir_match_scrutinee_coverage;
use super::coverage_hir_place::{
    hir_field_projection_source_coverage, hir_place_expr_coverage,
    hir_reference_owner_source_coverage,
};
use super::coverage_hir_projection::{
    callee_projects_reference_address, callee_projects_reference_field,
    compiler_field_load_base_and_offset, get_field_intrinsic_owner, get_field_ref_intrinsic_owner,
    intrinsic_projects_reference_address, intrinsic_projects_reference_field,
};
use super::coverage_hir_raw::should_count_raw_memory_call;
use super::coverage_hir_scope::HirCoverageContext;
use super::coverage_hir_transparent::transparent_raw_address_return_deref_projection_count;
use super::lower_raw_memory::{raw_memory_op_from_callee, raw_memory_op_from_intrinsic};

pub(super) fn hir_function_coverage(
    function: &HirFunction,
    module: &HirModule,
    types: &TypeCtx,
    string_literals: &[String],
) -> ResourceCoverageCounts {
    let mut counts = ResourceCoverageCounts::default();
    let mut context = HirCoverageContext::new(function, module);
    if let HirBody::Block(block) = &function.body {
        hir_block_coverage(&mut context, block, &mut counts, types, string_literals);
    }
    counts
}

pub(super) fn hir_expr_coverage(
    expr: &HirExpr,
    counts: &mut ResourceCoverageCounts,
    types: &TypeCtx,
    string_literals: &[String],
) {
    let mut context = HirCoverageContext::empty();
    context.hir_expr_coverage(expr, counts, types, string_literals);
}

fn hir_block_coverage(
    context: &mut HirCoverageContext<'_>,
    block: &HirBlock,
    counts: &mut ResourceCoverageCounts,
    types: &TypeCtx,
    string_literals: &[String],
) {
    context.push_scope();
    for line in &block.lines {
        context.hir_expr_coverage(&line.expr, counts, types, string_literals);
    }
    context.pop_scope();
}

impl HirCoverageContext<'_> {
    pub(super) fn hir_expr_coverage(
        &mut self,
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
            HirExprKind::Var(name) => {
                if self.var_is_callable_value_reference(name) {
                    counts.function_values += 1;
                } else {
                    counts.reads += 1;
                }
            }
            HirExprKind::Drop { .. } => {
                counts.drops += 1;
            }
            HirExprKind::FnValue(_) | HirExprKind::MemoizedFunctionValue(_) => {
                counts.function_values += 1;
            }
            HirExprKind::Call { callee, args } => {
                if callee_projects_reference_field(callee, args, expr.ty, types) {
                    counts.borrows += 1;
                    counts.deref_projections += 1;
                } else if callee_projects_reference_address(callee, args, types) {
                    counts.deref_projections += 1;
                } else {
                    counts.deref_projections +=
                        transparent_raw_address_return_deref_projection_count(
                            self, callee, args, expr.ty, types,
                        );
                }
                counts.direct_calls += 1;
                if self.call_is_explicit_drop(callee)
                    && args
                        .first()
                        .is_some_and(|arg| matches!(arg.kind, HirExprKind::AddrOf(_)))
                {
                    counts.drops += 1;
                }
                if raw_memory_op_from_callee(callee)
                    .filter(|operation| should_count_raw_memory_call(operation, args, types))
                    .is_some()
                {
                    counts.raw_memory_ops += 1;
                }
                for arg in args {
                    self.hir_expr_coverage(arg, counts, types, string_literals);
                }
            }
            HirExprKind::CallIndirect { callee, args, .. } => {
                counts.indirect_calls += 1;
                self.hir_expr_coverage(callee, counts, types, string_literals);
                for arg in args {
                    self.hir_expr_coverage(arg, counts, types, string_literals);
                }
            }
            HirExprKind::If {
                cond,
                then_branch,
                else_branch,
            } => {
                self.hir_expr_coverage(cond, counts, types, string_literals);
                self.hir_expr_coverage(then_branch, counts, types, string_literals);
                self.hir_expr_coverage(else_branch, counts, types, string_literals);
            }
            HirExprKind::While { cond, body } => {
                self.hir_expr_coverage(cond, counts, types, string_literals);
                self.hir_expr_coverage(body, counts, types, string_literals);
            }
            HirExprKind::Match { scrutinee, arms } => {
                hir_match_scrutinee_coverage(self, scrutinee, counts, types, string_literals);
                for arm in arms {
                    self.push_scope();
                    if let Some(bind_local) = &arm.bind_local {
                        self.declare_local(bind_local);
                    }
                    self.hir_expr_coverage(&arm.body, counts, types, string_literals);
                    self.pop_scope();
                }
            }
            HirExprKind::EnumConstruct { payload, .. } => {
                counts.constructs += 1;
                if let Some(payload) = payload {
                    self.hir_expr_coverage(payload, counts, types, string_literals);
                }
            }
            HirExprKind::StructConstruct { fields, .. } => {
                counts.constructs += 1;
                for field in fields {
                    self.hir_expr_coverage(field, counts, types, string_literals);
                }
            }
            HirExprKind::TupleConstruct { items } => {
                counts.constructs += 1;
                for item in items {
                    self.hir_expr_coverage(item, counts, types, string_literals);
                }
            }
            HirExprKind::Block(block) => {
                hir_block_coverage(self, block, counts, types, string_literals);
            }
            HirExprKind::Let { name, value, .. } => {
                counts.declares += 1;
                self.hir_expr_coverage(value, counts, types, string_literals);
                self.declare_local(name);
            }
            HirExprKind::Set { value, .. } => {
                counts.assigns += 1;
                self.hir_expr_coverage(value, counts, types, string_literals);
            }
            HirExprKind::Intrinsic { name, args, .. } => {
                if let Some(owner) =
                    get_field_ref_intrinsic_owner(name, args, expr.ty, types, string_literals)
                {
                    counts.borrows += 1;
                    if !matches!(owner.kind, HirExprKind::AddrOf(_)) {
                        counts.deref_projections += 1;
                    }
                    hir_reference_owner_source_coverage(owner, counts, types, string_literals);
                    return;
                }
                if let Some(owner) =
                    get_field_intrinsic_owner(name, args, expr.ty, types, string_literals)
                {
                    counts.reads += 1;
                    hir_field_projection_source_coverage(owner, counts, types, string_literals);
                    return;
                }
                if intrinsic_projects_reference_field(name, args, expr.ty, types) {
                    counts.borrows += 1;
                    if let Some(owner) = args.first() {
                        if !matches!(owner.kind, HirExprKind::AddrOf(_)) {
                            counts.deref_projections += 1;
                        }
                        hir_reference_owner_source_coverage(owner, counts, types, string_literals);
                    }
                    for arg in args.iter().skip(1) {
                        self.hir_expr_coverage(arg, counts, types, string_literals);
                    }
                    return;
                }
                if let Some((base, _)) =
                    compiler_field_load_base_and_offset(name, args, expr.ty, types)
                {
                    counts.reads += 1;
                    hir_field_projection_source_coverage(base, counts, types, string_literals);
                    return;
                }
                if intrinsic_projects_reference_address(name, args, types) {
                    counts.deref_projections += 1;
                }
                if raw_memory_op_from_intrinsic(name)
                    .filter(|operation| should_count_raw_memory_call(operation, args, types))
                    .is_some()
                {
                    counts.raw_memory_ops += 1;
                }
                if let Some(primitive) = CollectionSlotBorrowPrimitive::from_intrinsic_name(name) {
                    hir_collection_slot_borrow_coverage(primitive, args, counts, types);
                }
                if let Some(primitive) = CollectionSlotLifecyclePrimitive::from_intrinsic_name(name)
                {
                    if primitive.requires_storage_pair() {
                        counts.collection_storage_relocates += 1;
                    } else {
                        counts.collection_slot_lifecycle_ops += 1;
                    }
                }
                for arg in args {
                    self.hir_expr_coverage(arg, counts, types, string_literals);
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
}

fn hir_collection_slot_borrow_coverage(
    primitive: CollectionSlotBorrowPrimitive,
    args: &[HirExpr],
    counts: &mut ResourceCoverageCounts,
    types: &TypeCtx,
) {
    match primitive {
        CollectionSlotBorrowPrimitive::BorrowRef => {
            counts.collection_slot_lifecycle_ops += 1;
            counts.borrows += 1;
            if args.first().is_some_and(|arg| {
                super::coverage_hir_projection::expr_requires_reference_deref_for_projection(
                    types, arg,
                )
            }) {
                counts.deref_projections += 1;
            }
            counts.deref_projections += 1;
        }
    }
}
