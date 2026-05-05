extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::initialized_summary_condition::RawCellValueCondition;
use super::model::{PlaceProjection, RawMemoryFillUnit};
use super::report::ResourceCheckOperation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationFunctionSummary {
    pub(super) function: String,
    pub(super) return_cells: Vec<RawCellInitializationReturnCell>,
    pub(super) param_cells: Vec<RawCellInitializationParamCell>,
    pub(super) variant_param_cells: Vec<RawCellInitializationVariantParamCell>,
    pub(super) variant_param_ranges: Vec<RawCellInitializationVariantParamRange>,
    pub(super) variant_required_param_cells: Vec<RawCellInitializationVariantParamRequirement>,
    pub(super) variant_conditions: Vec<RawCellInitializationVariantCondition>,
    pub(super) param_destructions: Vec<RawCellDestructionParamAddress>,
    pub(super) param_moves: Vec<RawCellMoveParamAddress>,
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
pub(super) struct RawCellDestructionParamAddress {
    pub(super) param_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) operation: ResourceCheckOperation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellMoveParamAddress {
    pub(super) param_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) address_ty: TypeId,
    pub(super) cell_ty: TypeId,
    pub(super) operation: ResourceCheckOperation,
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
pub(super) struct RawCellInitializationVariantParamRange {
    pub(super) variant: String,
    pub(super) address_param_index: usize,
    pub(super) address_suffix: Vec<PlaceProjection>,
    pub(super) address_ty: TypeId,
    pub(super) count_param_index: usize,
    pub(super) count_suffix: Vec<PlaceProjection>,
    pub(super) count_ty: TypeId,
    pub(super) unit: RawMemoryFillUnit,
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
    pub(super) condition: RawCellValueCondition,
}
