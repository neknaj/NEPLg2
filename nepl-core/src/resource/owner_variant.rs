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
use super::owner_state::OwnerTable;
use super::place_utils::{place_with_suffix, places_overlap};
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;
use super::summary::{
    OwnerResolvedParameterVariant, OwnerReturnSummary, OwnerValueCondition, OwnerVariantCondition,
};

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
    arg: Place,
    source_suffix: Vec<super::model::PlaceProjection>,
    source_ty: TypeId,
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
            self.push_unique_consumption(PendingVariantOwnerConsumption {
                result: output.clone(),
                variant: normalize_variant_name(&entry.variant),
                arg: raw_aliases.canonicalize(arg),
                suffix: entry.source.suffix.clone(),
                ty: entry.source.ty,
            });
        }
        for entry in &summary.variant_projection_returns {
            let Some(arg) = args.get(entry.source.parameter_index) else {
                continue;
            };
            self.push_unique_return(PendingVariantOwnerReturn {
                result: output.clone(),
                variant: normalize_variant_name(&entry.variant),
                target_suffix: entry.suffix.clone(),
                target_ty: entry.ty,
                arg: raw_aliases.canonicalize(arg),
                source_suffix: entry.source.suffix.clone(),
                source_ty: entry.source.ty,
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
            let arg = raw_aliases.canonicalize(&entry.arg);
            let source = place_with_suffix(&arg, &entry.source_suffix, entry.source_ty);
            let target = place_with_suffix(scrutinee, &entry.target_suffix, entry.target_ty);
            engine.transfer_owner(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &source,
                &target,
                ResourceOwnerOperation::ReturnValue,
                span,
            );
            raw_views.clear(&target);
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
        for entry in &self.consumptions {
            if entry.result != *scrutinee || entry.variant != variant {
                continue;
            }
            let arg = raw_aliases.canonicalize(&entry.arg);
            let place = place_with_suffix(&arg, &entry.suffix, entry.ty);
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
            let source = place_with_suffix(scrutinee, &entry.suffix, entry.ty);
            raw_aliases.add_i32_condition(&source, entry.condition);
            if let Some(bind_local) = bind_local {
                let bind_suffix = payload_bind_suffix(&entry.suffix, &variant);
                let target = place_with_suffix(bind_local, bind_suffix, entry.ty);
                raw_aliases.add_i32_condition(&target, entry.condition);
            }
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
            let source = pending_return_source(entry, raw_aliases);
            let target = place_with_suffix(result, &entry.target_suffix, entry.target_ty);
            engine.transfer_owner(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &source,
                &target,
                ResourceOwnerOperation::ReturnValue,
                span,
            );
            raw_views.clear(&target);
            push_unique_source(&mut handled_sources, source, Vec::new(), entry.source_ty);
        }
        for entry in self
            .consumptions
            .iter()
            .filter(|entry| entry.result == *result)
        {
            let source = pending_consumption_source(entry, raw_aliases);
            if source_list_contains(&handled_sources, &source, &[], entry.ty) {
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
            push_unique_source(&mut handled_sources, source, Vec::new(), entry.ty);
        }
        self.resolve_result(result);
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
            let source = pending_return_source(entry, raw_aliases);
            let target = place_with_suffix(result, &entry.target_suffix, entry.target_ty);
            engine.transfer_owner(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &source,
                &target,
                ResourceOwnerOperation::ReturnValue,
                span,
            );
            raw_views.clear(&target);
            push_unique_source(&mut handled_sources, source, Vec::new(), entry.source_ty);
        }
        for entry in self
            .consumptions
            .iter()
            .filter(|entry| entry.result == *result && entry.variant == variant)
        {
            let source = pending_consumption_source(entry, raw_aliases);
            if source_list_contains(&handled_sources, &source, &[], entry.ty) {
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
            push_unique_source(&mut handled_sources, source, Vec::new(), entry.ty);
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
                arg: entry.arg.clone(),
                source_suffix: entry.source_suffix.clone(),
                source_ty: entry.source_ty,
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
    }

    pub(super) fn clear_result(&mut self, result: &Place) {
        self.consumptions.retain(|entry| entry.result != *result);
        self.returns.retain(|entry| entry.result != *result);
        self.unreachable_variants
            .retain(|entry| entry.result != *result);
        self.payload_conditions
            .retain(|entry| entry.result != *result);
    }

    fn resolve_result(&mut self, result: &Place) {
        let mut resolved_sources = Vec::new();
        for entry in self
            .consumptions
            .iter()
            .filter(|entry| entry.result == *result)
        {
            push_unique_source(
                &mut resolved_sources,
                entry.arg.clone(),
                entry.suffix.clone(),
                entry.ty,
            );
        }
        for entry in self.returns.iter().filter(|entry| entry.result == *result) {
            push_unique_source(
                &mut resolved_sources,
                entry.arg.clone(),
                entry.source_suffix.clone(),
                entry.source_ty,
            );
        }
        self.consumptions.retain(|entry| {
            entry.result != *result
                && !source_list_contains(&resolved_sources, &entry.arg, &entry.suffix, entry.ty)
        });
        self.returns.retain(|entry| {
            entry.result != *result
                && !source_list_contains(
                    &resolved_sources,
                    &entry.arg,
                    &entry.source_suffix,
                    entry.source_ty,
                )
        });
        self.unreachable_variants
            .retain(|entry| entry.result != *result);
        self.payload_conditions
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
            let source = pending_return_source(entry, raw_aliases);
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
        out
    }

    fn record_unreachable_variants(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        conditions: &[OwnerVariantCondition],
    ) {
        for condition in conditions {
            if matches!(
                owner_value_condition_truth(raw_aliases, args, &condition.condition),
                Some(false)
            ) {
                self.push_unique_unreachable(PendingUnreachableVariant {
                    result: output.clone(),
                    variant: normalize_variant_name(&condition.variant),
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
}

fn pending_consumption_source(
    entry: &PendingVariantOwnerConsumption,
    raw_aliases: &RawCellAddressAliases,
) -> Place {
    let arg = raw_aliases.canonicalize(&entry.arg);
    place_with_suffix(&arg, &entry.suffix, entry.ty)
}

fn pending_return_source(
    entry: &PendingVariantOwnerReturn,
    raw_aliases: &RawCellAddressAliases,
) -> Place {
    let arg = raw_aliases.canonicalize(&entry.arg);
    place_with_suffix(&arg, &entry.source_suffix, entry.source_ty)
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

fn push_unique_source(
    out: &mut Vec<(Place, Vec<super::model::PlaceProjection>, TypeId)>,
    arg: Place,
    suffix: Vec<super::model::PlaceProjection>,
    ty: TypeId,
) {
    if !source_list_contains(out, &arg, &suffix, ty) {
        out.push((arg, suffix, ty));
    }
}

fn source_list_contains(
    sources: &[(Place, Vec<super::model::PlaceProjection>, TypeId)],
    arg: &Place,
    suffix: &[super::model::PlaceProjection],
    ty: TypeId,
) -> bool {
    sources
        .iter()
        .any(|(existing_arg, existing_suffix, existing_ty)| {
            existing_arg == arg && existing_suffix == suffix && *existing_ty == ty
        })
}

fn owner_value_condition_truth(
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    condition: &OwnerValueCondition,
) -> Option<bool> {
    match condition {
        OwnerValueCondition::Param { source, condition } => {
            let arg = args.get(source.parameter_index)?;
            let place = place_with_suffix(arg, &source.suffix, source.ty);
            let place = raw_aliases.canonicalize(&place);
            raw_aliases.i32_condition_truth(&place, *condition)
        }
        OwnerValueCondition::Any(conditions) => {
            let mut has_unknown = false;
            for condition in conditions {
                match owner_value_condition_truth(raw_aliases, args, condition) {
                    Some(true) => return Some(true),
                    Some(false) => {}
                    None => has_unknown = true,
                }
            }
            if has_unknown {
                None
            } else {
                Some(false)
            }
        }
        OwnerValueCondition::All(conditions) => {
            let mut has_unknown = false;
            for condition in conditions {
                match owner_value_condition_truth(raw_aliases, args, condition) {
                    Some(true) => {}
                    Some(false) => return Some(false),
                    None => has_unknown = true,
                }
            }
            if has_unknown {
                None
            } else {
                Some(true)
            }
        }
    }
}

fn payload_bind_suffix<'a>(
    suffix: &'a [super::model::PlaceProjection],
    variant: &str,
) -> &'a [super::model::PlaceProjection] {
    let Some(super::model::PlaceProjection::EnumPayload {
        variant: suffix_variant,
    }) = suffix.first()
    else {
        return suffix;
    };
    if normalize_variant_name(suffix_variant) == variant {
        &suffix[1..]
    } else {
        suffix
    }
}

fn normalize_variant_name(variant: &str) -> String {
    String::from(variant.rsplit("::").next().unwrap_or(variant))
}

fn match_pattern_variant_name(pattern: &ResourceMatchPattern) -> Option<String> {
    let ResourceMatchPattern::Variant(variant) = pattern else {
        return None;
    };
    Some(normalize_variant_name(variant))
}
