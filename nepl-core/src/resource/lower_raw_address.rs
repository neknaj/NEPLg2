use alloc::vec::Vec;

use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::resource_primitives::MemoryHelperPrimitive;
use crate::runtime_helpers::helper_base_name;
use crate::span::Span;
use crate::types::TypeId;

use super::compiler_memory_place::{mem_ptr_raw_field_place, region_token_raw_field_place};
use super::lower::LoweringEnvironment;
use super::lower_call::func_ref_base_name;
use super::lower_layout_intrinsic::{
    layout_intrinsic_i64_value, layout_intrinsic_i64_value_from_callee,
};
use super::lower_raw_address_place::{
    raw_address_alias_target, raw_address_place_from_actual_argument, reference_target_type,
    region_token_place_from_actual_arg,
};
pub(super) use super::lower_raw_address_return::push_transparent_raw_address_return_projection;
use super::lower_raw_address_source::{push_raw_address_op, RawAddressOffset, RawAddressSource};
use super::model::{
    Place, PlaceProjection, RawAddressAliasKind, RawAddressViewKind, ResourceOffset, ResourceOp,
    StorageOrigin,
};
use super::place_utils::reference_target_place;
use super::result_variant::ResultVariant;
use super::scalar_primitive::I32ArithmeticPrimitive;

pub(super) fn push_core_mem_wrapper_semantics(
    callee: &FuncRef,
    hir_args: &[HirExpr],
    arg_places: &[Place],
    output: &Place,
    ops: &mut Vec<ResourceOp>,
    env: &LoweringEnvironment,
    span: Span,
) -> bool {
    match func_ref_base_name(callee).and_then(MemoryHelperPrimitive::from_base_name) {
        Some(MemoryHelperPrimitive::MemPtrWrap) => {
            let Some(raw) = arg_places.first() else {
                return false;
            };
            ops.push(ResourceOp::RawAddressAlias {
                source: raw.clone(),
                target: mem_ptr_raw_field_place(env.types, output, env.types.i32()),
                kind: RawAddressAliasKind::InternalHelper,
                span,
            });
            true
        }
        Some(MemoryHelperPrimitive::MemPtrAddr) => {
            let Some(ptr) = arg_places.first() else {
                return false;
            };
            push_raw_address_op(
                mem_ptr_raw_field_place(env.types, ptr, output.ty),
                output.clone(),
                Some(RawAddressViewKind::InternalHelper),
                ops,
                span,
            );
            true
        }
        Some(MemoryHelperPrimitive::MemPtrAdd) => {
            let Some(ptr) = arg_places.first() else {
                return false;
            };
            let mut raw = mem_ptr_raw_field_place(env.types, ptr, env.types.i32());
            let view_kind = Some(RawAddressViewKind::MemPtrOffset);
            raw = raw_address_place_with_offset(
                raw,
                raw_address_offset_from_actual_arg(1, hir_args, arg_places, env),
                env.types.i32(),
            );
            push_raw_address_op(
                raw,
                mem_ptr_raw_field_place(env.types, output, env.types.i32()),
                view_kind,
                ops,
                span,
            );
            true
        }
        Some(MemoryHelperPrimitive::RegionNew) => {
            let Some(ptr) = arg_places.first() else {
                return false;
            };
            let target = region_token_raw_field_place(env.types, output, env.types.i32());
            ops.push(ResourceOp::RawAddressAlias {
                source: mem_ptr_raw_field_place(env.types, ptr, env.types.i32()),
                target,
                kind: RawAddressAliasKind::InternalHelper,
                span,
            });
            true
        }
        Some(MemoryHelperPrimitive::RegionPtr) => {
            let Some(source) =
                region_token_raw_source_from_actual_arg(0, hir_args, arg_places, env)
            else {
                return false;
            };
            let source = source.into_place_and_view(env.types.i32());
            push_raw_address_op(
                source.place,
                mem_ptr_raw_field_place(env.types, output, env.types.i32()),
                Some(RawAddressViewKind::NonOwningProjection),
                ops,
                span,
            );
            true
        }
        Some(MemoryHelperPrimitive::RegionPtrAt) => {
            let Some(source) =
                region_token_raw_source_from_actual_arg(0, hir_args, arg_places, env)
            else {
                return false;
            };
            let source = source
                .with_added_offset(raw_address_offset_from_actual_arg(
                    1, hir_args, arg_places, env,
                ))
                .into_non_owning_view()
                .into_place_and_view(env.types.i32());
            let Some(ok_payload) = ResultVariant::Ok.payload_place(env.types, output) else {
                return false;
            };
            push_raw_address_op(
                source.place,
                mem_ptr_raw_field_place(env.types, &ok_payload, env.types.i32()),
                source.view_kind,
                ops,
                span,
            );
            true
        }
        Some(MemoryHelperPrimitive::RegionTokenRawRef) => {
            let Some(source) =
                region_token_raw_source_from_actual_arg(0, hir_args, arg_places, env)
            else {
                return false;
            };
            let Some(target_ty) = reference_target_type(env.types, output.ty) else {
                return false;
            };
            push_raw_address_op(
                source.base,
                reference_target_place(output, target_ty),
                Some(RawAddressViewKind::InternalHelper),
                ops,
                span,
            );
            true
        }
        _ => false,
    }
}

pub(super) fn push_core_mem_owner_storage_origin(
    callee: &FuncRef,
    output: &Place,
    ops: &mut Vec<ResourceOp>,
    env: &LoweringEnvironment,
    span: Span,
) {
    if func_ref_base_name(callee).and_then(MemoryHelperPrimitive::from_base_name)
        != Some(MemoryHelperPrimitive::RegionNew)
    {
        return;
    }
    ops.push(ResourceOp::StorageOrigin {
        target: region_token_raw_field_place(env.types, output, env.types.i32()),
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
                .or_else(|| layout_intrinsic_i64_value_from_callee(callee, env))
        }
        HirExprKind::Intrinsic {
            name,
            type_args,
            args,
        } => layout_intrinsic_i64_value(name, type_args, env)
            .or_else(|| i32_const_from_actual_named_expr(helper_base_name(name), args, env)),
        _ => None,
    }
}

fn raw_address_source_from_actual_named_expr(
    name: &str,
    args: &[HirExpr],
    arg_places: &[Place],
    env: &LoweringEnvironment,
) -> Option<RawAddressSource> {
    match MemoryHelperPrimitive::from_symbol(name) {
        Some(MemoryHelperPrimitive::MemPtrAddr) if args.len() == 1 && arg_places.len() == 1 => {
            return raw_address_source_from_actual_arg(0, args, arg_places, env)
                .map(RawAddressSource::into_internal_view);
        }
        Some(MemoryHelperPrimitive::MemPtrWrap) if args.len() == 1 && arg_places.len() == 1 => {
            return raw_address_source_from_actual_arg(0, args, arg_places, env);
        }
        Some(MemoryHelperPrimitive::StrAddr) if args.len() == 1 && arg_places.len() == 1 => {
            return raw_address_source_from_actual_arg(0, args, arg_places, env)
                .map(RawAddressSource::into_internal_view);
        }
        Some(MemoryHelperPrimitive::MemPtrAdd) if args.len() >= 2 && arg_places.len() >= 2 => {
            return raw_address_source_from_actual_arg(0, args, arg_places, env).map(|source| {
                source
                    .with_added_offset(raw_address_offset_from_actual_arg(1, args, arg_places, env))
            });
        }
        Some(MemoryHelperPrimitive::RegionNew) if args.len() >= 2 && !arg_places.is_empty() => {
            return raw_address_source_from_actual_arg(0, args, arg_places, env);
        }
        _ => {}
    }
    match I32ArithmeticPrimitive::from_symbol(name) {
        Some(I32ArithmeticPrimitive::Add) if args.len() == 2 && arg_places.len() == 2 => {
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
        Some(I32ArithmeticPrimitive::Sub) if args.len() == 2 && arg_places.len() == 2 => {
            raw_address_source_from_actual_arg(0, args, arg_places, env).map(|source| {
                source.with_subtracted_offset(raw_address_offset_from_actual_arg(
                    1, args, arg_places, env,
                ))
            })
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
        RawAddressOffset::SymbolicPlusKnown { place, bytes } => {
            let raw = raw.with_projection(
                PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place }),
                raw_ty,
            );
            match usize::try_from(bytes) {
                Ok(bytes) => raw.with_projection(
                    PlaceProjection::StorageOffset(ResourceOffset::Known(bytes)),
                    raw_ty,
                ),
                Err(_) => raw.with_projection(
                    PlaceProjection::StorageOffset(ResourceOffset::Unknown),
                    raw_ty,
                ),
            }
        }
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
        internal_view: false,
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
        base: region_token_raw_field_place(env.types, &token, env.types.i32()),
        offset: RawAddressOffset::Known(0),
        explicit_offset: false,
        non_owning_view: false,
        internal_view: false,
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
    let op = I32ArithmeticPrimitive::from_symbol(name)?;
    match op {
        I32ArithmeticPrimitive::Mul => {
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
        I32ArithmeticPrimitive::Add | I32ArithmeticPrimitive::Sub => op.checked_i64(
            i32_const_from_actual_arg(&args[0], env)?,
            i32_const_from_actual_arg(&args[1], env)?,
        ),
    }
}
