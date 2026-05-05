use crate::effects::{
    intrinsic_is_raw_memory_effect, raw_memory_op_from_name as effect_raw_memory_op_from_name,
};
use crate::hir::FuncRef;

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
