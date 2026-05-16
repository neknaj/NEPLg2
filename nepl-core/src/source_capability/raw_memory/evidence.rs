use alloc::collections::BTreeSet;

use crate::effects::{RawBodyMemoryOp, RawMemoryOp};
use crate::resource_primitives::{
    compiler_memory_type_from_constructor_name, MemoryHelperPrimitive,
};

#[derive(Debug, Default)]
pub(super) struct RawMemoryEvidence {
    pub(super) structural_boundary: bool,
    pub(super) operations: BTreeSet<RawMemoryOp>,
    pub(super) raw_body_operations: BTreeSet<RawBodyMemoryOp>,
}

impl RawMemoryEvidence {
    pub(super) fn has_any_evidence(&self) -> bool {
        self.structural_boundary
            || !self.operations.is_empty()
            || !self.raw_body_operations.is_empty()
    }

    pub(super) fn merge(&mut self, other: Self) {
        self.structural_boundary |= other.structural_boundary;
        self.operations.extend(other.operations);
        self.raw_body_operations.extend(other.raw_body_operations);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawMemoryBoundaryEvidence {
    RawAddressBoundaryHelper,
    RestrictedConstructor,
}

impl RawMemoryBoundaryEvidence {
    pub(super) fn from_symbol(name: &str) -> Option<Self> {
        if compiler_memory_type_from_constructor_name(name).is_some() {
            return Some(Self::RestrictedConstructor);
        }
        if MemoryHelperPrimitive::from_symbol(name).is_some() {
            return Some(Self::RawAddressBoundaryHelper);
        }
        None
    }
}
