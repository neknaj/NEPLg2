use crate::hir::HirExpr;
use crate::types::TypeCtx;

use super::lower_raw_address_place::is_named_struct_type;
use super::model::RawMemoryOp;

pub(super) fn should_count_raw_memory_call(
    operation: &RawMemoryOp,
    args: &[HirExpr],
    types: &TypeCtx,
) -> bool {
    match operation {
        RawMemoryOp::Load
        | RawMemoryOp::Store
        | RawMemoryOp::Dealloc
        | RawMemoryOp::Realloc
        | RawMemoryOp::Fill
        | RawMemoryOp::BulkCopy
        | RawMemoryOp::BulkMove => args
            .first()
            .map(|arg| !is_named_struct_type(types, arg.ty, "MemPtr"))
            .unwrap_or(true),
        RawMemoryOp::Alloc
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow
        | RawMemoryOp::Other { .. } => true,
    }
}
