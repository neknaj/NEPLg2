use crate::effects::{
    intrinsic_is_raw_memory_effect, raw_memory_op_from_name as effect_raw_memory_op_from_name,
};
use crate::hir::{FuncRef, HirExpr};
use crate::resource_primitives::type_is_raw_pointer;
use crate::types::TypeCtx;

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
        | RawMemoryOp::LoadU8
        | RawMemoryOp::StoreU8
        | RawMemoryOp::Dealloc
        | RawMemoryOp::Realloc
        | RawMemoryOp::FillBytes
        | RawMemoryOp::Fill
        | RawMemoryOp::BulkCopy
        | RawMemoryOp::BulkMove => args
            .first()
            .map(|arg| !type_is_raw_pointer(types, arg.ty))
            .unwrap_or(true),
        RawMemoryOp::Alloc | RawMemoryOp::MemorySize | RawMemoryOp::MemoryGrow => true,
    }
}
