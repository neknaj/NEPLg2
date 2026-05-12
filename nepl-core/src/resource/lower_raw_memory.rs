use crate::effects::{
    intrinsic_is_raw_memory_effect, raw_memory_op_from_name as effect_raw_memory_op_from_name,
};
use crate::hir::{FuncRef, HirExpr};
use crate::types::TypeCtx;

use super::lower_raw_address_place::is_named_struct_type;
use super::model::RawMemoryOp;

pub(super) fn raw_memory_op_from_callee(callee: &FuncRef) -> Option<RawMemoryOp> {
    match callee {
        FuncRef::Builtin(name) | FuncRef::User(name, _, _) => raw_memory_op_from_name(name),
        FuncRef::Trait { .. } => None,
    }
}

pub(super) fn raw_memory_op_from_intrinsic(name: &str) -> Option<RawMemoryOp> {
    if intrinsic_is_raw_memory_effect(name) {
        raw_memory_op_from_name(name)
    } else {
        None
    }
}

pub(super) fn raw_memory_op_from_name(name: &str) -> Option<RawMemoryOp> {
    effect_raw_memory_op_from_name(name)
}

pub(super) fn raw_memory_call_uses_direct_raw_address(
    operation: &RawMemoryOp,
    args: &[HirExpr],
    types: &TypeCtx,
) -> bool {
    match operation {
        RawMemoryOp::Load
        | RawMemoryOp::Store
        | RawMemoryOp::Dealloc
        | RawMemoryOp::Realloc
        | RawMemoryOp::FillBytes
        | RawMemoryOp::Fill
        | RawMemoryOp::BulkCopy
        | RawMemoryOp::BulkMove => args
            .first()
            .map(|arg| !is_named_struct_type(types, arg.ty, "MemPtr"))
            .unwrap_or(true),
        RawMemoryOp::Alloc | RawMemoryOp::MemorySize | RawMemoryOp::MemoryGrow => true,
    }
}
