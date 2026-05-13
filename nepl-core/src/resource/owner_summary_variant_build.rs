use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceFunction, ResourceOp};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_summary_leaf::owner_seed_leaf_places;
use super::owner_summary_record::{OwnerParameterConditionSource, OwnerParameterStorageSource};
use super::owner_summary_variant_paths::collect_variant_consumed_owner_parameters_from_nested_return;
use super::owner_variant::PendingVariantOwnerEffects;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceOwnerCheckDeferred;
use super::storage_origin::StorageOriginTable;
use super::summary::{
    OwnerReturnSummary, OwnerVariantCondition, OwnerVariantConsumedExtentRequirement,
    OwnerVariantParameterIndex, OwnerVariantPayloadCondition, OwnerVariantProjectionReturn,
    OwnerVariantProjectionSource,
};

pub(super) fn collect_variant_consumed_owner_parameters_from_return(
    index_out: &mut Vec<OwnerVariantParameterIndex>,
    source_out: &mut Vec<OwnerVariantProjectionSource>,
    extent_out: &mut Vec<OwnerVariantConsumedExtentRequirement>,
    condition_out: &mut Vec<OwnerVariantCondition>,
    payload_condition_out: &mut Vec<OwnerVariantPayloadCondition>,
    function: &ResourceFunction,
    types: &TypeCtx,
    summaries: &[OwnerReturnSummary],
    parameter_storage_sources: &[OwnerParameterStorageSource],
    parameter_condition_sources: &[OwnerParameterConditionSource],
    ops: &[ResourceOp],
    return_value: &Place,
    return_out: &mut Vec<OwnerVariantProjectionReturn>,
) {
    let engine = ResourceOwnerCheckEngine {
        function: function.name.as_str(),
        types,
        summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
        owner_extent_requirements: Vec::new(),
    };
    let mut owners = OwnerTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let raw_views = RawAddressViewTable::default();
    let mut storage_origins = StorageOriginTable::default();
    let function_aliases = FunctionAliasTable::default();
    let pending_reallocs = PendingRawReallocs::default();
    let variant_owner_effects = PendingVariantOwnerEffects::default();
    for (index, param) in function.params.iter().enumerate() {
        for leaf in owner_seed_leaf_places(types, function, index, &param.place) {
            owners.allocate(&leaf.place);
            raw_aliases.mark(&leaf.place);
            storage_origins.mark_owned(&leaf.place);
        }
    }

    collect_variant_consumed_owner_parameters_from_nested_return(
        index_out,
        source_out,
        extent_out,
        condition_out,
        payload_condition_out,
        &engine,
        &owners,
        &raw_aliases,
        &raw_views,
        &storage_origins,
        &function_aliases,
        &pending_reallocs,
        &variant_owner_effects,
        parameter_storage_sources,
        parameter_condition_sources,
        ops,
        return_value,
        return_out,
    );
}
