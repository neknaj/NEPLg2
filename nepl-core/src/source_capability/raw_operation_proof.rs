use alloc::collections::BTreeSet;

use crate::effects::{RawBodyMemoryOp, RawMemoryOp};

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

    pub(in crate::source_capability) fn has_direct_raw_evidence(&self) -> bool {
        !self.operations.is_empty() || !self.raw_body_operations.is_empty()
    }
}
