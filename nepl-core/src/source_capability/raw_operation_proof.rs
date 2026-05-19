use alloc::collections::BTreeSet;

use crate::effects::{RawBodyMemoryOp, RawMemoryOp};
use crate::source_capability::raw_body_operation_compat::raw_body_operation_supports_boundary;
use crate::source_capability::raw_operation_compat::raw_memory_operation_supports_boundary;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::source_capability) enum RawOperationBoundaryContract {
    None,
    RawMemoryOperation(RawMemoryOp),
}

impl RawOperationBoundaryContract {
    pub(in crate::source_capability) fn operation(self) -> Option<RawMemoryOp> {
        match self {
            RawOperationBoundaryContract::None => None,
            RawOperationBoundaryContract::RawMemoryOperation(operation) => Some(operation),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub(in crate::source_capability) struct RawOperationFunctionEvidence {
    operations: BTreeSet<RawMemoryOp>,
    raw_body_operations: BTreeSet<RawBodyMemoryOp>,
}

impl RawOperationFunctionEvidence {
    pub(in crate::source_capability) fn insert_operation(&mut self, operation: RawMemoryOp) {
        self.operations.insert(operation);
    }

    pub(in crate::source_capability) fn insert_raw_body_operation(
        &mut self,
        operation: RawBodyMemoryOp,
    ) {
        self.raw_body_operations.insert(operation);
    }

    pub(in crate::source_capability) fn supports_operation(&self, operation: RawMemoryOp) -> bool {
        self.operations
            .iter()
            .copied()
            .any(|evidence| raw_memory_operation_supports_boundary(evidence, operation))
            || self
                .raw_body_operations
                .iter()
                .copied()
                .any(|evidence| raw_body_operation_supports_boundary(evidence, operation))
    }
}
