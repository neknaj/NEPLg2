extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::{TypeCtx, TypeId};

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place, PlaceRoot, ResourceMatchPattern, StorageId};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::summarize_owner_storage_extent_for_owner;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_return_apply_place::summary_projection_place;
use super::owner_state::OwnerTable;
use super::owner_summary_record::{OwnerParameterConditionSource, OwnerParameterStorageSource};
use super::owner_variant_apply::{
    apply_pending_variant_owner_return, consume_pending_variant_owner, pending_consumption_source,
    pending_return_source, push_or_merge_variant_extent_requirement, reserved_owner_state,
};
use super::owner_variant_source_list::{push_unique_source, source_list_overlaps};
use super::owner_variant_utils::{
    owner_projection_sources_for_place, payload_bind_suffix, push_unique_owner_variant_condition,
    push_unique_variant_consumed_source, push_unique_variant_projection_return,
};
use super::owner_variant_value_condition::PendingVariantValueCondition;
use super::place_utils::{
    place_suffix_after_prefix, place_with_suffix, places_overlap, replace_place_prefix,
};
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;
use super::summary::{
    OwnerExtentSummary, OwnerProjectionReturnOwner, OwnerProjectionSource,
    OwnerResolvedParameterVariant, OwnerVariantCondition, OwnerVariantConsumedExtentRequirement,
    OwnerVariantParameterIndex, OwnerVariantProjectionReturn, OwnerVariantProjectionSource,
};
use super::variant_name::{match_pattern_variant_name, normalize_variant_name};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingVariantOwnerConsumption {
    pub(super) result: Place,
    pub(super) variant: String,
    pub(super) arg: Place,
    pub(super) suffix: Vec<super::model::PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) extent: Option<PendingVariantOwnerExtentRequirement>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingVariantOwnerReturn {
    pub(super) result: Place,
    pub(super) variant: String,
    pub(super) target_suffix: Vec<super::model::PlaceProjection>,
    pub(super) target_ty: TypeId,
    pub(super) source_condition: Option<OwnerProjectionSource>,
    pub(super) source: PendingVariantOwnerReturnSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum PendingVariantOwnerReturnSource {
    Parameter {
        arg: Place,
        source_suffix: Vec<super::model::PlaceProjection>,
        source_ty: TypeId,
        extent_requirement: Option<PendingVariantOwnerExtentRequirement>,
        returned_extent: super::model::OwnerStorageExtent,
    },
    Fresh {
        extent: super::model::OwnerStorageExtent,
    },
    UnknownSource {
        extent: super::model::OwnerStorageExtent,
    },
    Maybe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingVariantOwnerExtentRequirement {
    pub(super) expected: super::model::OwnerStorageExtent,
    pub(super) operation: ResourceOwnerOperation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingUnreachableVariant {
    pub(super) result: Place,
    pub(super) variant: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingVariantPayloadValueCondition {
    pub(super) result: Place,
    pub(super) variant: String,
    pub(super) suffix: Vec<super::model::PlaceProjection>,
    pub(super) ty: TypeId,
    pub(super) condition: super::model::I32ValueCondition,
}

#[derive(Debug, Clone, Default)]
pub(super) struct PendingVariantOwnerEffects {
    pub(super) consumptions: Vec<PendingVariantOwnerConsumption>,
    pub(super) returns: Vec<PendingVariantOwnerReturn>,
    pub(super) unreachable_variants: Vec<PendingUnreachableVariant>,
    pub(super) payload_conditions: Vec<PendingVariantPayloadValueCondition>,
    pub(super) value_conditions: Vec<PendingVariantValueCondition>,
}

#[derive(Default)]
pub(super) struct PendingVariantOwnerEffectProfile {
    pub(super) consumptions: usize,
    pub(super) returns: usize,
    pub(super) parameter_returns: usize,
    pub(super) fresh_returns: usize,
    pub(super) unknown_returns: usize,
    pub(super) maybe_returns: usize,
    pub(super) temporary_sources: usize,
    pub(super) unreachable_variants: usize,
    pub(super) payload_conditions: usize,
    pub(super) value_conditions: usize,
    pub(super) scrutinee_owner_entries: usize,
}

impl PendingVariantOwnerEffects {
    pub(super) fn profile_result_effects(
        &self,
        raw_aliases: &RawCellAddressAliases,
        result: &Place,
        scrutinee: &Place,
    ) -> PendingVariantOwnerEffectProfile {
        let mut profile = PendingVariantOwnerEffectProfile::default();
        for entry in self.consumptions.iter().filter(|entry| entry.result == *result) {
            profile.consumptions += 1;
            let source = pending_consumption_source(entry, raw_aliases);
            profile.temporary_sources += usize::from(matches!(source.root, PlaceRoot::Temporary(_)));
        }
        for entry in self.returns.iter().filter(|entry| entry.result == *result) {
            profile.returns += 1;
            match &entry.source {
                PendingVariantOwnerReturnSource::Parameter { .. } => profile.parameter_returns += 1,
                PendingVariantOwnerReturnSource::Fresh { .. } => profile.fresh_returns += 1,
                PendingVariantOwnerReturnSource::UnknownSource { .. } => profile.unknown_returns += 1,
                PendingVariantOwnerReturnSource::Maybe => profile.maybe_returns += 1,
            }
            if pending_return_source(entry, raw_aliases)
                .is_some_and(|source| matches!(source.root, PlaceRoot::Temporary(_)))
            {
                profile.temporary_sources += 1;
            }
        }
        profile.unreachable_variants = self.unreachable_variants.iter().filter(|entry| entry.result == *result).count();
        profile.payload_conditions = self.payload_conditions.iter().filter(|entry| entry.result == *result).count();
        profile.value_conditions = self.value_conditions.iter().filter(|entry| entry.result == *result).count();
        profile.scrutinee_owner_entries = self.consumptions.iter().filter(|entry| entry.result == *scrutinee).count()
            + self.returns.iter().filter(|entry| entry.result == *scrutinee).count();
        profile
    }

    pub(super) fn reject_reserved_source_use(
        &self,
        engine: &mut ResourceOwnerCheckEngine<'_>,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) -> bool {
        let Some(source) = self.reserved_source_for(engine.types, owners, raw_aliases, place)
        else {
            return false;
        };
        let state = reserved_owner_state(owners, &source);
        engine.push_unavailable(operation, &source, state, span);
        true
    }

    pub(super) fn apply_match_arm_returns(
        &self,
        engine: &mut ResourceOwnerCheckEngine<'_>,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        scrutinee: &Place,
        pattern: &ResourceMatchPattern,
        span: Span,
    ) {
        let Some(variant) = match_pattern_variant_name(pattern) else {
            return;
        };
        if self.variant_is_unreachable(scrutinee, &variant) {
            return;
        }
        let matching_returns = self
            .returns
            .iter()
            .filter(|entry| entry.result == *scrutinee && entry.variant == variant)
            .cloned()
            .collect::<Vec<_>>();
        let available =
            snapshot_return_availability(&matching_returns, engine, owners, raw_aliases);
        let mut applied_targets = BTreeMap::<AppliedOwnerSourceKey, Vec<Place>>::new();
        for (index, entry) in matching_returns.iter().enumerate() {
            if should_skip_unavailable_alternative(&matching_returns, &available, index) {
                continue;
            }
            let target = summary_projection_place(scrutinee, &entry.target_suffix, entry.target_ty);
            let source_identity = available
                .get(index)
                .and_then(|availability| availability.source_identity.as_ref());
            if let Some(source_identity) = source_identity {
                let applied_candidates =
                    mutually_exclusive_applied_targets(&applied_targets, source_identity, &target);
                if applied_candidates.into_iter().any(|applied_target| {
                    copy_exclusive_applied_target(
                        owners,
                        raw_aliases,
                        raw_views,
                        storage_origins,
                        &applied_target,
                        &target,
                    )
                }) {
                    if available[index].source.is_some() {
                        applied_targets
                            .entry(source_identity.clone())
                            .or_default()
                            .push(target);
                    }
                    continue;
                }
            }
            let applied_source = apply_pending_variant_owner_return(
                engine,
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                entry,
                scrutinee,
                span,
            );
            if let Some(source) = applied_source {
                let source_identity = available
                    .get(index)
                    .and_then(|availability| availability.source_identity.clone())
                    .unwrap_or(AppliedOwnerSourceKey::Place(source.clone()));
                applied_targets
                    .entry(source_identity)
                    .or_default()
                    .push(target);
            }
        }
    }

    pub(super) fn apply_match_arm(
        &mut self,
        engine: &mut ResourceOwnerCheckEngine<'_>,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        scrutinee: &Place,
        pattern: &ResourceMatchPattern,
        span: Span,
    ) {
        let Some(variant) = match_pattern_variant_name(pattern) else {
            return;
        };
        if self.variant_is_unreachable(scrutinee, &variant) {
            return;
        }
        let mut handled_sources = Vec::new();
        for entry in self
            .returns
            .iter()
            .filter(|entry| entry.result == *scrutinee && entry.variant == variant)
        {
            if let Some(source) = pending_return_source(entry, raw_aliases) {
                let ty = source.ty;
                push_unique_source(&mut handled_sources, source, Vec::new(), ty);
            }
        }
        for entry in &self.consumptions {
            if entry.result != *scrutinee || entry.variant != variant {
                continue;
            }
            let arg = raw_aliases.canonicalize(&entry.arg);
            let place = summary_projection_place(&arg, &entry.suffix, entry.ty);
            if source_list_overlaps(&handled_sources, &place, &[], place.ty) {
                continue;
            }
            if consume_pending_variant_owner(
                engine,
                owners,
                raw_aliases,
                storage_origins,
                entry,
                &place,
                span,
            ) {
                raw_views.clear(&place);
            }
        }
        self.resolve_result(scrutinee);
    }

    pub(super) fn apply_match_arm_payload_conditions(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        scrutinee: &Place,
        pattern: &ResourceMatchPattern,
        bind_local: Option<&Place>,
    ) {
        let Some(variant) = match_pattern_variant_name(pattern) else {
            return;
        };
        if self.variant_is_unreachable(scrutinee, &variant) {
            return;
        }
        for entry in &self.payload_conditions {
            if entry.result != *scrutinee || entry.variant != variant {
                continue;
            }
            let source = summary_projection_place(scrutinee, &entry.suffix, entry.ty);
            raw_aliases.add_i32_condition(&source, entry.condition);
            if let Some(bind_local) = bind_local {
                let bind_suffix = payload_bind_suffix(&entry.suffix, &variant);
                let bind_ty = if bind_suffix.is_empty() {
                    bind_local.ty
                } else {
                    entry.ty
                };
                let target = place_with_suffix(bind_local, bind_suffix, bind_ty);
                raw_aliases.add_i32_condition(&target, entry.condition);
            }
        }
    }

    pub(super) fn apply_match_arm_value_conditions(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        scrutinee: &Place,
        pattern: &ResourceMatchPattern,
    ) {
        let Some(variant) = match_pattern_variant_name(pattern) else {
            return;
        };
        if self.variant_is_unreachable(scrutinee, &variant) {
            return;
        }
        for entry in &self.value_conditions {
            entry.apply_if_selected(raw_aliases, scrutinee, &variant);
        }
    }

    pub(super) fn collect_match_arm_value_condition_summaries(
        &self,
        out: &mut Vec<OwnerVariantCondition>,
        raw_aliases: &RawCellAddressAliases,
        parameter_condition_sources: &[OwnerParameterConditionSource],
        output_variant: &str,
        scrutinee: &Place,
        pattern: &ResourceMatchPattern,
    ) {
        let Some(selected_variant) = match_pattern_variant_name(pattern) else {
            return;
        };
        if self.variant_is_unreachable(scrutinee, &selected_variant) {
            return;
        }
        let output_variant = normalize_variant_name(output_variant);
        for entry in &self.value_conditions {
            let Some(condition) = entry.selected_summary_condition(
                raw_aliases,
                parameter_condition_sources,
                scrutinee,
                &selected_variant,
                output_variant.clone(),
            ) else {
                continue;
            };
            push_unique_owner_variant_condition(out, condition);
        }
    }

    pub(super) fn materialize_result_owner_effects(
        &mut self,
        engine: &mut ResourceOwnerCheckEngine<'_>,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        result: &Place,
        span: Span,
    ) {
        let mut handled_sources = Vec::new();
        for entry in self.returns.iter().filter(|entry| {
            entry.result == *result && !self.variant_is_unreachable(result, &entry.variant)
        }) {
            if let Some(source) = pending_return_source(entry, raw_aliases) {
                let ty = source.ty;
                if source_list_overlaps(&handled_sources, &source, &[], ty) {
                    continue;
                }
            }
            if let Some(source) = apply_pending_variant_owner_return(
                engine,
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                entry,
                result,
                span,
            ) {
                let ty = source.ty;
                push_unique_source(&mut handled_sources, source, Vec::new(), ty);
            }
        }
        for entry in self.consumptions.iter().filter(|entry| {
            entry.result == *result && !self.variant_is_unreachable(result, &entry.variant)
        }) {
            let source = pending_consumption_source(entry, raw_aliases);
            let ty = source.ty;
            if source_list_overlaps(&handled_sources, &source, &[], ty) {
                continue;
            }
            if consume_pending_variant_owner(
                engine,
                owners,
                raw_aliases,
                storage_origins,
                entry,
                &source,
                span,
            ) {
                raw_views.clear(&source);
                push_unique_source(&mut handled_sources, source, Vec::new(), ty);
            }
        }
        self.resolve_result(result);
    }

    pub(super) fn has_result_effects(&self, result: &Place) -> bool {
        self.consumptions
            .iter()
            .any(|entry| entry.result == *result)
            || self.returns.iter().any(|entry| entry.result == *result)
    }

    pub(super) fn result_effects_have_temporary_sources(
        &self,
        raw_aliases: &RawCellAddressAliases,
        result: &Place,
    ) -> bool {
        for entry in self
            .consumptions
            .iter()
            .filter(|entry| entry.result == *result)
        {
            let effect_source = pending_consumption_source(entry, raw_aliases);
            if matches!(effect_source.root, PlaceRoot::Temporary(_)) {
                return true;
            }
        }
        for entry in self.returns.iter().filter(|entry| entry.result == *result) {
            let Some(effect_source) = pending_return_source(entry, raw_aliases) else {
                continue;
            };
            if matches!(effect_source.root, PlaceRoot::Temporary(_)) {
                return true;
            }
        }
        false
    }

    pub(super) fn move_result_effect_sources_out(
        &mut self,
        engine: &mut ResourceOwnerCheckEngine<'_>,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        result: &Place,
        operation: ResourceOwnerOperation,
        span: Span,
    ) {
        let mut handled_sources = Vec::new();
        let returns = self
            .returns
            .iter()
            .filter(|entry| entry.result == *result)
            .cloned()
            .collect::<Vec<_>>();
        for entry in returns {
            let Some(source) = pending_return_source(&entry, raw_aliases) else {
                continue;
            };
            let ty = source.ty;
            if source_list_overlaps(&handled_sources, &source, &[], ty) {
                continue;
            }
            if !engine.place_is_copy_owner_view(owners, raw_aliases, &source)
                && engine.has_transferable_owner(owners, raw_aliases, &source)
            {
                engine.move_owner_out(
                    owners,
                    raw_aliases,
                    storage_origins,
                    &source,
                    operation,
                    span,
                );
                raw_views.clear(&source);
            }
            push_unique_source(&mut handled_sources, source, Vec::new(), ty);
        }

        let consumptions = self
            .consumptions
            .iter()
            .filter(|entry| entry.result == *result)
            .cloned()
            .collect::<Vec<_>>();
        for entry in consumptions {
            let source = pending_consumption_source(&entry, raw_aliases);
            let ty = source.ty;
            if source_list_overlaps(&handled_sources, &source, &[], ty) {
                continue;
            }
            if consume_pending_variant_owner(
                engine,
                owners,
                raw_aliases,
                storage_origins,
                &entry,
                &source,
                span,
            ) {
                raw_views.clear(&source);
            }
            push_unique_source(&mut handled_sources, source, Vec::new(), ty);
        }
        self.resolve_result(result);
    }

    pub(super) fn materialize_return_owner_for_target(
        &mut self,
        engine: &mut ResourceOwnerCheckEngine<'_>,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        target: &Place,
        span: Span,
    ) -> bool {
        let matching_returns = self
            .returns
            .iter()
            .filter(|entry| {
                summary_projection_place(&entry.result, &entry.target_suffix, entry.target_ty)
                    == *target
            })
            .cloned()
            .collect::<Vec<_>>();
        if matching_returns.is_empty() {
            return false;
        }

        let mut materialized_sources = Vec::new();
        let mut materialized = false;
        let available =
            snapshot_return_availability(&matching_returns, engine, owners, raw_aliases);
        for (index, entry) in matching_returns.iter().enumerate() {
            if should_skip_unavailable_alternative(&matching_returns, &available, index) {
                continue;
            }
            if let Some(source) = apply_pending_variant_owner_return(
                engine,
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                entry,
                &entry.result,
                span,
            ) {
                let ty = source.ty;
                push_unique_source(&mut materialized_sources, source, Vec::new(), ty);
                materialized = true;
            }
        }
        if !materialized_sources.is_empty() {
            self.retain_unmaterialized_sources(raw_aliases, &materialized_sources);
        }
        materialized
    }

    pub(super) fn collect_result_owner_effect_summaries(
        &self,
        _engine: &ResourceOwnerCheckEngine<'_>,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        _raw_views: &RawAddressViewTable,
        result: &Place,
        parameter_storage_sources: &[OwnerParameterStorageSource],
        parameter_condition_sources: &[OwnerParameterConditionSource],
        index_out: &mut Vec<OwnerVariantParameterIndex>,
        source_out: &mut Vec<OwnerVariantProjectionSource>,
        extent_out: &mut Vec<OwnerVariantConsumedExtentRequirement>,
        return_out: &mut Vec<OwnerVariantProjectionReturn>,
    ) {
        for entry in self
            .consumptions
            .iter()
            .filter(|entry| entry.result == *result)
        {
            let source = pending_consumption_source(entry, raw_aliases);
            for parameter_source in owner_projection_sources_for_place(
                owners,
                raw_aliases,
                &source,
                parameter_storage_sources,
            ) {
                push_unique_variant_consumed_source(
                    index_out,
                    source_out,
                    normalize_variant_name(&entry.variant),
                    parameter_source,
                );
            }
            if let Some(requirement) = &entry.extent {
                for consumed in owner_projection_sources_for_place(
                    owners,
                    raw_aliases,
                    &source,
                    parameter_storage_sources,
                ) {
                    push_or_merge_variant_extent_requirement(
                        extent_out,
                        OwnerVariantConsumedExtentRequirement {
                            variant: normalize_variant_name(&entry.variant),
                            owner: consumed,
                            extent: summarize_owner_storage_extent_for_owner(
                                raw_aliases,
                                parameter_condition_sources,
                                &source,
                                &requirement.expected,
                            ),
                            operation: requirement.operation,
                        },
                    );
                }
            }
        }
        for entry in self.returns.iter().filter(|entry| entry.result == *result) {
            match &entry.source {
                PendingVariantOwnerReturnSource::Parameter { .. } => {
                    let Some(source) = pending_return_source(entry, raw_aliases) else {
                        continue;
                    };
                    for parameter_source in owner_projection_sources_for_place(
                        owners,
                        raw_aliases,
                        &source,
                        parameter_storage_sources,
                    ) {
                        let source_condition =
                            super::owner_variant_utils::source_condition_for_projection_source(
                                &parameter_source,
                            );
                        if let PendingVariantOwnerReturnSource::Parameter {
                            extent_requirement: Some(requirement),
                            ..
                        } = &entry.source
                        {
                            push_or_merge_variant_extent_requirement(
                                extent_out,
                                OwnerVariantConsumedExtentRequirement {
                                    variant: normalize_variant_name(&entry.variant),
                                    owner: parameter_source.clone(),
                                    extent: summarize_owner_storage_extent_for_owner(
                                        raw_aliases,
                                        parameter_condition_sources,
                                        &source,
                                        &requirement.expected,
                                    ),
                                    operation: requirement.operation,
                                },
                            );
                        }
                        push_unique_variant_projection_return(
                            return_out,
                            OwnerVariantProjectionReturn {
                                variant: normalize_variant_name(&entry.variant),
                                suffix: entry.target_suffix.clone(),
                                ty: entry.target_ty,
                                source_condition,
                                owner: OwnerProjectionReturnOwner::Parameter {
                                    returned_extent: match &entry.source {
                                        PendingVariantOwnerReturnSource::Parameter {
                                            returned_extent,
                                            ..
                                        } => summarize_owner_storage_extent_for_owner(
                                            raw_aliases,
                                            parameter_condition_sources,
                                            &source,
                                            returned_extent,
                                        ),
                                        PendingVariantOwnerReturnSource::Fresh { .. }
                                        | PendingVariantOwnerReturnSource::UnknownSource {
                                            ..
                                        }
                                        | PendingVariantOwnerReturnSource::Maybe => {
                                            OwnerExtentSummary::Unknown
                                        }
                                    },
                                    source: parameter_source,
                                },
                            },
                        );
                    }
                }
                PendingVariantOwnerReturnSource::Fresh { extent } => {
                    let target =
                        summary_projection_place(result, &entry.target_suffix, entry.target_ty);
                    push_unique_variant_projection_return(
                        return_out,
                        OwnerVariantProjectionReturn {
                            variant: normalize_variant_name(&entry.variant),
                            suffix: entry.target_suffix.clone(),
                            ty: entry.target_ty,
                            source_condition: entry.source_condition.clone(),
                            owner: OwnerProjectionReturnOwner::Fresh {
                                extent: summarize_owner_storage_extent_for_owner(
                                    raw_aliases,
                                    parameter_condition_sources,
                                    &target,
                                    extent,
                                ),
                            },
                        },
                    );
                }
                PendingVariantOwnerReturnSource::UnknownSource { extent } => {
                    let target =
                        summary_projection_place(result, &entry.target_suffix, entry.target_ty);
                    push_unique_variant_projection_return(
                        return_out,
                        OwnerVariantProjectionReturn {
                            variant: normalize_variant_name(&entry.variant),
                            suffix: entry.target_suffix.clone(),
                            ty: entry.target_ty,
                            source_condition: entry.source_condition.clone(),
                            owner: OwnerProjectionReturnOwner::UnknownSource {
                                extent: summarize_owner_storage_extent_for_owner(
                                    raw_aliases,
                                    parameter_condition_sources,
                                    &target,
                                    extent,
                                ),
                            },
                        },
                    );
                }
                PendingVariantOwnerReturnSource::Maybe => {
                    push_unique_variant_projection_return(
                        return_out,
                        OwnerVariantProjectionReturn {
                            variant: normalize_variant_name(&entry.variant),
                            suffix: entry.target_suffix.clone(),
                            ty: entry.target_ty,
                            source_condition: entry.source_condition.clone(),
                            owner: OwnerProjectionReturnOwner::Maybe,
                        },
                    );
                }
            }
        }
    }

    pub(super) fn apply_resolved_parameter_variants(
        &mut self,
        engine: &mut ResourceOwnerCheckEngine<'_>,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        args: &[Place],
        variants: &[OwnerResolvedParameterVariant],
        span: Span,
    ) {
        for entry in variants {
            let Some(result) = args.get(entry.parameter_index) else {
                continue;
            };
            self.apply_resolved_result_variant(
                engine,
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                result,
                &entry.variant,
                span,
            );
        }
    }

    fn apply_resolved_result_variant(
        &mut self,
        engine: &mut ResourceOwnerCheckEngine<'_>,
        owners: &mut OwnerTable,
        raw_aliases: &mut RawCellAddressAliases,
        raw_views: &mut RawAddressViewTable,
        storage_origins: &mut StorageOriginTable,
        result: &Place,
        variant: &str,
        span: Span,
    ) {
        let variant = normalize_variant_name(variant);
        if self.variant_is_unreachable(result, &variant) {
            self.resolve_result(result);
            return;
        }
        let mut handled_sources = Vec::new();
        for entry in self
            .returns
            .iter()
            .filter(|entry| entry.result == *result && entry.variant == variant)
        {
            if let Some(source) = apply_pending_variant_owner_return(
                engine,
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                entry,
                result,
                span,
            ) {
                let ty = source.ty;
                push_unique_source(&mut handled_sources, source, Vec::new(), ty);
            }
        }
        for entry in self
            .consumptions
            .iter()
            .filter(|entry| entry.result == *result && entry.variant == variant)
        {
            let source = pending_consumption_source(entry, raw_aliases);
            let ty = source.ty;
            if source_list_overlaps(&handled_sources, &source, &[], ty) {
                continue;
            }
            if consume_pending_variant_owner(
                engine,
                owners,
                raw_aliases,
                storage_origins,
                entry,
                &source,
                span,
            ) {
                raw_views.clear(&source);
                push_unique_source(&mut handled_sources, source, Vec::new(), ty);
            }
        }
        self.resolve_result(result);
    }

    pub(super) fn match_arm_reachable(
        &self,
        scrutinee: &Place,
        pattern: &ResourceMatchPattern,
    ) -> bool {
        let Some(variant) = match_pattern_variant_name(pattern) else {
            return true;
        };
        !self.variant_is_unreachable(scrutinee, &variant)
    }

    fn reserved_source_for(
        &self,
        types: &TypeCtx,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
    ) -> Option<Place> {
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        for entry in &self.consumptions {
            let source = pending_consumption_source(entry, raw_aliases);
            let resolved_source = resolve_owner_alias_place(owners, raw_aliases, &source);
            if types.is_copy(resolved_source.ty) {
                continue;
            }
            if places_overlap(&resolved_place, &resolved_source) {
                return Some(resolved_source);
            }
        }
        for entry in &self.returns {
            let Some(source) = pending_return_source(entry, raw_aliases) else {
                continue;
            };
            let resolved_source = resolve_owner_alias_place(owners, raw_aliases, &source);
            if types.is_copy(resolved_source.ty) {
                continue;
            }
            if places_overlap(&resolved_place, &resolved_source) {
                return Some(resolved_source);
            }
        }
        None
    }

    fn retain_unmaterialized_sources(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        materialized_sources: &[(Place, Vec<super::model::PlaceProjection>, TypeId)],
    ) {
        self.consumptions.retain(|entry| {
            let source = pending_consumption_source(entry, raw_aliases);
            let ty = source.ty;
            !source_list_overlaps(materialized_sources, &source, &[], ty)
        });
        self.returns.retain(|entry| {
            let Some(source) = pending_return_source(entry, raw_aliases) else {
                return true;
            };
            let ty = source.ty;
            !source_list_overlaps(materialized_sources, &source, &[], ty)
        });
    }
}

fn snapshot_return_availability(
    returns: &[PendingVariantOwnerReturn],
    engine: &ResourceOwnerCheckEngine<'_>,
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
) -> Vec<PendingReturnAvailability> {
    returns
        .iter()
        .map(|entry| {
            let source = pending_return_source(entry, raw_aliases);
            let source_identity = source
                .as_ref()
                .map(|source| applied_owner_source_key(owners, raw_aliases, source));
            let transferable = source
                .as_ref()
                .map(|source| engine.has_transferable_owner(owners, raw_aliases, source))
                .unwrap_or(true);
            PendingReturnAvailability {
                source,
                source_identity,
                transferable,
            }
        })
        .collect()
}

struct PendingReturnAvailability {
    source: Option<Place>,
    source_identity: Option<AppliedOwnerSourceKey>,
    transferable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum AppliedOwnerSourceKey {
    OwnerSignature(Vec<(Vec<super::model::PlaceProjection>, OwnerSourceStateKey)>),
    Place(Place),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum OwnerSourceStateKey {
    Live(StorageId),
    MaybeFreed(Option<StorageId>),
    Reserved(Option<StorageId>),
    Moved,
    Freed,
}

fn applied_owner_source_key(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    source: &Place,
) -> AppliedOwnerSourceKey {
    let resolved = resolve_owner_alias_place(owners, raw_aliases, source);
    if let Some(signature) = owner_storage_signature(owners, &resolved) {
        return AppliedOwnerSourceKey::OwnerSignature(signature);
    }
    AppliedOwnerSourceKey::Place(resolved)
}

fn owner_storage_signature(
    owners: &OwnerTable,
    source: &Place,
) -> Option<Vec<(Vec<super::model::PlaceProjection>, OwnerSourceStateKey)>> {
    let mut signature = Vec::new();
    let mut has_transferable = false;
    if let Some(state) = owners.state(source) {
        if let Some(state) = owner_source_state_key(state) {
            has_transferable |= matches!(
                state,
                OwnerSourceStateKey::Live(_) | OwnerSourceStateKey::MaybeFreed(Some(_))
            );
            signature.push((Vec::new(), state));
        }
    }
    for entry in owners.descendant_entries(source) {
        let Some(state) = owner_source_state_key(entry.state) else {
            continue;
        };
        has_transferable |= matches!(
            state,
            OwnerSourceStateKey::Live(_) | OwnerSourceStateKey::MaybeFreed(Some(_))
        );
        let Some(suffix) = place_suffix_after_prefix(&entry.place, source) else {
            continue;
        };
        signature.push((suffix, state));
    }
    signature.sort();
    has_transferable.then_some(signature)
}

fn owner_source_state_key(state: OwnerState) -> Option<OwnerSourceStateKey> {
    match state {
        OwnerState::Live { storage, .. } => Some(OwnerSourceStateKey::Live(storage)),
        OwnerState::MaybeFreed { storage } => Some(OwnerSourceStateKey::MaybeFreed(storage)),
        OwnerState::Reserved { storage } => Some(OwnerSourceStateKey::Reserved(storage)),
        OwnerState::Moved => Some(OwnerSourceStateKey::Moved),
        OwnerState::Freed => Some(OwnerSourceStateKey::Freed),
        OwnerState::NoFreeObligation => None,
    }
}

fn should_skip_unavailable_alternative(
    returns: &[PendingVariantOwnerReturn],
    available: &[PendingReturnAvailability],
    index: usize,
) -> bool {
    let Some(entry_availability) = available.get(index) else {
        return false;
    };
    if entry_availability.transferable {
        return false;
    }
    let Some(entry) = returns.get(index) else {
        return false;
    };
    returns
        .iter()
        .enumerate()
        .any(|(candidate_index, candidate)| {
            available
                .get(candidate_index)
                .is_some_and(|candidate| candidate.transferable)
                && candidate.result == entry.result
                && candidate.variant == entry.variant
                && candidate.target_suffix == entry.target_suffix
                && candidate.target_ty == entry.target_ty
                && places_are_mutually_exclusive(
                    entry_availability.source.as_ref(),
                    available
                        .get(candidate_index)
                        .and_then(|candidate| candidate.source.as_ref()),
                )
        })
}

fn places_are_mutually_exclusive(left: Option<&Place>, right: Option<&Place>) -> bool {
    let (Some(left), Some(right)) = (left, right) else {
        return false;
    };
    if left.root != right.root {
        return false;
    }
    for (left, right) in left.projections.iter().zip(&right.projections) {
        if left == right {
            continue;
        }
        return match (left, right) {
            (
                super::model::PlaceProjection::EnumPayload { variant: left },
                super::model::PlaceProjection::EnumPayload { variant: right },
            ) => normalize_variant_name(left) != normalize_variant_name(right),
            _ => false,
        };
    }
    false
}

fn mutually_exclusive_applied_targets(
    applied_targets: &BTreeMap<AppliedOwnerSourceKey, Vec<Place>>,
    source_identity: &AppliedOwnerSourceKey,
    target: &Place,
) -> Vec<Place> {
    applied_targets
        .get(source_identity)
        .into_iter()
        .flatten()
        .filter_map(|applied_target| {
            if applied_target.ty == target.ty
                && places_are_mutually_exclusive(Some(applied_target), Some(target))
            {
                return Some(applied_target.clone());
            }
            None
        })
        .collect()
}

fn copy_exclusive_applied_target(
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    raw_views: &mut RawAddressViewTable,
    storage_origins: &mut StorageOriginTable,
    source: &Place,
    target: &Place,
) -> bool {
    if source.ty != target.ty {
        return false;
    }
    if target_has_nonreplaceable_state(owners, target) || !owners.has_transferable_owner(source) {
        return false;
    }
    let entries = transferable_entries_under(owners, source);
    if entries.is_empty() {
        return false;
    }
    if owners.state(source).is_some_and(|state| {
        matches!(
            state,
            OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed
        )
    }) {
        return false;
    }
    if owners.descendant_entries(source).iter().any(|unavailable| {
        matches!(
            unavailable.state,
            OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed
        ) && entries.iter().any(|entry| {
            !places_are_mutually_exclusive(Some(&unavailable.place), Some(&entry.place))
        })
    }) {
        return false;
    }
    for entry in entries {
        let Some(copied) = replace_place_prefix(&entry.place, source, target) else {
            return false;
        };
        owners.set_state(&copied, entry.state);
        raw_aliases.copy_exclusive_variant_facts(&entry.place, &copied);
        raw_views.copy(&entry.place, &copied);
        storage_origins.copy_origin(&entry.place, &copied);
    }
    true
}

fn transferable_entries_under(
    owners: &OwnerTable,
    source: &Place,
) -> Vec<super::model::OwnerStateEntry> {
    owners
        .live_entries_under(source)
        .into_iter()
        .filter(|entry| {
            matches!(
                entry.state,
                OwnerState::Live { .. } | OwnerState::MaybeFreed { .. }
            )
        })
        .collect()
}

fn target_has_nonreplaceable_state(owners: &OwnerTable, target: &Place) -> bool {
    owners
        .state(target)
        .into_iter()
        .chain(
            owners
                .descendant_entries(target)
                .into_iter()
                .map(|entry| entry.state),
        )
        .any(|state| !matches!(state, OwnerState::NoFreeObligation))
}

#[cfg(test)]
mod alternative_tests {
    use super::*;
    use alloc::string::String;
    use alloc::vec;

    fn condition(variant: &str) -> OwnerProjectionSource {
        OwnerProjectionSource {
            parameter_index: 0,
            suffix: vec![super::super::model::PlaceProjection::EnumPayload {
                variant: String::from(variant),
            }],
            ty: TypeId(1),
        }
    }

    fn pending(variant: &str) -> PendingVariantOwnerReturn {
        PendingVariantOwnerReturn {
            result: Place::unknown(TypeId(2)),
            variant: String::from("Ok"),
            target_suffix: vec![super::super::model::PlaceProjection::EnumPayload {
                variant: String::from("Ok"),
            }],
            target_ty: TypeId(1),
            source_condition: Some(condition(variant)),
            source: PendingVariantOwnerReturnSource::Maybe,
        }
    }

    fn availability(variant: &str, transferable: bool) -> PendingReturnAvailability {
        let source = place_with_suffix(
            &Place::unknown(TypeId(2)),
            &[super::super::model::PlaceProjection::EnumPayload {
                variant: String::from(variant),
            }],
            TypeId(1),
        );
        PendingReturnAvailability {
            source: Some(source.clone()),
            source_identity: Some(AppliedOwnerSourceKey::Place(source)),
            transferable,
        }
    }

    #[test]
    fn exclusive_return_identity_collapses_raw_alias_forms() {
        let canonical = Place::local(String::from("canonical"), TypeId(1));
        let left_alias = Place::local(String::from("left_alias"), TypeId(1));
        let right_alias = Place::local(String::from("right_alias"), TypeId(1));
        let mut owners = OwnerTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        owners.allocate(&canonical);
        raw_aliases.mark(&canonical);
        raw_aliases.copy_alias_if_tracked(&canonical, &left_alias);
        raw_aliases.copy_alias_if_tracked(&canonical, &right_alias);

        let left_key = applied_owner_source_key(&owners, &raw_aliases, &left_alias);
        let right_key = applied_owner_source_key(&owners, &raw_aliases, &right_alias);
        assert_eq!(
            left_key,
            AppliedOwnerSourceKey::OwnerSignature(vec![(
                Vec::new(),
                OwnerSourceStateKey::Live(StorageId(0)),
            )])
        );
        assert_eq!(left_key, right_key);

        owners.set_state(&left_alias, OwnerState::NoFreeObligation);
        let left_key = applied_owner_source_key(&owners, &raw_aliases, &left_alias);
        let right_key = applied_owner_source_key(&owners, &raw_aliases, &right_alias);
        assert_eq!(
            left_key,
            AppliedOwnerSourceKey::OwnerSignature(vec![(
                Vec::new(),
                OwnerSourceStateKey::Live(StorageId(0)),
            )])
        );
        assert_eq!(left_key, right_key);

        owners.set_state(&left_alias, OwnerState::Moved);
        assert_eq!(
            applied_owner_source_key(&owners, &raw_aliases, &left_alias),
            AppliedOwnerSourceKey::Place(left_alias.clone())
        );
        owners.set_state(&left_alias, OwnerState::NoFreeObligation);

        let left_aggregate = Place::local(String::from("left_aggregate"), TypeId(2));
        let right_aggregate = Place::local(String::from("right_aggregate"), TypeId(2));
        let left_leaf = place_with_suffix(
            &left_aggregate,
            &[super::super::model::PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            }],
            TypeId(1),
        );
        let right_leaf = replace_place_prefix(&left_leaf, &left_aggregate, &right_aggregate)
            .expect("aggregate leaf prefix");
        owners.set_state(
            &left_leaf,
            OwnerState::Live {
                storage: StorageId(7),
                extent: super::super::model::OwnerStorageExtent::Unknown,
            },
        );
        owners.set_state(
            &right_leaf,
            OwnerState::Live {
                storage: StorageId(7),
                extent: super::super::model::OwnerStorageExtent::Unknown,
            },
        );
        assert_eq!(
            applied_owner_source_key(&owners, &raw_aliases, &left_aggregate),
            applied_owner_source_key(&owners, &raw_aliases, &right_aggregate)
        );

        let swapped_left = Place::local(String::from("swapped_left"), TypeId(2));
        let swapped_right = Place::local(String::from("swapped_right"), TypeId(2));
        let field = |base: &Place, index: usize| {
            place_with_suffix(
                base,
                &[super::super::model::PlaceProjection::Field {
                    index,
                    offset_bytes: index * 8,
                }],
                TypeId(1),
            )
        };
        owners.set_state(
            &field(&swapped_left, 0),
            OwnerState::Live {
                storage: StorageId(8),
                extent: super::super::model::OwnerStorageExtent::Unknown,
            },
        );
        owners.set_state(
            &field(&swapped_left, 1),
            OwnerState::Live {
                storage: StorageId(9),
                extent: super::super::model::OwnerStorageExtent::Unknown,
            },
        );
        owners.set_state(
            &field(&swapped_right, 0),
            OwnerState::Live {
                storage: StorageId(9),
                extent: super::super::model::OwnerStorageExtent::Unknown,
            },
        );
        owners.set_state(
            &field(&swapped_right, 1),
            OwnerState::Live {
                storage: StorageId(8),
                extent: super::super::model::OwnerStorageExtent::Unknown,
            },
        );
        assert_ne!(
            applied_owner_source_key(&owners, &raw_aliases, &swapped_left),
            applied_owner_source_key(&owners, &raw_aliases, &swapped_right)
        );
    }

    #[test]
    fn unavailable_alternatives_do_not_suppress_owner_diagnostics() {
        let a = pending("A");
        let b = pending("B");
        let returns = vec![a.clone(), b.clone()];
        assert!(!should_skip_unavailable_alternative(
            &returns,
            &[availability("A", false), availability("B", false)],
            0
        ));
        assert!(should_skip_unavailable_alternative(
            &returns,
            &[availability("A", false), availability("B", true)],
            0
        ));
        assert!(!should_skip_unavailable_alternative(
            &returns,
            &[availability("A", true), availability("B", false)],
            0
        ));
        assert!(should_skip_unavailable_alternative(
            &returns,
            &[availability("A", true), availability("B", false)],
            1
        ));
    }

    #[test]
    fn applied_target_requires_same_source_and_enum_exclusion() {
        let source = Place::local(String::from("source"), TypeId(1));
        let other_source = Place::unknown(TypeId(3));
        let root = Place::unknown(TypeId(2));
        let left = place_with_suffix(
            &root,
            &[super::super::model::PlaceProjection::EnumPayload {
                variant: String::from("Left"),
            }],
            TypeId(1),
        );
        let right = place_with_suffix(
            &root,
            &[super::super::model::PlaceProjection::EnumPayload {
                variant: String::from("Right"),
            }],
            TypeId(1),
        );
        let source_key = AppliedOwnerSourceKey::Place(source.clone());
        let other_source_key = AppliedOwnerSourceKey::Place(other_source.clone());
        let applied = BTreeMap::from([(source_key.clone(), vec![left.clone()])]);
        assert_eq!(
            mutually_exclusive_applied_targets(&applied, &source_key, &right,),
            vec![left.clone()]
        );
        assert_eq!(
            mutually_exclusive_applied_targets(&applied, &other_source_key, &right,),
            Vec::new()
        );
        let mut wrong_type_right = right.clone();
        wrong_type_right.ty = TypeId(4);
        assert_eq!(
            mutually_exclusive_applied_targets(&applied, &source_key, &wrong_type_right,),
            Vec::new()
        );
        assert_eq!(
            mutually_exclusive_applied_targets(&applied, &source_key, &left,),
            Vec::new()
        );
        let struct_root = Place::unknown(TypeId(5));
        let first_field = place_with_suffix(
            &struct_root,
            &[super::super::model::PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            }],
            TypeId(1),
        );
        let second_field = place_with_suffix(
            &struct_root,
            &[super::super::model::PlaceProjection::Field {
                index: 1,
                offset_bytes: 8,
            }],
            TypeId(1),
        );
        let struct_applied = BTreeMap::from([(source_key.clone(), vec![first_field])]);
        assert_eq!(
            mutually_exclusive_applied_targets(&struct_applied, &source_key, &second_field),
            Vec::new()
        );
    }

    #[test]
    fn exclusive_applied_targets_keep_sibling_owner_aliases_disjoint() {
        let mut owners = OwnerTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut raw_views = RawAddressViewTable::default();
        let mut storage_origins = StorageOriginTable::default();
        let source = Place::local(String::from("source"), TypeId(1));
        let root = Place::local(String::from("result"), TypeId(2));
        let left = place_with_suffix(
            &root,
            &[super::super::model::PlaceProjection::EnumPayload {
                variant: String::from("Left"),
            }],
            TypeId(1),
        );
        let right = place_with_suffix(
            &root,
            &[super::super::model::PlaceProjection::EnumPayload {
                variant: String::from("Right"),
            }],
            TypeId(1),
        );
        owners.allocate(&source);
        raw_aliases.mark(&source);
        owners.set_state(&left, super::super::model::OwnerState::NoFreeObligation);
        owners.set_state(&right, super::super::model::OwnerState::NoFreeObligation);

        assert!(copy_exclusive_applied_target(
            &mut owners,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &source,
            &left,
        ));
        assert!(copy_exclusive_applied_target(
            &mut owners,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &left,
            &right,
        ));

        assert_eq!(owners.state(&left), owners.state(&right));
        assert!(!raw_aliases.aliases_for(&left).contains(&right));
        assert!(!raw_aliases.aliases_for(&right).contains(&left));
        assert!(raw_aliases.contains_marked_alias(&left));
        assert!(raw_aliases.contains_marked_alias(&right));
    }

    #[test]
    fn exclusive_applied_targets_copy_aggregate_owner_subtrees() {
        let mut owners = OwnerTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut raw_views = RawAddressViewTable::default();
        let mut storage_origins = StorageOriginTable::default();
        let source = Place::local(String::from("source"), TypeId(1));
        let result = Place::local(String::from("result"), TypeId(2));
        let left = place_with_suffix(
            &result,
            &[super::super::model::PlaceProjection::EnumPayload {
                variant: String::from("Left"),
            }],
            TypeId(1),
        );
        let right = place_with_suffix(
            &result,
            &[super::super::model::PlaceProjection::EnumPayload {
                variant: String::from("Right"),
            }],
            TypeId(1),
        );
        let field = |base: &Place, index: usize| {
            place_with_suffix(
                base,
                &[super::super::model::PlaceProjection::Field {
                    index,
                    offset_bytes: index * 8,
                }],
                TypeId(3),
            )
        };
        let source_first = field(&source, 0);
        let source_second = field(&source, 1);
        let left_first = field(&left, 0);
        let left_second = field(&left, 1);
        let right_first = field(&right, 0);
        let right_second = field(&right, 1);
        owners.allocate(&source_first);
        owners.allocate(&source_second);
        raw_aliases.mark(&source_first);
        raw_aliases.mark(&source_second);
        owners.set_state(
            &left_first,
            super::super::model::OwnerState::NoFreeObligation,
        );
        owners.set_state(
            &left_second,
            super::super::model::OwnerState::NoFreeObligation,
        );
        owners.set_state(
            &right_first,
            super::super::model::OwnerState::NoFreeObligation,
        );
        owners.set_state(
            &right_second,
            super::super::model::OwnerState::NoFreeObligation,
        );

        assert!(copy_exclusive_applied_target(
            &mut owners,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &source,
            &left,
        ));
        assert!(copy_exclusive_applied_target(
            &mut owners,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &left,
            &right,
        ));

        assert_eq!(owners.state(&left_first), owners.state(&right_first));
        assert_eq!(owners.state(&left_second), owners.state(&right_second));
        assert!(!raw_aliases.aliases_for(&left_first).contains(&right_first));
        assert!(!raw_aliases
            .aliases_for(&left_second)
            .contains(&right_second));

        let blocked = Place::local(String::from("blocked"), TypeId(2));
        let blocked_first = field(&blocked, 0);
        let blocked_second = field(&blocked, 1);
        owners.set_state(&source_first, OwnerState::Moved);
        owners.set_state(&blocked_first, OwnerState::NoFreeObligation);
        owners.set_state(&blocked_second, OwnerState::NoFreeObligation);
        assert!(!copy_exclusive_applied_target(
            &mut owners,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &source,
            &blocked,
        ));
        assert_eq!(
            owners.state(&blocked_first),
            Some(OwnerState::NoFreeObligation)
        );
        assert_eq!(
            owners.state(&blocked_second),
            Some(OwnerState::NoFreeObligation)
        );
    }

    #[test]
    fn exclusive_applied_target_allows_unavailable_enum_siblings() {
        let mut owners = OwnerTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut raw_views = RawAddressViewTable::default();
        let mut storage_origins = StorageOriginTable::default();
        let source = Place::local(String::from("source"), TypeId(2));
        let target = Place::local(String::from("target"), TypeId(2));
        let payload = |base: &Place, variant: &str| {
            place_with_suffix(
                base,
                &[
                    super::super::model::PlaceProjection::EnumPayload {
                        variant: String::from(variant),
                    },
                    super::super::model::PlaceProjection::Field {
                        index: 0,
                        offset_bytes: 0,
                    },
                ],
                TypeId(1),
            )
        };
        let source_ready = payload(&source, "Ready");
        let source_empty = payload(&source, "Empty");
        let target_ready = payload(&target, "Ready");
        let target_empty = payload(&target, "Empty");
        owners.allocate(&source_ready);
        owners.set_state(&source_empty, OwnerState::Moved);
        owners.set_state(&target_ready, OwnerState::NoFreeObligation);
        owners.set_state(&target_empty, OwnerState::NoFreeObligation);

        assert!(copy_exclusive_applied_target(
            &mut owners,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &source,
            &target,
        ));
        assert!(matches!(
            owners.state(&target_ready),
            Some(OwnerState::Live { .. })
        ));
        assert_eq!(
            owners.state(&target_empty),
            Some(OwnerState::NoFreeObligation)
        );
    }

    #[test]
    fn exclusive_target_copy_preserves_storage_and_existing_target() {
        let source = Place::local(String::from("source"), TypeId(1));
        let target = Place::local(String::from("target"), TypeId(1));
        let occupied = Place::local(String::from("occupied"), TypeId(1));
        let mut owners = OwnerTable::default();
        let mut raw_aliases = RawCellAddressAliases::default();
        let mut raw_views = RawAddressViewTable::default();
        let mut storage_origins = StorageOriginTable::default();
        owners.allocate(&source);
        raw_aliases.mark(&source);
        raw_views.mark(&source);
        storage_origins.mark_owned(&source);
        let source_state = owners.state(&source);
        assert!(copy_exclusive_applied_target(
            &mut owners,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &source,
            &target,
        ));
        assert_eq!(owners.state(&target), source_state);
        assert!(raw_views.contains(&target));
        assert!(storage_origins.expects_owned(&target));

        owners.allocate(&occupied);
        let occupied_state = owners.state(&occupied);
        assert!(!copy_exclusive_applied_target(
            &mut owners,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &source,
            &occupied,
        ));
        assert_eq!(owners.state(&occupied), occupied_state);

        owners.set_state(&occupied, super::super::model::OwnerState::Moved);
        assert!(!copy_exclusive_applied_target(
            &mut owners,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &source,
            &occupied,
        ));
        assert_eq!(
            owners.state(&occupied),
            Some(super::super::model::OwnerState::Moved)
        );
        owners.set_state(&occupied, super::super::model::OwnerState::Freed);
        assert!(!copy_exclusive_applied_target(
            &mut owners,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &source,
            &occupied,
        ));
        assert_eq!(
            owners.state(&occupied),
            Some(super::super::model::OwnerState::Freed)
        );
    }
}
