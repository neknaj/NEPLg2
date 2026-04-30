extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::model::PlaceProjection;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationFunctionSummary {
    pub(super) function: String,
    pub(super) return_cells: Vec<RawCellInitializationReturnCell>,
    pub(super) param_cells: Vec<RawCellInitializationParamCell>,
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
pub(super) struct RawCellInitializationParamCell {
    pub(super) param_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) holds_raw_address: bool,
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
    pub(super) condition: RawCellValueCondition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawCellValueCondition {
    EqZero,
    NeZero,
    Positive,
    NonPositive,
    Negative,
    NonNegative,
}

impl RawCellValueCondition {
    pub(super) fn holds(self, value: i32) -> bool {
        match self {
            RawCellValueCondition::EqZero => value == 0,
            RawCellValueCondition::NeZero => value != 0,
            RawCellValueCondition::Positive => value > 0,
            RawCellValueCondition::NonPositive => value <= 0,
            RawCellValueCondition::Negative => value < 0,
            RawCellValueCondition::NonNegative => value >= 0,
        }
    }
}
