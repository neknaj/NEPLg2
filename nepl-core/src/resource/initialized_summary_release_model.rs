extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::model::PlaceProjection;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawCellReleaseRequirementKind {
    Store,
    Dealloc,
    Realloc,
    Fill,
    BulkDestination,
    BulkSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellReleaseParamRequirement {
    pub(super) param_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) kind: RawCellReleaseRequirementKind,
}
