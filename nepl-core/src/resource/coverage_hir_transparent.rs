extern crate alloc;

use alloc::collections::BTreeSet;

use crate::hir::{FuncRef, HirBody, HirExpr, HirExprKind, HirFunction};
use crate::resource_primitives::{type_is_owner_token, type_is_raw_pointer, MemoryHelperPrimitive};
use crate::runtime_helpers::helper_base_name;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::coverage_hir_projection::expr_requires_reference_deref_for_projection;
use super::coverage_hir_scope::HirCoverageContext;
use super::scalar_primitive::I32ArithmeticPrimitive;

const TRANSPARENT_RAW_ADDRESS_COVERAGE_DEPTH_LIMIT: usize = 8;

pub(super) fn transparent_raw_address_return_deref_projection_count(
    context: &HirCoverageContext<'_>,
    callee: &FuncRef,
    args: &[HirExpr],
    output_ty: TypeId,
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
    let Some(function) = context.function(name) else {
        return 0;
    };
    transparent_raw_address_reference_param_indices(function, context, types, 0)
        .iter()
        .filter_map(|index| args.get(*index))
        .filter(|arg| expr_requires_reference_deref_for_projection(types, arg))
        .count()
}

fn transparent_raw_address_output_can_carry_value(types: &TypeCtx, ty: TypeId) -> bool {
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

fn type_is_reference(types: &TypeCtx, ty: TypeId) -> bool {
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
