use alloc::vec::Vec;

use crate::span::Span;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceConditionFact, ResourceMatchArm, ResourceOp};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_control::OwnerMatchPathState;
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
use super::owner_summary_variant_payload_conditions::OwnerVariantPayloadConditionAccumulator;
use super::owner_summary_variant_profile::{OwnerVariantProfilePhase, OwnerVariantReturnProfile};
use super::owner_summary_variant_return::record_variant_projection_returns;
use super::owner_summary_variant_return_sources::returned_owner_returns_for_value;
use super::owner_variant::PendingVariantOwnerEffects;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceOwnerCheckDeferred;
use super::storage_origin::StorageOriginTable;
use super::summary::{
    OwnerHostSizeReturn, OwnerTypeSizeReturn, OwnerVariantCondition,
    OwnerVariantConsumedExtentRequirement, OwnerVariantParameterIndex,
    OwnerVariantProjectionReturn, OwnerVariantProjectionSource,
};
use super::variant_name::normalize_variant_name;

pub(super) fn collect_variant_consumed_owner_parameters_from_nested_return(
    index_out: &mut Vec<OwnerVariantParameterIndex>,
    source_out: &mut Vec<OwnerVariantProjectionSource>,
    extent_out: &mut Vec<OwnerVariantConsumedExtentRequirement>,
    condition_out: &mut Vec<OwnerVariantCondition>,
    payload_condition_out: &mut OwnerVariantPayloadConditionAccumulator,
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
    profile: &mut OwnerVariantReturnProfile,
    depth: usize,
) -> OwnerMatchPathState {
    profile.observe_nested(depth);
    let payload_observations_before = payload_condition_out.observation_count();
    let state_clone_timer = profile.start();
    let mut engine = ResourceOwnerCheckEngine {
        function: engine.function,
        types: engine.types,
        summaries: engine.summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
        owner_extent_requirements: engine.owner_extent_requirements.clone(),
        memory_span_requirements: engine.memory_span_requirements.clone(),
        params: engine.params,
        owner_leaf_projection_cache: Default::default(),
    };
    let mut owners = owners.clone();
    let mut raw_aliases = raw_aliases.clone();
    let mut raw_views = raw_views.clone();
    let mut storage_origins = storage_origins.clone();
    let mut function_aliases = function_aliases.clone();
    let mut pending_reallocs = pending_reallocs.clone();
    let mut variant_owner_effects = variant_owner_effects.clone();
    profile.finish(state_clone_timer, OwnerVariantProfilePhase::StateClone);
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
                profile.observe_branch_fork();
                let branch_fork_timer = profile.start();
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
                profile.finish(branch_fork_timer, OwnerVariantProfilePhase::BranchFork);
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
                    profile,
                    depth + 1,
                );
                profile.observe_branch_fork();
                let branch_fork_timer = profile.start();
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
                profile.finish(branch_fork_timer, OwnerVariantProfilePhase::BranchFork);
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
                    profile,
                    depth + 1,
                );
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                span,
                ..
            } if output == return_value => {
                if profile.is_enabled() {
                    profile.observe_pending_effects(
                        variant_owner_effects.profile_result_effects(
                            &raw_aliases,
                            output,
                            scrutinee,
                        ),
                    );
                }
                for arm in arms {
                    if !variant_owner_effects.match_arm_reachable(scrutinee, &arm.pattern) {
                        continue;
                    }
                    profile.observe_match_arm();
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
                        profile,
                        depth + 1,
                    );
                }
            }
            _ => {}
        }
        profile.observe_sequential_replay(1);
        let replay_timer = profile.start();
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
        profile.finish_sequential_replay(replay_timer, op, return_value, depth);
    }
    let terminal_timer = profile.start();
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
    profile.finish(terminal_timer, OwnerVariantProfilePhase::Terminal);
    if payload_condition_out.observation_count() == payload_observations_before {
        payload_condition_out.observe_unknown_path();
    }
    OwnerMatchPathState {
        owners,
        function_aliases,
        raw_aliases,
        raw_views,
        storage_origins,
        pending_reallocs,
        variant_owner_effects,
    }
}

fn collect_variant_consumed_owner_parameters_from_path(
    index_out: &mut Vec<OwnerVariantParameterIndex>,
    source_out: &mut Vec<OwnerVariantProjectionSource>,
    extent_out: &mut Vec<OwnerVariantConsumedExtentRequirement>,
    condition_out: &mut Vec<OwnerVariantCondition>,
    payload_condition_out: &mut OwnerVariantPayloadConditionAccumulator,
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
    profile: &mut OwnerVariantReturnProfile,
    depth: usize,
) -> OwnerMatchPathState {
    profile.observe_path(depth);
    let state_clone_timer = profile.start();
    let mut path_engine = ResourceOwnerCheckEngine {
        function: engine.function,
        types: engine.types,
        summaries: engine.summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
        owner_extent_requirements: engine.owner_extent_requirements.clone(),
        memory_span_requirements: engine.memory_span_requirements.clone(),
        params: engine.params,
        owner_leaf_projection_cache: Default::default(),
    };
    let mut path_owners = owners.clone();
    let mut path_raw_aliases = raw_aliases.clone();
    let mut path_raw_views = raw_views.clone();
    let mut path_storage_origins = storage_origins.clone();
    let mut path_function_aliases = function_aliases.clone();
    let mut path_pending_reallocs = pending_reallocs.clone();
    let mut path_variant_owner_effects = variant_owner_effects.clone();
    profile.finish(state_clone_timer, OwnerVariantProfilePhase::StateClone);
    let match_entry_timer = profile.start();
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
    profile.finish(match_entry_timer, OwnerVariantProfilePhase::MatchEntry);
    let Some(constructed_variant) = construct_variant_for_value(path_ops, path_value) else {
        profile.observe_recursive_path();
        return collect_variant_consumed_owner_parameters_from_nested_return(
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
            profile,
            depth,
        );
    };
    profile.observe_constructed_path();
    profile.observe_path_replay(path_ops.len());
    let path_replay_timer = profile.start();
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
    profile.finish(path_replay_timer, OwnerVariantProfilePhase::PathReplay);
    let terminal_timer = profile.start();
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
    let mut path_payload_conditions = Vec::new();
    if let Some((condition_fact, truthy_path)) = branch_condition {
        collect_owner_variant_payload_conditions(
            &mut path_payload_conditions,
            path_engine.types,
            &constructed_variant,
            path_value,
            condition_fact,
            truthy_path,
            &path_raw_aliases,
        );
    }
    collect_owner_variant_known_payload_conditions(
        &mut path_payload_conditions,
        path_engine.types,
        &constructed_variant,
        path_value,
        &path_raw_aliases,
    );
    payload_condition_out.merge_path(
        normalize_variant_name(&constructed_variant.variant),
        path_payload_conditions,
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
    profile.finish(terminal_timer, OwnerVariantProfilePhase::Terminal);
    OwnerMatchPathState {
        owners: path_owners,
        function_aliases: path_function_aliases,
        raw_aliases: path_raw_aliases,
        raw_views: path_raw_views,
        storage_origins: path_storage_origins,
        pending_reallocs: path_pending_reallocs,
        variant_owner_effects: path_variant_owner_effects,
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

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec::Vec;

    use crate::types::{TypeCtx, TypeId};

    use super::*;
    use crate::resource::model::{AggregateKind, StorageOrigin};
    use crate::resource::summary::OwnerReturnSummaryIndex;

    fn run_path(path_ops: &[ResourceOp], path_value: &Place) -> OwnerMatchPathState {
        let types = TypeCtx::new();
        let summaries = Vec::new();
        let summary_index = OwnerReturnSummaryIndex::new(&summaries);
        let engine = ResourceOwnerCheckEngine {
            function: "variant_path_oracle",
            types: &types,
            summaries: &summary_index,
            diagnostics: Vec::new(),
            deferred: ResourceOwnerCheckDeferred::default(),
            owner_extent_requirements: Vec::new(),
            memory_span_requirements: Vec::new(),
            params: &[],
            owner_leaf_projection_cache: Default::default(),
        };
        let mut index_out = Vec::new();
        let mut source_out = Vec::new();
        let mut extent_out = Vec::new();
        let mut condition_out = Vec::new();
        let mut payload_condition_out = OwnerVariantPayloadConditionAccumulator::default();
        let mut host_size_out = Vec::new();
        let mut type_size_out = Vec::new();
        let mut return_out = Vec::new();
        let mut profile = OwnerVariantReturnProfile::new("variant_path_oracle");

        collect_variant_consumed_owner_parameters_from_path(
            &mut index_out,
            &mut source_out,
            &mut extent_out,
            &mut condition_out,
            &mut payload_condition_out,
            &engine,
            &OwnerTable::default(),
            &RawCellAddressAliases::default(),
            &RawAddressViewTable::default(),
            &StorageOriginTable::default(),
            &FunctionAliasTable::default(),
            &PendingRawReallocs::default(),
            &PendingVariantOwnerEffects::default(),
            &[],
            &[],
            path_ops,
            path_value,
            None,
            None,
            &mut host_size_out,
            &mut type_size_out,
            &mut return_out,
            &mut profile,
            0,
        )
    }

    #[test]
    fn constructed_path_returns_state_after_final_replay_op() {
        let value = Place::local("value".to_string(), TypeId(0));
        let retained = Place::local("retained".to_string(), TypeId(0));
        let span = Span::dummy();
        let state = run_path(
            &[
                ResourceOp::Construct {
                    output: value.clone(),
                    kind: AggregateKind::Enum {
                        name: "Result".to_string(),
                        variant: "Ok".to_string(),
                    },
                    inputs: Vec::new(),
                    span,
                },
                ResourceOp::StorageOrigin {
                    target: retained.clone(),
                    origin: StorageOrigin::Owned,
                    span,
                },
            ],
            &value,
        );

        assert_eq!(
            state.storage_origins.origin(&retained),
            Some(StorageOrigin::Owned)
        );
    }

    #[test]
    fn recursive_path_returns_state_after_final_replay_op() {
        let value = Place::local("value".to_string(), TypeId(0));
        let retained = Place::local("retained".to_string(), TypeId(0));
        let span = Span::dummy();
        let state = run_path(
            &[ResourceOp::StorageOrigin {
                target: retained.clone(),
                origin: StorageOrigin::Owned,
                span,
            }],
            &value,
        );

        assert_eq!(
            state.storage_origins.origin(&retained),
            Some(StorageOrigin::Owned)
        );
    }
}
