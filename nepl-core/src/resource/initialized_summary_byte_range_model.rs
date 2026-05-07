extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::model::PlaceProjection;

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
pub(super) struct RawCellInitializationVariantParamByteRange {
    pub(super) variant: String,
    pub(super) address_param_index: usize,
    pub(super) address_suffix: Vec<PlaceProjection>,
    pub(super) address_ty: TypeId,
    pub(super) count: RawCellInitializationParamCount,
    pub(super) unit: InitializedRawRangeUnit,
    pub(super) ty: TypeId,
}
