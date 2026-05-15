use alloc::collections::BTreeSet;

use crate::effects::{RawBodyMemoryOp, RawMemoryOp};
use crate::runtime_helpers::helper_base_name;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawAddressBoundaryHelper {
    MemPtrWrap,
    MemPtrAddr,
    MemPtrAdd,
    RegionNew,
    RegionPtr,
    RegionPtrAt,
    RegionTokenPtrRef,
    StrAddr,
    StrFromAddrUnchecked,
}

impl RawAddressBoundaryHelper {
    fn from_symbol(name: &str) -> Option<Self> {
        let helper = match helper_base_name(name) {
            "mem_ptr_wrap" => Self::MemPtrWrap,
            "mem_ptr_addr" => Self::MemPtrAddr,
            "mem_ptr_add" => Self::MemPtrAdd,
            "region_new" => Self::RegionNew,
            "region_ptr" => Self::RegionPtr,
            "region_ptr_at" => Self::RegionPtrAt,
            "region_token_ptr_ref" => Self::RegionTokenPtrRef,
            "str_addr" => Self::StrAddr,
            "str_from_addr_unchecked" => Self::StrFromAddrUnchecked,
            _ => return None,
        };
        Some(helper)
    }
}

impl RawMemoryBoundaryEvidence {
    pub(super) fn from_symbol(name: &str) -> Option<Self> {
        if matches!(name, "MemPtr" | "RegionToken") {
            return Some(Self::RestrictedConstructor);
        }
        if RawAddressBoundaryHelper::from_symbol(name).is_some() {
            return Some(Self::RawAddressBoundaryHelper);
        }
        None
    }
}
