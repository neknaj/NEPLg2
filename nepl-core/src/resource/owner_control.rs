use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeKind;

use super::condition_fact::record_condition_fact_value_constraints;
use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place, ResourceConditionFact, ResourceMatchArm, ResourceOp};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::{match_arm_variant_payload_name, match_bind_payload_place};
use super::raw_realloc::{
    raw_realloc_condition_outcome, PendingRawReallocs, RawReallocConditionOutcome,
};
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;

impl ResourceOwnerCheckEngine<'_> {
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
            then_variant_owner_effects.materialize_result_owner_effects(
                self,
                &mut then_owners,
                &mut then_raw_aliases,
                &mut then_raw_views,
                &mut then_storage_origins,
                then_value,
                span,
            );
            if !then_variant_owner_effects.reject_reserved_source_use(
                self,
                &then_owners,
                &then_raw_aliases,
                then_value,
                ResourceOwnerOperation::BranchValue,
                span,
            ) {
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
                then_pending_reallocs.copy_result(then_value, output);
                then_variant_owner_effects.copy_result(then_value, output);
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
            else_variant_owner_effects.materialize_result_owner_effects(
                self,
                &mut else_owners,
                &mut else_raw_aliases,
                &mut else_raw_views,
                &mut else_storage_origins,
                else_value,
                span,
            );
            if !else_variant_owner_effects.reject_reserved_source_use(
                self,
                &else_owners,
                &else_raw_aliases,
                else_value,
                ResourceOwnerOperation::BranchValue,
                span,
            ) {
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
                else_pending_reallocs.copy_result(else_value, output);
                else_variant_owner_effects.copy_result(else_value, output);
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
        body_ops: &[ResourceOp],
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

        let mut body_owners = condition_owners.clone();
        let mut body_function_aliases = condition_function_aliases.clone();
        let mut body_raw_aliases = condition_raw_aliases.clone();
        let mut body_raw_views = condition_raw_views.clone();
        let mut body_storage_origins = condition_storage_origins.clone();
        let mut body_pending_reallocs = condition_pending_reallocs.clone();
        let mut body_variant_owner_effects = condition_variant_owner_effects.clone();
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
            RawCellAddressAliases::merge_paths(&[condition_raw_aliases, body_raw_aliases]);
        *owners = OwnerTable::merge_paths_with_raw_aliases(
            &[condition_owners, body_owners],
            &merged_raw_aliases,
        );
        *function_aliases =
            FunctionAliasTable::merge_paths(&[condition_function_aliases, body_function_aliases]);
        *raw_aliases = merged_raw_aliases;
        *raw_views = RawAddressViewTable::merge_paths(&[condition_raw_views, body_raw_views]);
        *storage_origins =
            StorageOriginTable::merge_paths(&[condition_storage_origins, body_storage_origins]);
        *pending_reallocs =
            PendingRawReallocs::merge_paths(&[condition_pending_reallocs, body_pending_reallocs]);
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
        let mut arm_paths = Vec::new();
        let mut function_alias_paths = Vec::new();
        let mut raw_alias_paths = Vec::new();
        let mut raw_view_paths = Vec::new();
        let mut storage_origin_paths = Vec::new();
        let mut pending_realloc_paths = Vec::new();
        let mut variant_owner_effect_paths = Vec::new();

        for arm in arms {
            if !variant_owner_effects.match_arm_reachable(scrutinee, &arm.pattern) {
                continue;
            }
            let mut arm_owners = owners.clone();
            let mut arm_function_aliases = function_aliases.clone();
            let mut arm_raw_aliases = raw_aliases.clone();
            let mut arm_raw_views = raw_views.clone();
            let mut arm_storage_origins = storage_origins.clone();
            let mut arm_pending_reallocs = pending_reallocs.clone();
            let mut arm_variant_owner_effects = variant_owner_effects.clone();
            if let Some(selected_variant) = match_arm_variant_payload_name(arm) {
                let mut inactive_payloads =
                    arm_owners.sibling_enum_payload_places(scrutinee, selected_variant);
                let resolved_scrutinee = super::owner_alias::resolve_owner_alias_place(
                    &arm_owners,
                    &arm_raw_aliases,
                    scrutinee,
                );
                if resolved_scrutinee != *scrutinee {
                    for inactive_payload in arm_owners
                        .sibling_enum_payload_places(&resolved_scrutinee, selected_variant)
                    {
                        if !inactive_payloads.contains(&inactive_payload) {
                            inactive_payloads.push(inactive_payload);
                        }
                    }
                }
                for inactive_payload in inactive_payloads {
                    arm_owners.set_state(&inactive_payload, OwnerState::NoFreeObligation);
                    arm_raw_aliases.clear(&inactive_payload);
                    arm_raw_views.clear(&inactive_payload);
                    arm_storage_origins.clear(&inactive_payload);
                }
            }
            arm_variant_owner_effects.apply_match_arm_returns(
                self,
                &mut arm_owners,
                &mut arm_raw_aliases,
                &mut arm_raw_views,
                &mut arm_storage_origins,
                scrutinee,
                &arm.pattern,
                span,
            );
            if let Some(bind_local) = &arm.bind_local {
                if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
                    if !arm_variant_owner_effects.reject_reserved_source_use(
                        self,
                        &arm_owners,
                        &arm_raw_aliases,
                        &source,
                        ResourceOwnerOperation::MatchValue,
                        span,
                    ) {
                        self.transfer_owner(
                            &mut arm_owners,
                            &mut arm_raw_aliases,
                            &arm_raw_views,
                            &mut arm_storage_origins,
                            &source,
                            bind_local,
                            ResourceOwnerOperation::MatchValue,
                            span,
                        );
                    }
                    arm_function_aliases.copy_alias(&source, bind_local);
                    arm_raw_views.copy(&source, bind_local);
                    arm_pending_reallocs.copy_result(&source, bind_local);
                    arm_variant_owner_effects.copy_result(&source, bind_local);
                    arm_variant_owner_effects.apply_match_arm_payload_conditions(
                        &mut arm_raw_aliases,
                        scrutinee,
                        &arm.pattern,
                        Some(bind_local),
                    );
                } else {
                    arm_raw_aliases.clear(bind_local);
                    arm_raw_views.clear(bind_local);
                    arm_storage_origins.clear(bind_local);
                    arm_pending_reallocs.clear_result(bind_local);
                    arm_variant_owner_effects.clear_result(bind_local);
                }
            } else {
                arm_variant_owner_effects.apply_match_arm_payload_conditions(
                    &mut arm_raw_aliases,
                    scrutinee,
                    &arm.pattern,
                    None,
                );
            }
            arm_variant_owner_effects.apply_match_arm(
                self,
                &mut arm_owners,
                &mut arm_raw_aliases,
                &mut arm_raw_views,
                &mut arm_storage_origins,
                scrutinee,
                &arm.pattern,
                span,
            );
            self.check_ops(
                &mut arm_owners,
                &mut arm_function_aliases,
                &mut arm_raw_aliases,
                &mut arm_raw_views,
                &mut arm_storage_origins,
                &mut arm_pending_reallocs,
                &mut arm_variant_owner_effects,
                &arm.ops,
            );
            if !self.place_is_never(&arm.value) {
                arm_variant_owner_effects.materialize_result_owner_effects(
                    self,
                    &mut arm_owners,
                    &mut arm_raw_aliases,
                    &mut arm_raw_views,
                    &mut arm_storage_origins,
                    &arm.value,
                    span,
                );
                if !arm_variant_owner_effects.reject_reserved_source_use(
                    self,
                    &arm_owners,
                    &arm_raw_aliases,
                    &arm.value,
                    ResourceOwnerOperation::MatchValue,
                    span,
                ) {
                    self.transfer_owner(
                        &mut arm_owners,
                        &mut arm_raw_aliases,
                        &arm_raw_views,
                        &mut arm_storage_origins,
                        &arm.value,
                        output,
                        ResourceOwnerOperation::MatchValue,
                        span,
                    );
                    arm_pending_reallocs.copy_result(&arm.value, output);
                    arm_variant_owner_effects.copy_result(&arm.value, output);
                }
                arm_paths.push(arm_owners);
                function_alias_paths.push(arm_function_aliases);
                raw_alias_paths.push(arm_raw_aliases);
                raw_view_paths.push(arm_raw_views);
                storage_origin_paths.push(arm_storage_origins);
                pending_realloc_paths.push(arm_pending_reallocs);
                variant_owner_effect_paths.push(arm_variant_owner_effects);
            }
        }
        if !arm_paths.is_empty() {
            let merged_raw_aliases = RawCellAddressAliases::merge_paths(&raw_alias_paths);
            *owners = OwnerTable::merge_paths_with_raw_aliases(&arm_paths, &merged_raw_aliases);
            *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
            *raw_aliases = merged_raw_aliases;
            *raw_views = RawAddressViewTable::merge_paths(&raw_view_paths);
            *storage_origins = StorageOriginTable::merge_paths(&storage_origin_paths);
            *pending_reallocs = PendingRawReallocs::merge_paths(&pending_realloc_paths);
            *variant_owner_effects =
                PendingVariantOwnerEffects::merge_paths(&variant_owner_effect_paths);
        }
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
                            &pending.source,
                            &pending.result,
                            ResourceOwnerOperation::ReallocInput,
                            span,
                        );
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
