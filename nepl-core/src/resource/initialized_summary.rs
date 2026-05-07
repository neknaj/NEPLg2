extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::model::PlaceProjection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationFunctionSummary {
    pub(super) function: String,
    pub(super) return_cells: Vec<RawCellInitializationReturnCell>,
    pub(super) return_byte_ranges: Vec<RawCellInitializationReturnByteRange>,
    pub(super) param_cells: Vec<RawCellInitializationParamCell>,
    pub(super) param_release_requirements: Vec<RawCellReleaseParamRequirement>,
    pub(super) variant_param_cells: Vec<RawCellInitializationVariantParamCell>,
    pub(super) variant_required_param_cells: Vec<RawCellInitializationVariantParamRequirement>,
    pub(super) variant_conditions: Vec<RawCellInitializationVariantCondition>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationReturnCell {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) holds_raw_address: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationReturnByteRange {
    pub(super) address_suffix: Vec<PlaceProjection>,
    pub(super) address_ty: TypeId,
    pub(super) count_suffix: Vec<PlaceProjection>,
    pub(super) count_ty: TypeId,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationParamCell {
    pub(super) param_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) holds_raw_address: bool,
}

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationVariantParamCell {
    pub(super) variant: String,
    pub(super) param_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) holds_raw_address: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationVariantParamRequirement {
    pub(super) variant: String,
    pub(super) param_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationVariantCondition {
    pub(super) variant: String,
    pub(super) param_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) condition: super::initialized_summary_condition::RawCellValueCondition,
}
