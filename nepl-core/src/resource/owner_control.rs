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
    use super::*;

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
}
