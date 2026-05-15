extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::{FuncRef, HirBody, HirExpr, HirExprKind, HirFunction};
use crate::runtime_helpers::helper_base_name;
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::lower::LoweringEnvironment;
use super::lower_call::func_ref_base_name;
use super::lower_raw_address::{i32_const_from_actual_arg, i32_const_from_size_of_call};
use super::lower_raw_address_place::{
    is_named_struct_type, raw_address_alias_target, raw_address_place_from_actual_argument,
    region_token_place_from_actual_arg, region_token_raw_field_place,
};
use super::lower_raw_address_source::{push_raw_address_op, RawAddressOffset, RawAddressSource};
use super::model::{Place, ResourceOp};

pub(super) fn push_transparent_raw_address_return_projection(
    callee: &FuncRef,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    output: &Place,
    ops: &mut Vec<ResourceOp>,
    env: &LoweringEnvironment,
    span: Span,
) {
    let FuncRef::User(name, _, _) = callee else {
        return;
    };
    if function_has_dedicated_raw_address_lowering(name) {
        return;
    }
    let Some(function) = env.function(name) else {
        return;
    };
    if function.params.len() != hir_args.len() || hir_args.len() != arg_places.len() {
        return;
    }
    if !raw_address_output_can_carry_value(env, output.ty) {
        return;
    }
    let Some(return_expr) = function_return_expr(function) else {
        return;
    };
    let Some(source) = raw_address_source_from_return_expr(
        return_expr,
        function,
        hir_args,
        arg_places,
        env,
        RawAddressReturnContext::DirectReturn,
    ) else {
        return;
    };
    let source = source.into_place_and_view(env.types.i32());
    push_raw_address_op(
        source.place,
        raw_address_alias_target(output, env),
        source.view_kind,
        ops,
        span,
    );
}

fn function_has_dedicated_raw_address_lowering(name: &str) -> bool {
    matches!(helper_base_name(name), "mem_ptr_addr" | "region_ptr")
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

fn raw_address_source_from_return_expr(
    expr: &HirExpr,
    function: &HirFunction,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
    context: RawAddressReturnContext,
) -> Option<RawAddressSource> {
    match &expr.kind {
        HirExprKind::Var(name) => {
            let index = function_param_index(function, name)?;
            let hir_arg = hir_args.get(index)?;
            let arg_place = arg_places.get(index)?;
            let base = raw_address_place_from_actual_argument(hir_arg, arg_place, env);
            if matches!(context, RawAddressReturnContext::DirectReturn) && base == *arg_place {
                return None;
            }
            Some(RawAddressSource {
                base,
                offset: RawAddressOffset::Known(0),
                explicit_offset: false,
                non_owning_view: false,
            })
        }
        HirExprKind::Call { callee, args } => raw_address_source_from_return_named_call(
            func_ref_base_name(callee)?,
            args,
            expr.ty,
            function,
            hir_args,
            arg_places,
            env,
        ),
        HirExprKind::Intrinsic { name, args, .. } => raw_address_source_from_return_named_call(
            helper_base_name(name),
            args,
            expr.ty,
            function,
            hir_args,
            arg_places,
            env,
        ),
        HirExprKind::StructConstruct { name, fields, .. } if name == "MemPtr" => {
            raw_address_source_from_return_expr(
                fields.first()?,
                function,
                hir_args,
                arg_places,
                env,
                RawAddressReturnContext::AddressOperand,
            )
        }
        HirExprKind::Deref(inner) => {
            raw_address_source_from_return_expr(inner, function, hir_args, arg_places, env, context)
        }
        _ => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RawAddressReturnContext {
    DirectReturn,
    AddressOperand,
}

fn raw_address_source_from_return_operand_expr(
    expr: &HirExpr,
    function: &HirFunction,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    raw_address_source_from_return_expr(
        expr,
        function,
        hir_args,
        arg_places,
        env,
        RawAddressReturnContext::AddressOperand,
    )
}

fn raw_address_source_from_return_named_call(
    name: &str,
    args: &[HirExpr],
    return_ty: TypeId,
    function: &HirFunction,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    match helper_base_name(name) {
        "add" if args.len() == 2 => raw_address_source_from_return_operand_expr(
            &args[0], function, hir_args, arg_places, env,
        )
        .map(|source| {
            source.with_added_offset(raw_address_offset_from_return_expr(
                &args[1], function, hir_args, arg_places, env,
            ))
        })
        .or_else(|| {
            let offset = i32_const_from_return_expr(&args[0], function, hir_args, env)?;
            raw_address_source_from_return_operand_expr(
                &args[1], function, hir_args, arg_places, env,
            )
            .map(|source| source.with_added_offset(RawAddressOffset::Known(offset)))
        }),
        "sub" if args.len() == 2 => raw_address_source_from_return_operand_expr(
            &args[0], function, hir_args, arg_places, env,
        )
        .map(|source| {
            source.with_subtracted_offset(raw_address_offset_from_return_expr(
                &args[1], function, hir_args, arg_places, env,
            ))
        }),
        "mem_ptr_addr" if args.len() == 1 => raw_address_source_from_return_operand_expr(
            &args[0], function, hir_args, arg_places, env,
        )
        .map(RawAddressSource::into_non_owning_view),
        "mem_ptr_wrap" | "str_from_addr_unchecked" if args.len() == 1 => {
            raw_address_source_from_return_operand_expr(
                &args[0], function, hir_args, arg_places, env,
            )
        }
        "str_addr" if args.len() == 1 => raw_address_source_from_return_operand_expr(
            &args[0], function, hir_args, arg_places, env,
        )
        .map(RawAddressSource::into_non_owning_view),
        "mem_ptr_add" if args.len() >= 2 => raw_address_source_from_return_operand_expr(
            &args[0], function, hir_args, arg_places, env,
        )
        .map(|source| {
            source.with_added_offset(raw_address_offset_from_return_expr(
                &args[1], function, hir_args, arg_places, env,
            ))
        }),
        "get" | "get_field"
            if args.len() >= 2
                && literal_field_name(env, &args[1]) == Some("raw")
                && is_named_struct_type(env.types, args[0].ty, "MemPtr") =>
        {
            raw_address_source_from_return_operand_expr(
                &args[0], function, hir_args, arg_places, env,
            )
        }
        "get" | "get_field"
            if args.len() >= 2
                && is_named_struct_type(env.types, args[0].ty, "RegionToken")
                && matches!(
                    env.types.get_ref(
                        env.types
                            .resolve_named_type_id(env.types.resolve_id(return_ty))
                    ),
                    TypeKind::I32
                )
                && literal_field_name(env, &args[1])
                    .is_none_or(|field_name| field_name == "raw") =>
        {
            raw_address_source_from_region_token_raw_expr(
                &args[0], function, hir_args, arg_places, env,
            )
        }
        "region_new" if args.len() >= 2 => raw_address_source_from_return_operand_expr(
            &args[0], function, hir_args, arg_places, env,
        ),
        "region_token_raw_ref" if args.len() == 1 => raw_address_source_from_region_token_raw_expr(
            &args[0], function, hir_args, arg_places, env,
        ),
        _ => None,
    }
}

fn raw_address_offset_from_return_expr(
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

fn i32_const_from_return_expr(
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

fn raw_address_source_from_region_token_raw_expr(
    expr: &HirExpr,
    function: &HirFunction,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    match &expr.kind {
        HirExprKind::Var(name) => {
            let index = function_param_index(function, name)?;
            let token = region_token_place_from_actual_arg(
                hir_args.get(index)?,
                arg_places.get(index)?,
                env,
            )?;
            Some(RawAddressSource {
                base: region_token_raw_field_place(&token, env.types.i32()),
                offset: RawAddressOffset::Known(0),
                explicit_offset: false,
                non_owning_view: false,
            })
        }
        HirExprKind::Call { callee, args }
            if matches!(func_ref_base_name(callee), Some("region_new")) =>
        {
            raw_address_source_from_return_operand_expr(
                args.first()?,
                function,
                hir_args,
                arg_places,
                env,
            )
        }
        HirExprKind::Intrinsic { name, args, .. } if helper_base_name(name) == "region_new" => {
            raw_address_source_from_return_operand_expr(
                args.first()?,
                function,
                hir_args,
                arg_places,
                env,
            )
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

fn function_param_index(function: &HirFunction, name: &str) -> Option<usize> {
    function.params.iter().position(|param| param.name == name)
}

fn raw_address_output_can_carry_value(env: &LoweringEnvironment, ty: TypeId) -> bool {
    let resolved = env.types.resolve_named_type_id(env.types.resolve_id(ty));
    if matches!(env.types.get_ref(resolved), TypeKind::Reference(_, _)) {
        return false;
    }
    matches!(env.types.get_ref(resolved), TypeKind::I32 | TypeKind::Str)
        || is_named_struct_type(env.types, ty, "MemPtr")
        || is_named_struct_type(env.types, ty, "RegionToken")
}

fn literal_field_name<'a>(env: &'a LoweringEnvironment, expr: &HirExpr) -> Option<&'a str> {
    match &expr.kind {
        HirExprKind::LiteralStr(index) => {
            env.string_literals.get(*index as usize).map(String::as_str)
        }
        _ => None,
    }
}
