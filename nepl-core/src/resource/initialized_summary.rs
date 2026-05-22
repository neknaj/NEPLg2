extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::initialized_summary_byte_range_model::{
    RawCellInitializationParamByteRange, RawCellInitializationReturnByteRange,
    RawCellInitializationVariantParamByteRange,
};
pub(super) use super::initialized_summary_release_model::{
    RawCellReleaseParamRequirement, RawCellReleaseRequirementKind,
};
use super::model::PlaceProjection;
use super::summary_index::{FunctionSummary, SummaryIndex};
use super::summary_projection::SummaryProjection;

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

pub(super) type RawCellInitializationFunctionSummaryIndex<'a> =
    SummaryIndex<'a, RawCellInitializationFunctionSummary>;

impl FunctionSummary for RawCellInitializationFunctionSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationReturnCell {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) holds_raw_address: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationParamCell {
    pub(super) param_index: usize,
    pub(super) suffix: Vec<SummaryProjection>,
    pub(super) ty: TypeId,
    pub(super) holds_raw_address: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationVariantParamCell {
    pub(super) variant: String,
    pub(super) param_index: usize,
    pub(super) suffix: Vec<SummaryProjection>,
    pub(super) ty: TypeId,
    pub(super) holds_raw_address: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationVariantParamRequirement {
    pub(super) variant: String,
    pub(super) param_index: usize,
    pub(super) suffix: Vec<SummaryProjection>,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationVariantCondition {
    pub(super) variant: String,
    pub(super) param_index: usize,
    pub(super) suffix: Vec<SummaryProjection>,
    pub(super) ty: TypeId,
    pub(super) condition: super::initialized_summary_condition::RawCellValueCondition,
}
