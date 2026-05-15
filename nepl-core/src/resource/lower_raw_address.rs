use alloc::string::String;
use alloc::vec::Vec;

use crate::hir::{FuncRef, HirBody, HirExpr, HirExprKind};
use crate::layout::storage_size_bytes;
use crate::runtime_helpers::helper_base_name;
use crate::span::Span;
use crate::types::TypeId;

use super::lower::LoweringEnvironment;
use super::lower_call::func_ref_base_name;
use super::lower_raw_address_place::{
    raw_address_alias_target, raw_address_place_from_actual_argument, reference_target_type,
    region_token_place_from_actual_arg, region_token_raw_field_place,
};
pub(super) use super::lower_raw_address_return::push_transparent_raw_address_return_projection;
use super::lower_raw_address_source::{push_raw_address_op, RawAddressOffset, RawAddressSource};
use super::model::{
    Place, PlaceProjection, RawAddressViewKind, ResourceOffset, ResourceOp, StorageOrigin,
};
use super::place_utils::{enum_payload_type, mem_ptr_raw_field_place, reference_target_place};

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
            push_raw_address_op(
                mem_ptr_raw_field_place(ptr, output.ty),
                output.clone(),
                Some(RawAddressViewKind::NonOwningProjection),
                ops,
                span,
            );
        }
        Some("mem_ptr_add") => {
            let Some(ptr) = arg_places.first() else {
                return;
            };
            let mut raw = mem_ptr_raw_field_place(ptr, env.types.i32());
            let view_kind = Some(RawAddressViewKind::MemPtrOffset);
            raw = raw_address_place_with_offset(
                raw,
                raw_address_offset_from_actual_arg(1, hir_args, arg_places, env),
                env.types.i32(),
            );
            push_raw_address_op(
                raw,
                mem_ptr_raw_field_place(output, env.types.i32()),
                view_kind,
                ops,
                span,
            );
        }
        Some("region_new") => {
            let Some(ptr) = arg_places.first() else {
                return;
            };
            let target = region_token_raw_field_place(output, env.types.i32());
            ops.push(ResourceOp::RawAddressAlias {
                source: mem_ptr_raw_field_place(ptr, env.types.i32()),
                target,
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
                Some(RawAddressViewKind::NonOwningProjection),
                ops,
                span,
            );
        }
        Some("region_ptr_at") => {
            let Some(source) =
                region_token_raw_source_from_actual_arg(0, hir_args, arg_places, env)
            else {
                return;
            };
            let Some(payload_ty) = enum_payload_type(env.types, output.ty, "Ok") else {
                return;
            };
            let source = source
                .with_added_offset(raw_address_offset_from_actual_arg(
                    1, hir_args, arg_places, env,
                ))
                .into_non_owning_view()
                .into_place_and_view(env.types.i32());
            let ok_payload = output.clone().with_projection(
                PlaceProjection::EnumPayload {
                    variant: String::from("Ok"),
                },
                payload_ty,
            );
            push_raw_address_op(
                source.place,
                mem_ptr_raw_field_place(&ok_payload, env.types.i32()),
                source.view_kind,
                ops,
                span,
            );
        }
        Some("region_token_raw_ref") => {
            let Some(source) =
                region_token_raw_source_from_actual_arg(0, hir_args, arg_places, env)
            else {
                return;
            };
            let Some(target_ty) = reference_target_type(env.types, output.ty) else {
                return;
            };
            push_raw_address_op(
                source.base,
                reference_target_place(output, target_ty),
                Some(RawAddressViewKind::NonOwningProjection),
                ops,
                span,
            );
        }
        _ => {}
    }
}

pub(super) fn push_core_mem_owner_storage_origin(
    callee: &FuncRef,
    output: &Place,
    ops: &mut Vec<ResourceOp>,
    env: &LoweringEnvironment,
    span: Span,
) {
    if !matches!(func_ref_base_name(callee), Some("region_new")) {
        return;
    }
    ops.push(ResourceOp::StorageOrigin {
        target: region_token_raw_field_place(output, env.types.i32()),
        origin: StorageOrigin::Owned,
        span,
    });
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
        source.view_kind,
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
                    source.with_added_offset(raw_address_offset_from_actual_arg(
                        0, args, arg_places, env,
                    ))
                })
            } else {
                raw_address_source_from_actual_arg(0, args, arg_places, env).map(|source| {
                    source.with_added_offset(raw_address_offset_from_actual_arg(
                        1, args, arg_places, env,
                    ))
                })
            }
        }
        "sub" if args.len() == 2 && arg_places.len() == 2 => {
            raw_address_source_from_actual_arg(0, args, arg_places, env).map(|source| {
                source.with_subtracted_offset(raw_address_offset_from_actual_arg(
                    1, args, arg_places, env,
                ))
            })
        }
        "mem_ptr_addr" if args.len() == 1 && arg_places.len() == 1 => {
            raw_address_source_from_actual_arg(0, args, arg_places, env)
                .map(RawAddressSource::into_non_owning_view)
        }
        "mem_ptr_wrap" if args.len() == 1 && arg_places.len() == 1 => {
            raw_address_source_from_actual_arg(0, args, arg_places, env)
        }
        "str_addr" if args.len() == 1 && arg_places.len() == 1 => {
            raw_address_source_from_actual_arg(0, args, arg_places, env)
                .map(RawAddressSource::into_non_owning_view)
        }
        "mem_ptr_add" if args.len() >= 2 && arg_places.len() >= 2 => {
            raw_address_source_from_actual_arg(0, args, arg_places, env).map(|source| {
                source
                    .with_added_offset(raw_address_offset_from_actual_arg(1, args, arg_places, env))
            })
        }
        "region_new" if args.len() >= 2 && !arg_places.is_empty() => {
            raw_address_source_from_actual_arg(0, args, arg_places, env)
        }
        _ => None,
    }
}

fn raw_address_offset_from_actual_arg(
    index: usize,
    args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> RawAddressOffset {
    if let Some(value) = args
        .get(index)
        .and_then(|arg| i32_const_from_actual_arg(arg, env))
    {
        return RawAddressOffset::Known(value);
    }
    arg_places
        .get(index)
        .map(RawAddressOffset::symbolic)
        .unwrap_or(RawAddressOffset::Unknown)
}

fn raw_address_place_with_offset(raw: Place, offset: RawAddressOffset, raw_ty: TypeId) -> Place {
    match offset {
        RawAddressOffset::Known(0) => raw,
        RawAddressOffset::Known(bytes) if bytes > 0 => match usize::try_from(bytes) {
            Ok(bytes) => raw.with_projection(
                PlaceProjection::StorageOffset(ResourceOffset::Known(bytes)),
                raw_ty,
            ),
            Err(_) => raw.with_projection(
                PlaceProjection::StorageOffset(ResourceOffset::Unknown),
                raw_ty,
            ),
        },
        RawAddressOffset::Symbolic { place } => raw.with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place }),
            raw_ty,
        ),
        RawAddressOffset::Known(_) | RawAddressOffset::Unknown => raw.with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::Unknown),
            raw_ty,
        ),
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
        non_owning_view: false,
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
        non_owning_view: false,
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
