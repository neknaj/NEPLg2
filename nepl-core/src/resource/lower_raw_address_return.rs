extern crate alloc;

use alloc::vec::Vec;

use crate::hir::{FuncRef, HirBody, HirExpr, HirExprKind, HirFunction};
use crate::intrinsic_kinds::FieldAccessorKind;
use crate::resource_primitives::{
    type_is_owner_token, type_is_raw_pointer, CompilerMemoryFieldSpec, MemoryHelperPrimitive,
};
use crate::runtime_helpers::helper_base_name;
use crate::span::Span;
use crate::types::{TypeId, TypeKind};

use super::compiler_memory_place::region_token_raw_field_place;
use super::lower::LoweringEnvironment;
use super::lower_call::func_ref_base_name;
use super::lower_raw_address_place::{
    raw_address_alias_target, raw_address_place_from_actual_argument,
    region_token_place_from_actual_arg,
};
use super::lower_raw_address_return_util::{
    function_param_index, i32_const_from_return_expr, literal_field_name,
    raw_address_offset_from_return_expr,
};
use super::lower_raw_address_source::{push_raw_address_op, RawAddressOffset, RawAddressSource};
use super::model::{Place, ResourceOp};
use super::scalar_primitive::I32ArithmeticPrimitive;

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
    MemoryHelperPrimitive::from_symbol(name)
        .is_some_and(MemoryHelperPrimitive::has_dedicated_raw_address_lowering)
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
        HirExprKind::StructConstruct { fields, .. } if type_is_raw_pointer(env.types, expr.ty) => {
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
    match MemoryHelperPrimitive::from_symbol(name) {
        Some(MemoryHelperPrimitive::MemPtrAddr) if args.len() == 1 => {
            return raw_address_source_from_return_operand_expr(
                &args[0], function, hir_args, arg_places, env,
            )
            .map(RawAddressSource::into_non_owning_view);
        }
        Some(MemoryHelperPrimitive::MemPtrWrap | MemoryHelperPrimitive::StrFromAddrUnchecked)
            if args.len() == 1 =>
        {
            return raw_address_source_from_return_operand_expr(
                &args[0], function, hir_args, arg_places, env,
            );
        }
        Some(MemoryHelperPrimitive::StrAddr) if args.len() == 1 => {
            return raw_address_source_from_return_operand_expr(
                &args[0], function, hir_args, arg_places, env,
            )
            .map(RawAddressSource::into_non_owning_view);
        }
        Some(MemoryHelperPrimitive::MemPtrAdd) if args.len() >= 2 => {
            return raw_address_source_from_return_operand_expr(
                &args[0], function, hir_args, arg_places, env,
            )
            .map(|source| {
                source.with_added_offset(raw_address_offset_from_return_expr(
                    &args[1], function, hir_args, arg_places, env,
                ))
            });
        }
        Some(MemoryHelperPrimitive::RegionNew) if args.len() >= 2 => {
            return raw_address_source_from_return_operand_expr(
                &args[0], function, hir_args, arg_places, env,
            );
        }
        Some(MemoryHelperPrimitive::RegionTokenRawRef) if args.len() == 1 => {
            return raw_address_source_from_region_token_raw_expr(
                &args[0], function, hir_args, arg_places, env,
            );
        }
        _ => {}
    }
    if let Some(source) = raw_address_source_from_return_arithmetic_expr(
        name, args, function, hir_args, arg_places, env,
    ) {
        return Some(source);
    }
    let field_accessor = FieldAccessorKind::from_call_base_name(helper_base_name(name));
    if field_accessor == Some(FieldAccessorKind::Get)
        && args.len() >= 2
        && literal_field_name(env, &args[1]) == Some(CompilerMemoryFieldSpec::RawI32.name())
        && type_is_raw_pointer(env.types, args[0].ty)
    {
        return raw_address_source_from_return_operand_expr(
            &args[0], function, hir_args, arg_places, env,
        );
    }
    if field_accessor == Some(FieldAccessorKind::Get)
        && args.len() >= 2
        && type_is_owner_token(env.types, args[0].ty)
        && matches!(
            env.types.get_ref(
                env.types
                    .resolve_named_type_id(env.types.resolve_id(return_ty))
            ),
            TypeKind::I32
        )
        && literal_field_name(env, &args[1])
            .is_none_or(|field_name| field_name == CompilerMemoryFieldSpec::RawI32.name())
    {
        return raw_address_source_from_region_token_raw_expr(
            &args[0], function, hir_args, arg_places, env,
        );
    }
    None
}

fn raw_address_source_from_return_arithmetic_expr(
    name: &str,
    args: &[HirExpr],
    function: &HirFunction,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    match I32ArithmeticPrimitive::from_symbol(name) {
        Some(I32ArithmeticPrimitive::Add) if args.len() == 2 => {
            raw_address_source_from_return_operand_expr(
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
            })
        }
        Some(I32ArithmeticPrimitive::Sub) if args.len() == 2 => {
            raw_address_source_from_return_operand_expr(
                &args[0], function, hir_args, arg_places, env,
            )
            .map(|source| {
                source.with_subtracted_offset(raw_address_offset_from_return_expr(
                    &args[1], function, hir_args, arg_places, env,
                ))
            })
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
                base: region_token_raw_field_place(env.types, &token, env.types.i32()),
                offset: RawAddressOffset::Known(0),
                explicit_offset: false,
                non_owning_view: false,
            })
        }
        HirExprKind::Call { callee, args }
            if func_ref_base_name(callee).and_then(MemoryHelperPrimitive::from_base_name)
                == Some(MemoryHelperPrimitive::RegionNew) =>
        {
            raw_address_source_from_return_operand_expr(
                args.first()?,
                function,
                hir_args,
                arg_places,
                env,
            )
        }
        HirExprKind::Intrinsic { name, args, .. }
            if MemoryHelperPrimitive::from_symbol(name)
                == Some(MemoryHelperPrimitive::RegionNew) =>
        {
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

fn raw_address_output_can_carry_value(env: &LoweringEnvironment, ty: TypeId) -> bool {
    let resolved = env.types.resolve_named_type_id(env.types.resolve_id(ty));
    if matches!(env.types.get_ref(resolved), TypeKind::Reference(_, _)) {
        return false;
    }
    matches!(env.types.get_ref(resolved), TypeKind::I32 | TypeKind::Str)
        || type_is_raw_pointer(env.types, ty)
        || type_is_owner_token(env.types, ty)
}
