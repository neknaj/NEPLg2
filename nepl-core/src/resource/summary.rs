use alloc::string::String;
use alloc::vec::Vec;

use super::model::PlaceProjection;
use crate::types::TypeId;

pub(super) use super::borrow_summary::compute_borrow_token_return_summaries;
pub(super) use super::owner_summary::compute_owner_return_summaries;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BorrowTokenReturnSummary {
    pub(super) function: String,
    pub(super) parameter_indices: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerReturnSummary {
    pub(super) function: String,
    pub(super) parameter_indices: Vec<usize>,
    pub(super) parameter_sources: Vec<OwnerProjectionSource>,
    pub(super) consumed_parameter_indices: Vec<usize>,
    pub(super) consumed_parameter_sources: Vec<OwnerProjectionSource>,
    pub(super) variant_consumed_parameter_indices: Vec<OwnerVariantParameterIndex>,
    pub(super) variant_consumed_parameter_sources: Vec<OwnerVariantProjectionSource>,
    pub(super) variant_projection_returns: Vec<OwnerVariantProjectionReturnSource>,
    pub(super) returns_fresh_owner: bool,
    pub(super) returns_maybe_owner: bool,
    pub(super) projection_returns: Vec<OwnerProjectionReturnSummary>,
    pub(super) projection_markers: Vec<OwnerProjectionMarker>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerProjectionSource {
    pub(super) parameter_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerVariantParameterIndex {
    pub(super) variant: String,
    pub(super) parameter_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerVariantProjectionSource {
    pub(super) variant: String,
    pub(super) source: OwnerProjectionSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerVariantProjectionReturnSource {
    pub(super) variant: String,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) source: OwnerProjectionSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerProjectionMarker {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerProjectionReturnSummary {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) parameter_indices: Vec<usize>,
    pub(super) parameter_sources: Vec<OwnerProjectionSource>,
    pub(super) returns_fresh_owner: bool,
    pub(super) returns_maybe_owner: bool,
}
