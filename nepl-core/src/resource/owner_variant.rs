extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeId;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceMatchPattern};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::summarize_owner_storage_extent;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_return_apply_source::summary_projection_place;
use super::owner_state::OwnerTable;
use super::owner_summary_record::{OwnerParameterConditionSource, OwnerParameterStorageSource};
use super::owner_variant_apply::{
    apply_pending_variant_owner_return, consume_pending_variant_owner, pending_consumption_source,
    pending_return_source, push_or_merge_variant_extent_requirement, reserved_owner_state,
};
use super::owner_variant_utils::{
    owner_projection_sources_for_place, payload_bind_suffix, push_unique_owner_variant_condition,
    push_unique_source, push_unique_variant_consumed_source, push_unique_variant_projection_return,
    source_list_contains,
};
use super::owner_variant_value_condition::PendingVariantValueCondition;
use super::place_utils::{place_with_suffix, places_overlap};
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;
use super::summary::{
    OwnerExtentSummary, OwnerProjectionReturnOwner, OwnerResolvedParameterVariant,
    OwnerVariantCondition, OwnerVariantConsumedExtentRequirement, OwnerVariantParameterIndex,
    OwnerVariantProjectionReturn, OwnerVariantProjectionSource,
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
        let Some(source) = self.reserved_source_for(owners, raw_aliases, place) else {
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
        for entry in &self.returns {
            if entry.result != *scrutinee || entry.variant != variant {
                continue;
            }
            apply_pending_variant_owner_return(
                engine,
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                entry,
                scrutinee,
                span,
            );
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
            if source_list_contains(&handled_sources, &place, &[], place.ty) {
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
        for entry in self.returns.iter().filter(|entry| entry.result == *result) {
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
            .filter(|entry| entry.result == *result)
        {
            let source = pending_consumption_source(entry, raw_aliases);
            let ty = source.ty;
            if source_list_contains(&handled_sources, &source, &[], ty) {
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
        for entry in matching_returns {
            if let Some(source) = apply_pending_variant_owner_return(
                engine,
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &entry,
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
                            extent: summarize_owner_storage_extent(
                                raw_aliases,
                                parameter_condition_sources,
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
                                    extent: summarize_owner_storage_extent(
                                        raw_aliases,
                                        parameter_condition_sources,
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
                                owner: OwnerProjectionReturnOwner::Parameter {
                                    returned_extent: match &entry.source {
                                        PendingVariantOwnerReturnSource::Parameter {
                                            returned_extent,
                                            ..
                                        } => summarize_owner_storage_extent(
                                            raw_aliases,
                                            parameter_condition_sources,
                                            returned_extent,
                                        ),
                                        PendingVariantOwnerReturnSource::Fresh { .. }
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
                    push_unique_variant_projection_return(
                        return_out,
                        OwnerVariantProjectionReturn {
                            variant: normalize_variant_name(&entry.variant),
                            suffix: entry.target_suffix.clone(),
                            ty: entry.target_ty,
                            owner: OwnerProjectionReturnOwner::Fresh {
                                extent: summarize_owner_storage_extent(
                                    raw_aliases,
                                    parameter_condition_sources,
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
            if source_list_contains(&handled_sources, &source, &[], ty) {
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
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        place: &Place,
    ) -> Option<Place> {
        let resolved_place = resolve_owner_alias_place(owners, raw_aliases, place);
        for entry in &self.consumptions {
            let source = pending_consumption_source(entry, raw_aliases);
            let resolved_source = resolve_owner_alias_place(owners, raw_aliases, &source);
            if places_overlap(&resolved_place, &resolved_source) {
                return Some(resolved_source);
            }
        }
        for entry in &self.returns {
            let Some(source) = pending_return_source(entry, raw_aliases) else {
                continue;
            };
            let resolved_source = resolve_owner_alias_place(owners, raw_aliases, &source);
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
            !source_list_contains(materialized_sources, &source, &[], ty)
        });
        self.returns.retain(|entry| {
            let Some(source) = pending_return_source(entry, raw_aliases) else {
                return true;
            };
            let ty = source.ty;
            !source_list_contains(materialized_sources, &source, &[], ty)
        });
    }
}
