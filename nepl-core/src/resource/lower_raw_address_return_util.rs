use alloc::string::String;

use crate::hir::{HirExpr, HirExprKind, HirFunction};
use crate::runtime_helpers::helper_base_name;

use super::lower::LoweringEnvironment;
use super::lower_call::func_ref_base_name;
use super::lower_raw_address::{i32_const_from_actual_arg, i32_const_from_size_of_call};
use super::lower_raw_address_source::RawAddressOffset;
use super::model::Place;

pub(super) fn raw_address_offset_from_return_expr(
    expr: &HirExpr,
    function: &HirFunction,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> RawAddressOffset {
    if let Some(value) = i32_const_from_return_expr(expr, function, hir_args, env) {
        return RawAddressOffset::Known(value);
    }
    match &expr.kind {
        HirExprKind::Var(name) => {
            let Some(index) = function_param_index(function, name) else {
                return RawAddressOffset::Unknown;
            };
            arg_places
                .get(index)
                .map(RawAddressOffset::symbolic)
                .unwrap_or(RawAddressOffset::Unknown)
        }
        _ => RawAddressOffset::Unknown,
    }
}

pub(super) fn function_param_index(function: &HirFunction, name: &str) -> Option<usize> {
    function.params.iter().position(|param| param.name == name)
}

pub(super) fn literal_field_name<'a>(
    env: &'a LoweringEnvironment,
    expr: &HirExpr,
) -> Option<&'a str> {
    match &expr.kind {
        HirExprKind::LiteralStr(index) => {
            env.string_literals.get(*index as usize).map(String::as_str)
        }
        _ => None,
    }
}

pub(super) fn i32_const_from_return_expr(
    expr: &HirExpr,
    function: &HirFunction,
    hir_args: &[HirExpr],
    env: &LoweringEnvironment,
) -> Option<i64> {
    match &expr.kind {
        HirExprKind::LiteralI32(value) => Some(i64::from(*value)),
        HirExprKind::Var(name) => {
            let index = function_param_index(function, name)?;
            i32_const_from_actual_arg(hir_args.get(index)?, env)
        }
        HirExprKind::Call { callee, args } => i32_const_from_return_named_expr(
            func_ref_base_name(callee)?,
            args,
            function,
            hir_args,
            env,
        )
        .or_else(|| i32_const_from_size_of_call(callee, env)),
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => {
            if helper_base_name(name) == "size_of" && type_args.len() == 1 {
                i64::try_from(crate::layout::storage_size_bytes(env.types, type_args[0])).ok()
            } else {
                i32_const_from_return_named_expr(
                    helper_base_name(name),
                    args,
                    function,
                    hir_args,
                    env,
                )
            }
        }
        _ => None,
    }
}

fn i32_const_from_return_named_expr(
    name: &str,
    args: &[HirExpr],
    function: &HirFunction,
    hir_args: &[HirExpr],
    env: &LoweringEnvironment,
) -> Option<i64> {
    if args.len() != 2 {
        return None;
    }
    match helper_base_name(name) {
        "add" => i32_const_from_return_expr(&args[0], function, hir_args, env)?.checked_add(
            i32_const_from_return_expr(&args[1], function, hir_args, env)?,
        ),
        "sub" => i32_const_from_return_expr(&args[0], function, hir_args, env)?.checked_sub(
            i32_const_from_return_expr(&args[1], function, hir_args, env)?,
        ),
        "mul" => {
            let left = i32_const_from_return_expr(&args[0], function, hir_args, env);
            if matches!(left, Some(0)) {
                return Some(0);
            }
            let right = i32_const_from_return_expr(&args[1], function, hir_args, env);
            if matches!(right, Some(0)) {
                return Some(0);
            }
            left?.checked_mul(right?)
        }
        _ => None,
    }
}
