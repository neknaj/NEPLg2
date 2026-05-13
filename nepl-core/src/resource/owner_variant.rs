extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeId;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place, ResourceMatchPattern, StorageId};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_return_apply_source::{
    owner_projection_source_place_for_arg, summary_projection_place,
};
use super::owner_state::OwnerTable;
use super::owner_summary_record::{OwnerParameterConditionSource, OwnerParameterStorageSource};
use super::owner_variant_utils::{
    owner_projection_sources_for_place, owner_value_condition_truth, payload_bind_suffix,
    push_unique_owner_variant_condition, push_unique_source, push_unique_variant_consumed_source,
    push_unique_variant_projection_return, source_list_contains,
};
use super::owner_variant_value_condition::PendingVariantValueCondition;
use super::place_utils::{place_with_suffix, places_overlap};
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;
use super::summary::{
    OwnerProjectionReturnOwner, OwnerResolvedParameterVariant, OwnerReturnSummary,
    OwnerVariantCondition, OwnerVariantParameterIndex, OwnerVariantProjectionReturn,
    OwnerVariantProjectionSource,
};
use super::variant_name::{match_pattern_variant_name, normalize_variant_name};

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingVariantOwnerConsumption {
    result: Place,
    variant: String,
    arg: Place,
    suffix: Vec<super::model::PlaceProjection>,
    ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingVariantOwnerReturn {
    result: Place,
    variant: String,
    target_suffix: Vec<super::model::PlaceProjection>,
    target_ty: TypeId,
    source: PendingVariantOwnerReturnSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PendingVariantOwnerReturnSource {
    Parameter {
        arg: Place,
        source_suffix: Vec<super::model::PlaceProjection>,
        source_ty: TypeId,
    },
    Fresh,
    Maybe,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingUnreachableVariant {
    result: Place,
    variant: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingVariantPayloadValueCondition {
    result: Place,
    variant: String,
    suffix: Vec<super::model::PlaceProjection>,
    ty: TypeId,
    condition: super::model::I32ValueCondition,
}

#[derive(Debug, Clone, Default)]
pub(super) struct PendingVariantOwnerEffects {
    consumptions: Vec<PendingVariantOwnerConsumption>,
    returns: Vec<PendingVariantOwnerReturn>,
    unreachable_variants: Vec<PendingUnreachableVariant>,
    payload_conditions: Vec<PendingVariantPayloadValueCondition>,
    value_conditions: Vec<PendingVariantValueCondition>,
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

    pub(super) fn record_call(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        summary: &OwnerReturnSummary,
    ) {
        self.clear_result(output);
        self.record_unreachable_variants(raw_aliases, output, args, &summary.variant_conditions);
        for entry in &summary.variant_conditions {
            let variant = normalize_variant_name(&entry.variant);
            let Some(condition) = PendingVariantValueCondition::from_summary_condition(
                raw_aliases,
                output,
                args,
                variant,
                entry,
            ) else {
                continue;
            };
            self.push_unique_value_condition(condition);
        }
        for entry in &summary.variant_consumed_parameter_indices {
            if summary
                .consumed_parameter_indices
                .contains(&entry.parameter_index)
            {
                continue;
            }
            let Some(arg) = args.get(entry.parameter_index) else {
                continue;
            };
            self.push_unique_consumption(PendingVariantOwnerConsumption {
                result: output.clone(),
                variant: normalize_variant_name(&entry.variant),
                arg: raw_aliases.canonicalize(arg),
                suffix: Vec::new(),
                ty: arg.ty,
            });
        }
        for entry in &summary.variant_consumed_parameter_sources {
            if summary
                .consumed_parameter_indices
                .contains(&entry.source.parameter_index)
                || summary
                    .consumed_parameter_sources
                    .iter()
                    .any(|source| source == &entry.source)
            {
                continue;
            }
            let Some(arg) = args.get(entry.source.parameter_index) else {
                continue;
            };
            let source_place = owner_projection_source_place_for_arg(arg, &entry.source);
            self.push_unique_consumption(PendingVariantOwnerConsumption {
                result: output.clone(),
                variant: normalize_variant_name(&entry.variant),
                arg: raw_aliases.canonicalize(arg),
                suffix: entry.source.suffix.clone(),
                ty: source_place.ty,
            });
        }
        for entry in &summary.variant_projection_returns {
            let source = match &entry.owner {
                OwnerProjectionReturnOwner::Parameter(source) => {
                    let Some(arg) = args.get(source.parameter_index) else {
                        continue;
                    };
                    PendingVariantOwnerReturnSource::Parameter {
                        arg: raw_aliases.canonicalize(arg),
                        source_suffix: source.suffix.clone(),
                        source_ty: summary_projection_place(arg, &source.suffix, source.ty).ty,
                    }
                }
                OwnerProjectionReturnOwner::Fresh => PendingVariantOwnerReturnSource::Fresh,
                OwnerProjectionReturnOwner::Maybe => PendingVariantOwnerReturnSource::Maybe,
            };
            self.push_unique_return(PendingVariantOwnerReturn {
                result: output.clone(),
                variant: normalize_variant_name(&entry.variant),
                target_suffix: entry.suffix.clone(),
                target_ty: summary_projection_place(output, &entry.suffix, entry.ty).ty,
                source,
            });
        }
        for entry in &summary.variant_payload_conditions {
            self.push_unique_payload_condition(PendingVariantPayloadValueCondition {
                result: output.clone(),
                variant: normalize_variant_name(&entry.variant),
                suffix: entry.suffix.clone(),
                ty: entry.ty,
                condition: entry.condition,
            });
        }
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
            engine.move_owner_out(
                owners,
                raw_aliases,
                storage_origins,
                &place,
                ResourceOwnerOperation::CallArgument,
                span,
            );
            raw_views.clear(&place);
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
            engine.move_owner_out(
                owners,
                raw_aliases,
                storage_origins,
                &source,
                ResourceOwnerOperation::CallArgument,
                span,
            );
            raw_views.clear(&source);
            push_unique_source(&mut handled_sources, source, Vec::new(), ty);
        }
        self.resolve_result(result);
    }

    pub(super) fn collect_result_owner_effect_summaries(
        &self,
        engine: &ResourceOwnerCheckEngine<'_>,
        owners: &OwnerTable,
        raw_aliases: &RawCellAddressAliases,
        raw_views: &RawAddressViewTable,
        result: &Place,
        parameter_storage_sources: &[OwnerParameterStorageSource],
        index_out: &mut Vec<OwnerVariantParameterIndex>,
        source_out: &mut Vec<OwnerVariantProjectionSource>,
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
        }
        for entry in self.returns.iter().filter(|entry| entry.result == *result) {
            match &entry.source {
                PendingVariantOwnerReturnSource::Parameter { .. } => {
                    let Some(source) = pending_return_source(entry, raw_aliases) else {
                        continue;
                    };
                    if engine.place_is_non_owning_raw_address_view(
                        owners,
                        raw_aliases,
                        raw_views,
                        &source,
                    ) {
                        continue;
                    }
                    for parameter_source in owner_projection_sources_for_place(
                        owners,
                        raw_aliases,
                        &source,
                        parameter_storage_sources,
                    ) {
                        push_unique_variant_projection_return(
                            return_out,
                            OwnerVariantProjectionReturn {
                                variant: normalize_variant_name(&entry.variant),
                                suffix: entry.target_suffix.clone(),
                                ty: entry.target_ty,
                                owner: OwnerProjectionReturnOwner::Parameter(parameter_source),
                            },
                        );
                    }
                }
                PendingVariantOwnerReturnSource::Fresh => {
                    push_unique_variant_projection_return(
                        return_out,
                        OwnerVariantProjectionReturn {
                            variant: normalize_variant_name(&entry.variant),
                            suffix: entry.target_suffix.clone(),
                            ty: entry.target_ty,
                            owner: OwnerProjectionReturnOwner::Fresh,
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
            engine.move_owner_out(
                owners,
                raw_aliases,
                storage_origins,
                &source,
                ResourceOwnerOperation::CallArgument,
                span,
            );
            raw_views.clear(&source);
            push_unique_source(&mut handled_sources, source, Vec::new(), ty);
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

    pub(super) fn copy_result(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let copies = self
            .consumptions
            .iter()
            .filter(|entry| entry.result == *source)
            .map(|entry| PendingVariantOwnerConsumption {
                result: target.clone(),
                variant: entry.variant.clone(),
                arg: entry.arg.clone(),
                suffix: entry.suffix.clone(),
                ty: entry.ty,
            })
            .collect::<Vec<_>>();
        let return_copies = self
            .returns
            .iter()
            .filter(|entry| entry.result == *source)
            .map(|entry| PendingVariantOwnerReturn {
                result: target.clone(),
                variant: entry.variant.clone(),
                target_suffix: entry.target_suffix.clone(),
                target_ty: entry.target_ty,
                source: entry.source.clone(),
            })
            .collect::<Vec<_>>();
        let unreachable_copies = self
            .unreachable_variants
            .iter()
            .filter(|entry| entry.result == *source)
            .map(|entry| PendingUnreachableVariant {
                result: target.clone(),
                variant: entry.variant.clone(),
            })
            .collect::<Vec<_>>();
        let payload_condition_copies = self
            .payload_conditions
            .iter()
            .filter(|entry| entry.result == *source)
            .map(|entry| PendingVariantPayloadValueCondition {
                result: target.clone(),
                variant: entry.variant.clone(),
                suffix: entry.suffix.clone(),
                ty: entry.ty,
                condition: entry.condition,
            })
            .collect::<Vec<_>>();
        let value_condition_copies = self
            .value_conditions
            .iter()
            .filter(|entry| entry.result == *source)
            .map(|entry| entry.with_result(target.clone()))
            .collect::<Vec<_>>();
        self.clear_result(target);
        for entry in copies {
            self.push_unique_consumption(entry);
        }
        for entry in return_copies {
            self.push_unique_return(entry);
        }
        for entry in unreachable_copies {
            self.push_unique_unreachable(entry);
        }
        for entry in payload_condition_copies {
            self.push_unique_payload_condition(entry);
        }
        for entry in value_condition_copies {
            self.push_unique_value_condition(entry);
        }
    }

    pub(super) fn clear_result(&mut self, result: &Place) {
        self.consumptions.retain(|entry| entry.result != *result);
        self.returns.retain(|entry| entry.result != *result);
        self.unreachable_variants
            .retain(|entry| entry.result != *result);
        self.payload_conditions
            .retain(|entry| entry.result != *result);
        self.value_conditions
            .retain(|entry| entry.result != *result);
    }

    fn resolve_result(&mut self, result: &Place) {
        let mut resolved_sources = Vec::new();
        for entry in self
            .consumptions
            .iter()
            .filter(|entry| entry.result == *result)
        {
            let ty = summary_projection_place(&entry.arg, &entry.suffix, entry.ty).ty;
            push_unique_source(
                &mut resolved_sources,
                entry.arg.clone(),
                entry.suffix.clone(),
                ty,
            );
        }
        for entry in self.returns.iter().filter(|entry| entry.result == *result) {
            if let PendingVariantOwnerReturnSource::Parameter {
                arg,
                source_suffix,
                source_ty,
            } = &entry.source
            {
                let ty = summary_projection_place(arg, source_suffix, *source_ty).ty;
                push_unique_source(
                    &mut resolved_sources,
                    arg.clone(),
                    source_suffix.clone(),
                    ty,
                );
            }
        }
        self.consumptions.retain(|entry| {
            let ty = summary_projection_place(&entry.arg, &entry.suffix, entry.ty).ty;
            entry.result != *result
                && !source_list_contains(&resolved_sources, &entry.arg, &entry.suffix, ty)
        });
        self.returns.retain(|entry| {
            if entry.result == *result {
                return false;
            }
            let PendingVariantOwnerReturnSource::Parameter {
                arg,
                source_suffix,
                source_ty,
            } = &entry.source
            else {
                return true;
            };
            let ty = summary_projection_place(arg, source_suffix, *source_ty).ty;
            !source_list_contains(&resolved_sources, arg, source_suffix, ty)
        });
        self.unreachable_variants
            .retain(|entry| entry.result != *result);
        self.payload_conditions
            .retain(|entry| entry.result != *result);
        self.value_conditions
            .retain(|entry| entry.result != *result);
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

    pub(super) fn merge_paths(paths: &[PendingVariantOwnerEffects]) -> Self {
        let Some(first) = paths.first() else {
            return Self::default();
        };
        let mut out = Self::default();
        for entry in &first.consumptions {
            if paths
                .iter()
                .skip(1)
                .all(|path| path.consumptions.iter().any(|existing| existing == entry))
            {
                out.push_unique_consumption(entry.clone());
            }
        }
        for entry in &first.returns {
            if paths
                .iter()
                .skip(1)
                .all(|path| path.returns.iter().any(|existing| existing == entry))
            {
                out.push_unique_return(entry.clone());
            }
        }
        for entry in &first.unreachable_variants {
            if paths.iter().skip(1).all(|path| {
                path.unreachable_variants
                    .iter()
                    .any(|existing| existing == entry)
            }) {
                out.push_unique_unreachable(entry.clone());
            }
        }
        for entry in &first.payload_conditions {
            if paths.iter().skip(1).all(|path| {
                path.payload_conditions
                    .iter()
                    .any(|existing| existing == entry)
            }) {
                out.push_unique_payload_condition(entry.clone());
            }
        }
        for entry in &first.value_conditions {
            if paths.iter().skip(1).all(|path| {
                path.value_conditions
                    .iter()
                    .any(|existing| existing == entry)
            }) {
                out.push_unique_value_condition(entry.clone());
            }
        }
        out
    }

    fn record_unreachable_variants(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        conditions: &[OwnerVariantCondition],
    ) {
        let mut variants = Vec::new();
        for condition in conditions {
            if !variants.iter().any(|variant| variant == &condition.variant) {
                variants.push(condition.variant.clone());
            }
        }
        for variant in variants {
            let mut saw_condition = false;
            let mut all_conditions_false = true;
            for condition in conditions
                .iter()
                .filter(|condition| condition.variant == variant)
            {
                saw_condition = true;
                match owner_value_condition_truth(raw_aliases, args, &condition.condition) {
                    Some(false) => {}
                    Some(true) | None => {
                        all_conditions_false = false;
                        break;
                    }
                }
            }
            if saw_condition && all_conditions_false {
                self.push_unique_unreachable(PendingUnreachableVariant {
                    result: output.clone(),
                    variant: normalize_variant_name(&variant),
                });
            }
        }
    }

    fn variant_is_unreachable(&self, result: &Place, variant: &str) -> bool {
        self.unreachable_variants
            .iter()
            .any(|entry| entry.result == *result && entry.variant == variant)
    }

    fn push_unique_consumption(&mut self, entry: PendingVariantOwnerConsumption) {
        if self.consumptions.iter().any(|existing| existing == &entry) {
            return;
        }
        self.consumptions.push(entry);
    }

    fn push_unique_return(&mut self, entry: PendingVariantOwnerReturn) {
        if self.returns.iter().any(|existing| existing == &entry) {
            return;
        }
        self.returns.push(entry);
    }

    fn push_unique_unreachable(&mut self, entry: PendingUnreachableVariant) {
        if self
            .unreachable_variants
            .iter()
            .any(|existing| existing == &entry)
        {
            return;
        }
        self.unreachable_variants.push(entry);
    }

    fn push_unique_payload_condition(&mut self, entry: PendingVariantPayloadValueCondition) {
        if self
            .payload_conditions
            .iter()
            .any(|existing| existing == &entry)
        {
            return;
        }
        self.payload_conditions.push(entry);
    }

    fn push_unique_value_condition(&mut self, entry: PendingVariantValueCondition) {
        if self
            .value_conditions
            .iter()
            .any(|existing| existing == &entry)
        {
            return;
        }
        self.value_conditions.push(entry);
    }
}

fn pending_consumption_source(
    entry: &PendingVariantOwnerConsumption,
    raw_aliases: &RawCellAddressAliases,
) -> Place {
    let arg = raw_aliases.canonicalize(&entry.arg);
    summary_projection_place(&arg, &entry.suffix, entry.ty)
}

fn pending_return_source(
    entry: &PendingVariantOwnerReturn,
    raw_aliases: &RawCellAddressAliases,
) -> Option<Place> {
    let PendingVariantOwnerReturnSource::Parameter {
        arg,
        source_suffix,
        source_ty,
    } = &entry.source
    else {
        return None;
    };
    let arg = raw_aliases.canonicalize(arg);
    Some(summary_projection_place(&arg, source_suffix, *source_ty))
}

fn apply_pending_variant_owner_return(
    engine: &mut ResourceOwnerCheckEngine<'_>,
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    raw_views: &mut RawAddressViewTable,
    storage_origins: &mut StorageOriginTable,
    entry: &PendingVariantOwnerReturn,
    result: &Place,
    span: Span,
) -> Option<Place> {
    let target = summary_projection_place(result, &entry.target_suffix, entry.target_ty);
    let source = match &entry.source {
        PendingVariantOwnerReturnSource::Parameter { .. } => {
            let source = pending_return_source(entry, raw_aliases)?;
            raw_aliases.copy_scalar_facts_if_tracked(&source, &target);
            engine.transfer_owner_from_summary_effect(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &source,
                &target,
                ResourceOwnerOperation::ReturnValue,
                span,
            );
            Some(source)
        }
        PendingVariantOwnerReturnSource::Fresh => {
            owners.allocate(&target);
            raw_aliases.mark(&target);
            storage_origins.mark_owned(&target);
            None
        }
        PendingVariantOwnerReturnSource::Maybe => {
            owners.set_state(&target, OwnerState::MaybeFreed { storage: None });
            raw_aliases.mark(&target);
            storage_origins.mark_owned(&target);
            None
        }
    };
    raw_views.clear(&target);
    source
}

fn reserved_owner_state(owners: &OwnerTable, source: &Place) -> OwnerState {
    let storage = match owners.state(source) {
        Some(OwnerState::Live { storage }) => Some(storage),
        Some(OwnerState::MaybeFreed { storage } | OwnerState::Reserved { storage }) => storage,
        Some(OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed) | None => {
            first_storage_under(owners, source)
        }
    };
    OwnerState::Reserved { storage }
}

fn first_storage_under(owners: &OwnerTable, source: &Place) -> Option<StorageId> {
    owners
        .live_entries_under(source)
        .into_iter()
        .find_map(|entry| match entry.state {
            OwnerState::Live { storage } => Some(storage),
            OwnerState::MaybeFreed { storage } | OwnerState::Reserved { storage } => storage,
            OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed => None,
        })
}
