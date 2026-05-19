use crate::effects::{LlvmRawBodyMemoryOp, RawBodyMemoryOp, RawMemoryOp, WasmRawBodyMemoryOp};

pub(in crate::source_capability) fn raw_body_operation_supports_boundary(
    evidence: RawBodyMemoryOp,
    boundary: RawMemoryOp,
) -> bool {
    match evidence {
        RawBodyMemoryOp::Wasm(operation) => {
            wasm_raw_body_operation_supports_boundary(operation, boundary)
        }
        RawBodyMemoryOp::Llvm(operation) => {
            llvm_raw_body_operation_supports_boundary(operation, boundary)
        }
    }
}

fn wasm_raw_body_operation_supports_boundary(
    evidence: WasmRawBodyMemoryOp,
    boundary: RawMemoryOp,
) -> bool {
    match evidence {
        WasmRawBodyMemoryOp::Load => {
            matches!(boundary, RawMemoryOp::Load | RawMemoryOp::LoadU8)
        }
        WasmRawBodyMemoryOp::Store => matches!(
            boundary,
            RawMemoryOp::Store | RawMemoryOp::StoreU8 | RawMemoryOp::Dealloc
        ),
        WasmRawBodyMemoryOp::MemorySize => boundary == RawMemoryOp::MemorySize,
        WasmRawBodyMemoryOp::MemoryGrow => {
            matches!(boundary, RawMemoryOp::MemoryGrow | RawMemoryOp::Alloc)
        }
        WasmRawBodyMemoryOp::MemoryCopy => {
            matches!(boundary, RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove)
        }
        WasmRawBodyMemoryOp::MemoryFill => boundary == RawMemoryOp::FillBytes,
        WasmRawBodyMemoryOp::MemoryInit
        | WasmRawBodyMemoryOp::DataDrop
        | WasmRawBodyMemoryOp::Memory => false,
    }
}

fn llvm_raw_body_operation_supports_boundary(
    evidence: LlvmRawBodyMemoryOp,
    boundary: RawMemoryOp,
) -> bool {
    match evidence {
        LlvmRawBodyMemoryOp::Load => {
            matches!(boundary, RawMemoryOp::Load | RawMemoryOp::LoadU8)
        }
        LlvmRawBodyMemoryOp::Store => matches!(
            boundary,
            RawMemoryOp::Store | RawMemoryOp::StoreU8 | RawMemoryOp::Dealloc
        ),
        LlvmRawBodyMemoryOp::Memcpy => boundary == RawMemoryOp::BulkCopy,
        LlvmRawBodyMemoryOp::Memmove => boundary == RawMemoryOp::BulkMove,
        LlvmRawBodyMemoryOp::Memset => boundary == RawMemoryOp::FillBytes,
        LlvmRawBodyMemoryOp::Alloca
        | LlvmRawBodyMemoryOp::AtomicRmw
        | LlvmRawBodyMemoryOp::Cmpxchg
        | LlvmRawBodyMemoryOp::Fence => false,
    }
}
