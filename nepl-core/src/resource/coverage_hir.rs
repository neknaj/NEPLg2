extern crate alloc;

use alloc::collections::BTreeSet;
use alloc::string::String;

use crate::hir::{FuncRef, HirBlock, HirBody, HirExpr, HirExprKind, HirFunction, HirModule};
use crate::resource_primitives::{type_is_owner_token, type_is_raw_pointer, MemoryHelperPrimitive};
use crate::runtime_helpers::helper_base_name;
use crate::types::{TypeCtx, TypeKind};

use super::coverage::ResourceCoverageCounts;
use super::coverage_hir_match::hir_match_scrutinee_coverage;
use super::coverage_hir_place::{
    hir_field_projection_source_coverage, hir_place_expr_coverage,
    hir_reference_owner_source_coverage,
};
use super::coverage_hir_projection::{
    callee_projects_reference_address, callee_projects_reference_field,
    compiler_field_load_base_and_offset, expr_requires_reference_deref_for_projection,
    get_field_intrinsic_owner, get_field_ref_intrinsic_owner, intrinsic_projects_reference_address,
    intrinsic_projects_reference_field,
};
use super::coverage_hir_raw::should_count_raw_memory_call;
use super::coverage_hir_scope::HirCoverageContext;
use super::lower_raw_memory::{raw_memory_op_from_callee, raw_memory_op_from_intrinsic};
use super::scalar_primitive::I32ArithmeticPrimitive;

const TRANSPARENT_RAW_ADDRESS_COVERAGE_DEPTH_LIMIT: usize = 8;

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
            HirExprKind::FnValue(_) => {
                counts.function_values += 1;
            }
            HirExprKind::Call { callee, args } => {
                if callee_projects_reference_field(callee, args, expr.ty, types) {
                    counts.borrows += 1;
                    counts.deref_projections += 1;
                } else if callee_projects_reference_address(callee, args, types) {
                    counts.deref_projections += 1;
                } else {
                    counts.deref_projections += self
                        .transparent_raw_address_return_deref_projection_count(
                            callee, args, expr.ty, types,
                        );
                }
                counts.direct_calls += 1;
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

    fn transparent_raw_address_return_deref_projection_count(
        &self,
        callee: &FuncRef,
        args: &[HirExpr],
        output_ty: crate::types::TypeId,
        types: &TypeCtx,
    ) -> usize {
        if !transparent_raw_address_output_can_carry_value(types, output_ty) {
            return 0;
        }
        let FuncRef::User(name, _, _) = callee else {
            return 0;
        };
        if MemoryHelperPrimitive::from_symbol(name)
            .is_some_and(MemoryHelperPrimitive::has_dedicated_raw_address_lowering)
        {
            return 0;
        }
        let Some(function) = self.function(name) else {
            return 0;
        };
        transparent_raw_address_reference_param_indices(function, self, types, 0)
            .iter()
            .filter_map(|index| args.get(*index))
            .filter(|arg| expr_requires_reference_deref_for_projection(types, arg))
            .count()
    }
}

fn transparent_raw_address_output_can_carry_value(
    types: &TypeCtx,
    ty: crate::types::TypeId,
) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    if matches!(types.get_ref(resolved), TypeKind::Reference(_, _)) {
        return false;
    }
    matches!(types.get_ref(resolved), TypeKind::I32 | TypeKind::Str)
        || type_is_raw_pointer(types, ty)
        || type_is_owner_token(types, ty)
}

fn transparent_raw_address_reference_param_indices(
    function: &HirFunction,
    context: &HirCoverageContext<'_>,
    types: &TypeCtx,
    depth: usize,
) -> BTreeSet<usize> {
    if depth >= TRANSPARENT_RAW_ADDRESS_COVERAGE_DEPTH_LIMIT {
        return BTreeSet::new();
    }
    let Some(expr) = function_return_expr(function) else {
        return BTreeSet::new();
    };
    transparent_raw_address_expr_reference_param_indices(expr, function, context, types, depth)
}

fn function_return_expr(function: &HirFunction) -> Option<&HirExpr> {
    let HirBody::Block(block) = &function.body else {
        return None;
    };
    block
        .lines
        .iter()
        .rev()
        .find(|line| !line.drop_result)
        .map(|line| &line.expr)
}

fn transparent_raw_address_expr_reference_param_indices(
    expr: &HirExpr,
    function: &HirFunction,
    context: &HirCoverageContext<'_>,
    types: &TypeCtx,
    depth: usize,
) -> BTreeSet<usize> {
    match &expr.kind {
        HirExprKind::Call { callee, args } => transparent_raw_address_call_reference_param_indices(
            callee, args, function, context, types, depth,
        ),
        HirExprKind::Intrinsic { name, args, .. } => {
            let callee = FuncRef::Builtin(helper_base_name(name).into());
            transparent_raw_address_call_reference_param_indices(
                &callee, args, function, context, types, depth,
            )
        }
        HirExprKind::Deref(inner) | HirExprKind::AddrOf(inner) => {
            transparent_raw_address_expr_reference_param_indices(
                inner, function, context, types, depth,
            )
        }
        HirExprKind::StructConstruct { fields, .. } => fields
            .iter()
            .flat_map(|field| {
                transparent_raw_address_expr_reference_param_indices(
                    field, function, context, types, depth,
                )
            })
            .collect(),
        _ => BTreeSet::new(),
    }
}

fn transparent_raw_address_call_reference_param_indices(
    callee: &FuncRef,
    args: &[HirExpr],
    function: &HirFunction,
    context: &HirCoverageContext<'_>,
    types: &TypeCtx,
    depth: usize,
) -> BTreeSet<usize> {
    let Some(name) = callee_base_name(callee) else {
        return BTreeSet::new();
    };
    if let Some(memory) = MemoryHelperPrimitive::from_base_name(name) {
        return transparent_memory_helper_reference_param_indices(
            memory, args, function, context, types, depth,
        );
    }
    if I32ArithmeticPrimitive::from_base_name(name).is_some() {
        return args
            .iter()
            .flat_map(|arg| {
                transparent_raw_address_expr_reference_param_indices(
                    arg, function, context, types, depth,
                )
            })
            .collect();
    }
    let FuncRef::User(name, _, _) = callee else {
        return BTreeSet::new();
    };
    let Some(callee_function) = context.function(name) else {
        return BTreeSet::new();
    };
    let projected =
        transparent_raw_address_reference_param_indices(callee_function, context, types, depth + 1);
    project_callee_reference_params_to_caller(function, args, &projected, types)
}

fn transparent_memory_helper_reference_param_indices(
    memory: MemoryHelperPrimitive,
    args: &[HirExpr],
    function: &HirFunction,
    context: &HirCoverageContext<'_>,
    types: &TypeCtx,
    depth: usize,
) -> BTreeSet<usize> {
    match memory {
        MemoryHelperPrimitive::RegionPtr
        | MemoryHelperPrimitive::RegionPtrAt
        | MemoryHelperPrimitive::RegionTokenRawRef => args
            .first()
            .and_then(|arg| reference_param_index(function, arg, types))
            .into_iter()
            .collect(),
        MemoryHelperPrimitive::MemPtrAddr
        | MemoryHelperPrimitive::MemPtrWrap
        | MemoryHelperPrimitive::MemPtrAdd
        | MemoryHelperPrimitive::RegionNew
        | MemoryHelperPrimitive::StrAddr
        | MemoryHelperPrimitive::StrFromAddrUnchecked => args
            .iter()
            .flat_map(|arg| {
                transparent_raw_address_expr_reference_param_indices(
                    arg, function, context, types, depth,
                )
            })
            .collect(),
    }
}

fn project_callee_reference_params_to_caller(
    caller: &HirFunction,
    args: &[HirExpr],
    projected: &BTreeSet<usize>,
    types: &TypeCtx,
) -> BTreeSet<usize> {
    projected
        .iter()
        .filter_map(|index| args.get(*index))
        .filter_map(|arg| reference_param_index(caller, arg, types))
        .collect()
}

fn reference_param_index(function: &HirFunction, expr: &HirExpr, types: &TypeCtx) -> Option<usize> {
    let HirExprKind::Var(name) = &expr.kind else {
        return None;
    };
    let index = function
        .params
        .iter()
        .position(|param| param.name == *name)?;
    type_is_reference(types, function.params[index].ty).then_some(index)
}

fn type_is_reference(types: &TypeCtx, ty: crate::types::TypeId) -> bool {
    matches!(
        types.get_ref(types.resolve_named_type_id(types.resolve_id(ty))),
        TypeKind::Reference(_, _)
    )
}

fn callee_base_name(callee: &FuncRef) -> Option<&str> {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Some(helper_base_name(name)),
        FuncRef::Trait { .. } => None,
    }
}
