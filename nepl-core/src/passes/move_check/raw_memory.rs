use crate::hir::{FuncRef, HirExpr};
use crate::types::TypeId;

use super::{is_mem_ptr_type, is_region_token_type};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawMemoryCallKind {
    Load,
    Store,
    Dealloc,
    Realloc,
    BulkCopy,
    ByteWrite,
}

pub(super) fn raw_memory_call_kind(
    callee: &FuncRef,
    args: &[HirExpr],
    result_ty: TypeId,
    tctx: &crate::types::TypeCtx,
) -> Option<RawMemoryCallKind> {
    let FuncRef::User(name, _, _) = callee else {
        return None;
    };
    if is_raw_memory_load_name(name)
        && args.len() == 1
        && tctx.same_type(args[0].ty, tctx.i32())
        && !tctx.is_copy(result_ty)
    {
        return Some(RawMemoryCallKind::Load);
    }
    if is_raw_memory_store_name(name)
        && args.len() >= 2
        && raw_place_arg_is_address_or_memptr(&args[0], tctx)
    {
        return Some(RawMemoryCallKind::Store);
    }
    if is_raw_memory_dealloc_name(name)
        && args.len() >= 2
        && (tctx.same_type(args[0].ty, tctx.i32()) || is_mem_ptr_type(tctx, args[0].ty))
    {
        return Some(RawMemoryCallKind::Dealloc);
    }
    if is_raw_memory_dealloc_name(name) && args.len() == 1 && is_region_token_type(tctx, args[0].ty)
    {
        return Some(RawMemoryCallKind::Dealloc);
    }
    if is_raw_memory_realloc_name(name)
        && args.len() >= 3
        && (tctx.same_type(args[0].ty, tctx.i32()) || is_mem_ptr_type(tctx, args[0].ty))
    {
        return Some(RawMemoryCallKind::Realloc);
    }
    if is_raw_memory_bulk_copy_name(name)
        && args.len() >= 3
        && raw_place_arg_is_address_or_memptr(&args[0], tctx)
        && raw_place_arg_is_address_or_memptr(&args[1], tctx)
    {
        return Some(RawMemoryCallKind::BulkCopy);
    }
    if is_raw_memory_byte_fill_name(name)
        && args.len() >= 2
        && raw_place_arg_is_address_or_memptr(&args[0], tctx)
    {
        return Some(RawMemoryCallKind::ByteWrite);
    }
    None
}

pub(super) fn raw_memory_helper_name_is_tracked(name: &str) -> bool {
    is_raw_memory_load_name(name)
        || is_raw_memory_store_name(name)
        || is_raw_memory_dealloc_name(name)
        || is_raw_memory_realloc_name(name)
        || is_raw_memory_bulk_copy_name(name)
        || is_raw_memory_byte_fill_name(name)
}

fn raw_place_arg_is_address_or_memptr(arg: &HirExpr, tctx: &crate::types::TypeCtx) -> bool {
    tctx.same_type(arg.ty, tctx.i32()) || is_mem_ptr_type(tctx, arg.ty)
}

fn is_raw_memory_load_name(name: &str) -> bool {
    name == "load" || name.starts_with("load_")
}

fn is_raw_memory_store_name(name: &str) -> bool {
    name == "store" || name.starts_with("store_")
}

fn is_raw_memory_byte_fill_name(name: &str) -> bool {
    name == "memset_u8"
        || name == "fill_u8"
        || name == "fill_i32"
        || name == "mem_fill"
        || name.starts_with("memset_u8_")
        || name.starts_with("fill_u8_")
        || name.starts_with("fill_i32_")
        || name.starts_with("mem_fill_")
}

fn is_raw_memory_dealloc_name(name: &str) -> bool {
    name == "dealloc"
        || name == "dealloc_raw"
        || name == "dealloc_ptr"
        || name == "dealloc_region"
        || name == "__nepl_rt_dealloc"
        || name.starts_with("dealloc_raw_")
        || name.starts_with("dealloc_ptr_")
        || name.starts_with("dealloc_region_")
        || name.starts_with("__nepl_rt_dealloc_")
}

fn is_raw_memory_realloc_name(name: &str) -> bool {
    name == "realloc"
        || name == "realloc_raw"
        || name == "realloc_ptr"
        || name == "__nepl_rt_realloc"
        || name.starts_with("realloc_raw_")
        || name.starts_with("realloc_ptr_")
        || name.starts_with("__nepl_rt_realloc_")
}

fn is_raw_memory_bulk_copy_name(name: &str) -> bool {
    name == "mem_copy"
        || name == "mem_move"
        || name.starts_with("mem_copy_")
        || name.starts_with("mem_move_")
}
