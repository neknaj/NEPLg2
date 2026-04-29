use alloc::string::String;

use crate::hir::{FuncRef, HirExpr, HirExprKind};
use crate::layout::storage_size_bytes;
use crate::types::{TypeId, TypeKind};

use super::provenance::{
    func_ref_name, is_mem_ptr_type, is_region_token_type, raw_memory_place_key,
    raw_memory_place_key_from_mem_ptr, raw_memory_place_key_from_region_token,
};
use super::MoveCheckContext;

fn mem_ptr_element_type(tctx: &crate::types::TypeCtx, ty: TypeId) -> Option<TypeId> {
    match tctx.get_ref(tctx.resolve_id(ty)) {
        TypeKind::Apply { base, args } => match tctx.get_ref(tctx.resolve_id(*base)) {
            TypeKind::Struct { name, .. } if name == "MemPtr" => args.first().copied(),
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn raw_dealloc_place_key(
    addr: &HirExpr,
    ctx: &MoveCheckContext,
    tctx: &crate::types::TypeCtx,
) -> Option<String> {
    if is_mem_ptr_type(tctx, addr.ty) {
        raw_memory_place_key_from_mem_ptr(addr, ctx, tctx)
    } else if is_region_token_type(tctx, addr.ty) {
        raw_memory_place_key_from_region_token(addr, ctx, tctx)
    } else if tctx.same_type(addr.ty, tctx.i32()) {
        raw_memory_place_key(addr, ctx, tctx)
    } else {
        None
    }
}

pub(super) fn raw_dealloc_size_arg_bytes(
    arg: Option<&HirExpr>,
    tctx: &crate::types::TypeCtx,
) -> Option<usize> {
    match arg.map(|arg| &arg.kind) {
        Some(HirExprKind::LiteralI32(value)) if *value > 0 => Some(*value as usize),
        Some(HirExprKind::Intrinsic {
            name, type_args, ..
        }) if name == "size_of" && type_args.len() == 1 => {
            Some(storage_size_bytes(tctx, type_args[0]))
        }
        _ => None,
    }
}

pub(super) fn raw_store_write_size_bytes(
    callee: &FuncRef,
    value: Option<&HirExpr>,
    tctx: &crate::types::TypeCtx,
) -> Option<usize> {
    match func_ref_name(callee) {
        Some(name) if name == "store_u8" || name.starts_with("store_u8_") => Some(1),
        Some(name) if name == "store_i32" || name.starts_with("store_i32_") => Some(4),
        _ => value.map(|value| storage_size_bytes(tctx, value.ty)),
    }
}

pub(super) fn raw_byte_write_size_arg_bytes(
    callee: &FuncRef,
    args: &[HirExpr],
    tctx: &crate::types::TypeCtx,
) -> Option<usize> {
    match func_ref_name(callee) {
        Some(name) if name == "fill_i32" || name.starts_with("fill_i32_") => {
            match args.get(1).map(|arg| &arg.kind) {
                Some(HirExprKind::LiteralI32(count)) if *count > 0 => Some((*count as usize) * 4),
                Some(HirExprKind::LiteralI32(_)) => Some(0),
                _ => None,
            }
        }
        _ => raw_dealloc_size_arg_bytes(args.get(1), tctx),
    }
}

pub(super) fn raw_bulk_copy_size_arg_bytes(
    args: &[HirExpr],
    tctx: &crate::types::TypeCtx,
) -> Option<usize> {
    let element_ty = args
        .get(0)
        .and_then(|arg| mem_ptr_element_type(tctx, arg.ty))
        .or_else(|| {
            args.get(1)
                .and_then(|arg| mem_ptr_element_type(tctx, arg.ty))
        });
    if let Some(element_ty) = element_ty {
        let count = match args.get(2).map(|arg| &arg.kind) {
            Some(HirExprKind::LiteralI32(value)) if *value > 0 => *value as usize,
            Some(HirExprKind::LiteralI32(_)) => return Some(0),
            _ => return None,
        };
        return Some(count.saturating_mul(storage_size_bytes(tctx, element_ty)));
    }
    raw_dealloc_size_arg_bytes(args.get(2), tctx)
}
