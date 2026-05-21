use crate::types::TypeCtx;

use super::model::{EffectOp, Place};
use super::place_utils::call_uses_checked_mem_ptr_wrapper;

pub(super) fn direct_call_invalidates_result(
    types: &TypeCtx,
    effect: &EffectOp,
    args: &[Place],
) -> bool {
    matches!(effect, EffectOp::InternalAlloc { .. })
        || (matches!(effect, EffectOp::UnsafeMemory { .. })
            && !call_uses_checked_mem_ptr_wrapper(types, args))
}
