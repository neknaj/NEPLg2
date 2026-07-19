use alloc::vec::Vec;
#[cfg(test)]
use alloc::vec;

use crate::span::Span;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceConditionFact, ResourceMatchArm, ResourceOp};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_control::{OwnerMatchEngineEffectAccumulator, OwnerMatchPathState};
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
    merge_eligible: bool,
    engine_effects: Option<OwnerMatchEngineEffectAccumulator>,
}

impl OwnerVariantTraversalResult {
    #[cfg(test)]
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
    let mut engine_effects = OwnerMatchEngineEffectAccumulator::default();
    #[cfg(test)]
    let mut generic_oracle = None;
    profile.finish(state_clone_timer, OwnerVariantProfilePhase::StateClone);
    for (index, op) in ops.iter().enumerate() {
        let mut specialized_authority = false;
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
                let mut branch_engine = ResourceOwnerCheckEngine {
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
                #[cfg(test)]
                let mut branch_generic_state = OwnerMatchPathState::from_parent(
                    &owners,
                    &function_aliases,
                    &raw_aliases,
                    &raw_views,
                    &storage_origins,
                    &pending_reallocs,
                    &variant_owner_effects,
                );
                #[cfg(test)]
                let mut branch_generic_engine = ResourceOwnerCheckEngine {
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
                let branch_condition_checkpoint =
                    branch_engine.match_engine_effect_checkpoint();
                let mut then_owners = owners.clone();
                let mut then_raw_aliases = raw_aliases.clone();
                let mut then_storage_origins = storage_origins.clone();
                let mut then_pending_reallocs = pending_reallocs.clone();
                branch_engine.apply_branch_condition_fact(
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
                profile.observe_branch_fork();
                let branch_fork_timer = profile.start();
                let mut else_owners = owners.clone();
                let mut else_raw_aliases = raw_aliases.clone();
                let mut else_storage_origins = storage_origins.clone();
                let mut else_pending_reallocs = pending_reallocs.clone();
                branch_engine.apply_branch_condition_fact(
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
                let branch_condition_effects =
                    branch_engine.match_engine_effect_delta(branch_condition_checkpoint);
                let mut then_result = collect_variant_consumed_owner_parameters_from_path(
                    index_out,
                    source_out,
                    extent_out,
                    condition_out,
                    payload_condition_out,
                    &branch_engine,
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
                    Some((output, *span)),
                    host_size_out,
                    type_size_out,
                    return_out,
                    profile,
                    depth + 1,
                );
                let mut else_result = collect_variant_consumed_owner_parameters_from_path(
                    index_out,
                    source_out,
                    extent_out,
                    condition_out,
                    payload_condition_out,
                    &branch_engine,
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
                    Some((output, *span)),
                    host_size_out,
                    type_size_out,
                    return_out,
                    profile,
                    depth + 1,
                );
                let mut branch_effects = OwnerMatchEngineEffectAccumulator::default();
                branch_effects.push(branch_condition_effects);
                branch_effects.extend(then_result.take_engine_effects());
                branch_effects.extend(else_result.take_engine_effects());
                let mut branch_paths = super::owner_control::OwnerMatchPathStates::default();
                #[cfg(test)]
                let then_evidence = OwnerVariantTraversalEvidence {
                    state_snapshot: then_result.state.oracle_snapshot(),
                    controls: then_result.controls,
                    merge_eligible: then_result.merge_eligible,
                    effect_authority_transferred: then_result.engine_effects.is_none(),
                };
                if then_result.merge_eligible {
                    branch_paths.push(then_result.state);
                }
                #[cfg(test)]
                let else_evidence = OwnerVariantTraversalEvidence {
                    state_snapshot: else_result.state.oracle_snapshot(),
                    controls: else_result.controls,
                    merge_eligible: else_result.merge_eligible,
                    effect_authority_transferred: else_result.engine_effects.is_none(),
                };
                if else_result.merge_eligible {
                    branch_paths.push(else_result.state);
                }
                if branch_effects.is_complete() {
                    super::owner_control::merge_match_path_states(
                        &mut owners,
                        &mut function_aliases,
                        &mut raw_aliases,
                        &mut raw_views,
                        &mut storage_origins,
                        &mut pending_reallocs,
                        &mut variant_owner_effects,
                        branch_paths,
                    );
                    branch_effects = branch_effects.absorb_into_and_retain(&mut engine);
                    specialized_authority = true;
                }
                engine_effects.extend(branch_effects);
                #[cfg(test)]
                if specialized_authority {
                    branch_generic_engine.check_ops(
                        &mut branch_generic_state.owners,
                        &mut branch_generic_state.function_aliases,
                        &mut branch_generic_state.raw_aliases,
                        &mut branch_generic_state.raw_views,
                        &mut branch_generic_state.storage_origins,
                        &mut branch_generic_state.pending_reallocs,
                        &mut branch_generic_state.variant_owner_effects,
                        &ops[index..=index],
                    );
                    assert_eq!(
                        branch_generic_state.oracle_snapshot(),
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
                    assert_eq!(
                        branch_generic_engine.match_oracle_snapshot(),
                        engine.match_oracle_snapshot()
                    );
                }
                #[cfg(test)]
                controls.push(OwnerVariantControlPaths {
                    op_index: index,
                    kind: OwnerVariantControlKind::Branch,
                    paths: vec![
                        OwnerVariantTraversalPath {
                            selector: OwnerVariantPathSelector::Then,
                            result: then_evidence,
                        },
                        OwnerVariantTraversalPath {
                            selector: OwnerVariantPathSelector::Else,
                            result: else_evidence,
                        },
                    ],
                });
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
                let mut reachable_paths = 0usize;
                let mut merged_paths = super::owner_control::OwnerMatchPathStates::default();
                let mut match_effects = OwnerMatchEngineEffectAccumulator::default();
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
                    let mut result = collect_variant_consumed_owner_parameters_from_path(
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
                        Some((output, *span)),
                        host_size_out,
                        type_size_out,
                        return_out,
                        profile,
                        depth + 1,
                    );
                    reachable_paths += 1;
                    let child_effects = result.take_engine_effects();
                    match_effects.extend(child_effects);
                    #[cfg(test)]
                    let evidence = {
                        let evidence = OwnerVariantTraversalEvidence {
                            state_snapshot: result.state.oracle_snapshot(),
                            controls: result.controls,
                            merge_eligible: result.merge_eligible,
                            effect_authority_transferred: result.engine_effects.is_none(),
                        };
                        evidence
                    };
                    #[cfg(test)]
                    paths.push(OwnerVariantTraversalPath {
                        selector: OwnerVariantPathSelector::MatchArm(_arm_index),
                        result: evidence,
                    });
                    if result.merge_eligible {
                        merged_paths.push(result.state);
                    }
                }
                if reachable_paths == 0 {
                    match_effects.mark_incomplete();
                }
                if match_effects.is_complete() {
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
                    match_effects = match_effects.absorb_into_and_retain(&mut engine);
                    specialized_authority = true;
                }
                engine_effects.extend(match_effects);
                #[cfg(test)]
                if specialized_authority {
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
        let specialized_match = matches!(
            op,
            ResourceOp::Match { output, .. } if output == return_value
        );
        let captures_sequential_effects = !matches!(
            op,
            ResourceOp::Branch { .. } | ResourceOp::Match { .. }
        );
        if !specialized_match && !captures_sequential_effects {
            engine_effects.mark_incomplete();
        }
        let replay_checkpoint = engine.match_engine_effect_checkpoint();
        profile.observe_sequential_replay(1);
        let replay_timer = profile.start();
        if !specialized_authority {
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
        #[cfg(test)]
        if !specialized_match {
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
        merge_eligible: true,
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
    control_output: Option<(&Place, Span)>,
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
    let mut engine_effects = OwnerMatchEngineEffectAccumulator::default();
    let entry_checkpoint = path_engine.match_engine_effect_checkpoint();
    let mut match_entry_reachable = true;
    profile.finish(state_clone_timer, OwnerVariantProfilePhase::StateClone);
    let match_entry_timer = profile.start();
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
    engine_effects.push(path_engine.match_engine_effect_delta(entry_checkpoint));
    profile.finish(match_entry_timer, OwnerVariantProfilePhase::MatchEntry);
    let Some(constructed_variant) = construct_variant_for_value(path_ops, path_value) else {
        profile.observe_recursive_path();
        let mut result = collect_variant_consumed_owner_parameters_from_nested_return(
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
        engine_effects.extend(result.take_engine_effects());
        result.engine_effects = Some(engine_effects);
        if match_entry_reachable {
            if let Some((output, control_span)) = control_output {
                let finalize_checkpoint = path_engine.match_engine_effect_checkpoint();
                result.merge_eligible = if match_arm.is_some() {
                    path_engine.finalize_match_arm_value(
                        &mut result.state,
                        output,
                        path_value,
                        control_span,
                    )
                } else {
                    path_engine.finalize_branch_path(
                        &mut result.state,
                        output,
                        path_value,
                        control_span,
                    )
                };
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
    let leaf_effects_complete = !path_ops.iter().any(|op| {
        matches!(
            op,
            ResourceOp::Branch { .. } | ResourceOp::Match { .. }
        )
    });
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
    let mut result = OwnerVariantTraversalResult {
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
        merge_eligible: true,
        engine_effects: Some(engine_effects),
    };
    if match_entry_reachable {
        if let Some((output, control_span)) = control_output {
            let finalize_checkpoint = path_engine.match_engine_effect_checkpoint();
            result.merge_eligible = if match_arm.is_some() {
                path_engine.finalize_match_arm_value(
                    &mut result.state,
                    output,
                    path_value,
                    control_span,
                )
            } else {
                path_engine.finalize_branch_path(
                    &mut result.state,
                    output,
                    path_value,
                    control_span,
                )
            };
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
    use alloc::boxed::Box;
    use alloc::string::ToString;
    use alloc::vec::Vec;

    use crate::types::{EnumVariantInfo, TypeCtx, TypeId, TypeKind};

    use super::*;
    use crate::layout::composite_field_offset_bytes;
    use crate::resource::model::{
        AggregateKind, OwnerStorageExtent, StorageId, StorageOrigin,
    };
    use crate::resource::summary::{
        OwnerExtentSummary, OwnerProjectionSource, OwnerReturnSummaryIndex,
    };

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

    fn loop_with_nested_match_diagnostic(
        types: &mut TypeCtx,
        loop_retained: &Place,
    ) -> (ResourceOp, PendingVariantOwnerEffects) {
        let owner_ty = types.box_ty(types.unit());
        let condition = Place::local("loop_condition".to_string(), types.unit());
        let scrutinee = Place::local("loop_scrutinee".to_string(), types.unit());
        let bind = Place::local("loop_bind".to_string(), owner_ty);
        let arm_value = Place::local("loop_arm_value".to_string(), types.unit());
        let output = Place::local("loop_match_output".to_string(), types.unit());
        let span = Span::dummy();
        let mut effects = PendingVariantOwnerEffects::default();
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
        (
            ResourceOp::Loop {
                condition_ops: vec![ResourceOp::StorageOrigin {
                    target: loop_retained.clone(),
                    origin: StorageOrigin::Owned,
                    span,
                }],
                condition,
                condition_fact: None,
                body_ops: vec![ResourceOp::Match {
                    output,
                    scrutinee,
                    scrutinee_is_borrow_target: false,
                    arms: vec![ResourceMatchArm {
                        pattern: crate::resource::model::ResourceMatchPattern::Variant(
                            "Ok".to_string(),
                        ),
                        bind_local: Some(bind),
                        bind_source_name: None,
                        bind_mode: Some(crate::resource::model::ResourceMatchBindMode::Owned),
                        ops: vec![ResourceOp::StorageOrigin {
                            target: loop_retained.clone(),
                            origin: StorageOrigin::Owned,
                            span,
                        }],
                        value: arm_value,
                        span,
                    }],
                    span,
                }],
                span,
            },
            effects,
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
        run_path_with_seeded_summary_snapshot(
            types,
            &OwnerTable::default(),
            &[],
            path_ops,
            path_value,
            variant_owner_effects,
            match_arm,
            match_output,
        )
    }

    fn run_path_with_seeded_summary_snapshot(
        types: &TypeCtx,
        owners: &OwnerTable,
        parameter_storage_sources: &[OwnerParameterStorageSource],
        path_ops: &[ResourceOp],
        path_value: &Place,
        variant_owner_effects: &PendingVariantOwnerEffects,
        match_arm: Option<(&Place, &ResourceMatchArm, Span)>,
        match_output: Option<&Place>,
    ) -> (OwnerVariantTraversalResult, OwnerVariantSummarySnapshot) {
        run_path_with_seeded_extent_summary_snapshot(
            types,
            owners,
            &RawCellAddressAliases::default(),
            parameter_storage_sources,
            &[],
            &[],
            path_ops,
            path_value,
            variant_owner_effects,
            match_arm,
            match_output,
        )
    }

    fn run_path_with_seeded_extent_summary_snapshot(
        types: &TypeCtx,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        parameter_storage_sources: &[OwnerParameterStorageSource],
        parameter_condition_sources: &[OwnerParameterConditionSource],
        owner_extent_requirements: &[crate::resource::owner_extent::PendingOwnerExtentRequirement],
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
            owner_extent_requirements: owner_extent_requirements.to_vec(),
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
            owners,
            raw_aliases,
            &RawAddressViewTable::default(),
            &StorageOriginTable::default(),
            &FunctionAliasTable::default(),
            &PendingRawReallocs::default(),
            variant_owner_effects,
            parameter_storage_sources,
            parameter_condition_sources,
            path_ops,
            path_value,
            None,
            match_arm,
            match_output.map(|output| {
                (
                    output,
                    match_arm.map(|(_, _, span)| span).unwrap_or_else(Span::dummy),
                )
            }),
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
    fn constructed_path_records_consumed_root_parameter_index() {
        let mut types = TypeCtx::new();
        let owner_ty = types.box_ty(types.unit());
        let value = Place::local("value".to_string(), types.unit());
        let parameter = Place::local("parameter".to_string(), owner_ty);
        let moved = Place::local("moved".to_string(), owner_ty);
        let span = Span::dummy();
        let mut owners = OwnerTable::default();
        owners.allocate(&parameter);
        let sources = [OwnerParameterStorageSource {
            storage: StorageId(0),
            source: OwnerProjectionSource {
                parameter_index: 2,
                suffix: Vec::new(),
                ty: owner_ty,
            },
            place: parameter.clone(),
        }];
        let ops = [
            ResourceOp::Move {
                source: parameter,
                output: moved,
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

        let (result, snapshot) = run_path_with_seeded_summary_snapshot(
            &types,
            &owners,
            &sources,
            &ops,
            &value,
            &PendingVariantOwnerEffects::default(),
            None,
            None,
        );

        assert!(result.engine_effects().is_complete());
        assert_eq!(
            snapshot.indices,
            vec![OwnerVariantParameterIndex {
                variant: "Ok".to_string(),
                parameter_index: 2,
            }]
        );
        assert!(snapshot.sources.is_empty());
        assert!(snapshot.extents.is_empty());
    }

    #[test]
    fn constructed_path_records_consumed_parameter_projection_source() {
        let mut types = TypeCtx::new();
        let owner_ty = types.box_ty(types.unit());
        let parameter_fields = [types.i32(), owner_ty];
        let parameter_ty = types.tuple(parameter_fields.to_vec());
        let value = Place::local("value".to_string(), types.unit());
        let parameter = Place::local("parameter".to_string(), parameter_ty);
        let suffix = vec![crate::resource::model::PlaceProjection::TupleField {
            index: 1,
            offset_bytes: composite_field_offset_bytes(&types, &parameter_fields, 1),
        }];
        let projected = parameter.clone().with_projection(suffix[0].clone(), owner_ty);
        let moved = Place::local("moved".to_string(), owner_ty);
        let span = Span::dummy();
        let mut owners = OwnerTable::default();
        owners.allocate(&projected);
        let source = OwnerProjectionSource {
            parameter_index: 3,
            suffix,
            ty: owner_ty,
        };
        let sources = [OwnerParameterStorageSource {
            storage: StorageId(0),
            source: source.clone(),
            place: projected.clone(),
        }];
        let ops = [
            ResourceOp::Move {
                source: projected,
                output: moved,
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

        let (result, snapshot) = run_path_with_seeded_summary_snapshot(
            &types,
            &owners,
            &sources,
            &ops,
            &value,
            &PendingVariantOwnerEffects::default(),
            None,
            None,
        );

        assert!(result.engine_effects().is_complete());
        assert!(snapshot.indices.is_empty());
        assert_eq!(
            snapshot.sources,
            vec![OwnerVariantProjectionSource {
                variant: "Ok".to_string(),
                source,
            }]
        );
        assert!(snapshot.extents.is_empty());
    }

    #[test]
    fn constructed_path_records_consumed_parameter_extent_requirement() {
        let mut types = TypeCtx::new();
        let owner_ty = types.box_ty(types.unit());
        let parameter_fields = [types.i32(), owner_ty];
        let parameter_ty = types.tuple(parameter_fields.to_vec());
        let value = Place::local("value".to_string(), types.unit());
        let parameter = Place::local("parameter".to_string(), parameter_ty);
        let suffix = vec![crate::resource::model::PlaceProjection::TupleField {
            index: 1,
            offset_bytes: composite_field_offset_bytes(&types, &parameter_fields, 1),
        }];
        let projected = parameter.clone().with_projection(suffix[0].clone(), owner_ty);
        let moved = Place::local("moved".to_string(), owner_ty);
        let size = Place::local("size".to_string(), types.i32());
        let span = Span::dummy();
        let mut owners = OwnerTable::default();
        owners.allocate(&projected);
        let source = OwnerProjectionSource {
            parameter_index: 4,
            suffix,
            ty: owner_ty,
        };
        let sources = [OwnerParameterStorageSource {
            storage: StorageId(0),
            source: source.clone(),
            place: projected.clone(),
        }];
        let size_source = OwnerProjectionSource {
            parameter_index: 5,
            suffix: Vec::new(),
            ty: types.i32(),
        };
        let condition_sources = [OwnerParameterConditionSource {
            source: size_source.clone(),
            place: size.clone(),
        }];
        let requirements = [crate::resource::owner_extent::PendingOwnerExtentRequirement {
            owner: projected.clone(),
            expected: OwnerStorageExtent::PayloadBytes {
                bytes: Box::new(size),
            },
            operation: crate::resource::report::ResourceOwnerOperation::CallArgument,
        }];
        let ops = [
            ResourceOp::Move {
                source: projected,
                output: moved,
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

        let (result, snapshot) = run_path_with_seeded_extent_summary_snapshot(
            &types,
            &owners,
            &RawCellAddressAliases::default(),
            &sources,
            &condition_sources,
            &requirements,
            &ops,
            &value,
            &PendingVariantOwnerEffects::default(),
            None,
            None,
        );

        assert!(result.engine_effects().is_complete());
        assert!(snapshot.indices.is_empty());
        assert_eq!(
            snapshot.sources,
            vec![OwnerVariantProjectionSource {
                variant: "Ok".to_string(),
                source: source.clone(),
            }]
        );
        assert_eq!(
            snapshot.extents,
            vec![OwnerVariantConsumedExtentRequirement {
                variant: "Ok".to_string(),
                owner: source,
                extent: OwnerExtentSummary::PayloadBytesParameter(size_source),
                operation: crate::resource::report::ResourceOwnerOperation::CallArgument,
            }]
        );
    }

    #[test]
    fn constructed_path_records_known_variant_payload_conditions() {
        let types = TypeCtx::new();
        let payload = Place::local("payload".to_string(), types.i32());
        let value = Place::local("value".to_string(), types.unit());
        let span = Span::dummy();
        let mut raw_aliases = RawCellAddressAliases::default();
        raw_aliases.add_i32_condition(
            &payload,
            crate::resource::model::I32ValueCondition::Positive,
        );
        let ops = [ResourceOp::Construct {
            output: value.clone(),
            kind: AggregateKind::Enum {
                name: "Result".to_string(),
                variant: "Ok".to_string(),
            },
            inputs: vec![payload],
            span,
        }];

        let (result, snapshot) = run_path_with_seeded_extent_summary_snapshot(
            &types,
            &OwnerTable::default(),
            &raw_aliases,
            &[],
            &[],
            &[],
            &ops,
            &value,
            &PendingVariantOwnerEffects::default(),
            None,
            None,
        );

        assert!(result.engine_effects().is_complete());
        let payload_suffix = vec![crate::resource::model::PlaceProjection::EnumPayload {
            variant: "Ok".to_string(),
        }];
        assert_eq!(
            snapshot.payload_conditions,
            vec![
                OwnerVariantPayloadCondition {
                    variant: "Ok".to_string(),
                    suffix: payload_suffix.clone(),
                    ty: types.i32(),
                    condition: crate::resource::model::I32ValueCondition::NeZero,
                },
                OwnerVariantPayloadCondition {
                    variant: "Ok".to_string(),
                    suffix: payload_suffix.clone(),
                    ty: types.i32(),
                    condition: crate::resource::model::I32ValueCondition::Positive,
                },
                OwnerVariantPayloadCondition {
                    variant: "Ok".to_string(),
                    suffix: payload_suffix,
                    ty: types.i32(),
                    condition: crate::resource::model::I32ValueCondition::NonNegative,
                },
            ]
        );
        assert!(snapshot.indices.is_empty());
        assert!(snapshot.sources.is_empty());
        assert!(snapshot.extents.is_empty());
        assert_eq!(
            snapshot.conditions,
            vec![OwnerVariantCondition {
                variant: "Ok".to_string(),
                condition: OwnerValueCondition::Always,
            }]
        );
        assert!(snapshot.host_sizes.is_empty());
        assert!(snapshot.type_sizes.is_empty());
        assert!(snapshot.returns.is_empty());
    }

    #[test]
    fn constructed_path_records_host_and_type_size_payload_returns() {
        let types = TypeCtx::new();
        let host_size = Place::local("host_size".to_string(), types.i32());
        let type_size = Place::local("type_size".to_string(), types.i32());
        let value = Place::local("value".to_string(), types.unit());
        let span = Span::dummy();
        let mut raw_aliases = RawCellAddressAliases::default();
        raw_aliases.set_host_size_kind(
            &host_size,
            crate::resource::host_size_contract::HostSizeKind::ArgsCount,
        );
        raw_aliases.set_i32_type_size(&type_size, types.unit());
        let ops = [ResourceOp::Construct {
            output: value.clone(),
            kind: AggregateKind::Enum {
                name: "Result".to_string(),
                variant: "Ok".to_string(),
            },
            inputs: vec![host_size, type_size],
            span,
        }];

        let (result, snapshot) = run_path_with_seeded_extent_summary_snapshot(
            &types,
            &OwnerTable::default(),
            &raw_aliases,
            &[],
            &[],
            &[],
            &ops,
            &value,
            &PendingVariantOwnerEffects::default(),
            None,
            None,
        );

        assert!(result.engine_effects().is_complete());
        let payload_suffix = crate::resource::model::PlaceProjection::EnumPayload {
            variant: "Ok".to_string(),
        };
        assert_eq!(
            snapshot.host_sizes,
            vec![OwnerHostSizeReturn {
                suffix: vec![payload_suffix.clone()],
                ty: types.i32(),
                kind: crate::resource::host_size_contract::HostSizeKind::ArgsCount,
            }]
        );
        assert_eq!(
            snapshot.type_sizes,
            vec![OwnerTypeSizeReturn {
                suffix: vec![
                    payload_suffix,
                    crate::resource::model::PlaceProjection::TupleField {
                        index: 1,
                        offset_bytes: 0,
                    },
                ],
                ty: types.i32(),
                element_ty: types.unit(),
            }]
        );
        assert_eq!(
            snapshot.conditions,
            vec![OwnerVariantCondition {
                variant: "Ok".to_string(),
                condition: OwnerValueCondition::Always,
            }]
        );
        assert!(snapshot.indices.is_empty());
        assert!(snapshot.sources.is_empty());
        assert!(snapshot.extents.is_empty());
        assert!(snapshot.payload_conditions.is_empty());
        assert!(snapshot.returns.is_empty());
    }

    #[test]
    fn constructed_path_records_returned_parameter_owner_projection() {
        let mut types = TypeCtx::new();
        let owner_ty = types.box_ty(types.unit());
        let result_ty = types.register_named(
            "ProjectionReturnResult".to_string(),
            TypeKind::Enum {
                name: "ProjectionReturnResult".to_string(),
                type_params: Vec::new(),
                variants: vec![
                    EnumVariantInfo {
                        name: "Ok".to_string(),
                        payload: Some(owner_ty),
                    },
                    EnumVariantInfo {
                        name: "Err".to_string(),
                        payload: Some(types.unit()),
                    },
                ],
            },
        );
        let parameter = Place::local("parameter".to_string(), owner_ty);
        let value = Place::local("value".to_string(), result_ty);
        let span = Span::dummy();
        let mut owners = OwnerTable::default();
        owners.allocate(&parameter);
        let source = OwnerProjectionSource {
            parameter_index: 6,
            suffix: Vec::new(),
            ty: owner_ty,
        };
        let sources = [OwnerParameterStorageSource {
            storage: StorageId(0),
            source: source.clone(),
            place: parameter.clone(),
        }];
        let ops = [ResourceOp::Construct {
            output: value.clone(),
            kind: AggregateKind::Enum {
                name: "ProjectionReturnResult".to_string(),
                variant: "Ok".to_string(),
            },
            inputs: vec![parameter],
            span,
        }];

        let (result, snapshot) = run_path_with_seeded_summary_snapshot(
            &types,
            &owners,
            &sources,
            &ops,
            &value,
            &PendingVariantOwnerEffects::default(),
            None,
            None,
        );

        assert!(result.engine_effects().is_complete());
        assert_eq!(
            snapshot.returns,
            vec![OwnerVariantProjectionReturn {
                variant: "Ok".to_string(),
                suffix: vec![crate::resource::model::PlaceProjection::EnumPayload {
                    variant: "Ok".to_string(),
                }],
                ty: owner_ty,
                source_condition: None,
                owner: crate::resource::summary::OwnerProjectionReturnOwner::Parameter {
                    source,
                    returned_extent: OwnerExtentSummary::Unknown,
                },
            }]
        );
        assert!(snapshot.indices.is_empty());
        assert!(snapshot.sources.is_empty());
        assert!(snapshot.extents.is_empty());
        assert!(snapshot.payload_conditions.is_empty());
        assert!(snapshot.host_sizes.is_empty());
        assert!(snapshot.type_sizes.is_empty());
        assert_eq!(
            snapshot.conditions,
            vec![OwnerVariantCondition {
                variant: "Ok".to_string(),
                condition: OwnerValueCondition::Always,
            }]
        );
    }

    #[test]
    fn constructed_path_keeps_all_variant_summary_channels_nonempty() {
        let mut types = TypeCtx::new();
        let owner_ty = types.box_ty(types.unit());
        let payload_fields = [owner_ty, types.i32(), types.i32(), types.i32()];
        let payload_ty = types.tuple(payload_fields.to_vec());
        let result_ty = types.register_named(
            "CombinedVariantSummaryResult".to_string(),
            TypeKind::Enum {
                name: "CombinedVariantSummaryResult".to_string(),
                type_params: Vec::new(),
                variants: vec![EnumVariantInfo {
                    name: "Ok".to_string(),
                    payload: Some(payload_ty),
                }],
            },
        );
        let returned = Place::local("returned".to_string(), owner_ty);
        let consumed_root = Place::local("consumed_root".to_string(), owner_ty);
        let projection_base_ty = types.tuple(vec![types.i32(), owner_ty]);
        let projection_base = Place::local("projection_base".to_string(), projection_base_ty);
        let projection_suffix = vec![crate::resource::model::PlaceProjection::TupleField {
            index: 1,
            offset_bytes: composite_field_offset_bytes(&types, &[types.i32(), owner_ty], 1),
        }];
        let consumed_projection =
            projection_base
                .clone()
                .with_projection(projection_suffix[0].clone(), owner_ty);
        let host_size = Place::local("host_size".to_string(), types.i32());
        let type_size = Place::local("type_size".to_string(), types.i32());
        let conditioned = Place::local("conditioned".to_string(), types.i32());
        let extent_size = Place::local("extent_size".to_string(), types.i32());
        let moved_root = Place::local("moved_root".to_string(), owner_ty);
        let moved_projection = Place::local("moved_projection".to_string(), owner_ty);
        let payload = Place::local("payload".to_string(), payload_ty);
        let value = Place::local("value".to_string(), result_ty);
        let span = Span::dummy();
        let mut owners = OwnerTable::default();
        owners.allocate(&returned);
        owners.allocate(&consumed_root);
        owners.allocate(&consumed_projection);
        let returned_source = OwnerProjectionSource {
            parameter_index: 10,
            suffix: Vec::new(),
            ty: owner_ty,
        };
        let consumed_root_source = OwnerProjectionSource {
            parameter_index: 11,
            suffix: Vec::new(),
            ty: owner_ty,
        };
        let consumed_projection_source = OwnerProjectionSource {
            parameter_index: 12,
            suffix: projection_suffix,
            ty: owner_ty,
        };
        let storage_sources = [
            OwnerParameterStorageSource {
                storage: StorageId(0),
                source: returned_source,
                place: returned.clone(),
            },
            OwnerParameterStorageSource {
                storage: StorageId(1),
                source: consumed_root_source,
                place: consumed_root.clone(),
            },
            OwnerParameterStorageSource {
                storage: StorageId(2),
                source: consumed_projection_source,
                place: consumed_projection.clone(),
            },
        ];
        let extent_size_source = OwnerProjectionSource {
            parameter_index: 13,
            suffix: Vec::new(),
            ty: types.i32(),
        };
        let condition_sources = [OwnerParameterConditionSource {
            source: extent_size_source,
            place: extent_size.clone(),
        }];
        let extent_requirements = [
            crate::resource::owner_extent::PendingOwnerExtentRequirement {
                owner: consumed_projection.clone(),
                expected: OwnerStorageExtent::PayloadBytes {
                    bytes: Box::new(extent_size),
                },
                operation: crate::resource::report::ResourceOwnerOperation::CallArgument,
            },
        ];
        let mut raw_aliases = RawCellAddressAliases::default();
        raw_aliases.set_host_size_kind(
            &host_size,
            crate::resource::host_size_contract::HostSizeKind::ArgsCount,
        );
        raw_aliases.set_i32_type_size(&type_size, types.unit());
        raw_aliases.add_i32_condition(
            &conditioned,
            crate::resource::model::I32ValueCondition::Positive,
        );
        let payload_offsets = (0..payload_fields.len())
            .map(|index| composite_field_offset_bytes(&types, &payload_fields, index))
            .collect();
        let ops = [
            ResourceOp::Move {
                source: consumed_root,
                output: moved_root,
                span,
            },
            ResourceOp::Move {
                source: consumed_projection,
                output: moved_projection,
                span,
            },
            ResourceOp::Construct {
                output: payload.clone(),
                kind: AggregateKind::Tuple {
                    field_offsets: payload_offsets,
                },
                inputs: vec![returned, host_size, type_size, conditioned],
                span,
            },
            ResourceOp::Construct {
                output: value.clone(),
                kind: AggregateKind::Enum {
                    name: "CombinedVariantSummaryResult".to_string(),
                    variant: "Ok".to_string(),
                },
                inputs: vec![payload],
                span,
            },
        ];

        let (result, snapshot) = run_path_with_seeded_extent_summary_snapshot(
            &types,
            &owners,
            &raw_aliases,
            &storage_sources,
            &condition_sources,
            &extent_requirements,
            &ops,
            &value,
            &PendingVariantOwnerEffects::default(),
            None,
            None,
        );

        assert!(result.engine_effects().is_complete());
        assert_eq!(snapshot.indices.len(), 1);
        assert_eq!(snapshot.sources.len(), 1);
        assert_eq!(snapshot.extents.len(), 1);
        assert_eq!(snapshot.conditions.len(), 1);
        assert_eq!(snapshot.payload_conditions.len(), 3);
        assert_eq!(snapshot.host_sizes.len(), 1);
        assert_eq!(snapshot.type_sizes.len(), 1);
        assert_eq!(snapshot.returns.len(), 1);
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
    fn constructed_path_with_loop_replay_keeps_complete_effects() {
        let mut types = TypeCtx::new();
        let value = Place::local("value".to_string(), TypeId(0));
        let loop_retained = Place::local("loop_retained".to_string(), TypeId(0));
        let retained = Place::local("retained".to_string(), TypeId(0));
        let span = Span::dummy();
        let (loop_op, effects) = loop_with_nested_match_diagnostic(&mut types, &loop_retained);
        let ops = [
            loop_op,
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
        ];
        let mut result = run_path_with_match(
            &types,
            &ops,
            &value,
            &effects,
            None,
            None,
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
        let mut generic_engine = make_engine();
        let mut generic_state = OwnerMatchPathState::from_parent(
            &OwnerTable::default(),
            &FunctionAliasTable::default(),
            &RawCellAddressAliases::default(),
            &RawAddressViewTable::default(),
            &StorageOriginTable::default(),
            &PendingRawReallocs::default(),
            &effects,
        );
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
        let mut absorbed_engine = make_engine();
        result.take_engine_effects().absorb_into(&mut absorbed_engine);

        assert_eq!(result.state.oracle_snapshot(), generic_state.oracle_snapshot());
        assert_eq!(
            absorbed_engine.match_oracle_snapshot(),
            generic_engine.match_oracle_snapshot()
        );
        assert_eq!(absorbed_engine.match_oracle_snapshot().diagnostic_count(), 1);
        assert_eq!(
            result.state.storage_origins.origin(&loop_retained),
            Some(StorageOrigin::Owned)
        );
        assert_eq!(
            result.state.storage_origins.origin(&retained),
            Some(StorageOrigin::Owned)
        );
    }

    #[test]
    fn recursive_path_captures_loop_and_post_op_effects_in_order() {
        let mut types = TypeCtx::new();
        let value = Place::local("value".to_string(), TypeId(0));
        let loop_retained = Place::local("loop_retained".to_string(), TypeId(0));
        let retained = Place::local("retained".to_string(), TypeId(0));
        let span = Span::dummy();
        let (loop_op, effects) = loop_with_nested_match_diagnostic(&mut types, &loop_retained);
        let ops = [
                loop_op,
                ResourceOp::StorageOrigin {
                    target: retained.clone(),
                    origin: StorageOrigin::Owned,
                    span,
                },
            ];
        let mut result = run_path_with_match(
            &types,
            &ops,
            &value,
            &effects,
            None,
            None,
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
        let mut generic_engine = make_engine();
        let mut generic_state = OwnerMatchPathState::from_parent(
            &OwnerTable::default(),
            &FunctionAliasTable::default(),
            &RawCellAddressAliases::default(),
            &RawAddressViewTable::default(),
            &StorageOriginTable::default(),
            &PendingRawReallocs::default(),
            &effects,
        );
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
        let mut absorbed_engine = make_engine();
        result.take_engine_effects().absorb_into(&mut absorbed_engine);

        assert_eq!(result.state.oracle_snapshot(), generic_state.oracle_snapshot());
        assert_eq!(
            absorbed_engine.match_oracle_snapshot(),
            generic_engine.match_oracle_snapshot()
        );
        assert_eq!(absorbed_engine.match_oracle_snapshot().diagnostic_count(), 1);
        assert_eq!(
            result.state.storage_origins.origin(&loop_retained),
            Some(StorageOrigin::Owned)
        );
        assert_eq!(
            result.state.storage_origins.origin(&retained),
            Some(StorageOrigin::Owned)
        );
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
