use alloc::vec::Vec;

use crate::hir::{FuncRef, HirBody, HirExpr, HirExprKind};
use crate::layout::storage_size_bytes;
use crate::runtime_helpers::helper_base_name;
use crate::span::Span;

use super::lower::LoweringEnvironment;
use super::lower_raw_address_place::{
    mem_ptr_raw_field_place, raw_address_alias_target, raw_address_place_from_actual_argument,
    region_token_place_from_actual_arg, region_token_raw_field_place,
};
pub(super) use super::lower_raw_address_return::push_transparent_raw_address_return_projection;
use super::lower_raw_address_source::{push_raw_address_op, RawAddressOffset, RawAddressSource};
use super::model::{Place, PlaceProjection, ResourceOffset, ResourceOp};

pub(super) fn push_core_mem_wrapper_semantics(
    callee: &FuncRef,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    output: &Place,
    ops: &mut Vec<ResourceOp>,
    env: &LoweringEnvironment,
    span: Span,
) {
    match func_ref_base_name(callee) {
        Some("mem_ptr_wrap") => {
            let Some(raw) = arg_places.first() else {
                return;
            };
            ops.push(ResourceOp::RawAddressAlias {
                source: raw.clone(),
                target: mem_ptr_raw_field_place(output, env.types.i32()),
                span,
            });
        }
        Some("mem_ptr_addr") => {
            let Some(ptr) = arg_places.first() else {
                return;
            };
            ops.push(ResourceOp::RawAddressAlias {
                source: mem_ptr_raw_field_place(ptr, output.ty),
                target: output.clone(),
                span,
            });
        }
        Some("mem_ptr_add") => {
            let Some(ptr) = arg_places.first() else {
                return;
            };
            let mut raw = mem_ptr_raw_field_place(ptr, env.types.i32());
            let is_view = true;
            match hir_args.get(1).and_then(non_negative_i32_literal_bytes) {
                Some(0) => {}
                Some(bytes) => {
                    raw = raw.with_projection(
                        PlaceProjection::StorageOffset(ResourceOffset { bytes: Some(bytes) }),
                        env.types.i32(),
                    );
                }
                None => {
                    raw = raw.with_projection(
                        PlaceProjection::StorageOffset(ResourceOffset { bytes: None }),
                        env.types.i32(),
                    );
                }
            }
            push_raw_address_op(
                raw,
                mem_ptr_raw_field_place(output, env.types.i32()),
                is_view,
                ops,
                span,
            );
        }
        Some("region_new") => {
            let Some(ptr) = arg_places.first() else {
                return;
            };
            ops.push(ResourceOp::RawAddressAlias {
                source: mem_ptr_raw_field_place(ptr, env.types.i32()),
                target: region_token_raw_field_place(output, env.types.i32()),
                span,
            });
        }
        Some("region_ptr") => {
            let Some(source) =
                region_token_raw_source_from_actual_arg(0, hir_args, arg_places, env)
            else {
                return;
            };
            let source = source.into_place_and_view(env.types.i32());
            push_raw_address_op(
                source.place,
                mem_ptr_raw_field_place(output, env.types.i32()),
                true,
                ops,
                span,
            );
        }
        _ => {}
    }
}

pub(super) fn push_named_raw_address_semantics(
    name: &str,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    output: &Place,
    ops: &mut Vec<ResourceOp>,
    env: &LoweringEnvironment,
    span: Span,
) {
    let Some(source) = raw_address_source_from_actual_named_expr(name, hir_args, arg_places, env)
    else {
        return;
    };
    let source = source.into_place_and_view(env.types.i32());
    push_raw_address_op(
        source.place,
        raw_address_alias_target(output, env),
        source.is_view,
        ops,
        span,
    );
}

pub(super) fn i32_const_from_actual_arg(expr: &HirExpr, env: &LoweringEnvironment) -> Option<i64> {
    match &expr.kind {
        HirExprKind::LiteralI32(value) => Some(i64::from(*value)),
        HirExprKind::Call { callee, args } => {
            i32_const_from_actual_named_expr(func_ref_base_name(callee)?, args, env)
                .or_else(|| i32_const_from_size_of_call(callee, env))
        }
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => {
            if helper_base_name(name) == "size_of" && type_args.len() == 1 {
                i64::try_from(storage_size_bytes(env.types, type_args[0])).ok()
            } else {
                i32_const_from_actual_named_expr(helper_base_name(name), args, env)
            }
        }
        _ => None,
    }
}

fn raw_address_source_from_actual_named_expr(
    name: &str,
    args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    match helper_base_name(name) {
        "add" if args.len() == 2 && arg_places.len() == 2 => {
            if i32_const_from_actual_arg(&args[0], env).is_some()
                && i32_const_from_actual_arg(&args[1], env).is_none()
            {
                raw_address_source_from_actual_arg(1, args, arg_places, env).map(|source| {
                    source.with_added_offset(i32_const_from_actual_arg(&args[0], env))
                })
            } else {
                raw_address_source_from_actual_arg(0, args, arg_places, env).map(|source| {
                    source.with_added_offset(i32_const_from_actual_arg(&args[1], env))
                })
            }
        }
        "sub" if args.len() == 2 && arg_places.len() == 2 => {
            raw_address_source_from_actual_arg(0, args, arg_places, env).map(|source| {
                source.with_subtracted_offset(i32_const_from_actual_arg(&args[1], env))
            })
        }
        "mem_ptr_addr" | "mem_ptr_wrap" | "str_addr"
            if args.len() == 1 && arg_places.len() == 1 =>
        {
            raw_address_source_from_actual_arg(0, args, arg_places, env)
        }
        "mem_ptr_add" if args.len() >= 2 && arg_places.len() >= 2 => {
            raw_address_source_from_actual_arg(0, args, arg_places, env)
                .map(|source| source.with_added_offset(i32_const_from_actual_arg(&args[1], env)))
        }
        "region_new" if args.len() >= 2 && !arg_places.is_empty() => {
            raw_address_source_from_actual_arg(0, args, arg_places, env)
        }
        _ => None,
    }
}

fn raw_address_source_from_actual_arg(
    index: usize,
    args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    Some(RawAddressSource {
        base: raw_address_place_from_actual_argument(args.get(index)?, arg_places.get(index)?, env),
        offset: RawAddressOffset::Known(0),
        explicit_offset: false,
    })
}

fn region_token_raw_source_from_actual_arg(
    index: usize,
    args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    let token = region_token_place_from_actual_arg(args.get(index)?, arg_places.get(index)?, env)?;
    Some(RawAddressSource {
        base: region_token_raw_field_place(&token, env.types.i32()),
        offset: RawAddressOffset::Known(0),
        explicit_offset: false,
    })
}

fn i32_const_from_actual_named_expr(
    name: &str,
    args: &[HirExpr],
    env: &LoweringEnvironment,
) -> Option<i64> {
    if args.len() != 2 {
        return None;
    }
    match name {
        "add" => i32_const_from_actual_arg(&args[0], env)?
            .checked_add(i32_const_from_actual_arg(&args[1], env)?),
        "sub" => i32_const_from_actual_arg(&args[0], env)?
            .checked_sub(i32_const_from_actual_arg(&args[1], env)?),
        "mul" => {
            let left = i32_const_from_actual_arg(&args[0], env);
            if matches!(left, Some(0)) {
                return Some(0);
            }
            let right = i32_const_from_actual_arg(&args[1], env);
            if matches!(right, Some(0)) {
                return Some(0);
            }
            left?.checked_mul(right?)
        }
        _ => None,
    }
}

pub(super) fn i32_const_from_size_of_call(
    callee: &FuncRef,
    env: &LoweringEnvironment,
) -> Option<i64> {
    match callee {
        FuncRef::User(name, type_args, _)
            if helper_base_name(name) == "size_of" && type_args.len() == 1 =>
        {
            i64::try_from(storage_size_bytes(env.types, type_args[0])).ok()
        }
        FuncRef::User(name, _, _) if helper_base_name(name) == "size_of" => {
            let function = env.function(name)?;
            let HirBody::Block(block) = &function.body else {
                return None;
            };
            if block.lines.len() != 1 {
                return None;
            }
            let HirExprKind::Intrinsic {
                name, type_args, ..
            } = &block.lines[0].expr.kind
            else {
                return None;
            };
            if helper_base_name(name) == "size_of" && type_args.len() == 1 {
                i64::try_from(storage_size_bytes(env.types, type_args[0])).ok()
            } else {
                None
            }
        }
        _ => None,
    }
}

fn non_negative_i32_literal_bytes(expr: &HirExpr) -> Option<usize> {
    match &expr.kind {
        HirExprKind::LiteralI32(value) if *value >= 0 => Some(*value as usize),
        _ => None,
    }
}

pub(super) fn func_ref_base_name(callee: &FuncRef) -> Option<&str> {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => Some(helper_base_name(name)),
        FuncRef::Trait { .. } => None,
    }
}
