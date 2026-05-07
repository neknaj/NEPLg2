extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::cell_state_raw_range::InitializedRawRangeUnit;
pub(super) use super::initialized_summary_release_model::{
    RawCellReleaseParamRequirement, RawCellReleaseRequirementKind,
};
use super::model::PlaceProjection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationFunctionSummary {
    pub(super) function: String,
    pub(super) return_cells: Vec<RawCellInitializationReturnCell>,
    pub(super) return_byte_ranges: Vec<RawCellInitializationReturnByteRange>,
    pub(super) param_cells: Vec<RawCellInitializationParamCell>,
    pub(super) param_byte_ranges: Vec<RawCellInitializationParamByteRange>,
    pub(super) param_release_requirements: Vec<RawCellReleaseParamRequirement>,
    pub(super) variant_param_cells: Vec<RawCellInitializationVariantParamCell>,
    pub(super) variant_param_byte_ranges: Vec<RawCellInitializationVariantParamByteRange>,
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
    pub(super) count: RawCellInitializationReturnCount,
    pub(super) unit: InitializedRawRangeUnit,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RawCellInitializationReturnCount {
    ReturnValueProjection {
        suffix: Vec<PlaceProjection>,
        ty: TypeId,
    },
    KnownI32 {
        value: i32,
        ty: TypeId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationParamCell {
    pub(super) param_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) holds_raw_address: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationParamByteRange {
    pub(super) address_param_index: usize,
    pub(super) address_suffix: Vec<PlaceProjection>,
    pub(super) address_ty: TypeId,
    pub(super) count: RawCellInitializationParamCount,
    pub(super) unit: InitializedRawRangeUnit,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum RawCellInitializationParamCount {
    ParamProjection {
        param_index: usize,
        suffix: Vec<PlaceProjection>,
        ty: TypeId,
    },
    KnownI32 {
        value: i32,
        ty: TypeId,
    },
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
pub(super) struct RawCellInitializationVariantParamByteRange {
    pub(super) variant: String,
    pub(super) address_param_index: usize,
    pub(super) address_suffix: Vec<PlaceProjection>,
    pub(super) address_ty: TypeId,
    pub(super) count: RawCellInitializationParamCount,
    pub(super) unit: InitializedRawRangeUnit,
    pub(super) ty: TypeId,
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
