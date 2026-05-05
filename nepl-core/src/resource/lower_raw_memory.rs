extern crate alloc;

use alloc::string::String;

use crate::effects::{intrinsic_is_raw_memory_effect, raw_callee_is_raw_memory_effect};
use crate::hir::FuncRef;
use crate::runtime_helpers::{
    helper_base_name, ALLOC_RUNTIME_ABI, DEALLOC_RUNTIME_ABI, REALLOC_RUNTIME_ABI,
};

use super::model::{RawMemoryFillUnit, RawMemoryOp};

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
    if !raw_callee_is_raw_memory_effect(name) && !intrinsic_is_raw_memory_effect(name) {
        return None;
    }
    let base = helper_base_name(name);
    let operation = match base {
        ALLOC_RUNTIME_ABI | "alloc_raw" | "alloc" => RawMemoryOp::Alloc,
        DEALLOC_RUNTIME_ABI | "dealloc_raw" | "dealloc" => RawMemoryOp::Dealloc,
        REALLOC_RUNTIME_ABI | "realloc_raw" | "realloc" => RawMemoryOp::Realloc,
        "load" => RawMemoryOp::Load,
        "store" => RawMemoryOp::Store,
        "mem_copy" => RawMemoryOp::BulkCopy,
        "mem_move" => RawMemoryOp::BulkMove,
        "memset_u8" | "fill_u8" => RawMemoryOp::Fill {
            unit: RawMemoryFillUnit::Byte,
        },
        "fill_i32" => RawMemoryOp::Fill {
            unit: RawMemoryFillUnit::I32,
        },
        "mem_size" => RawMemoryOp::MemorySize,
        "mem_grow" => RawMemoryOp::MemoryGrow,
        "mem_fill" => RawMemoryOp::Fill {
            unit: RawMemoryFillUnit::Byte,
        },
        other if other.starts_with("load_") => RawMemoryOp::Load,
        other if other.starts_with("store_") => RawMemoryOp::Store,
        other => RawMemoryOp::Other {
            name: String::from(other),
        },
    };
    Some(operation)
}
