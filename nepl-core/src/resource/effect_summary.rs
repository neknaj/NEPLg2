use alloc::string::String;
use alloc::vec::Vec;

use super::effect_identity::RawIdentityOrigin;
use super::model::PlaceProjection;
use super::summary_index::{FunctionSummary, SummaryIndex};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawIdentityReturnSummary {
    pub(super) function: String,
    pub(super) parameter_returns: Vec<RawIdentityParameterReturn>,
    pub(super) internal_alloc_returns: Vec<RawIdentityReturnProjection>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawIdentityParameterReturn {
    pub(super) parameter_index: usize,
    pub(super) source_projections: Vec<PlaceProjection>,
    pub(super) source_ty: crate::types::TypeId,
    pub(super) return_projections: Vec<PlaceProjection>,
    pub(super) return_ty: crate::types::TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawIdentityReturnProjection {
    pub(super) projections: Vec<PlaceProjection>,
    pub(super) ty: crate::types::TypeId,
    pub(super) origins: Vec<RawIdentityOrigin>,
}

pub(super) type RawIdentityReturnSummaryIndex<'a> = SummaryIndex<'a, RawIdentityReturnSummary>;

impl FunctionSummary for RawIdentityReturnSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawPointerReturnSummary {
    pub(super) function: String,
    pub(super) parameter_returns: Vec<RawPointerParameterReturn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawPointerParameterReturn {
    pub(super) parameter_index: usize,
    pub(super) source_projections: Vec<PlaceProjection>,
    pub(super) source_ty: crate::types::TypeId,
    pub(super) return_projections: Vec<PlaceProjection>,
    pub(super) return_ty: crate::types::TypeId,
}

pub(super) type RawPointerReturnSummaryIndex<'a> = SummaryIndex<'a, RawPointerReturnSummary>;

impl FunctionSummary for RawPointerReturnSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}
