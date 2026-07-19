use alloc::vec::Vec;
#[cfg(test)]
use alloc::vec;

use crate::span::Span;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceConditionFact, ResourceMatchArm, ResourceOp};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_control::OwnerMatchPathState;
#[cfg(test)]
use super::owner_control::OwnerMatchEngineEffectAccumulator;
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
#[cfg(test)]
use super::summary::{OwnerValueCondition, OwnerVariantPayloadCondition};
#[cfg(test)]
use super::owner_summary_canonicalize::canonicalize_variant_summary_channels;
use super::variant_name::normalize_variant_name;

pub(super) struct OwnerVariantTraversalResult {
    pub(super) state: OwnerMatchPathState,
    #[cfg(test)]
    controls: Vec<OwnerVariantControlPaths>,
    #[cfg(test)]
    merge_eligible: bool,
    #[cfg(test)]
    engine_effects: Option<OwnerMatchEngineEffectAccumulator>,
}

#[cfg(test)]
impl OwnerVariantTraversalResult {
    fn engine_effects(&self) -> &OwnerMatchEngineEffectAccumulator {
        self.engine_effects
            .as_ref()
            .expect("match engine effects were already transferred")
    }

    fn take_engine_effects(&mut self) -> OwnerMatchEngineEffectAccumulator {
        self.engine_effects
            .take()
            .expect("match engine effects were already transferred")
    }
}

#[cfg(test)]
struct OwnerVariantControlPaths {
    op_index: usize,
    kind: OwnerVariantControlKind,
    paths: Vec<OwnerVariantTraversalPath>,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OwnerVariantControlKind {
    Branch,
    Match,
}

#[cfg(test)]
struct OwnerVariantTraversalPath {
    selector: OwnerVariantPathSelector,
    result: OwnerVariantTraversalEvidence,
}

#[cfg(test)]
struct OwnerVariantTraversalEvidence {
    state_snapshot: super::owner_control::OwnerMatchOracleSnapshot,
    controls: Vec<OwnerVariantControlPaths>,
    merge_eligible: bool,
    effect_authority_transferred: bool,
}

#[cfg(test)]
impl OwnerVariantTraversalResult {
    fn into_evidence(self) -> OwnerVariantTraversalEvidence {
        OwnerVariantTraversalEvidence {
            state_snapshot: self.state.oracle_snapshot(),
            controls: self.controls,
            merge_eligible: self.merge_eligible,
            effect_authority_transferred: false,
        }
    }
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OwnerVariantPathSelector {
    Then,
    Else,
    MatchArm(usize),
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnerVariantSummarySnapshot {
    indices: Vec<OwnerVariantParameterIndex>,
    sources: Vec<OwnerVariantProjectionSource>,
    extents: Vec<OwnerVariantConsumedExtentRequirement>,
    conditions: Vec<OwnerVariantCondition>,
    payload_conditions: Vec<OwnerVariantPayloadCondition>,
    host_sizes: Vec<OwnerHostSizeReturn>,
    type_sizes: Vec<OwnerTypeSizeReturn>,
    returns: Vec<OwnerVariantProjectionReturn>,
}

#[cfg(test)]
impl OwnerVariantSummarySnapshot {
    fn canonicalize(&mut self) {
        canonicalize_variant_summary_channels(
            &mut self.indices,
            &mut self.sources,
            &mut self.extents,
            &mut self.conditions,
            &mut self.payload_conditions,
            &mut self.host_sizes,
            &mut self.type_sizes,
            &mut self.returns,
        );
    }
}

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
) -> OwnerVariantTraversalResult {
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
    #[cfg(test)]
    let mut controls = Vec::new();
    #[cfg(test)]
    let mut engine_effects = OwnerMatchEngineEffectAccumulator::default();
    #[cfg(test)]
    let mut generic_oracle = None;
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
                let then_result = collect_variant_consumed_owner_parameters_from_path(
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
                let else_result = collect_variant_consumed_owner_parameters_from_path(
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
                    None,
                    host_size_out,
                    type_size_out,
                    return_out,
                    profile,
                    depth + 1,
                );
                #[cfg(test)]
                engine_effects.mark_incomplete();
                #[cfg(test)]
                controls.push(OwnerVariantControlPaths {
                    op_index: index,
                    kind: OwnerVariantControlKind::Branch,
                    paths: vec![
                        OwnerVariantTraversalPath {
                            selector: OwnerVariantPathSelector::Then,
                            result: then_result.into_evidence(),
                        },
                        OwnerVariantTraversalPath {
                            selector: OwnerVariantPathSelector::Else,
                            result: else_result.into_evidence(),
                        },
                    ],
                });
                #[cfg(not(test))]
                let _ = (then_result, else_result);
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
                #[cfg(test)]
                let mut paths = Vec::new();
                #[cfg(test)]
                let mut reachable_paths = 0usize;
                #[cfg(test)]
                let mut merged_paths = super::owner_control::OwnerMatchPathStates::default();
                #[cfg(test)]
                let mut generic_state = OwnerMatchPathState::from_parent(
                    &owners,
                    &function_aliases,
                    &raw_aliases,
                    &raw_views,
                    &storage_origins,
                    &pending_reallocs,
                    &variant_owner_effects,
                );
                #[cfg(test)]
                let mut generic_engine = ResourceOwnerCheckEngine {
                    function: engine.function,
                    types: engine.types,
                    summaries: engine.summaries,
                    diagnostics: engine.diagnostics.clone(),
                    deferred: engine.deferred.clone(),
                    owner_extent_requirements: engine.owner_extent_requirements.clone(),
                    memory_span_requirements: engine.memory_span_requirements.clone(),
                    params: engine.params,
                    owner_leaf_projection_cache: Default::default(),
                };
                for (_arm_index, arm) in arms.iter().enumerate() {
                    if !variant_owner_effects.match_arm_reachable(scrutinee, &arm.pattern) {
                        continue;
                    }
                    profile.observe_match_arm();
                    let result = collect_variant_consumed_owner_parameters_from_path(
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
                        Some(output),
                        host_size_out,
                        type_size_out,
                        return_out,
                        profile,
                        depth + 1,
                    );
                    #[cfg(test)]
                    let mut result = result;
                    #[cfg(test)]
                    let evidence = {
                        reachable_paths += 1;
                        let child_effects = result.take_engine_effects();
                        engine_effects.extend(child_effects);
                        let evidence = OwnerVariantTraversalEvidence {
                            state_snapshot: result.state.oracle_snapshot(),
                            controls: result.controls,
                            merge_eligible: result.merge_eligible,
                            effect_authority_transferred: result.engine_effects.is_none(),
                        };
                        if result.merge_eligible {
                            merged_paths.push(result.state);
                        }
                        evidence
                    };
                    #[cfg(test)]
                    paths.push(OwnerVariantTraversalPath {
                        selector: OwnerVariantPathSelector::MatchArm(_arm_index),
                        result: evidence,
                    });
                    #[cfg(not(test))]
                    let _ = result;
                }
                #[cfg(test)]
                if reachable_paths == 0 {
                    engine_effects.mark_incomplete();
                }
                #[cfg(test)]
                {
                    super::owner_control::merge_match_path_states(
                        &mut owners,
                        &mut function_aliases,
                        &mut raw_aliases,
                        &mut raw_views,
                        &mut storage_origins,
                        &mut pending_reallocs,
                        &mut variant_owner_effects,
                        merged_paths,
                    );
                    generic_engine.check_ops(
                        &mut generic_state.owners,
                        &mut generic_state.function_aliases,
                        &mut generic_state.raw_aliases,
                        &mut generic_state.raw_views,
                        &mut generic_state.storage_origins,
                        &mut generic_state.pending_reallocs,
                        &mut generic_state.variant_owner_effects,
                        &ops[index..=index],
                    );
                    assert_eq!(generic_state.oracle_snapshot(), OwnerMatchPathState::from_parent(
                        &owners,
                        &function_aliases,
                        &raw_aliases,
                        &raw_views,
                        &storage_origins,
                        &pending_reallocs,
                        &variant_owner_effects,
                    ).oracle_snapshot());
                    generic_oracle = Some((generic_engine, generic_state));
                }
                #[cfg(test)]
                controls.push(OwnerVariantControlPaths {
                    op_index: index,
                    kind: OwnerVariantControlKind::Match,
                    paths,
                });
            }
            _ => {}
        }
        #[cfg(test)]
        let specialized_match = matches!(
            op,
            ResourceOp::Match { output, .. } if output == return_value
        );
        #[cfg(test)]
        let captures_sequential_effects = !matches!(
            op,
            ResourceOp::Branch { .. } | ResourceOp::Loop { .. } | ResourceOp::Match { .. }
        );
        #[cfg(test)]
        if !specialized_match && !captures_sequential_effects {
            engine_effects.mark_incomplete();
        }
        #[cfg(test)]
        let replay_checkpoint = engine.match_engine_effect_checkpoint();
        profile.observe_sequential_replay(1);
        let replay_timer = profile.start();
        #[cfg(not(test))]
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
        #[cfg(test)]
        if !specialized_match {
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
            if let Some((oracle_engine, oracle_state)) = generic_oracle.as_mut() {
                oracle_engine.check_ops(
                    &mut oracle_state.owners,
                    &mut oracle_state.function_aliases,
                    &mut oracle_state.raw_aliases,
                    &mut oracle_state.raw_views,
                    &mut oracle_state.storage_origins,
                    &mut oracle_state.pending_reallocs,
                    &mut oracle_state.variant_owner_effects,
                    &ops[index..=index],
                );
                assert_eq!(
                    oracle_state.oracle_snapshot(),
                    OwnerMatchPathState::from_parent(
                        &owners,
                        &function_aliases,
                        &raw_aliases,
                        &raw_views,
                        &storage_origins,
                        &pending_reallocs,
                        &variant_owner_effects,
                    )
                    .oracle_snapshot()
                );
            }
        }
        #[cfg(test)]
        if captures_sequential_effects {
            engine_effects.push(engine.match_engine_effect_delta(replay_checkpoint));
        }
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
    OwnerVariantTraversalResult {
        state: OwnerMatchPathState {
            owners,
            function_aliases,
            raw_aliases,
            raw_views,
            storage_origins,
            pending_reallocs,
            variant_owner_effects,
        },
        #[cfg(test)]
        controls,
        #[cfg(test)]
        merge_eligible: true,
        #[cfg(test)]
        engine_effects: Some(engine_effects),
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
    _match_output: Option<&Place>,
    host_size_out: &mut Vec<OwnerHostSizeReturn>,
    type_size_out: &mut Vec<OwnerTypeSizeReturn>,
    return_out: &mut Vec<OwnerVariantProjectionReturn>,
    profile: &mut OwnerVariantReturnProfile,
    depth: usize,
) -> OwnerVariantTraversalResult {
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
    #[cfg(test)]
    let mut engine_effects = OwnerMatchEngineEffectAccumulator::default();
    #[cfg(test)]
    let entry_checkpoint = path_engine.match_engine_effect_checkpoint();
    #[cfg(test)]
    let mut match_entry_reachable = true;
    profile.finish(state_clone_timer, OwnerVariantProfilePhase::StateClone);
    let match_entry_timer = profile.start();
    #[cfg(not(test))]
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
    #[cfg(test)]
    if let Some((scrutinee, arm, span)) = match_arm {
        if let Some(state) = path_engine.prepare_match_arm_path(
            owners,
            function_aliases,
            raw_aliases,
            raw_views,
            storage_origins,
            pending_reallocs,
            variant_owner_effects,
            scrutinee,
            arm,
            span,
        ) {
            path_owners = state.owners;
            path_function_aliases = state.function_aliases;
            path_raw_aliases = state.raw_aliases;
            path_raw_views = state.raw_views;
            path_storage_origins = state.storage_origins;
            path_pending_reallocs = state.pending_reallocs;
            path_variant_owner_effects = state.variant_owner_effects;
        } else {
            match_entry_reachable = false;
            engine_effects.mark_incomplete();
        }
    }
    #[cfg(test)]
    engine_effects.push(path_engine.match_engine_effect_delta(entry_checkpoint));
    profile.finish(match_entry_timer, OwnerVariantProfilePhase::MatchEntry);
    let Some(constructed_variant) = construct_variant_for_value(path_ops, path_value) else {
        profile.observe_recursive_path();
        let result = collect_variant_consumed_owner_parameters_from_nested_return(
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
        #[cfg(test)]
        let mut result = result;
        #[cfg(test)]
        {
            engine_effects.extend(result.take_engine_effects());
            result.engine_effects = Some(engine_effects);
        }
        #[cfg(test)]
        if match_entry_reachable {
            if let Some(output) = _match_output {
                let finalize_checkpoint = path_engine.match_engine_effect_checkpoint();
                result.merge_eligible = path_engine.finalize_match_arm_value(
                    &mut result.state,
                    output,
                    path_value,
                    match_arm.map(|(_, _, span)| span).unwrap_or_else(Span::dummy),
                );
                result
                    .engine_effects
                    .as_mut()
                    .expect("recursive match engine effects must remain owned by the root")
                    .push(path_engine.match_engine_effect_delta(finalize_checkpoint));
            }
        } else {
            result.merge_eligible = false;
        }
        return result;
    };
    profile.observe_constructed_path();
    profile.observe_path_replay(path_ops.len());
    let path_replay_timer = profile.start();
    #[cfg(test)]
    let leaf_effects_complete = !path_ops.iter().any(|op| {
        matches!(
            op,
            ResourceOp::Branch { .. } | ResourceOp::Loop { .. } | ResourceOp::Match { .. }
        )
    });
    #[cfg(test)]
    let leaf_checkpoint = path_engine.match_engine_effect_checkpoint();
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
    #[cfg(test)]
    if leaf_effects_complete {
        engine_effects.push(path_engine.match_engine_effect_delta(leaf_checkpoint));
    } else {
        engine_effects.mark_incomplete();
    }
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
    let result = OwnerVariantTraversalResult {
        state: OwnerMatchPathState {
            owners: path_owners,
            function_aliases: path_function_aliases,
            raw_aliases: path_raw_aliases,
            raw_views: path_raw_views,
            storage_origins: path_storage_origins,
            pending_reallocs: path_pending_reallocs,
            variant_owner_effects: path_variant_owner_effects,
        },
        #[cfg(test)]
        controls: Vec::new(),
        #[cfg(test)]
        merge_eligible: true,
        #[cfg(test)]
        engine_effects: Some(engine_effects),
    };
    #[cfg(test)]
    let mut result = result;
    #[cfg(test)]
    if match_entry_reachable {
        if let Some(output) = _match_output {
            let finalize_checkpoint = path_engine.match_engine_effect_checkpoint();
            result.merge_eligible = path_engine.finalize_match_arm_value(
                &mut result.state,
                output,
                path_value,
                match_arm.map(|(_, _, span)| span).unwrap_or_else(Span::dummy),
            );
            result
                .engine_effects
                .as_mut()
                .expect("constructed match engine effects must remain owned by the root")
                .push(path_engine.match_engine_effect_delta(finalize_checkpoint));
        }
    } else {
        result.merge_eligible = false;
    }
    result
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

    fn run_path(path_ops: &[ResourceOp], path_value: &Place) -> OwnerVariantTraversalResult {
        let types = TypeCtx::new();
        run_path_with_match(
            &types,
            path_ops,
            path_value,
            &PendingVariantOwnerEffects::default(),
            None,
            None,
        )
    }

    fn run_path_with_match(
        types: &TypeCtx,
        path_ops: &[ResourceOp],
        path_value: &Place,
        variant_owner_effects: &PendingVariantOwnerEffects,
        match_arm: Option<(&Place, &ResourceMatchArm, Span)>,
        match_output: Option<&Place>,
    ) -> OwnerVariantTraversalResult {
        run_path_with_summary_snapshot(
            types,
            path_ops,
            path_value,
            variant_owner_effects,
            match_arm,
            match_output,
        )
        .0
    }

    fn run_path_with_summary_snapshot(
        types: &TypeCtx,
        path_ops: &[ResourceOp],
        path_value: &Place,
        variant_owner_effects: &PendingVariantOwnerEffects,
        match_arm: Option<(&Place, &ResourceMatchArm, Span)>,
        match_output: Option<&Place>,
    ) -> (OwnerVariantTraversalResult, OwnerVariantSummarySnapshot) {
        let summaries = Vec::new();
        let summary_index = OwnerReturnSummaryIndex::new(&summaries);
        let engine = ResourceOwnerCheckEngine {
            function: "variant_path_oracle",
            types,
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

        let result = collect_variant_consumed_owner_parameters_from_path(
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
            variant_owner_effects,
            &[],
            &[],
            path_ops,
            path_value,
            None,
            match_arm,
            match_output,
            &mut host_size_out,
            &mut type_size_out,
            &mut return_out,
            &mut profile,
            0,
        );
        let mut snapshot = OwnerVariantSummarySnapshot {
            indices: index_out,
            sources: source_out,
            extents: extent_out,
            conditions: condition_out,
            payload_conditions: payload_condition_out.conditions_snapshot(),
            host_sizes: host_size_out,
            type_sizes: type_size_out,
            returns: return_out,
        };
        snapshot.canonicalize();
        (result, snapshot)
    }

    fn linear_match_condition_reference(
        ops: &[ResourceOp],
        return_value: &Place,
        effects: &PendingVariantOwnerEffects,
    ) -> Vec<OwnerVariantCondition> {
        let mut conditions = Vec::new();
        for op in ops {
            let ResourceOp::Match {
                output,
                scrutinee,
                arms,
                ..
            } = op
            else {
                continue;
            };
            if output != return_value {
                continue;
            }
            for arm in arms {
                let crate::resource::model::ResourceMatchPattern::Variant(variant) = &arm.pattern
                else {
                    panic!("condition reference requires an explicit variant pattern");
                };
                if effects.unreachable_variants.iter().any(|entry| {
                    entry.result == *scrutinee && entry.variant == *variant
                }) {
                    continue;
                }
                conditions.push(OwnerVariantCondition {
                    variant: variant.clone(),
                    condition: OwnerValueCondition::Always,
                });
            }
        }
        conditions.sort_unstable();
        conditions.dedup();
        conditions
    }

    #[test]
    fn constructed_path_returns_state_after_final_replay_op() {
        let value = Place::local("value".to_string(), TypeId(0));
        let retained = Place::local("retained".to_string(), TypeId(0));
        let span = Span::dummy();
        let result = run_path(
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
            result.state.storage_origins.origin(&retained),
            Some(StorageOrigin::Owned)
        );
        assert!(result.engine_effects().is_complete());
    }

    #[test]
    fn recursive_path_returns_state_after_final_replay_op() {
        let value = Place::local("value".to_string(), TypeId(0));
        let retained = Place::local("retained".to_string(), TypeId(0));
        let span = Span::dummy();
        let result = run_path(
            &[ResourceOp::StorageOrigin {
                target: retained.clone(),
                origin: StorageOrigin::Owned,
                span,
            }],
            &value,
        );

        assert_eq!(
            result.state.storage_origins.origin(&retained),
            Some(StorageOrigin::Owned)
        );
        assert!(result.engine_effects().is_complete());
    }

    #[test]
    fn constructed_path_with_control_replay_keeps_engine_effects_incomplete() {
        let value = Place::local("value".to_string(), TypeId(0));
        let branch_output = Place::local("branch_output".to_string(), TypeId(0));
        let then_value = Place::local("then_value".to_string(), TypeId(0));
        let else_value = Place::local("else_value".to_string(), TypeId(0));
        let condition = Place::local("condition".to_string(), TypeId(0));
        let span = Span::dummy();
        let ops = [
            ResourceOp::Branch {
                output: branch_output,
                condition,
                condition_fact: None,
                then_ops: Vec::new(),
                then_value,
                else_ops: Vec::new(),
                else_value,
                span,
            },
            ResourceOp::Construct {
                output: value.clone(),
                kind: AggregateKind::Enum {
                    name: "Result".to_string(),
                    variant: "Ok".to_string(),
                },
                inputs: Vec::new(),
                span,
            },
        ];
        assert_eq!(
            construct_variant_for_value(&ops, &value).map(|variant| variant.variant),
            Some("Ok".to_string())
        );
        let result = run_path(&ops, &value);

        assert!(!result.engine_effects().is_complete());
    }

    #[test]
    fn constructed_never_match_path_keeps_complete_effects_but_excludes_state() {
        let types = TypeCtx::new();
        let value = Place::local("never_value".to_string(), types.never());
        let output = Place::local("output".to_string(), types.unit());
        let scrutinee = Place::local("scrutinee".to_string(), types.unit());
        let span = Span::dummy();
        let arm = ResourceMatchArm {
            pattern: crate::resource::model::ResourceMatchPattern::Wildcard,
            bind_local: None,
            bind_source_name: None,
            bind_mode: None,
            ops: Vec::new(),
            value: value.clone(),
            span,
        };
        let ops = [ResourceOp::Construct {
            output: value.clone(),
            kind: AggregateKind::Enum {
                name: "Result".to_string(),
                variant: "Ok".to_string(),
            },
            inputs: Vec::new(),
            span,
        }];

        let result = run_path_with_match(
            &types,
            &ops,
            &value,
            &PendingVariantOwnerEffects::default(),
            Some((&scrutinee, &arm, span)),
            Some(&output),
        );

        assert!(result.engine_effects().is_complete());
        assert!(!result.merge_eligible);
    }

    #[test]
    fn unreachable_match_path_is_incomplete_and_merge_ineligible() {
        let types = TypeCtx::new();
        let value = Place::local("value".to_string(), types.unit());
        let output = Place::local("output".to_string(), types.unit());
        let scrutinee = Place::local("scrutinee".to_string(), types.unit());
        let span = Span::dummy();
        let arm = ResourceMatchArm {
            pattern: crate::resource::model::ResourceMatchPattern::Variant("Err".to_string()),
            bind_local: None,
            bind_source_name: None,
            bind_mode: None,
            ops: Vec::new(),
            value: value.clone(),
            span,
        };
        let ops = [ResourceOp::Construct {
            output: value.clone(),
            kind: AggregateKind::Enum {
                name: "Result".to_string(),
                variant: "Ok".to_string(),
            },
            inputs: Vec::new(),
            span,
        }];
        let mut effects = PendingVariantOwnerEffects::default();
        effects.unreachable_variants.push(
            crate::resource::owner_variant::PendingUnreachableVariant {
                result: scrutinee.clone(),
                variant: "Err".to_string(),
            },
        );

        let result = run_path_with_match(
            &types,
            &ops,
            &value,
            &effects,
            Some((&scrutinee, &arm, span)),
            Some(&output),
        );

        assert!(!result.engine_effects().is_complete());
        assert!(!result.merge_eligible);
    }

    #[test]
    fn recursive_all_never_match_preserves_parent_state_and_effects() {
        let types = TypeCtx::new();
        let value = Place::local("value".to_string(), types.unit());
        let never_value = Place::local("never_value".to_string(), types.never());
        let parent_retained = Place::local("parent_retained".to_string(), types.unit());
        let retained = Place::local("retained".to_string(), types.unit());
        let scrutinee = Place::local("scrutinee".to_string(), types.unit());
        let span = Span::dummy();
        let arm = ResourceMatchArm {
            pattern: crate::resource::model::ResourceMatchPattern::Wildcard,
            bind_local: None,
            bind_source_name: None,
            bind_mode: None,
            ops: vec![
                ResourceOp::Construct {
                    output: never_value.clone(),
                    kind: AggregateKind::Enum {
                        name: "Result".to_string(),
                        variant: "Never".to_string(),
                    },
                    inputs: Vec::new(),
                    span,
                },
                ResourceOp::StorageOrigin {
                    target: never_value.clone(),
                    origin: StorageOrigin::Owned,
                    span,
                },
            ],
            value: never_value.clone(),
            span,
        };
        let result = run_path(
            &[
                ResourceOp::StorageOrigin {
                    target: parent_retained.clone(),
                    origin: StorageOrigin::Owned,
                    span,
                },
                ResourceOp::Match {
                    output: value.clone(),
                    scrutinee,
                    scrutinee_is_borrow_target: false,
                    arms: vec![arm],
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

        assert!(result.engine_effects().is_complete());
        assert_eq!(result.engine_effects().delta_count(), 6);
        assert!(result.merge_eligible);
        assert_eq!(result.controls.len(), 1);
        assert_eq!(result.controls[0].kind, OwnerVariantControlKind::Match);
        assert_eq!(result.controls[0].paths.len(), 1);
        assert_eq!(
            result.controls[0].paths[0].selector,
            OwnerVariantPathSelector::MatchArm(0)
        );
        assert!(!result.controls[0].paths[0].result.merge_eligible);
        assert!(
            result.controls[0].paths[0]
                .result
                .effect_authority_transferred
        );
        assert_eq!(
            result.controls[0].paths[0]
                .result
                .state_snapshot
                .storage_origin(&never_value),
            Some(StorageOrigin::Owned)
        );
        assert_eq!(result.state.storage_origins.origin(&never_value), None);
        assert_eq!(
            result.state.storage_origins.origin(&parent_retained),
            Some(StorageOrigin::Owned)
        );
        assert_eq!(
            result.state.storage_origins.origin(&retained),
            Some(StorageOrigin::Owned)
        );
    }

    #[test]
    fn consecutive_recursive_matches_thread_specialized_state_and_effects() {
        let mut types = TypeCtx::new();
        let owner_ty = types.box_ty(types.unit());
        let value = Place::local("value".to_string(), types.unit());
        let first_unreachable = Place::local("first_unreachable".to_string(), types.unit());
        let first_value = Place::local("first_value".to_string(), types.unit());
        let first_retained = Place::local("first_retained".to_string(), types.unit());
        let second_value = Place::local("second_value".to_string(), types.unit());
        let second_retained = Place::local("second_retained".to_string(), types.unit());
        let post_retained = Place::local("post_retained".to_string(), types.unit());
        let first_scrutinee = Place::local("first_scrutinee".to_string(), types.unit());
        let second_scrutinee = Place::local("second_scrutinee".to_string(), types.unit());
        let first_bind = Place::local("first_bind".to_string(), owner_ty);
        let second_bind = Place::local("second_bind".to_string(), owner_ty);
        let span = Span::dummy();
        let arm = |arm_value: Place, retained: Place, bind_local: Place| ResourceMatchArm {
            pattern: crate::resource::model::ResourceMatchPattern::Variant("Ok".to_string()),
            bind_local: Some(bind_local),
            bind_source_name: None,
            bind_mode: Some(crate::resource::model::ResourceMatchBindMode::Owned),
            ops: vec![
                ResourceOp::Construct {
                    output: arm_value.clone(),
                    kind: AggregateKind::Enum {
                        name: "Result".to_string(),
                        variant: "Ok".to_string(),
                    },
                    inputs: Vec::new(),
                    span,
                },
                ResourceOp::StorageOrigin {
                    target: retained,
                    origin: StorageOrigin::Owned,
                    span,
                },
            ],
            value: arm_value,
            span,
        };
        let unreachable_arm = ResourceMatchArm {
            pattern: crate::resource::model::ResourceMatchPattern::Variant("Err".to_string()),
            bind_local: None,
            bind_source_name: None,
            bind_mode: None,
            ops: vec![ResourceOp::Construct {
                output: first_unreachable.clone(),
                kind: AggregateKind::Enum {
                    name: "Result".to_string(),
                    variant: "Err".to_string(),
                },
                inputs: Vec::new(),
                span,
            }],
            value: first_unreachable,
            span,
        };
        let ops = [
            ResourceOp::Match {
                output: value.clone(),
                scrutinee: first_scrutinee.clone(),
                scrutinee_is_borrow_target: false,
                arms: vec![
                    unreachable_arm,
                    arm(first_value, first_retained.clone(), first_bind),
                ],
                span,
            },
            ResourceOp::Match {
                output: value.clone(),
                scrutinee: second_scrutinee.clone(),
                scrutinee_is_borrow_target: false,
                arms: vec![arm(second_value, second_retained.clone(), second_bind)],
                span,
            },
            ResourceOp::StorageOrigin {
                target: post_retained.clone(),
                origin: StorageOrigin::Owned,
                span,
            },
        ];
        let mut effects = PendingVariantOwnerEffects::default();
        for scrutinee in [&first_scrutinee, &second_scrutinee] {
            effects.consumptions.push(
                crate::resource::owner_variant::PendingVariantOwnerConsumption {
                    result: scrutinee.clone(),
                    variant: "Ok".to_string(),
                    arg: scrutinee.clone().with_projection(
                        crate::resource::model::PlaceProjection::EnumPayload {
                            variant: "Ok".to_string(),
                        },
                        owner_ty,
                    ),
                    suffix: Vec::new(),
                    ty: owner_ty,
                    extent: None,
                },
            );
        }
        effects.unreachable_variants.push(
            crate::resource::owner_variant::PendingUnreachableVariant {
                result: first_scrutinee.clone(),
                variant: "Err".to_string(),
            },
        );
        let (mut result, summary_snapshot) = run_path_with_summary_snapshot(
            &types,
            &ops,
            &value,
            &effects,
            None,
            None,
        );

        assert!(result.engine_effects().is_complete());
        assert_eq!(
            summary_snapshot.conditions,
            linear_match_condition_reference(&ops, &value, &effects)
        );
        assert_eq!(
            summary_snapshot,
            OwnerVariantSummarySnapshot {
                indices: Vec::new(),
                sources: Vec::new(),
                extents: Vec::new(),
                conditions: vec![OwnerVariantCondition {
                    variant: "Ok".to_string(),
                    condition: OwnerValueCondition::Always,
                }],
                payload_conditions: Vec::new(),
                host_sizes: Vec::new(),
                type_sizes: Vec::new(),
                returns: Vec::new(),
            }
        );
        assert_eq!(result.engine_effects().delta_count(), 8);
        assert_eq!(result.controls.len(), 2);
        assert_eq!(result.controls[0].op_index, 0);
        assert_eq!(result.controls[1].op_index, 1);
        for control in &result.controls {
            assert_eq!(control.kind, OwnerVariantControlKind::Match);
            assert_eq!(control.paths.len(), 1);
            assert!(control.paths[0].result.merge_eligible);
            assert!(control.paths[0].result.effect_authority_transferred);
        }
        assert_eq!(
            result.controls[0].paths[0].selector,
            OwnerVariantPathSelector::MatchArm(1)
        );
        assert_eq!(
            result.controls[1].paths[0].selector,
            OwnerVariantPathSelector::MatchArm(0)
        );
        assert_eq!(
            result.controls[0].paths[0]
                .result
                .state_snapshot
                .storage_origin(&first_retained),
            Some(StorageOrigin::Owned)
        );
        assert_eq!(
            result.controls[0].paths[0]
                .result
                .state_snapshot
                .storage_origin(&second_retained),
            None
        );
        assert_eq!(
            result.controls[1].paths[0]
                .result
                .state_snapshot
                .storage_origin(&first_retained),
            Some(StorageOrigin::Owned)
        );
        assert_eq!(
            result.controls[1].paths[0]
                .result
                .state_snapshot
                .storage_origin(&second_retained),
            Some(StorageOrigin::Owned)
        );
        assert_eq!(
            result.controls[1].paths[0]
                .result
                .state_snapshot
                .storage_origin(&post_retained),
            None
        );
        assert_eq!(
            result.state.storage_origins.origin(&first_retained),
            Some(StorageOrigin::Owned)
        );
        assert_eq!(
            result.state.storage_origins.origin(&second_retained),
            Some(StorageOrigin::Owned)
        );
        assert_eq!(
            result.state.storage_origins.origin(&post_retained),
            Some(StorageOrigin::Owned)
        );

        let summaries = Vec::new();
        let summary_index = OwnerReturnSummaryIndex::new(&summaries);
        let make_engine = || ResourceOwnerCheckEngine {
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
        let mut generic_state = OwnerMatchPathState::from_parent(
            &OwnerTable::default(),
            &FunctionAliasTable::default(),
            &RawCellAddressAliases::default(),
            &RawAddressViewTable::default(),
            &StorageOriginTable::default(),
            &PendingRawReallocs::default(),
            &effects,
        );
        let mut generic_engine = make_engine();
        generic_engine.check_ops(
            &mut generic_state.owners,
            &mut generic_state.function_aliases,
            &mut generic_state.raw_aliases,
            &mut generic_state.raw_views,
            &mut generic_state.storage_origins,
            &mut generic_state.pending_reallocs,
            &mut generic_state.variant_owner_effects,
            &ops,
        );
        let generic_effects = generic_engine.match_oracle_snapshot();
        let mut absorbed_engine = make_engine();
        result
            .take_engine_effects()
            .absorb_into(&mut absorbed_engine);

        assert_eq!(generic_effects, absorbed_engine.match_oracle_snapshot());
        assert_eq!(generic_effects.diagnostic_count(), 2);
        assert_eq!(generic_state.oracle_snapshot(), result.state.oracle_snapshot());
        assert!(result.engine_effects.is_none());
    }

    #[test]
    fn recursive_match_transfers_complete_child_effects_once() {
        let mut types = TypeCtx::new();
        let unit = types.unit();
        let owner_ty = types.box_ty(unit);
        let value = Place::local("value".to_string(), types.unit());
        let child_value = Place::local("child_value".to_string(), types.unit());
        let unreachable_value = Place::local("unreachable_value".to_string(), types.unit());
        let never_value = Place::local("never_value".to_string(), types.never());
        let output = Place::local("output".to_string(), types.unit());
        let retained = Place::local("retained".to_string(), types.unit());
        let outer_scrutinee = Place::local("outer_scrutinee".to_string(), types.unit());
        let nested_scrutinee = Place::local("nested_scrutinee".to_string(), types.unit());
        let span = Span::dummy();
        let nested_arm = ResourceMatchArm {
            pattern: crate::resource::model::ResourceMatchPattern::Wildcard,
            bind_local: None,
            bind_source_name: None,
            bind_mode: None,
            ops: vec![
                ResourceOp::Construct {
                    output: child_value.clone(),
                    kind: AggregateKind::Enum {
                        name: "Result".to_string(),
                        variant: "Ok".to_string(),
                    },
                    inputs: Vec::new(),
                    span,
                },
                ResourceOp::StorageOrigin {
                    target: child_value.clone(),
                    origin: StorageOrigin::Owned,
                    span,
                },
            ],
            value: child_value.clone(),
            span,
        };
        let unreachable_arm = ResourceMatchArm {
            pattern: crate::resource::model::ResourceMatchPattern::Variant("Err".to_string()),
            bind_local: None,
            bind_source_name: None,
            bind_mode: None,
            ops: vec![ResourceOp::Construct {
                output: unreachable_value.clone(),
                kind: AggregateKind::Enum {
                    name: "Result".to_string(),
                    variant: "Err".to_string(),
                },
                inputs: Vec::new(),
                span,
            }],
            value: unreachable_value,
            span,
        };
        let never_arm = ResourceMatchArm {
            pattern: crate::resource::model::ResourceMatchPattern::Variant("Never".to_string()),
            bind_local: None,
            bind_source_name: None,
            bind_mode: None,
            ops: vec![
                ResourceOp::Construct {
                    output: never_value.clone(),
                    kind: AggregateKind::Enum {
                        name: "Result".to_string(),
                        variant: "Never".to_string(),
                    },
                    inputs: Vec::new(),
                    span,
                },
                ResourceOp::StorageOrigin {
                    target: never_value.clone(),
                    origin: StorageOrigin::Owned,
                    span,
                },
            ],
            value: never_value.clone(),
            span,
        };
        let ops = [
            ResourceOp::Match {
                output: value.clone(),
                scrutinee: nested_scrutinee.clone(),
                scrutinee_is_borrow_target: false,
                arms: vec![unreachable_arm, never_arm, nested_arm],
                span,
            },
            ResourceOp::StorageOrigin {
                target: retained.clone(),
                origin: StorageOrigin::Owned,
                span,
            },
        ];
        let bind_local = Place::local("payload".to_string(), owner_ty);
        let reserved_source = outer_scrutinee.clone().with_projection(
            crate::resource::model::PlaceProjection::EnumPayload {
                variant: "Ok".to_string(),
            },
            owner_ty,
        );
        let outer_arm = ResourceMatchArm {
            pattern: crate::resource::model::ResourceMatchPattern::Variant("Ok".to_string()),
            bind_local: Some(bind_local),
            bind_source_name: None,
            bind_mode: Some(crate::resource::model::ResourceMatchBindMode::Owned),
            ops: ops.to_vec(),
            value: value.clone(),
            span,
        };
        let mut effects = PendingVariantOwnerEffects::default();
        effects.consumptions.push(
            crate::resource::owner_variant::PendingVariantOwnerConsumption {
                result: outer_scrutinee.clone(),
                variant: "Ok".to_string(),
                arg: reserved_source,
                suffix: Vec::new(),
                ty: owner_ty,
                extent: None,
            },
        );
        effects.unreachable_variants.push(
            crate::resource::owner_variant::PendingUnreachableVariant {
                result: nested_scrutinee,
                variant: "Err".to_string(),
            },
        );

        let mut result = run_path_with_match(
            &types,
            &ops,
            &value,
            &effects,
            Some((&outer_scrutinee, &outer_arm, span)),
            Some(&output),
        );

        assert!(result.engine_effects().is_complete());
        assert_eq!(result.engine_effects().delta_count(), 9);
        assert!(result.merge_eligible);
        assert_eq!(result.controls.len(), 1);
        assert_eq!(result.controls[0].kind, OwnerVariantControlKind::Match);
        assert_eq!(result.controls[0].paths.len(), 2);
        assert_eq!(
            result.controls[0].paths[0].selector,
            OwnerVariantPathSelector::MatchArm(1)
        );
        assert!(!result.controls[0].paths[0].result.merge_eligible);
        assert_eq!(
            result.controls[0].paths[0]
                .result
                .state_snapshot
                .storage_origin(&never_value),
            Some(StorageOrigin::Owned)
        );
        assert!(result.controls[0].paths[0]
            .result
            .effect_authority_transferred);
        assert_eq!(
            result.controls[0].paths[1].selector,
            OwnerVariantPathSelector::MatchArm(2)
        );
        assert!(result.controls[0].paths[1].result.merge_eligible);
        assert_eq!(
            result.controls[0].paths[1]
                .result
                .state_snapshot
                .storage_origin(&value),
            Some(StorageOrigin::Owned)
        );
        assert!(result.controls[0].paths[1]
            .result
            .effect_authority_transferred);
        assert_eq!(result.state.storage_origins.origin(&never_value), None);
        assert_eq!(
            result.state.storage_origins.origin(&output),
            Some(StorageOrigin::Owned)
        );
        assert_eq!(
            result.state.storage_origins.origin(&retained),
            Some(StorageOrigin::Owned)
        );

        let summaries = Vec::new();
        let summary_index = OwnerReturnSummaryIndex::new(&summaries);
        let make_engine = || ResourceOwnerCheckEngine {
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
        let generic_state = OwnerMatchPathState::from_parent(
            &OwnerTable::default(),
            &FunctionAliasTable::default(),
            &RawCellAddressAliases::default(),
            &RawAddressViewTable::default(),
            &StorageOriginTable::default(),
            &PendingRawReallocs::default(),
            &effects,
        );
        let (generic_state, generic_effects) = make_engine().run_generic_match_oracle(
            generic_state,
            &output,
            &outer_scrutinee,
            &[outer_arm],
            span,
            &[],
        );
        let mut absorbed_engine = make_engine();
        result
            .take_engine_effects()
            .absorb_into(&mut absorbed_engine);

        assert_eq!(generic_effects, absorbed_engine.match_oracle_snapshot());
        assert_eq!(generic_state, result.state.oracle_snapshot());
        assert_eq!(generic_effects.diagnostic_count(), 1);
        assert!(result.engine_effects.is_none());
    }

    #[test]
    fn recursive_control_paths_preserve_branch_and_nested_match_hierarchy() {
        let root_value = Place::local("root_value".to_string(), TypeId(0));
        let then_value = Place::local("then_value".to_string(), TypeId(0));
        let else_value = Place::local("else_value".to_string(), TypeId(0));
        let arm_value = Place::local("arm_value".to_string(), TypeId(0));
        let condition = Place::local("condition".to_string(), TypeId(0));
        let scrutinee = Place::local("scrutinee".to_string(), TypeId(0));
        let else_retained = Place::local("else_retained".to_string(), TypeId(0));
        let span = Span::dummy();
        let enum_construct = |output: Place| ResourceOp::Construct {
            output,
            kind: AggregateKind::Enum {
                name: "Result".to_string(),
                variant: "Ok".to_string(),
            },
            inputs: Vec::new(),
            span,
        };
        let result = run_path(
            &[ResourceOp::Branch {
                output: root_value.clone(),
                condition,
                condition_fact: None,
                then_ops: vec![ResourceOp::Match {
                    output: then_value.clone(),
                    scrutinee,
                    scrutinee_is_borrow_target: false,
                    arms: vec![ResourceMatchArm {
                        pattern: crate::resource::model::ResourceMatchPattern::Wildcard,
                        bind_local: None,
                        bind_source_name: None,
                        bind_mode: None,
                        ops: vec![
                            enum_construct(arm_value.clone()),
                            ResourceOp::StorageOrigin {
                                target: arm_value.clone(),
                                origin: StorageOrigin::Owned,
                                span,
                            },
                        ],
                        value: arm_value,
                        span,
                    }],
                    span,
                }],
                then_value: then_value.clone(),
                else_ops: vec![
                    enum_construct(else_value.clone()),
                    ResourceOp::StorageOrigin {
                        target: else_retained.clone(),
                        origin: StorageOrigin::Owned,
                        span,
                    },
                ],
                else_value,
                span,
            }],
            &root_value,
        );

        assert_eq!(result.controls.len(), 1);
        let branch = &result.controls[0];
        assert_eq!(branch.op_index, 0);
        assert_eq!(branch.kind, OwnerVariantControlKind::Branch);
        assert_eq!(branch.paths.len(), 2);
        assert_eq!(branch.paths[0].selector, OwnerVariantPathSelector::Then);
        assert_eq!(branch.paths[1].selector, OwnerVariantPathSelector::Else);
        assert_eq!(branch.paths[0].result.controls.len(), 1);
        assert!(branch.paths[1].result.controls.is_empty());
        let nested_match = &branch.paths[0].result.controls[0];
        assert_eq!(nested_match.op_index, 0);
        assert_eq!(nested_match.kind, OwnerVariantControlKind::Match);
        assert_eq!(nested_match.paths.len(), 1);
        assert!(nested_match.paths[0].result.merge_eligible);
        assert_eq!(
            nested_match.paths[0].selector,
            OwnerVariantPathSelector::MatchArm(0)
        );
        assert_eq!(
            nested_match.paths[0]
                .result
                .state_snapshot
                .storage_origin(&then_value),
            Some(StorageOrigin::Owned)
        );
        assert_eq!(
            branch.paths[1]
                .result
                .state_snapshot
                .storage_origin(&else_retained),
            Some(StorageOrigin::Owned)
        );
    }
}
