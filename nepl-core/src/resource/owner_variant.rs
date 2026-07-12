extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::{TypeCtx, TypeId};

use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceRoot, ResourceMatchPattern};
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
use super::place_utils::{place_with_suffix, places_overlap};
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

impl PendingVariantOwnerEffects {
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
        let mut applied_targets = BTreeMap::<Place, Vec<Place>>::new();
        for (index, entry) in matching_returns.iter().enumerate() {
            if should_skip_unavailable_alternative(&matching_returns, &available, index) {
                continue;
            }
            let target = summary_projection_place(scrutinee, &entry.target_suffix, entry.target_ty);
            let source = available
                .get(index)
                .and_then(|availability| availability.source.as_ref());
            if let Some(source) = source {
                if let Some(applied_target) =
                    mutually_exclusive_applied_target(&applied_targets, source, &target)
                {
                    if copy_exclusive_applied_target(
                        owners,
                        raw_aliases,
                        raw_views,
                        storage_origins,
                        applied_target,
                        &target,
                    ) {
                        applied_targets
                            .entry(source.clone())
                            .or_default()
                            .push(target);
                        continue;
                    }
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
                applied_targets.entry(source).or_default().push(target);
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
            let transferable = source
                .as_ref()
                .map(|source| engine.has_transferable_owner(owners, raw_aliases, source))
                .unwrap_or(true);
            PendingReturnAvailability {
                source,
                transferable,
            }
        })
        .collect()
}

struct PendingReturnAvailability {
    source: Option<Place>,
    transferable: bool,
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

fn mutually_exclusive_applied_target<'a>(
    applied_targets: &'a BTreeMap<Place, Vec<Place>>,
    source: &Place,
    target: &Place,
) -> Option<&'a Place> {
    applied_targets
        .get(source)?
        .iter()
        .find(|applied_target| places_are_mutually_exclusive(Some(applied_target), Some(target)))
}

fn copy_exclusive_applied_target(
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    raw_views: &mut RawAddressViewTable,
    storage_origins: &mut StorageOriginTable,
    source: &Place,
    target: &Place,
) -> bool {
    if owners.has_tracked_state_under(target) || !owners.has_transferable_owner(source) {
        return false;
    }
    let Some(state) = owners.state(source) else {
        return false;
    };
    owners.set_state(target, state);
    raw_aliases.copy_alias_if_tracked_preserving_target(source, target);
    raw_views.copy(source, target);
    storage_origins.copy_origin(source, target);
    true
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
        PendingReturnAvailability {
            source: Some(place_with_suffix(
                &Place::unknown(TypeId(2)),
                &[super::super::model::PlaceProjection::EnumPayload {
                    variant: String::from(variant),
                }],
                TypeId(1),
            )),
            transferable,
        }
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
        let applied = BTreeMap::from([(source.clone(), vec![left.clone()])]);
        assert_eq!(
            mutually_exclusive_applied_target(&applied, &source, &right),
            Some(&left)
        );
        assert_eq!(
            mutually_exclusive_applied_target(&applied, &other_source, &right),
            None
        );
        assert_eq!(
            mutually_exclusive_applied_target(&applied, &source, &left),
            None
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
    }
}
