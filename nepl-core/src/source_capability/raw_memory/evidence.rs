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
