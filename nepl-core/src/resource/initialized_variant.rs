extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::model::{Place, ResourceMatchPattern};
use super::place_utils::place_with_suffix;
use super::report::ResourceCheckOperation;

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

#[derive(Debug, Clone, Default)]
pub(super) struct PendingVariantRawCellInitializations {
    entries: Vec<PendingVariantRawCellInitialization>,
    requirements: Vec<PendingVariantRawCellRequirement>,
}

impl PendingVariantRawCellInitializations {
    pub(super) fn record_call(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        summary: &RawCellInitializationFunctionSummary,
    ) {
        self.clear_result(output);
        for cell in &summary.variant_param_cells {
            let Some(arg) = args.get(cell.param_index) else {
                continue;
            };
            self.push_unique_entry(PendingVariantRawCellInitialization {
                result: output.clone(),
                variant: normalize_variant_name(&cell.variant),
                arg: raw_aliases.canonicalize(arg),
                suffix: cell.suffix.clone(),
                ty: cell.ty,
                holds_raw_address: cell.holds_raw_address,
            });
        }
        for cell in &summary.variant_required_param_cells {
            let Some(arg) = args.get(cell.param_index) else {
                continue;
            };
            self.push_unique_requirement(PendingVariantRawCellRequirement {
                result: output.clone(),
                variant: normalize_variant_name(&cell.variant),
                arg: raw_aliases.canonicalize(arg),
                suffix: cell.suffix.clone(),
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
        for requirement in &self.requirements {
            if requirement.result != *scrutinee || requirement.variant != variant {
                continue;
            }
            let arg = raw_aliases.canonicalize(&requirement.arg);
            let place = place_with_suffix(&arg, &requirement.suffix, requirement.ty);
            engine.ensure_available(
                cells,
                &place,
                ResourceCheckOperation::RawMemoryLoadCell,
                span,
            );
        }
        for entry in &self.entries {
            if entry.result != *scrutinee || entry.variant != variant {
                continue;
            }
            let arg = raw_aliases.canonicalize(&entry.arg);
            let place = place_with_suffix(&arg, &entry.suffix, entry.ty);
            cells.mark_initialized(&place);
            if entry.holds_raw_address {
                mark_known_raw_address(raw_aliases, &place);
            }
        }
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
        self.clear_result(target);
        for entry in copies {
            self.push_unique_entry(entry);
        }
        for entry in requirement_copies {
            self.push_unique_requirement(entry);
        }
    }

    pub(super) fn clear_result(&mut self, result: &Place) {
        self.entries.retain(|entry| entry.result != *result);
        self.requirements.retain(|entry| entry.result != *result);
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
        out
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
}

pub(super) fn normalize_variant_name(variant: &str) -> String {
    String::from(variant.rsplit("::").next().unwrap_or(variant))
}

fn match_pattern_variant_name(pattern: &ResourceMatchPattern) -> Option<String> {
    let ResourceMatchPattern::Variant(variant) = pattern else {
        return None;
    };
    Some(normalize_variant_name(variant))
}

fn mark_known_raw_address(raw_aliases: &mut RawCellAddressAliases, place: &Place) {
    if !raw_aliases.contains_exact(place) {
        raw_aliases.mark(place);
    }
}
