use alloc::vec::Vec;

use crate::span::Span;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceConditionFact, ResourceMatchArm, ResourceOp};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_summary_consumed::consumed_owner_parameters;
use super::owner_summary_host_size_return::record_host_size_returns;
use super::owner_summary_record::{OwnerParameterConditionSource, OwnerParameterStorageSource};
use super::owner_summary_type_size_return::record_type_size_returns;
use super::owner_summary_variant_conditions::{
    collect_owner_variant_known_payload_conditions, collect_owner_variant_payload_conditions,
};
use super::owner_summary_variant_construct::construct_variant_for_value;
use super::owner_summary_variant_path_conditions::record_owner_variant_path_condition;
use super::owner_summary_variant_return::record_variant_projection_returns;
use super::owner_summary_variant_return_sources::returned_owner_returns_for_value;
use super::owner_variant::PendingVariantOwnerEffects;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceOwnerCheckDeferred;
use super::storage_origin::StorageOriginTable;
use super::summary::{
    OwnerHostSizeReturn, OwnerTypeSizeReturn, OwnerVariantCondition,
    OwnerVariantConsumedExtentRequirement, OwnerVariantParameterIndex,
    OwnerVariantPayloadCondition, OwnerVariantProjectionReturn, OwnerVariantProjectionSource,
};

pub(super) fn collect_variant_consumed_owner_parameters_from_nested_return(
    index_out: &mut Vec<OwnerVariantParameterIndex>,
    source_out: &mut Vec<OwnerVariantProjectionSource>,
    extent_out: &mut Vec<OwnerVariantConsumedExtentRequirement>,
    condition_out: &mut Vec<OwnerVariantCondition>,
    payload_condition_out: &mut Vec<OwnerVariantPayloadCondition>,
    engine: &ResourceOwnerCheckEngine<'_>,
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    raw_views: &RawAddressViewTable,
    storage_origins: &StorageOriginTable,
    function_aliases: &FunctionAliasTable,
    pending_reallocs: &PendingRawReallocs,
    variant_owner_effects: &PendingVariantOwnerEffects,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    parameter_condition_sources: &[OwnerParameterConditionSource],
    ops: &[ResourceOp],
    return_value: &Place,
    host_size_out: &mut Vec<OwnerHostSizeReturn>,
    type_size_out: &mut Vec<OwnerTypeSizeReturn>,
    return_out: &mut Vec<OwnerVariantProjectionReturn>,
) {
    let mut engine = ResourceOwnerCheckEngine {
        function: engine.function,
        types: engine.types,
        summaries: engine.summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
        owner_extent_requirements: engine.owner_extent_requirements.clone(),
        memory_span_requirements: engine.memory_span_requirements.clone(),
        params: engine.params,
    };
    let mut owners = owners.clone();
    let mut raw_aliases = raw_aliases.clone();
    let mut raw_views = raw_views.clone();
    let mut storage_origins = storage_origins.clone();
    let mut function_aliases = function_aliases.clone();
    let mut pending_reallocs = pending_reallocs.clone();
    let mut variant_owner_effects = variant_owner_effects.clone();
    for (index, op) in ops.iter().enumerate() {
        match op {
            ResourceOp::Branch {
                output,
                condition_fact,
                then_ops,
                then_value,
                else_ops,
                else_value,
                span,
                ..
            } if output == return_value => {
                let mut then_owners = owners.clone();
                let mut then_raw_aliases = raw_aliases.clone();
                let mut then_storage_origins = storage_origins.clone();
                let mut then_pending_reallocs = pending_reallocs.clone();
                engine.apply_branch_condition_fact(
                    &mut then_owners,
                    &mut then_raw_aliases,
                    &raw_views,
                    &mut then_storage_origins,
                    &mut then_pending_reallocs,
                    condition_fact.as_ref(),
                    true,
                    *span,
                );
                collect_variant_consumed_owner_parameters_from_path(
                    index_out,
                    source_out,
                    extent_out,
                    condition_out,
                    payload_condition_out,
                    &engine,
                    &then_owners,
                    &then_raw_aliases,
                    &raw_views,
                    &then_storage_origins,
                    &function_aliases,
                    &then_pending_reallocs,
                    &variant_owner_effects,
                    parameter_storage_sources,
                    parameter_condition_sources,
                    then_ops,
                    then_value,
                    condition_fact.as_ref().map(|fact| (fact, true)),
                    None,
                    host_size_out,
                    type_size_out,
                    return_out,
                );
                let mut else_owners = owners.clone();
                let mut else_raw_aliases = raw_aliases.clone();
                let mut else_storage_origins = storage_origins.clone();
                let mut else_pending_reallocs = pending_reallocs.clone();
                engine.apply_branch_condition_fact(
                    &mut else_owners,
                    &mut else_raw_aliases,
                    &raw_views,
                    &mut else_storage_origins,
                    &mut else_pending_reallocs,
                    condition_fact.as_ref(),
                    false,
                    *span,
                );
                collect_variant_consumed_owner_parameters_from_path(
                    index_out,
                    source_out,
                    extent_out,
                    condition_out,
                    payload_condition_out,
                    &engine,
                    &else_owners,
                    &else_raw_aliases,
                    &raw_views,
                    &else_storage_origins,
                    &function_aliases,
                    &else_pending_reallocs,
                    &variant_owner_effects,
                    parameter_storage_sources,
                    parameter_condition_sources,
                    else_ops,
                    else_value,
                    condition_fact.as_ref().map(|fact| (fact, false)),
                    None,
                    host_size_out,
                    type_size_out,
                    return_out,
                );
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                span,
                ..
            } if output == return_value => {
                for arm in arms {
                    if !variant_owner_effects.match_arm_reachable(scrutinee, &arm.pattern) {
                        continue;
                    }
                    collect_variant_consumed_owner_parameters_from_path(
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
                        &arm.ops,
                        &arm.value,
                        None,
                        Some((scrutinee, arm, *span)),
                        host_size_out,
                        type_size_out,
                        return_out,
                    );
                }
            }
            _ => {}
        }
        engine.check_ops(
            &mut owners,
            &mut function_aliases,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &mut pending_reallocs,
            &mut variant_owner_effects,
            &ops[index..=index],
        );
    }
    variant_owner_effects.collect_result_owner_effect_summaries(
        &engine,
        &owners,
        &raw_aliases,
        &raw_views,
        return_value,
        parameter_storage_sources,
        parameter_condition_sources,
        index_out,
        source_out,
        extent_out,
        return_out,
    );
}

fn collect_variant_consumed_owner_parameters_from_path(
    index_out: &mut Vec<OwnerVariantParameterIndex>,
    source_out: &mut Vec<OwnerVariantProjectionSource>,
    extent_out: &mut Vec<OwnerVariantConsumedExtentRequirement>,
    condition_out: &mut Vec<OwnerVariantCondition>,
    payload_condition_out: &mut Vec<OwnerVariantPayloadCondition>,
    engine: &ResourceOwnerCheckEngine<'_>,
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    raw_views: &RawAddressViewTable,
    storage_origins: &StorageOriginTable,
    function_aliases: &FunctionAliasTable,
    pending_reallocs: &PendingRawReallocs,
    variant_owner_effects: &PendingVariantOwnerEffects,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    parameter_condition_sources: &[OwnerParameterConditionSource],
    path_ops: &[ResourceOp],
    path_value: &Place,
    branch_condition: Option<(&ResourceConditionFact, bool)>,
    match_arm: Option<(&Place, &ResourceMatchArm, Span)>,
    host_size_out: &mut Vec<OwnerHostSizeReturn>,
    type_size_out: &mut Vec<OwnerTypeSizeReturn>,
    return_out: &mut Vec<OwnerVariantProjectionReturn>,
) {
    let mut path_engine = ResourceOwnerCheckEngine {
        function: engine.function,
        types: engine.types,
        summaries: engine.summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
        owner_extent_requirements: engine.owner_extent_requirements.clone(),
        memory_span_requirements: engine.memory_span_requirements.clone(),
        params: engine.params,
    };
    let mut path_owners = owners.clone();
    let mut path_raw_aliases = raw_aliases.clone();
    let mut path_raw_views = raw_views.clone();
    let mut path_storage_origins = storage_origins.clone();
    let mut path_function_aliases = function_aliases.clone();
    let mut path_pending_reallocs = pending_reallocs.clone();
    let mut path_variant_owner_effects = variant_owner_effects.clone();
    super::owner_summary_variant_match::apply_match_arm_entry(
        &mut path_engine,
        &mut path_owners,
        &mut path_raw_aliases,
        &mut path_raw_views,
        &mut path_storage_origins,
        &mut path_function_aliases,
        &mut path_pending_reallocs,
        &mut path_variant_owner_effects,
        match_arm,
    );
    let Some(constructed_variant) = construct_variant_for_value(path_ops, path_value) else {
        collect_variant_consumed_owner_parameters_from_nested_return(
            index_out,
            source_out,
            extent_out,
            condition_out,
            payload_condition_out,
            &path_engine,
            &path_owners,
            &path_raw_aliases,
            &path_raw_views,
            &path_storage_origins,
            &path_function_aliases,
            &path_pending_reallocs,
            &path_variant_owner_effects,
            parameter_storage_sources,
            parameter_condition_sources,
            path_ops,
            path_value,
            host_size_out,
            type_size_out,
            return_out,
        );
        return;
    };
    path_engine.check_ops(
        &mut path_owners,
        &mut path_function_aliases,
        &mut path_raw_aliases,
        &mut path_raw_views,
        &mut path_storage_origins,
        &mut path_pending_reallocs,
        &mut path_variant_owner_effects,
        path_ops,
    );
    record_owner_variant_path_condition(
        condition_out,
        variant_owner_effects,
        parameter_condition_sources,
        &constructed_variant.variant,
        raw_aliases,
        &path_raw_aliases,
        branch_condition,
        match_arm,
    );
    if let Some((condition_fact, truthy_path)) = branch_condition {
        collect_owner_variant_payload_conditions(
            payload_condition_out,
            path_engine.types,
            &constructed_variant,
            path_value,
            condition_fact,
            truthy_path,
            &path_raw_aliases,
        );
    }
    collect_owner_variant_known_payload_conditions(
        payload_condition_out,
        path_engine.types,
        &constructed_variant,
        path_value,
        &path_raw_aliases,
    );
    record_host_size_returns(host_size_out, &path_raw_aliases, path_value);
    record_type_size_returns(type_size_out, &path_raw_aliases, path_value);

    let (projection_returns, returned_sources) = returned_owner_returns_for_value(
        &path_owners,
        &path_raw_aliases,
        path_value,
        parameter_storage_sources,
        parameter_condition_sources,
    );
    let ambiguous_return_sources = record_variant_projection_returns(
        return_out,
        path_engine.types,
        path_value.ty,
        &constructed_variant.variant,
        &projection_returns,
        parameter_storage_sources,
    );
    let (indices, sources) =
        consumed_owner_parameters(&path_owners, parameter_storage_sources, &returned_sources);
    let extent_requirements = super::owner_extent::summarize_consumed_extent_requirements(
        &path_raw_aliases,
        parameter_storage_sources,
        parameter_condition_sources,
        &path_engine.owner_extent_requirements,
        &indices,
        &sources,
    );
    let variant = super::variant_name::normalize_variant_name(&constructed_variant.variant);
    for parameter_index in indices {
        push_unique_variant_parameter_index(
            index_out,
            OwnerVariantParameterIndex {
                variant: variant.clone(),
                parameter_index,
            },
        );
    }
    for source in sources {
        push_unique_variant_projection_source(
            source_out,
            OwnerVariantProjectionSource {
                variant: variant.clone(),
                source,
            },
        );
    }
    for source in ambiguous_return_sources {
        push_unique_variant_projection_source(
            source_out,
            OwnerVariantProjectionSource {
                variant: variant.clone(),
                source,
            },
        );
    }
    for requirement in extent_requirements {
        push_or_merge_variant_extent_requirement(
            extent_out,
            OwnerVariantConsumedExtentRequirement {
                variant: variant.clone(),
                owner: requirement.owner,
                extent: requirement.extent,
                operation: requirement.operation,
            },
        );
    }
}

fn push_unique_variant_parameter_index(
    out: &mut Vec<OwnerVariantParameterIndex>,
    entry: OwnerVariantParameterIndex,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

fn push_unique_variant_projection_source(
    out: &mut Vec<OwnerVariantProjectionSource>,
    entry: OwnerVariantProjectionSource,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

fn push_or_merge_variant_extent_requirement(
    out: &mut Vec<OwnerVariantConsumedExtentRequirement>,
    entry: OwnerVariantConsumedExtentRequirement,
) {
    if let Some(existing) = out.iter_mut().find(|existing| {
        existing.variant == entry.variant
            && existing.owner == entry.owner
            && existing.operation == entry.operation
    }) {
        existing.extent = super::owner_extent::merge_owner_extent_summaries(
            existing.extent.clone(),
            entry.extent,
        );
        return;
    }
    out.push(entry);
}
