use alloc::collections::BTreeSet;

use crate::effects::{RawBodyMemoryOp, RawMemoryOp};
use crate::resource_primitives::{
    compiler_memory_type_from_constructor_name, MemoryHelperPrimitive,
};

#[derive(Debug, Default)]
pub(in crate::source_capability) struct RawMemoryEvidence {
    pub(in crate::source_capability) structural_boundary: bool,
    pub(in crate::source_capability) raw_address_view_boundary: bool,
    pub(in crate::source_capability) operations: BTreeSet<RawMemoryOp>,
    pub(in crate::source_capability) raw_body_operations: BTreeSet<RawBodyMemoryOp>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::source_capability) enum RawMemoryStructuralEvidence {
    RestrictedConstructor,
}

impl RawMemoryStructuralEvidence {
    pub(in crate::source_capability) fn from_symbol(name: &str) -> Option<Self> {
        if compiler_memory_type_from_constructor_name(name).is_some() {
            return Some(Self::RestrictedConstructor);
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::source_capability) enum RawAddressViewEvidence {
    MemoryHelperPrimitive,
}

impl RawAddressViewEvidence {
    pub(in crate::source_capability) fn from_symbol(name: &str) -> Option<Self> {
        if MemoryHelperPrimitive::from_symbol(name)
            .is_some_and(MemoryHelperPrimitive::is_raw_address_view_boundary_evidence)
        {
            return Some(Self::MemoryHelperPrimitive);
        }
        None
    }
}
