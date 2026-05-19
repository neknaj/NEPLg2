use crate::effects::RawMemoryOp;

pub(in crate::source_capability) fn raw_memory_operation_supports_boundary(
    evidence: RawMemoryOp,
    boundary: RawMemoryOp,
) -> bool {
    if evidence == boundary {
        return true;
    }
    match (evidence, boundary) {
        (RawMemoryOp::MemoryGrow, RawMemoryOp::Alloc) => true,
        (RawMemoryOp::Store, RawMemoryOp::Dealloc) => true,
        (RawMemoryOp::StoreU8, RawMemoryOp::FillBytes) => true,
        (RawMemoryOp::Store, RawMemoryOp::Fill) => true,
        _ => false,
    }
}

pub(in crate::source_capability) fn raw_memory_operation_set_supports_boundary(
    evidence: impl IntoIterator<Item = RawMemoryOp>,
    boundary: RawMemoryOp,
) -> bool {
    let mut has_alloc = false;
    let mut has_dealloc = false;
    let mut has_compatible = false;
    for operation in evidence {
        has_alloc |= operation == RawMemoryOp::Alloc;
        has_dealloc |= operation == RawMemoryOp::Dealloc;
        has_compatible |= raw_memory_operation_supports_boundary(operation, boundary);
    }
    match boundary {
        RawMemoryOp::Realloc => has_alloc && has_dealloc,
        RawMemoryOp::Alloc
        | RawMemoryOp::Dealloc
        | RawMemoryOp::Load
        | RawMemoryOp::Store
        | RawMemoryOp::LoadU8
        | RawMemoryOp::StoreU8
        | RawMemoryOp::BulkCopy
        | RawMemoryOp::BulkMove
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow
        | RawMemoryOp::FillBytes
        | RawMemoryOp::Fill => has_compatible,
    }
}
