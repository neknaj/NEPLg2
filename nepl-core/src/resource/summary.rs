use alloc::string::String;
use alloc::vec::Vec;

use super::host_memory_contract::HostMemorySpan;
use super::host_size_contract::HostSizeKind;
use super::model::{I32ValueCondition, PlaceProjection, StorageOrigin};
use super::report::ResourceOwnerOperation;
use super::summary_index::{FunctionSummary, SummaryIndex};
use crate::types::TypeId;

pub(super) use super::borrow_summary::compute_borrow_token_return_summaries;
pub(super) use super::owner_summary::compute_owner_return_summaries_with_recomputations;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BorrowTokenReturnSummary {
    pub(super) function: String,
    pub(super) parameter_indices: Vec<usize>,
}

pub(super) type BorrowTokenReturnSummaryIndex<'a> = SummaryIndex<'a, BorrowTokenReturnSummary>;

impl FunctionSummary for BorrowTokenReturnSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerReturnSummary {
    pub(super) function: String,
    pub(super) type_params: Vec<TypeId>,
    pub(super) parameter_indices: Vec<usize>,
    pub(super) parameter_sources: Vec<OwnerProjectionSource>,
    pub(super) parameter_return_extents: Vec<OwnerParameterReturnExtent>,
    pub(super) consumed_parameter_indices: Vec<usize>,
    pub(super) consumed_parameter_sources: Vec<OwnerProjectionSource>,
    pub(super) consumed_extent_requirements: Vec<OwnerConsumedExtentRequirement>,
    pub(super) memory_span_requirements: Vec<OwnerMemorySpanRequirement>,
    pub(super) host_size_returns: Vec<OwnerHostSizeReturn>,
    pub(super) type_size_returns: Vec<OwnerTypeSizeReturn>,
    pub(super) variant_consumed_parameter_indices: Vec<OwnerVariantParameterIndex>,
    pub(super) variant_consumed_parameter_sources: Vec<OwnerVariantProjectionSource>,
    pub(super) variant_consumed_extent_requirements: Vec<OwnerVariantConsumedExtentRequirement>,
    pub(super) variant_projection_returns: Vec<OwnerVariantProjectionReturn>,
    pub(super) resolved_parameter_variants: Vec<OwnerResolvedParameterVariant>,
    pub(super) variant_conditions: Vec<OwnerVariantCondition>,
    pub(super) variant_payload_conditions: Vec<OwnerVariantPayloadCondition>,
    pub(super) non_owning_raw_view_returns: Vec<OwnerNonOwningRawViewReturn>,
    pub(super) returns_fresh_owner: bool,
    pub(super) returns_fresh_owner_extent: OwnerExtentSummary,
    pub(super) returns_maybe_owner: bool,
    pub(super) projection_returns: Vec<OwnerProjectionReturnSummary>,
    pub(super) projection_markers: Vec<OwnerProjectionMarker>,
    pub(super) storage_origin_markers: Vec<OwnerStorageOriginMarker>,
}

pub(super) type OwnerReturnSummaryIndex<'a> = SummaryIndex<'a, OwnerReturnSummary>;

impl FunctionSummary for OwnerReturnSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerProjectionSource {
    pub(super) parameter_index: usize,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum OwnerExtentSummary {
    Unknown,
    PayloadBytesParameter(OwnerProjectionSource),
    PayloadBytesParameterScaled {
        source: OwnerProjectionSource,
        scale: usize,
    },
    PayloadBytesParameterTypeSize {
        source: OwnerProjectionSource,
        element_ty: TypeId,
    },
    PayloadBytesI32Constant {
        value: i32,
        ty: TypeId,
    },
    RegionTokenSize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerConsumedExtentRequirement {
    pub(super) owner: OwnerProjectionSource,
    pub(super) extent: OwnerExtentSummary,
    pub(super) operation: ResourceOwnerOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerMemorySpanRequirement {
    pub(super) span: HostMemorySpan,
    pub(super) args: Vec<OwnerMemoryArgSummary>,
    pub(super) operation: ResourceOwnerOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum OwnerMemoryArgSummary {
    Unknown { ty: TypeId },
    Parameter(OwnerProjectionSource),
    I32Constant { value: i32, ty: TypeId },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerHostSizeReturn {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) kind: HostSizeKind,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerTypeSizeReturn {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) element_ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerParameterReturnExtent {
    pub(super) source: OwnerProjectionSource,
    pub(super) extent: OwnerExtentSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerVariantConsumedExtentRequirement {
    pub(super) variant: String,
    pub(super) owner: OwnerProjectionSource,
    pub(super) extent: OwnerExtentSummary,
    pub(super) operation: ResourceOwnerOperation,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerVariantParameterIndex {
    pub(super) variant: String,
    pub(super) parameter_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerVariantProjectionSource {
    pub(super) variant: String,
    pub(super) source: OwnerProjectionSource,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerVariantProjectionReturn {
    pub(super) variant: String,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) owner: OwnerProjectionReturnOwner,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum OwnerProjectionReturnOwner {
    Parameter {
        source: OwnerProjectionSource,
        returned_extent: OwnerExtentSummary,
    },
    Fresh {
        extent: OwnerExtentSummary,
    },
    UnknownSource {
        extent: OwnerExtentSummary,
    },
    Maybe,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerResolvedParameterVariant {
    pub(super) parameter_index: usize,
    pub(super) variant: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerVariantCondition {
    pub(super) variant: String,
    pub(super) condition: OwnerValueCondition,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum OwnerValueCondition {
    Always,
    Param {
        source: OwnerProjectionSource,
        condition: I32ValueCondition,
    },
    Any(Vec<OwnerValueCondition>),
    All(Vec<OwnerValueCondition>),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerVariantPayloadCondition {
    pub(super) variant: String,
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) condition: I32ValueCondition,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerProjectionMarker {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerNonOwningRawViewReturn {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) kind: OwnerNonOwningRawViewKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum OwnerNonOwningRawViewKind {
    AliasView,
    ProjectionView,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerStorageOriginMarker {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) origin: StorageOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct OwnerProjectionReturnSummary {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) parameter_indices: Vec<usize>,
    pub(super) parameter_sources: Vec<OwnerProjectionSource>,
    pub(super) parameter_return_extents: Vec<OwnerParameterReturnExtent>,
    pub(super) returns_fresh_owner: bool,
    pub(super) returns_fresh_owner_extent: OwnerExtentSummary,
    pub(super) returns_maybe_owner: bool,
}
