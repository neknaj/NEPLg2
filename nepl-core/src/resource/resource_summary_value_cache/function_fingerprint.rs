use crate::types::TypeCtx;

use super::super::model::ResourceFunction;
use super::super::owner_summary_type_params::owner_summary_type_params;
use super::body_hash::resource_function_body_hash;
use super::context::ResourceSummaryValueCacheContext;
use super::key::ResourceSummaryFunctionIdentity;
use super::type_boundary::{
    resource_summary_generic_type_argument_hash, resource_summary_type_parameter_boundary_hash,
};

/// Resource summary cache が関数の「局所的な証明入力」を対応付けるための fingerprint。
///
/// dependency closure は呼び出し関係から別に閉じるため、ここには関数自身の identity、
/// Resource IR body、型境界、generic 境界、source capability policy だけを入れる。
/// `TypeId`、`Span`、`SourceMap` は長寿命 snapshot に保存しない。
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct ResourceFunctionLocalFingerprint {
    identity: ResourceSummaryFunctionIdentity,
    body_hash: u64,
    type_parameter_boundary_hash: u64,
    generic_type_argument_hash: u64,
    source_capability_policy_hash: u64,
}

impl ResourceFunctionLocalFingerprint {
    pub(super) fn from_function(
        context: &ResourceSummaryValueCacheContext,
        types: &TypeCtx,
        function: &ResourceFunction,
    ) -> Option<Self> {
        let identity = ResourceSummaryFunctionIdentity::from_resource_function(function)?;
        let type_params = owner_summary_type_params(types, function);
        let source_capability_policy_hash = context
            .source_capability_policy_hash_for_function(function)?
            .as_u64();
        Some(Self {
            identity,
            body_hash: resource_function_body_hash(types, function)?,
            type_parameter_boundary_hash: resource_summary_type_parameter_boundary_hash(
                types,
                &type_params,
            )?,
            generic_type_argument_hash: resource_summary_generic_type_argument_hash(types, &[])?,
            source_capability_policy_hash,
        })
    }

    pub(super) fn same_identity(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}
