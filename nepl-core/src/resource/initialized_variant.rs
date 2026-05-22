extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use super::cell_state::{raw_cell_address_prefix, CellTable};
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_raw_memory_access::raw_memory_load_reads_zero_initialized_runtime_cell;
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary_variant_model::RawCellInitializationVariantCondition;
use super::initialized_variant_count::{
    pending_variant_count_place, pending_variant_count_source, PendingVariantRawByteRangeCount,
};
use super::model::{Place, ResourceMatchPattern};
use super::place_utils::projected_place_with_concrete_type;
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

#[derive(Debug, Clone, Default)]
pub(super) struct PendingVariantRawCellInitializations {
    entries: Vec<PendingVariantRawCellInitialization>,
    byte_ranges: Vec<PendingVariantRawByteRangeInitialization>,
    requirements: Vec<PendingVariantRawCellRequirement>,
    unreachable_variants: Vec<PendingUnreachableVariant>,
}

impl PendingVariantRawCellInitializations {
    pub(super) fn record_call(
        &mut self,
        types: &TypeCtx,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        summary: &RawCellInitializationFunctionSummary,
    ) {
        self.clear_result(output);
        self.record_unreachable_variants(
            types,
            raw_aliases,
            output,
            args,
            &summary.variant_conditions,
        );
        for cell in &summary.variant_param_cells {
            let Some(arg) = args.get(cell.param_index) else {
                continue;
            };
            let arg = raw_aliases.canonicalize(arg);
            let Some(suffix) =
                instantiate_summary_suffix_with_types(types, args, arg.ty, &cell.suffix, cell.ty)
            else {
                continue;
            };
            self.push_unique_entry(PendingVariantRawCellInitialization {
                result: output.clone(),
                variant: normalize_variant_name(&cell.variant),
                arg,
                suffix,
                ty: cell.ty,
                holds_raw_address: cell.holds_raw_address,
            });
        }
        for range in &summary.variant_param_byte_ranges {
            let Some(address_arg) = args.get(range.address_param_index) else {
                continue;
            };
            let address_arg = raw_aliases.canonicalize(address_arg);
            let Some(address_suffix) = instantiate_summary_suffix_with_types(
                types,
                args,
                address_arg.ty,
                &range.address_suffix,
                range.address_ty,
            ) else {
                continue;
            };
            let Some(count) = pending_variant_count_source(types, raw_aliases, args, &range.count)
            else {
                continue;
            };
            self.push_unique_byte_range(PendingVariantRawByteRangeInitialization {
                result: output.clone(),
                variant: normalize_variant_name(&range.variant),
                address_arg,
                address_suffix,
                address_ty: range.address_ty,
                count,
                unit: range.unit,
                ty: range.ty,
            });
        }
        for cell in &summary.variant_required_param_cells {
            let Some(arg) = args.get(cell.param_index) else {
                continue;
            };
            let arg = raw_aliases.canonicalize(arg);
            let Some(suffix) =
                instantiate_summary_suffix_with_types(types, args, arg.ty, &cell.suffix, cell.ty)
            else {
                continue;
            };
            self.push_unique_requirement(PendingVariantRawCellRequirement {
                result: output.clone(),
                variant: normalize_variant_name(&cell.variant),
                arg,
                suffix,
                ty: cell.ty,
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
        !self.variant_is_unreachable(scrutinee, &variant)
    }

    pub(super) fn copy_result(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let copies = self
            .entries
            .iter()
            .filter(|entry| entry.result == *source)
            .map(|entry| PendingVariantRawCellInitialization {
                result: target.clone(),
                variant: entry.variant.clone(),
                arg: entry.arg.clone(),
                suffix: entry.suffix.clone(),
                ty: entry.ty,
                holds_raw_address: entry.holds_raw_address,
            })
            .collect::<Vec<_>>();
        let requirement_copies = self
            .requirements
            .iter()
            .filter(|entry| entry.result == *source)
            .map(|entry| PendingVariantRawCellRequirement {
                result: target.clone(),
                variant: entry.variant.clone(),
                arg: entry.arg.clone(),
                suffix: entry.suffix.clone(),
                ty: entry.ty,
            })
            .collect::<Vec<_>>();
        let byte_range_copies = self
            .byte_ranges
            .iter()
            .filter(|entry| entry.result == *source)
            .map(|entry| PendingVariantRawByteRangeInitialization {
                result: target.clone(),
                variant: entry.variant.clone(),
                address_arg: entry.address_arg.clone(),
                address_suffix: entry.address_suffix.clone(),
                address_ty: entry.address_ty,
                count: entry.count.clone(),
                unit: entry.unit,
                ty: entry.ty,
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
        self.clear_result(target);
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
    }

    pub(super) fn clear_result(&mut self, result: &Place) {
        self.entries.retain(|entry| entry.result != *result);
        self.byte_ranges.retain(|entry| entry.result != *result);
        self.requirements.retain(|entry| entry.result != *result);
        self.unreachable_variants
            .retain(|entry| entry.result != *result);
    }

    pub(super) fn merge_paths(paths: &[PendingVariantRawCellInitializations]) -> Self {
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
        out
    }

    fn record_unreachable_variants(
        &mut self,
        types: &TypeCtx,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        conditions: &[RawCellInitializationVariantCondition],
    ) {
        for condition in conditions {
            let Some(arg) = args.get(condition.param_index) else {
                continue;
            };
            let Some(place) = instantiate_summary_suffix_on_base_with_types(
                types,
                args,
                arg,
                &condition.suffix,
                condition.ty,
            ) else {
                continue;
            };
            let place = raw_aliases.canonicalize(&place);
            let Some(value) = raw_aliases.i32_value(&place) else {
                continue;
            };
            if condition.condition.holds(value) {
                continue;
            }
            self.push_unique_unreachable(PendingUnreachableVariant {
                result: output.clone(),
                variant: normalize_variant_name(&condition.variant),
            });
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
}

fn mark_known_raw_address(raw_aliases: &mut RawCellAddressAliases, place: &Place) {
    if !raw_aliases.contains_exact(place) {
        raw_aliases.mark(place);
    }
}
