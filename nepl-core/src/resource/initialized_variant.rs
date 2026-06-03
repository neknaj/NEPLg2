extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use super::cell_state::{raw_cell_address_prefix, CellTable};
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_raw_memory_access::raw_memory_load_reads_zero_initialized_runtime_cell;
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary_condition::RawCellValueCondition;
use super::initialized_summary_variant_model::{
    RawCellInitializationVariantCondition, RawCellInitializationVariantValueCondition,
};
use super::initialized_variant_count::{
    pending_variant_count_place, pending_variant_count_source, PendingVariantRawByteRangeCount,
};
use super::model::{I32ValueCondition, Place, ResourceMatchPattern};
use super::owner_extent_summary::instantiate_summary_type;
use super::place_utils::{
    place_suffix_after_prefix, projected_place_with_concrete_type, replace_embedded_place_prefixes,
    replace_place_prefix,
};
use super::report::ResourceCheckOperation;
use super::summary_projection::{
    instantiate_summary_suffix_on_base_with_types, instantiate_summary_suffix_with_types,
};
use super::variant_name::{match_pattern_variant_name, normalize_variant_name};
use crate::types::TypeCtx;

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingVariantRawCellInitialization {
    result: Place,
    variant: String,
    arg: Place,
    suffix: Vec<super::model::PlaceProjection>,
    ty: crate::types::TypeId,
    holds_raw_address: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingVariantRawCellRequirement {
    result: Place,
    variant: String,
    arg: Place,
    suffix: Vec<super::model::PlaceProjection>,
    ty: crate::types::TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingVariantRawByteRangeInitialization {
    result: Place,
    variant: String,
    address_arg: Place,
    address_suffix: Vec<super::model::PlaceProjection>,
    address_ty: crate::types::TypeId,
    count: PendingVariantRawByteRangeCount,
    unit: InitializedRawRangeUnit,
    ty: crate::types::TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingUnreachableVariant {
    result: Place,
    variant: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingConcreteVariant {
    result: Place,
    variant: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingVariantValueCondition {
    result: Place,
    variant: String,
    place: Place,
    condition: I32ValueCondition,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct PendingVariantRawCellInitializations {
    entries: Vec<PendingVariantRawCellInitialization>,
    byte_ranges: Vec<PendingVariantRawByteRangeInitialization>,
    requirements: Vec<PendingVariantRawCellRequirement>,
    unreachable_variants: Vec<PendingUnreachableVariant>,
    concrete_variants: Vec<PendingConcreteVariant>,
    value_conditions: Vec<PendingVariantValueCondition>,
}

impl PendingVariantRawCellInitializations {
    pub(super) fn record_call(
        &mut self,
        types: &TypeCtx,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        type_args: &[crate::types::TypeId],
        summary: &RawCellInitializationFunctionSummary,
    ) {
        self.clear_result(output);
        self.record_unreachable_variants(
            types,
            raw_aliases,
            output,
            args,
            &summary.type_params,
            type_args,
            &summary.variant_conditions,
        );
        self.record_value_conditions(
            types,
            raw_aliases,
            output,
            args,
            &summary.type_params,
            type_args,
            &summary.variant_conditions,
        );
        for cell in &summary.variant_param_cells {
            let Some(arg) = args.get(cell.param_index) else {
                continue;
            };
            let arg = raw_aliases.canonicalize(arg);
            let cell_ty = instantiate_summary_type(&summary.type_params, type_args, cell.ty);
            let Some(suffix) =
                instantiate_summary_suffix_with_types(types, args, arg.ty, &cell.suffix, cell_ty)
            else {
                continue;
            };
            self.push_unique_entry(PendingVariantRawCellInitialization {
                result: output.clone(),
                variant: normalize_variant_name(&cell.variant),
                arg,
                suffix,
                ty: cell_ty,
                holds_raw_address: cell.holds_raw_address,
            });
        }
        for range in &summary.variant_param_byte_ranges {
            let Some(address_arg) = args.get(range.address_param_index) else {
                continue;
            };
            let address_arg = raw_aliases.canonicalize(address_arg);
            let address_ty =
                instantiate_summary_type(&summary.type_params, type_args, range.address_ty);
            let Some(address_suffix) = instantiate_summary_suffix_with_types(
                types,
                args,
                address_arg.ty,
                &range.address_suffix,
                address_ty,
            ) else {
                continue;
            };
            let Some(count) = pending_variant_count_source(
                types,
                raw_aliases,
                args,
                &summary.type_params,
                type_args,
                &range.count,
            ) else {
                continue;
            };
            self.push_unique_byte_range(PendingVariantRawByteRangeInitialization {
                result: output.clone(),
                variant: normalize_variant_name(&range.variant),
                address_arg,
                address_suffix,
                address_ty,
                count,
                unit: range.unit,
                ty: instantiate_summary_type(&summary.type_params, type_args, range.ty),
            });
        }
        for cell in &summary.variant_required_param_cells {
            let Some(arg) = args.get(cell.param_index) else {
                continue;
            };
            let arg = raw_aliases.canonicalize(arg);
            let cell_ty = instantiate_summary_type(&summary.type_params, type_args, cell.ty);
            let Some(suffix) =
                instantiate_summary_suffix_with_types(types, args, arg.ty, &cell.suffix, cell_ty)
            else {
                continue;
            };
            self.push_unique_requirement(PendingVariantRawCellRequirement {
                result: output.clone(),
                variant: normalize_variant_name(&cell.variant),
                arg,
                suffix,
                ty: cell_ty,
            });
        }
    }

    pub(super) fn apply_match_arm(
        &self,
        engine: &mut ResourceCheckEngine<'_>,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        scrutinee: &Place,
        pattern: &ResourceMatchPattern,
        span: crate::span::Span,
    ) {
        let Some(variant) = match_pattern_variant_name(pattern) else {
            return;
        };
        if self
            .concrete_variant(scrutinee)
            .is_some_and(|concrete| concrete != variant)
        {
            return;
        }
        if self.variant_is_unreachable(scrutinee, &variant) {
            return;
        }
        for range in &self.byte_ranges {
            if range.result != *scrutinee || range.variant != variant {
                continue;
            }
            let address_arg = raw_aliases.canonicalize(&range.address_arg);
            let address = projected_place_with_concrete_type(
                engine.types,
                &address_arg,
                &range.address_suffix,
                range.address_ty,
            );
            let count = pending_variant_count_place(engine.types, raw_aliases, &range.count);
            let count = raw_aliases.canonicalize_scalar(&count);
            cells.mark_initialized_raw_byte_range(&address, &count, range.unit, range.ty);
        }
        for requirement in &self.requirements {
            if requirement.result != *scrutinee || requirement.variant != variant {
                continue;
            }
            let arg = raw_aliases.canonicalize(&requirement.arg);
            let place = projected_place_with_concrete_type(
                engine.types,
                &arg,
                &requirement.suffix,
                requirement.ty,
            );
            let initialized_by_byte_range =
                raw_cell_address_prefix(&place).is_some_and(|address| {
                    cells.raw_cell_initialized_by_byte_range(
                        &address,
                        place.ty,
                        raw_aliases,
                        engine.types,
                    )
                });
            let loaded_from_untracked_source =
                raw_cell_address_prefix(&place).is_some_and(|address| {
                    raw_aliases
                        .aliases_for(&address)
                        .iter()
                        .any(|alias| cells.raw_cell_is_untracked_external(alias))
                        || raw_memory_load_reads_zero_initialized_runtime_cell(
                            cells,
                            raw_aliases,
                            &address,
                        )
                });
            if !initialized_by_byte_range && !loaded_from_untracked_source {
                engine.ensure_available(
                    cells,
                    &place,
                    ResourceCheckOperation::RawMemoryLoadCell,
                    span,
                );
            }
        }
        for condition in &self.value_conditions {
            if condition.result != *scrutinee || condition.variant != variant {
                continue;
            }
            let place = raw_aliases.canonicalize(&condition.place);
            raw_aliases.add_i32_condition(&place, condition.condition);
        }
        for entry in &self.entries {
            if entry.result != *scrutinee || entry.variant != variant {
                continue;
            }
            let arg = raw_aliases.canonicalize(&entry.arg);
            let place =
                projected_place_with_concrete_type(engine.types, &arg, &entry.suffix, entry.ty);
            cells.mark_initialized(&place);
            if entry.holds_raw_address {
                mark_known_raw_address(raw_aliases, &place);
            }
        }
    }

    pub(super) fn match_arm_reachable(
        &self,
        scrutinee: &Place,
        pattern: &ResourceMatchPattern,
    ) -> bool {
        let Some(variant) = match_pattern_variant_name(pattern) else {
            return true;
        };
        if let Some(concrete) = self.concrete_variant(scrutinee) {
            return concrete == variant;
        }
        !self.variant_is_unreachable(scrutinee, &variant)
    }

    pub(super) fn record_concrete_variant(&mut self, result: &Place, variant: &str) {
        self.concrete_variants
            .retain(|entry| entry.result != *result);
        self.push_unique_concrete(PendingConcreteVariant {
            result: result.clone(),
            variant: normalize_variant_name(variant),
        });
    }

    pub(super) fn copy_result(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let copies = self
            .entries
            .iter()
            .filter_map(|entry| {
                let result = replace_place_prefix(&entry.result, source, target)?;
                Some(PendingVariantRawCellInitialization {
                    result,
                    variant: entry.variant.clone(),
                    arg: replace_embedded_place_prefixes(&entry.arg, source, target),
                    suffix: entry.suffix.clone(),
                    ty: entry.ty,
                    holds_raw_address: entry.holds_raw_address,
                })
            })
            .collect::<Vec<_>>();
        let requirement_copies = self
            .requirements
            .iter()
            .filter_map(|entry| {
                let result = replace_place_prefix(&entry.result, source, target)?;
                Some(PendingVariantRawCellRequirement {
                    result,
                    variant: entry.variant.clone(),
                    arg: replace_embedded_place_prefixes(&entry.arg, source, target),
                    suffix: entry.suffix.clone(),
                    ty: entry.ty,
                })
            })
            .collect::<Vec<_>>();
        let byte_range_copies = self
            .byte_ranges
            .iter()
            .filter_map(|entry| {
                let result = replace_place_prefix(&entry.result, source, target)?;
                Some(PendingVariantRawByteRangeInitialization {
                    result,
                    variant: entry.variant.clone(),
                    address_arg: replace_embedded_place_prefixes(
                        &entry.address_arg,
                        source,
                        target,
                    ),
                    address_suffix: entry.address_suffix.clone(),
                    address_ty: entry.address_ty,
                    count: replace_byte_range_count_prefix(&entry.count, source, target),
                    unit: entry.unit,
                    ty: entry.ty,
                })
            })
            .collect::<Vec<_>>();
        let unreachable_copies = self
            .unreachable_variants
            .iter()
            .filter_map(|entry| {
                let result = replace_place_prefix(&entry.result, source, target)?;
                Some(PendingUnreachableVariant {
                    result,
                    variant: entry.variant.clone(),
                })
            })
            .collect::<Vec<_>>();
        let concrete_copies = self
            .concrete_variants
            .iter()
            .filter_map(|entry| {
                let result = replace_place_prefix(&entry.result, source, target)?;
                Some(PendingConcreteVariant {
                    result,
                    variant: entry.variant.clone(),
                })
            })
            .collect::<Vec<_>>();
        let value_condition_copies = self
            .value_conditions
            .iter()
            .filter_map(|entry| {
                let result = replace_place_prefix(&entry.result, source, target)?;
                Some(PendingVariantValueCondition {
                    result,
                    variant: entry.variant.clone(),
                    place: replace_embedded_place_prefixes(&entry.place, source, target),
                    condition: entry.condition,
                })
            })
            .collect::<Vec<_>>();
        self.remove_result_at_or_below(target);
        for entry in copies {
            self.push_unique_entry(entry);
        }
        for entry in requirement_copies {
            self.push_unique_requirement(entry);
        }
        for entry in byte_range_copies {
            self.push_unique_byte_range(entry);
        }
        for entry in unreachable_copies {
            self.push_unique_unreachable(entry);
        }
        for entry in concrete_copies {
            self.push_unique_concrete(entry);
        }
        for entry in value_condition_copies {
            self.push_unique_value_condition(entry);
        }
    }

    pub(super) fn clear_result(&mut self, result: &Place) {
        self.remove_result_at_or_below(result);
    }

    pub(super) fn resource_payload_entries_equal(&self, other: &Self) -> bool {
        // Concrete variant / unreachable variant / scalar value condition は match の
        // 到達性と値条件を狭める precision 情報である。一方、entries / byte_ranges /
        // requirements は variant payload に閉じた raw-cell proof を表すため、ここが
        // path 間で異なる場合は merged state へ畳むと安全性の証明そのものを失う。
        self.entries == other.entries
            && self.byte_ranges == other.byte_ranges
            && self.requirements == other.requirements
    }

    fn remove_result_at_or_below(&mut self, result: &Place) {
        self.entries
            .retain(|entry| place_suffix_after_prefix(&entry.result, result).is_none());
        self.byte_ranges
            .retain(|entry| place_suffix_after_prefix(&entry.result, result).is_none());
        self.requirements
            .retain(|entry| place_suffix_after_prefix(&entry.result, result).is_none());
        self.unreachable_variants
            .retain(|entry| place_suffix_after_prefix(&entry.result, result).is_none());
        self.concrete_variants
            .retain(|entry| place_suffix_after_prefix(&entry.result, result).is_none());
        self.value_conditions
            .retain(|entry| place_suffix_after_prefix(&entry.result, result).is_none());
    }

    pub(super) fn merge_path_refs(paths: &[&PendingVariantRawCellInitializations]) -> Self {
        let Some(first) = paths.first() else {
            return Self::default();
        };
        let mut out = Self::default();
        for entry in &first.entries {
            if paths
                .iter()
                .skip(1)
                .all(|path| path.entries.iter().any(|existing| existing == entry))
            {
                out.push_unique_entry(entry.clone());
            }
        }
        for entry in &first.requirements {
            if paths
                .iter()
                .skip(1)
                .all(|path| path.requirements.iter().any(|existing| existing == entry))
            {
                out.push_unique_requirement(entry.clone());
            }
        }
        for entry in &first.byte_ranges {
            if paths
                .iter()
                .skip(1)
                .all(|path| path.byte_ranges.iter().any(|existing| existing == entry))
            {
                out.push_unique_byte_range(entry.clone());
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
        for entry in &first.concrete_variants {
            if paths.iter().skip(1).all(|path| {
                path.concrete_variants
                    .iter()
                    .any(|existing| existing == entry)
            }) {
                out.push_unique_concrete(entry.clone());
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

    pub(super) fn concrete_variant(&self, result: &Place) -> Option<&str> {
        self.concrete_variants
            .iter()
            .find(|entry| entry.result == *result)
            .map(|entry| entry.variant.as_str())
    }

    fn record_unreachable_variants(
        &mut self,
        types: &TypeCtx,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        summary_type_params: &[crate::types::TypeId],
        type_args: &[crate::types::TypeId],
        conditions: &[RawCellInitializationVariantCondition],
    ) {
        let mut variants = Vec::new();
        for condition in conditions {
            if !variants.iter().any(|variant| variant == &condition.variant) {
                variants.push(condition.variant.clone());
            }
        }
        for variant in variants {
            let mut saw_path = false;
            let mut all_paths_false = true;
            for condition in conditions
                .iter()
                .filter(|condition| condition.variant == variant)
            {
                saw_path = true;
                if !variant_path_condition_is_known_false(
                    types,
                    raw_aliases,
                    args,
                    summary_type_params,
                    type_args,
                    condition,
                ) {
                    all_paths_false = false;
                    break;
                }
            }
            if saw_path && all_paths_false {
                self.push_unique_unreachable(PendingUnreachableVariant {
                    result: output.clone(),
                    variant: normalize_variant_name(&variant),
                });
            }
        }
    }

    fn record_value_conditions(
        &mut self,
        types: &TypeCtx,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        summary_type_params: &[crate::types::TypeId],
        type_args: &[crate::types::TypeId],
        conditions: &[RawCellInitializationVariantCondition],
    ) {
        let mut variants = Vec::new();
        for condition in conditions {
            if !variants.iter().any(|variant| variant == &condition.variant) {
                variants.push(condition.variant.clone());
            }
        }
        for variant in variants {
            let viable_paths = conditions
                .iter()
                .filter(|condition| condition.variant == variant)
                .filter(|condition| {
                    !variant_path_condition_is_known_false(
                        types,
                        raw_aliases,
                        args,
                        summary_type_params,
                        type_args,
                        condition,
                    )
                })
                .collect::<Vec<_>>();
            let Some(first_path) = viable_paths.first() else {
                continue;
            };
            for condition in &first_path.conditions {
                if !viable_paths
                    .iter()
                    .skip(1)
                    .all(|path| path.conditions.iter().any(|existing| existing == condition))
                {
                    continue;
                }
                let Some(place) = instantiate_variant_value_condition_place(
                    types,
                    raw_aliases,
                    args,
                    summary_type_params,
                    type_args,
                    condition,
                ) else {
                    continue;
                };
                self.push_unique_value_condition(PendingVariantValueCondition {
                    result: output.clone(),
                    variant: normalize_variant_name(&variant),
                    place,
                    condition: raw_cell_value_condition_to_i32(condition.condition),
                });
            }
        }
    }

    fn variant_is_unreachable(&self, result: &Place, variant: &str) -> bool {
        self.unreachable_variants
            .iter()
            .any(|entry| entry.result == *result && entry.variant == variant)
    }

    fn push_unique_entry(&mut self, entry: PendingVariantRawCellInitialization) {
        if self.entries.iter().any(|existing| existing == &entry) {
            return;
        }
        self.entries.push(entry);
    }

    fn push_unique_requirement(&mut self, entry: PendingVariantRawCellRequirement) {
        if self.requirements.iter().any(|existing| existing == &entry) {
            return;
        }
        self.requirements.push(entry);
    }

    fn push_unique_byte_range(&mut self, entry: PendingVariantRawByteRangeInitialization) {
        if self.byte_ranges.iter().any(|existing| existing == &entry) {
            return;
        }
        self.byte_ranges.push(entry);
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

    fn push_unique_concrete(&mut self, entry: PendingConcreteVariant) {
        if self
            .concrete_variants
            .iter()
            .any(|existing| existing == &entry)
        {
            return;
        }
        self.concrete_variants.push(entry);
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

fn variant_path_condition_is_known_false(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    summary_type_params: &[crate::types::TypeId],
    type_args: &[crate::types::TypeId],
    condition: &RawCellInitializationVariantCondition,
) -> bool {
    condition.conditions.iter().any(|condition| {
        variant_value_condition_truth(
            types,
            raw_aliases,
            args,
            summary_type_params,
            type_args,
            condition,
        )
        .is_some_and(|holds| !holds)
    })
}

fn variant_value_condition_truth(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    summary_type_params: &[crate::types::TypeId],
    type_args: &[crate::types::TypeId],
    condition: &RawCellInitializationVariantValueCondition,
) -> Option<bool> {
    let place = instantiate_variant_value_condition_place(
        types,
        raw_aliases,
        args,
        summary_type_params,
        type_args,
        condition,
    )?;
    raw_aliases
        .i32_value(&place)
        .map(|value| condition.condition.holds(value))
}

fn instantiate_variant_value_condition_place(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    summary_type_params: &[crate::types::TypeId],
    type_args: &[crate::types::TypeId],
    condition: &RawCellInitializationVariantValueCondition,
) -> Option<Place> {
    let arg = args.get(condition.param_index)?;
    let condition_ty = instantiate_summary_type(summary_type_params, type_args, condition.ty);
    let place = instantiate_summary_suffix_on_base_with_types(
        types,
        args,
        None,
        arg,
        &condition.suffix,
        condition_ty,
    )?;
    Some(raw_aliases.canonicalize(&place))
}

fn mark_known_raw_address(raw_aliases: &mut RawCellAddressAliases, place: &Place) {
    if !raw_aliases.contains_exact(place) {
        raw_aliases.mark(place);
    }
}

fn raw_cell_value_condition_to_i32(condition: RawCellValueCondition) -> I32ValueCondition {
    match condition {
        RawCellValueCondition::EqZero => I32ValueCondition::EqZero,
        RawCellValueCondition::NeZero => I32ValueCondition::NeZero,
        RawCellValueCondition::Positive => I32ValueCondition::Positive,
        RawCellValueCondition::NonPositive => I32ValueCondition::NonPositive,
        RawCellValueCondition::Negative => I32ValueCondition::Negative,
        RawCellValueCondition::NonNegative => I32ValueCondition::NonNegative,
    }
}

fn replace_byte_range_count_prefix(
    count: &PendingVariantRawByteRangeCount,
    source: &Place,
    target: &Place,
) -> PendingVariantRawByteRangeCount {
    match count {
        PendingVariantRawByteRangeCount::ArgProjection { arg, suffix, ty } => {
            PendingVariantRawByteRangeCount::ArgProjection {
                arg: replace_embedded_place_prefixes(arg, source, target),
                suffix: suffix.clone(),
                ty: *ty,
            }
        }
        PendingVariantRawByteRangeCount::KnownI32 { value, ty } => {
            PendingVariantRawByteRangeCount::KnownI32 {
                value: *value,
                ty: *ty,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeId;

    fn local(name: &str) -> Place {
        Place::local(String::from(name), TypeId(1))
    }

    fn field0(place: &Place) -> Place {
        place.clone().with_projection(
            super::super::model::PlaceProjection::Field {
                index: 0,
                offset_bytes: 0,
            },
            TypeId(2),
        )
    }

    #[test]
    fn copy_result_preserves_concrete_variant_under_aggregate_projection() {
        let source = local("source");
        let target = local("target");
        let mut pending = PendingVariantRawCellInitializations::default();

        pending.record_concrete_variant(&field0(&source), "VecStorage::Empty");
        pending.copy_result(&source, &target);

        assert_eq!(
            pending.concrete_variant(&field0(&target)),
            Some("Empty")
        );
    }

    #[test]
    fn clear_result_removes_concrete_variant_under_aggregate_projection() {
        let target = local("target");
        let mut pending = PendingVariantRawCellInitializations::default();

        pending.record_concrete_variant(&field0(&target), "VecStorage::Empty");
        pending.clear_result(&target);

        assert_eq!(pending.concrete_variant(&field0(&target)), None);
    }
}
