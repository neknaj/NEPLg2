use crate::hir::HirExpr;
use crate::types::TypeCtx;

use super::lower_raw_memory::raw_memory_call_uses_direct_raw_address;
use super::model::RawMemoryOp;

pub(super) fn should_count_raw_memory_call(
    operation: &RawMemoryOp,
    args: &[HirExpr],
    types: &TypeCtx,
) -> bool {
    raw_memory_call_uses_direct_raw_address(operation, args, types)
}
