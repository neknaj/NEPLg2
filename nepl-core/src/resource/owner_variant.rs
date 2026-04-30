extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeId;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceMatchPattern};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::place_utils::place_with_suffix;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;
use super::summary::OwnerReturnSummary;

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

#[derive(Debug, Clone, Default)]
pub(super) struct PendingVariantOwnerEffects {
    consumptions: Vec<PendingVariantOwnerConsumption>,
    returns: Vec<PendingVariantOwnerReturn>,
}

impl PendingVariantOwnerEffects {
    pub(super) fn record_call(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        summary: &OwnerReturnSummary,
    ) {
        self.clear_result(output);
        for entry in &summary.variant_consumed_parameter_indices {
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
        self.clear_result(target);
        for entry in copies {
            self.push_unique_consumption(entry);
        }
        for entry in return_copies {
            self.push_unique_return(entry);
        }
    }

    pub(super) fn clear_result(&mut self, result: &Place) {
        self.consumptions.retain(|entry| entry.result != *result);
        self.returns.retain(|entry| entry.result != *result);
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
        out
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
