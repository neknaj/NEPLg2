use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeKind;

use super::condition_fact::record_condition_fact_value_constraints;
use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place, ResourceConditionFact, ResourceMatchArm, ResourceOp};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_match_payload::retire_inactive_enum_payload_owners;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::{match_arm_variant_payload_name, match_bind_payload_place};
use super::raw_realloc::{
    raw_realloc_condition_outcome, PendingRawReallocs, RawReallocConditionOutcome,
};
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;

pub(super) struct OwnerMatchPathState {
    pub(super) owners: OwnerTable,
    pub(super) function_aliases: FunctionAliasTable,
    pub(super) raw_aliases: RawCellAddressAliases,
    pub(super) raw_views: RawAddressViewTable,
    pub(super) storage_origins: StorageOriginTable,
    pub(super) pending_reallocs: PendingRawReallocs,
    pub(super) variant_owner_effects: PendingVariantOwnerEffects,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerMatchOracleSnapshot {
    owners: (Vec<super::model::OwnerStateEntry>, usize),
    function_aliases: FunctionAliasTable,
    raw_aliases: RawCellAddressAliases,
    raw_views: Vec<(Place, super::owner_raw_view_model::RawAddressViewOwnership)>,
    storage_origins: (Vec<super::model::StorageOriginEntry>, Vec<(Place, Place)>),
    pending_reallocs: PendingRawReallocs,
    variant_owner_effects: PendingVariantOwnerEffects,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct OwnerMatchEngineOracleSnapshot {
    diagnostics: Vec<super::report::ResourceOwnerDiagnostic>,
    deferred: super::report::ResourceOwnerCheckDeferred,
    owner_extent_requirements: Vec<super::owner_extent::PendingOwnerExtentRequirement>,
    memory_span_requirements: Vec<super::summary::OwnerMemorySpanRequirement>,
}

impl OwnerMatchPathState {
    pub(super) fn from_parent(
        owners: &OwnerTable,
        function_aliases: &FunctionAliasTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &StorageOriginTable,
        pending_reallocs: &PendingRawReallocs,
        variant_owner_effects: &PendingVariantOwnerEffects,
    ) -> Self {
        Self {
            owners: owners.clone(),
            function_aliases: function_aliases.clone(),
            raw_aliases: raw_aliases.clone(),
            raw_views: raw_views.clone(),
            storage_origins: storage_origins.clone(),
            pending_reallocs: pending_reallocs.clone(),
            variant_owner_effects: variant_owner_effects.clone(),
        }
    }

    #[cfg(test)]
    pub(super) fn oracle_snapshot(&self) -> OwnerMatchOracleSnapshot {
        OwnerMatchOracleSnapshot {
            owners: self.owners.oracle_snapshot(),
            function_aliases: self.function_aliases.clone(),
            raw_aliases: self.raw_aliases.clone(),
            raw_views: self.raw_views.oracle_snapshot(),
            storage_origins: self.storage_origins.oracle_snapshot(),
            pending_reallocs: self.pending_reallocs.clone(),
            variant_owner_effects: self.variant_owner_effects.clone(),
        }
    }
}

#[derive(Default)]
pub(super) struct OwnerMatchPathStates {
    owner_paths: Vec<OwnerTable>,
    function_alias_paths: Vec<FunctionAliasTable>,
    raw_alias_paths: Vec<RawCellAddressAliases>,
    raw_view_paths: Vec<RawAddressViewTable>,
    storage_origin_paths: Vec<StorageOriginTable>,
    pending_realloc_paths: Vec<PendingRawReallocs>,
    variant_owner_effect_paths: Vec<PendingVariantOwnerEffects>,
}

impl OwnerMatchPathStates {
    pub(super) fn push(&mut self, path: OwnerMatchPathState) {
        self.owner_paths.push(path.owners);
        self.function_alias_paths.push(path.function_aliases);
        self.raw_alias_paths.push(path.raw_aliases);
        self.raw_view_paths.push(path.raw_views);
        self.storage_origin_paths.push(path.storage_origins);
        self.pending_realloc_paths.push(path.pending_reallocs);
        self.variant_owner_effect_paths
            .push(path.variant_owner_effects);
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn merge_match_path_states(
    owners: &mut OwnerTable,
    function_aliases: &mut FunctionAliasTable,
    raw_aliases: &mut RawCellAddressAliases,
    raw_views: &mut RawAddressViewTable,
    storage_origins: &mut StorageOriginTable,
    pending_reallocs: &mut PendingRawReallocs,
    variant_owner_effects: &mut PendingVariantOwnerEffects,
    paths: OwnerMatchPathStates,
) -> bool {
    if paths.owner_paths.is_empty() {
        return false;
    }
    let merged_raw_aliases = RawCellAddressAliases::merge_paths(&paths.raw_alias_paths);
    *owners = OwnerTable::merge_paths_with_raw_aliases(&paths.owner_paths, &merged_raw_aliases);
    *function_aliases = FunctionAliasTable::merge_paths(&paths.function_alias_paths);
    *raw_aliases = merged_raw_aliases;
    *raw_views = RawAddressViewTable::merge_paths(&paths.raw_view_paths);
    *storage_origins = StorageOriginTable::merge_paths(&paths.storage_origin_paths);
    *pending_reallocs = PendingRawReallocs::merge_paths(&paths.pending_realloc_paths);
    *variant_owner_effects =
        PendingVariantOwnerEffects::merge_paths(&paths.variant_owner_effect_paths);
    true
}

impl ResourceOwnerCheckEngine<'_> {
    #[cfg(test)]
    pub(super) fn match_oracle_snapshot(&self) -> OwnerMatchEngineOracleSnapshot {
        OwnerMatchEngineOracleSnapshot {
            diagnostics: self.diagnostics.clone(),
            deferred: self.deferred,
            owner_extent_requirements: self.owner_extent_requirements.clone(),
            memory_span_requirements: self.memory_span_requirements.clone(),
        }
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn run_generic_match_oracle(
        mut self,
        mut state: OwnerMatchPathState,
        output: &Place,
        scrutinee: &Place,
        arms: &[ResourceMatchArm],
        span: Span,
        post_ops: &[ResourceOp],
    ) -> (OwnerMatchOracleSnapshot, OwnerMatchEngineOracleSnapshot) {
        self.check_match(
            &mut state.owners,
            &mut state.function_aliases,
            &mut state.raw_aliases,
            &mut state.raw_views,
            &mut state.storage_origins,
            &mut state.pending_reallocs,
            &mut state.variant_owner_effects,
            output,
            scrutinee,
            arms,
            span,
        );
        self.check_ops(
            &mut state.owners,
            &mut state.function_aliases,
            &mut state.raw_aliases,
            &mut state.raw_views,
            &mut state.storage_origins,
            &mut state.pending_reallocs,
            &mut state.variant_owner_effects,
            post_ops,
        );
        (state.oracle_snapshot(), self.match_oracle_snapshot())
    }

    #[cfg(test)]
    #[allow(clippy::too_many_arguments)]
    pub(super) fn run_specialized_match_shadow_oracle(
        mut self,
        mut state: OwnerMatchPathState,
        output: &Place,
        scrutinee: &Place,
        arms: &[ResourceMatchArm],
        span: Span,
        post_ops: &[ResourceOp],
    ) -> (OwnerMatchOracleSnapshot, OwnerMatchEngineOracleSnapshot) {
        let mut arm_paths = OwnerMatchPathStates::default();
        for arm in arms {
            let Some(mut arm_state) = self.prepare_match_arm_path(
                &state.owners,
                &state.function_aliases,
                &state.raw_aliases,
                &state.raw_views,
                &state.storage_origins,
                &state.pending_reallocs,
                &state.variant_owner_effects,
                scrutinee,
                arm,
                span,
            ) else {
                continue;
            };
            self.check_ops(
                &mut arm_state.owners,
                &mut arm_state.function_aliases,
                &mut arm_state.raw_aliases,
                &mut arm_state.raw_views,
                &mut arm_state.storage_origins,
                &mut arm_state.pending_reallocs,
                &mut arm_state.variant_owner_effects,
                &arm.ops,
            );
            if self.finalize_match_arm_value(&mut arm_state, output, &arm.value, span) {
                arm_paths.push(arm_state);
            }
        }
        merge_match_path_states(
            &mut state.owners,
            &mut state.function_aliases,
            &mut state.raw_aliases,
            &mut state.raw_views,
            &mut state.storage_origins,
            &mut state.pending_reallocs,
            &mut state.variant_owner_effects,
            arm_paths,
        );
        self.check_ops(
            &mut state.owners,
            &mut state.function_aliases,
            &mut state.raw_aliases,
            &mut state.raw_views,
            &mut state.storage_origins,
            &mut state.pending_reallocs,
            &mut state.variant_owner_effects,
            post_ops,
        );
        (state.oracle_snapshot(), self.match_oracle_snapshot())
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_match_arm_path(
        &mut self,
        owners: &OwnerTable,
        function_aliases: &FunctionAliasTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &StorageOriginTable,
        pending_reallocs: &PendingRawReallocs,
        variant_owner_effects: &PendingVariantOwnerEffects,
        scrutinee: &Place,
        arm: &ResourceMatchArm,
        span: Span,
    ) -> Option<OwnerMatchPathState> {
        if !variant_owner_effects.match_arm_reachable(scrutinee, &arm.pattern) {
            return None;
        }
        let mut state = OwnerMatchPathState::from_parent(
            owners,
            function_aliases,
            raw_aliases,
            raw_views,
            storage_origins,
            pending_reallocs,
            variant_owner_effects,
        );
        if let Some(selected_variant) = match_arm_variant_payload_name(arm) {
            retire_inactive_enum_payload_owners(
                &mut state.owners,
                &mut state.raw_aliases,
                &mut state.raw_views,
                &mut state.storage_origins,
                scrutinee,
                &selected_variant,
            );
        }
        state.variant_owner_effects.apply_match_arm_returns(
            self,
            &mut state.owners,
            &mut state.raw_aliases,
            &mut state.raw_views,
            &mut state.storage_origins,
            scrutinee,
            &arm.pattern,
            span,
        );
        state
            .variant_owner_effects
            .apply_match_arm_value_conditions(&mut state.raw_aliases, scrutinee, &arm.pattern);
        if let Some(bind_local) = &arm.bind_local {
            if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
                if !state.variant_owner_effects.reject_reserved_source_use(
                    self,
                    &state.owners,
                    &state.raw_aliases,
                    &source,
                    ResourceOwnerOperation::MatchValue,
                    span,
                ) {
                    state
                        .raw_aliases
                        .copy_scalar_facts_if_tracked(&source, bind_local);
                    self.transfer_owner(
                        &mut state.owners,
                        &mut state.raw_aliases,
                        &state.raw_views,
                        &mut state.storage_origins,
                        &source,
                        bind_local,
                        ResourceOwnerOperation::MatchValue,
                        span,
                    );
                }
                state.function_aliases.copy_alias(&source, bind_local);
                state.raw_views.copy(&source, bind_local);
                state.pending_reallocs.copy_result(&source, bind_local);
                state.variant_owner_effects.copy_result(&source, bind_local);
                state
                    .variant_owner_effects
                    .apply_match_arm_payload_conditions(
                        &mut state.raw_aliases,
                        scrutinee,
                        &arm.pattern,
                        Some(bind_local),
                    );
            } else {
                state.raw_aliases.clear(bind_local);
                state.raw_views.clear(bind_local);
                state.storage_origins.clear(bind_local);
                state.pending_reallocs.clear_result(bind_local);
                state.variant_owner_effects.clear_result(bind_local);
            }
        } else {
            state
                .variant_owner_effects
                .apply_match_arm_payload_conditions(
                    &mut state.raw_aliases,
                    scrutinee,
                    &arm.pattern,
                    None,
                );
        }
        state.variant_owner_effects.apply_match_arm(
            self,
            &mut state.owners,
            &mut state.raw_aliases,
            &mut state.raw_views,
            &mut state.storage_origins,
            scrutinee,
            &arm.pattern,
            span,
        );
        Some(state)
    }

    pub(super) fn finalize_match_arm_value(
        &mut self,
        state: &mut OwnerMatchPathState,
        output: &Place,
        value: &Place,
        span: Span,
    ) -> bool {
        if self.place_is_never(value) {
            return false;
        }
        if !state.variant_owner_effects.has_result_effects(value) {
            state
                .variant_owner_effects
                .materialize_result_owner_effects(
                    self,
                    &mut state.owners,
                    &mut state.raw_aliases,
                    &mut state.raw_views,
                    &mut state.storage_origins,
                    value,
                    span,
                );
        }
        if !state.variant_owner_effects.reject_reserved_source_use(
            self,
            &state.owners,
            &state.raw_aliases,
            value,
            ResourceOwnerOperation::MatchValue,
            span,
        ) {
            state
                .raw_aliases
                .copy_scalar_facts_if_tracked(value, output);
            self.transfer_owner(
                &mut state.owners,
                &mut state.raw_aliases,
                &state.raw_views,
                &mut state.storage_origins,
                value,
                output,
                ResourceOwnerOperation::MatchValue,
                span,
            );
            state.raw_views.copy_non_owning(value, output);
            state.pending_reallocs.copy_result(value, output);
            state.variant_owner_effects.copy_result(value, output);
            if state
                .variant_owner_effects
                .result_effects_have_temporary_sources(&state.raw_aliases, output)
            {
                state
                    .variant_owner_effects
                    .materialize_result_owner_effects(
                        self,
                        &mut state.owners,
                        &mut state.raw_aliases,
                        &mut state.raw_views,
                        &mut state.storage_origins,
                        output,
                        span,
                    );
            }
        }
        true
    }

    pub(super) fn check_branch(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        output: &Place,
        condition_fact: Option<&ResourceConditionFact>,
        then_ops: &[ResourceOp],
        then_value: &Place,
        else_ops: &[ResourceOp],
        else_value: &Place,
        span: Span,
    ) {
        let mut then_owners = owners.clone();
        let mut else_owners = owners.clone();
        let mut then_function_aliases = function_aliases.clone();
        let mut else_function_aliases = function_aliases.clone();
        let mut then_raw_aliases = raw_aliases.clone();
        let mut else_raw_aliases = raw_aliases.clone();
        let mut then_raw_views = raw_views.clone();
        let mut else_raw_views = raw_views.clone();
        let mut then_storage_origins = storage_origins.clone();
        let mut else_storage_origins = storage_origins.clone();
        let mut then_pending_reallocs = pending_reallocs.clone();
        let mut else_pending_reallocs = pending_reallocs.clone();
        let mut then_variant_owner_effects = variant_owner_effects.clone();
        let mut else_variant_owner_effects = variant_owner_effects.clone();

        self.apply_branch_condition_fact(
            &mut then_owners,
            &mut then_raw_aliases,
            &then_raw_views,
            &mut then_storage_origins,
            &mut then_pending_reallocs,
            condition_fact,
            true,
            span,
        );
        self.apply_branch_condition_fact(
            &mut else_owners,
            &mut else_raw_aliases,
            &else_raw_views,
            &mut else_storage_origins,
            &mut else_pending_reallocs,
            condition_fact,
            false,
            span,
        );
        self.check_ops(
            &mut then_owners,
            &mut then_function_aliases,
            &mut then_raw_aliases,
            &mut then_raw_views,
            &mut then_storage_origins,
            &mut then_pending_reallocs,
            &mut then_variant_owner_effects,
            then_ops,
        );
        self.check_ops(
            &mut else_owners,
            &mut else_function_aliases,
            &mut else_raw_aliases,
            &mut else_raw_views,
            &mut else_storage_origins,
            &mut else_pending_reallocs,
            &mut else_variant_owner_effects,
            else_ops,
        );

        let mut owner_paths = Vec::new();
        let mut function_alias_paths = Vec::new();
        let mut raw_alias_paths = Vec::new();
        let mut raw_view_paths = Vec::new();
        let mut storage_origin_paths = Vec::new();
        let mut pending_realloc_paths = Vec::new();
        let mut variant_owner_effect_paths = Vec::new();
        if !self.place_is_never(then_value) {
            if !then_variant_owner_effects.has_result_effects(then_value) {
                then_variant_owner_effects.materialize_result_owner_effects(
                    self,
                    &mut then_owners,
                    &mut then_raw_aliases,
                    &mut then_raw_views,
                    &mut then_storage_origins,
                    then_value,
                    span,
                );
            }
            if !then_variant_owner_effects.reject_reserved_source_use(
                self,
                &then_owners,
                &then_raw_aliases,
                then_value,
                ResourceOwnerOperation::BranchValue,
                span,
            ) {
                then_raw_aliases.copy_scalar_facts_if_tracked(then_value, output);
                self.transfer_owner(
                    &mut then_owners,
                    &mut then_raw_aliases,
                    &then_raw_views,
                    &mut then_storage_origins,
                    then_value,
                    output,
                    ResourceOwnerOperation::BranchValue,
                    span,
                );
                then_raw_views.copy_non_owning(then_value, output);
                then_pending_reallocs.copy_result(then_value, output);
                then_variant_owner_effects.copy_result(then_value, output);
                if then_variant_owner_effects
                    .result_effects_have_temporary_sources(&then_raw_aliases, output)
                {
                    then_variant_owner_effects.materialize_result_owner_effects(
                        self,
                        &mut then_owners,
                        &mut then_raw_aliases,
                        &mut then_raw_views,
                        &mut then_storage_origins,
                        output,
                        span,
                    );
                }
            }
            owner_paths.push(then_owners);
            function_alias_paths.push(then_function_aliases);
            raw_alias_paths.push(then_raw_aliases);
            raw_view_paths.push(then_raw_views);
            storage_origin_paths.push(then_storage_origins);
            pending_realloc_paths.push(then_pending_reallocs);
            variant_owner_effect_paths.push(then_variant_owner_effects);
        }
        if !self.place_is_never(else_value) {
            if !else_variant_owner_effects.has_result_effects(else_value) {
                else_variant_owner_effects.materialize_result_owner_effects(
                    self,
                    &mut else_owners,
                    &mut else_raw_aliases,
                    &mut else_raw_views,
                    &mut else_storage_origins,
                    else_value,
                    span,
                );
            }
            if !else_variant_owner_effects.reject_reserved_source_use(
                self,
                &else_owners,
                &else_raw_aliases,
                else_value,
                ResourceOwnerOperation::BranchValue,
                span,
            ) {
                else_raw_aliases.copy_scalar_facts_if_tracked(else_value, output);
                self.transfer_owner(
                    &mut else_owners,
                    &mut else_raw_aliases,
                    &else_raw_views,
                    &mut else_storage_origins,
                    else_value,
                    output,
                    ResourceOwnerOperation::BranchValue,
                    span,
                );
                else_raw_views.copy_non_owning(else_value, output);
                else_pending_reallocs.copy_result(else_value, output);
                else_variant_owner_effects.copy_result(else_value, output);
                if else_variant_owner_effects
                    .result_effects_have_temporary_sources(&else_raw_aliases, output)
                {
                    else_variant_owner_effects.materialize_result_owner_effects(
                        self,
                        &mut else_owners,
                        &mut else_raw_aliases,
                        &mut else_raw_views,
                        &mut else_storage_origins,
                        output,
                        span,
                    );
                }
            }
            owner_paths.push(else_owners);
            function_alias_paths.push(else_function_aliases);
            raw_alias_paths.push(else_raw_aliases);
            raw_view_paths.push(else_raw_views);
            storage_origin_paths.push(else_storage_origins);
            pending_realloc_paths.push(else_pending_reallocs);
            variant_owner_effect_paths.push(else_variant_owner_effects);
        }
        if !owner_paths.is_empty() {
            let merged_raw_aliases = RawCellAddressAliases::merge_paths(&raw_alias_paths);
            *owners = OwnerTable::merge_paths_with_raw_aliases(&owner_paths, &merged_raw_aliases);
            *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
            *raw_aliases = merged_raw_aliases;
            *raw_views = RawAddressViewTable::merge_paths(&raw_view_paths);
            *storage_origins = StorageOriginTable::merge_paths(&storage_origin_paths);
            *pending_reallocs = PendingRawReallocs::merge_paths(&pending_realloc_paths);
            *variant_owner_effects =
                PendingVariantOwnerEffects::merge_paths(&variant_owner_effect_paths);
        }
    }

    pub(super) fn check_loop(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        condition_ops: &[ResourceOp],
        condition_fact: Option<&ResourceConditionFact>,
        body_ops: &[ResourceOp],
        span: Span,
    ) {
        let mut condition_owners = owners.clone();
        let mut condition_function_aliases = function_aliases.clone();
        let mut condition_raw_aliases = raw_aliases.clone();
        let mut condition_raw_views = raw_views.clone();
        let mut condition_storage_origins = storage_origins.clone();
        let mut condition_pending_reallocs = pending_reallocs.clone();
        let mut condition_variant_owner_effects = variant_owner_effects.clone();
        self.check_ops(
            &mut condition_owners,
            &mut condition_function_aliases,
            &mut condition_raw_aliases,
            &mut condition_raw_views,
            &mut condition_storage_origins,
            &mut condition_pending_reallocs,
            &mut condition_variant_owner_effects,
            condition_ops,
        );

        let mut exit_owners = condition_owners.clone();
        let mut exit_raw_aliases = condition_raw_aliases.clone();
        let mut exit_storage_origins = condition_storage_origins.clone();
        let mut exit_pending_reallocs = condition_pending_reallocs.clone();
        self.apply_branch_condition_fact(
            &mut exit_owners,
            &mut exit_raw_aliases,
            &condition_raw_views,
            &mut exit_storage_origins,
            &mut exit_pending_reallocs,
            condition_fact,
            false,
            span,
        );

        let mut body_owners = condition_owners;
        let mut body_function_aliases = condition_function_aliases.clone();
        let mut body_raw_aliases = condition_raw_aliases;
        let mut body_raw_views = condition_raw_views.clone();
        let mut body_storage_origins = condition_storage_origins;
        let mut body_pending_reallocs = condition_pending_reallocs;
        let mut body_variant_owner_effects = condition_variant_owner_effects.clone();
        self.apply_branch_condition_fact(
            &mut body_owners,
            &mut body_raw_aliases,
            &body_raw_views,
            &mut body_storage_origins,
            &mut body_pending_reallocs,
            condition_fact,
            true,
            span,
        );
        self.check_ops(
            &mut body_owners,
            &mut body_function_aliases,
            &mut body_raw_aliases,
            &mut body_raw_views,
            &mut body_storage_origins,
            &mut body_pending_reallocs,
            &mut body_variant_owner_effects,
            body_ops,
        );

        let merged_raw_aliases =
            RawCellAddressAliases::merge_paths(&[exit_raw_aliases, body_raw_aliases]);
        *owners = OwnerTable::merge_paths_with_raw_aliases(
            &[exit_owners, body_owners],
            &merged_raw_aliases,
        );
        *function_aliases =
            FunctionAliasTable::merge_paths(&[condition_function_aliases, body_function_aliases]);
        *raw_aliases = merged_raw_aliases;
        *raw_views = RawAddressViewTable::merge_paths(&[condition_raw_views, body_raw_views]);
        *storage_origins =
            StorageOriginTable::merge_paths(&[exit_storage_origins, body_storage_origins]);
        *pending_reallocs =
            PendingRawReallocs::merge_paths(&[exit_pending_reallocs, body_pending_reallocs]);
        *variant_owner_effects = PendingVariantOwnerEffects::merge_paths(&[
            condition_variant_owner_effects,
            body_variant_owner_effects,
        ]);
    }

    pub(super) fn check_match(
        &mut self,
        owners: &mut OwnerTable,
        function_aliases: &mut FunctionAliasTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_owner_effects: &mut PendingVariantOwnerEffects,
        output: &Place,
        scrutinee: &Place,
        arms: &[ResourceMatchArm],
        span: Span,
    ) {
        let mut arm_paths = OwnerMatchPathStates::default();

        for arm in arms {
            let Some(mut arm_state) = self.prepare_match_arm_path(
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
            ) else {
                continue;
            };
            self.check_ops(
                &mut arm_state.owners,
                &mut arm_state.function_aliases,
                &mut arm_state.raw_aliases,
                &mut arm_state.raw_views,
                &mut arm_state.storage_origins,
                &mut arm_state.pending_reallocs,
                &mut arm_state.variant_owner_effects,
                &arm.ops,
            );
            if self.finalize_match_arm_value(&mut arm_state, output, &arm.value, span) {
                arm_paths.push(arm_state);
            }
        }
        merge_match_path_states(
            owners,
            function_aliases,
            raw_aliases,
            raw_views,
            storage_origins,
            pending_reallocs,
            variant_owner_effects,
            arm_paths,
        );
    }

    pub(super) fn apply_branch_condition_fact(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        pending_reallocs: &mut PendingRawReallocs,
        fact: Option<&ResourceConditionFact>,
        truthy_path: bool,
        span: Span,
    ) {
        let Some(fact) = fact else {
            return;
        };
        record_condition_fact_value_constraints(raw_aliases, fact, truthy_path);
        if let Some((place, outcome)) = raw_realloc_condition_outcome(fact, truthy_path) {
            match outcome {
                RawReallocConditionOutcome::Success => {
                    if let Some(pending) = pending_reallocs.take_for_result(place) {
                        self.transfer_owner(
                            owners,
                            raw_aliases,
                            raw_views,
                            storage_origins,
                            &pending.storage_source,
                            &pending.result,
                            ResourceOwnerOperation::ReallocInput,
                            span,
                        );
                        owners.set_live_extent(&pending.result, pending.new_extent);
                        return;
                    }
                }
                RawReallocConditionOutcome::Failure => {
                    if pending_reallocs.take_for_result(place).is_some() {
                        self.discard_non_owned_raw_address_owner(
                            owners,
                            raw_aliases,
                            storage_origins,
                            place,
                        );
                        return;
                    }
                }
            }
        }
        match fact {
            ResourceConditionFact::EqZero { place } if truthy_path => {
                self.discard_non_owned_raw_address_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    place,
                );
            }
            ResourceConditionFact::NeZero { place } if !truthy_path => {
                self.discard_non_owned_raw_address_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    place,
                );
            }
            ResourceConditionFact::Positive { place } if !truthy_path => {
                self.discard_non_owned_raw_address_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    place,
                );
            }
            ResourceConditionFact::NonPositive { place } if truthy_path => {
                self.discard_non_owned_raw_address_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    place,
                );
            }
            ResourceConditionFact::Negative { place } if truthy_path => {
                self.discard_non_owned_raw_address_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    place,
                );
            }
            ResourceConditionFact::NonNegative { place } if !truthy_path => {
                self.discard_non_owned_raw_address_owner(
                    owners,
                    raw_aliases,
                    storage_origins,
                    place,
                );
            }
            ResourceConditionFact::EqZero { .. }
            | ResourceConditionFact::NeZero { .. }
            | ResourceConditionFact::Positive { .. }
            | ResourceConditionFact::NonPositive { .. }
            | ResourceConditionFact::Negative { .. }
            | ResourceConditionFact::NonNegative { .. }
            | ResourceConditionFact::I32Relation { .. }
            | ResourceConditionFact::Any(_)
            | ResourceConditionFact::All(_) => {}
        }
    }

    fn discard_non_owned_raw_address_owner(
        &mut self,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        storage_origins: &mut StorageOriginTable,
        place: &Place,
    ) {
        if owners.state(place).is_none() && owners.descendant_entries(place).is_empty() {
            return;
        }
        let resolved_place =
            super::owner_alias::resolve_owner_alias_place(owners, raw_aliases, place);
        let descendants = owners.descendant_entries(&resolved_place);
        owners.set_state(&resolved_place, OwnerState::NoFreeObligation);
        storage_origins.clear(&resolved_place);
        raw_aliases.clear(place);
        raw_aliases.clear(&resolved_place);
        for entry in descendants {
            owners.set_state(&entry.place, OwnerState::NoFreeObligation);
            storage_origins.clear(&entry.place);
            raw_aliases.clear(&entry.place);
        }
    }

    fn place_is_never(&self, place: &Place) -> bool {
        matches!(
            self.types.get_ref(self.types.resolve_id(place.ty)),
            TypeKind::Never
        )
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;
    use alloc::vec;
    use alloc::vec::Vec;

    use super::*;
    use crate::types::{TypeCtx, TypeId};

    fn clone_match_state(state: &OwnerMatchPathState) -> OwnerMatchPathState {
        OwnerMatchPathState::from_parent(
            &state.owners,
            &state.function_aliases,
            &state.raw_aliases,
            &state.raw_views,
            &state.storage_origins,
            &state.pending_reallocs,
            &state.variant_owner_effects,
        )
    }

    fn empty_match_state() -> OwnerMatchPathState {
        OwnerMatchPathState {
            owners: OwnerTable::default(),
            function_aliases: FunctionAliasTable::default(),
            raw_aliases: RawCellAddressAliases::default(),
            raw_views: RawAddressViewTable::default(),
            storage_origins: StorageOriginTable::default(),
            pending_reallocs: PendingRawReallocs::default(),
            variant_owner_effects: PendingVariantOwnerEffects::default(),
        }
    }

    #[test]
    fn empty_match_path_merge_preserves_parent_state() {
        let mut owners = OwnerTable::default();
        let mut function_aliases = FunctionAliasTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut raw_views = RawAddressViewTable::default();
        let mut storage_origins = StorageOriginTable::default();
        let mut pending_reallocs = PendingRawReallocs::default();
        let mut variant_owner_effects = PendingVariantOwnerEffects::default();

        assert!(!merge_match_path_states(
            &mut owners,
            &mut function_aliases,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &mut pending_reallocs,
            &mut variant_owner_effects,
            OwnerMatchPathStates::default(),
        ));
    }

    #[test]
    fn match_oracle_snapshot_observes_owner_allocation_identity() {
        let base = empty_match_state();
        let mut allocated = OwnerMatchPathState::from_parent(
            &base.owners,
            &base.function_aliases,
            &base.raw_aliases,
            &base.raw_views,
            &base.storage_origins,
            &base.pending_reallocs,
            &base.variant_owner_effects,
        );
        allocated
            .owners
            .allocate(&Place::local("owner".to_string(), TypeId(0)));

        let base_snapshot = base.oracle_snapshot();
        assert_eq!(base_snapshot, base.oracle_snapshot());
        assert_ne!(base_snapshot, allocated.oracle_snapshot());

        let owner = Place::local("owner".to_string(), TypeId(0));
        let other = Place::local("other".to_string(), TypeId(0));

        let mut changed = clone_match_state(&base);
        changed.function_aliases.set_alias(
            &owner,
            crate::function_identity::FunctionValueIdentity::new(
                "f".to_string(),
                None,
                TypeId(0),
                crate::ast::Effect::Pure,
                Vec::new(),
            ),
            super::super::model::ResourceFunctionValueKind::Plain,
        );
        assert_ne!(base_snapshot, changed.oracle_snapshot());

        let mut changed = clone_match_state(&base);
        changed.raw_aliases.mark(&owner);
        assert_ne!(base_snapshot, changed.oracle_snapshot());

        let mut changed = clone_match_state(&base);
        changed.raw_views.mark_non_owning(&owner);
        assert_ne!(base_snapshot, changed.oracle_snapshot());

        let mut forward = clone_match_state(&base);
        forward.raw_views.mark(&owner);
        forward.raw_views.mark_non_owning(&other);
        let mut reverse = clone_match_state(&base);
        reverse.raw_views.mark_non_owning(&other);
        reverse.raw_views.mark(&owner);
        assert_ne!(forward.oracle_snapshot(), reverse.oracle_snapshot());

        let mut changed = clone_match_state(&base);
        changed.storage_origins.mark_owned(&owner);
        changed
            .storage_origins
            .mark_origin_source_for_oracle(&owner, other.clone());
        assert_ne!(base_snapshot, changed.oracle_snapshot());

        let mut changed = clone_match_state(&base);
        changed.pending_reallocs.mark(
            &owner,
            &owner,
            &other,
            super::super::model::OwnerStorageExtent::Unknown,
            Vec::new(),
        );
        assert_ne!(base_snapshot, changed.oracle_snapshot());

        let mut changed = clone_match_state(&base);
        changed.variant_owner_effects.unreachable_variants.push(
            super::super::owner_variant::PendingUnreachableVariant {
                result: owner,
                variant: "Err".to_string(),
            },
        );
        assert_ne!(base_snapshot, changed.oracle_snapshot());
    }

    #[test]
    fn match_engine_oracle_snapshot_observes_all_side_effect_channels() {
        let types = TypeCtx::new();
        let summaries = Vec::new();
        let summary_index = super::super::summary::OwnerReturnSummaryIndex::new(&summaries);
        let mut engine = ResourceOwnerCheckEngine {
            function: "oracle",
            types: &types,
            summaries: &summary_index,
            diagnostics: Vec::new(),
            deferred: super::super::report::ResourceOwnerCheckDeferred::default(),
            owner_extent_requirements: Vec::new(),
            memory_span_requirements: Vec::new(),
            params: &[],
            owner_leaf_projection_cache: Default::default(),
        };
        let snapshot = engine.match_oracle_snapshot();
        assert!(snapshot.diagnostics.is_empty());
        assert_eq!(
            snapshot.deferred,
            super::super::report::ResourceOwnerCheckDeferred::default()
        );
        assert!(snapshot.owner_extent_requirements.is_empty());
        assert!(snapshot.memory_span_requirements.is_empty());

        let owner = Place::local("owner".to_string(), TypeId(0));
        engine.diagnostics.push(
            super::super::report::ResourceOwnerDiagnostic::OwnerUnavailable {
                function: "oracle".to_string(),
                operation: ResourceOwnerOperation::Read,
                place: owner.clone(),
                state: OwnerState::Moved,
                span: Span::empty(crate::span::FileId(0), 0),
            },
        );
        assert_ne!(snapshot, engine.match_oracle_snapshot());
        engine.diagnostics.clear();

        engine.deferred.match_merges = 1;
        assert_ne!(snapshot, engine.match_oracle_snapshot());
        engine.deferred.match_merges = 0;

        engine.owner_extent_requirements.push(
            super::super::owner_extent::PendingOwnerExtentRequirement {
                owner,
                expected: super::super::model::OwnerStorageExtent::Unknown,
                operation: ResourceOwnerOperation::Read,
            },
        );
        assert_ne!(snapshot, engine.match_oracle_snapshot());
        engine.owner_extent_requirements.clear();

        engine
            .memory_span_requirements
            .push(super::super::summary::OwnerMemorySpanRequirement {
                span: super::super::host_memory_contract::HostMemorySpan::IovDescriptor {
                    iovs_arg: 0,
                    iov_count_arg: 1,
                },
                args: Vec::new(),
                operation: ResourceOwnerOperation::Read,
            });
        assert_ne!(snapshot, engine.match_oracle_snapshot());
    }

    #[test]
    fn generic_match_oracle_runs_post_match_ops_on_retained_state() {
        let types = TypeCtx::new();
        let summaries = Vec::new();
        let summary_index = super::super::summary::OwnerReturnSummaryIndex::new(&summaries);
        let mut engine = ResourceOwnerCheckEngine {
            function: "oracle",
            types: &types,
            summaries: &summary_index,
            diagnostics: Vec::new(),
            deferred: super::super::report::ResourceOwnerCheckDeferred::default(),
            owner_extent_requirements: Vec::new(),
            memory_span_requirements: Vec::new(),
            params: &[],
            owner_leaf_projection_cache: Default::default(),
        };
        engine.deferred.match_merges = 7;
        let output = Place::local("output".to_string(), TypeId(0));
        let scrutinee = Place::local("scrutinee".to_string(), TypeId(0));
        let retained = Place::local("retained".to_string(), TypeId(0));
        let span = Span::empty(crate::span::FileId(0), 0);

        let (state, engine_state) = engine.run_generic_match_oracle(
            empty_match_state(),
            &output,
            &scrutinee,
            &[],
            span,
            &[ResourceOp::StorageOrigin {
                target: retained.clone(),
                origin: super::super::model::StorageOrigin::Owned,
                span,
            }],
        );

        assert_eq!(
            state.storage_origins.0,
            vec![super::super::model::StorageOriginEntry {
                place: retained,
                origin: super::super::model::StorageOrigin::Owned,
            }]
        );
        assert_eq!(engine_state.deferred.match_merges, 7);
        assert!(engine_state.diagnostics.is_empty());
        assert!(engine_state.owner_extent_requirements.is_empty());
        assert!(engine_state.memory_span_requirements.is_empty());
    }

    #[test]
    fn specialized_match_shadow_matches_generic_with_nonempty_arm_and_post_ops() {
        let types = TypeCtx::new();
        let summaries = Vec::new();
        let summary_index = super::super::summary::OwnerReturnSummaryIndex::new(&summaries);
        let make_engine = || ResourceOwnerCheckEngine {
            function: "oracle",
            types: &types,
            summaries: &summary_index,
            diagnostics: Vec::new(),
            deferred: super::super::report::ResourceOwnerCheckDeferred {
                match_merges: 7,
                ..Default::default()
            },
            owner_extent_requirements: Vec::new(),
            memory_span_requirements: Vec::new(),
            params: &[],
            owner_leaf_projection_cache: Default::default(),
        };
        let output = Place::local("output".to_string(), TypeId(0));
        let scrutinee = Place::local("scrutinee".to_string(), TypeId(0));
        let value = Place::local("value".to_string(), TypeId(0));
        let retained = Place::local("retained".to_string(), TypeId(0));
        let span = Span::empty(crate::span::FileId(0), 0);
        let arms = vec![ResourceMatchArm {
            pattern: super::super::model::ResourceMatchPattern::Wildcard,
            bind_local: None,
            bind_source_name: None,
            bind_mode: None,
            ops: Vec::new(),
            value,
            span,
        }];
        let post_ops = vec![ResourceOp::StorageOrigin {
            target: retained,
            origin: super::super::model::StorageOrigin::Owned,
            span,
        }];
        let generic_state = empty_match_state();
        let shadow_state = clone_match_state(&generic_state);

        let generic = make_engine().run_generic_match_oracle(
            generic_state,
            &output,
            &scrutinee,
            &arms,
            span,
            &post_ops,
        );
        let shadow = make_engine().run_specialized_match_shadow_oracle(
            shadow_state,
            &output,
            &scrutinee,
            &arms,
            span,
            &post_ops,
        );

        assert_eq!(generic, shadow);
    }

    #[test]
    fn specialized_match_shadow_skips_unreachable_arm_like_generic() {
        let types = TypeCtx::new();
        let summaries = Vec::new();
        let summary_index = super::super::summary::OwnerReturnSummaryIndex::new(&summaries);
        let make_engine = || ResourceOwnerCheckEngine {
            function: "oracle",
            types: &types,
            summaries: &summary_index,
            diagnostics: Vec::new(),
            deferred: super::super::report::ResourceOwnerCheckDeferred::default(),
            owner_extent_requirements: Vec::new(),
            memory_span_requirements: Vec::new(),
            params: &[],
            owner_leaf_projection_cache: Default::default(),
        };
        let output = Place::local("output".to_string(), types.unit());
        let scrutinee = Place::local("scrutinee".to_string(), types.unit());
        let unreachable_value = Place::local("unreachable_value".to_string(), types.unit());
        let retained_value = Place::local("retained_value".to_string(), types.unit());
        let span = Span::empty(crate::span::FileId(0), 0);
        let arms = vec![
            ResourceMatchArm {
                pattern: super::super::model::ResourceMatchPattern::Variant("Err".to_string()),
                bind_local: None,
                bind_source_name: None,
                bind_mode: None,
                ops: vec![ResourceOp::StorageOrigin {
                    target: unreachable_value.clone(),
                    origin: super::super::model::StorageOrigin::Owned,
                    span,
                }],
                value: unreachable_value,
                span,
            },
            ResourceMatchArm {
                pattern: super::super::model::ResourceMatchPattern::Wildcard,
                bind_local: None,
                bind_source_name: None,
                bind_mode: None,
                ops: vec![ResourceOp::StorageOrigin {
                    target: retained_value.clone(),
                    origin: super::super::model::StorageOrigin::Owned,
                    span,
                }],
                value: retained_value,
                span,
            },
        ];
        let mut generic_state = empty_match_state();
        generic_state
            .variant_owner_effects
            .unreachable_variants
            .push(super::super::owner_variant::PendingUnreachableVariant {
                result: scrutinee.clone(),
                variant: "Err".to_string(),
            });
        assert!(make_engine()
            .prepare_match_arm_path(
                &generic_state.owners,
                &generic_state.function_aliases,
                &generic_state.raw_aliases,
                &generic_state.raw_views,
                &generic_state.storage_origins,
                &generic_state.pending_reallocs,
                &generic_state.variant_owner_effects,
                &scrutinee,
                &arms[0],
                span,
            )
            .is_none());
        assert!(make_engine()
            .prepare_match_arm_path(
                &generic_state.owners,
                &generic_state.function_aliases,
                &generic_state.raw_aliases,
                &generic_state.raw_views,
                &generic_state.storage_origins,
                &generic_state.pending_reallocs,
                &generic_state.variant_owner_effects,
                &scrutinee,
                &arms[1],
                span,
            )
            .is_some());
        let shadow_state = clone_match_state(&generic_state);

        let generic = make_engine().run_generic_match_oracle(
            generic_state,
            &output,
            &scrutinee,
            &arms,
            span,
            &[],
        );
        let shadow = make_engine().run_specialized_match_shadow_oracle(
            shadow_state,
            &output,
            &scrutinee,
            &arms,
            span,
            &[],
        );

        assert_eq!(generic, shadow);
    }

    #[test]
    fn specialized_match_shadow_excludes_never_arm_like_generic() {
        let types = TypeCtx::new();
        let summaries = Vec::new();
        let summary_index = super::super::summary::OwnerReturnSummaryIndex::new(&summaries);
        let make_engine = || ResourceOwnerCheckEngine {
            function: "oracle",
            types: &types,
            summaries: &summary_index,
            diagnostics: Vec::new(),
            deferred: super::super::report::ResourceOwnerCheckDeferred::default(),
            owner_extent_requirements: Vec::new(),
            memory_span_requirements: Vec::new(),
            params: &[],
            owner_leaf_projection_cache: Default::default(),
        };
        let output = Place::local("output".to_string(), types.unit());
        let scrutinee = Place::local("scrutinee".to_string(), types.unit());
        let value = Place::local("never_value".to_string(), types.never());
        let retained = Place::local("retained".to_string(), types.unit());
        let span = Span::empty(crate::span::FileId(0), 0);
        let arms = vec![ResourceMatchArm {
            pattern: super::super::model::ResourceMatchPattern::Wildcard,
            bind_local: None,
            bind_source_name: None,
            bind_mode: None,
            ops: Vec::new(),
            value,
            span,
        }];
        let post_ops = vec![ResourceOp::StorageOrigin {
            target: retained,
            origin: super::super::model::StorageOrigin::Owned,
            span,
        }];
        let generic_state = empty_match_state();
        let mut direct_state = clone_match_state(&generic_state);
        assert!(!make_engine().finalize_match_arm_value(
            &mut direct_state,
            &output,
            &arms[0].value,
            span,
        ));
        let shadow_state = clone_match_state(&generic_state);

        let generic = make_engine().run_generic_match_oracle(
            generic_state,
            &output,
            &scrutinee,
            &arms,
            span,
            &post_ops,
        );
        let shadow = make_engine().run_specialized_match_shadow_oracle(
            shadow_state,
            &output,
            &scrutinee,
            &arms,
            span,
            &post_ops,
        );

        assert_eq!(generic, shadow);
    }
}
