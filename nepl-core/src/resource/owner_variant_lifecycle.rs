extern crate alloc;

use alloc::vec::Vec;

use super::model::Place;
use super::owner_return_apply_source::summary_projection_place;
use super::owner_variant::{
    PendingUnreachableVariant, PendingVariantOwnerConsumption, PendingVariantOwnerEffects,
    PendingVariantOwnerReturn, PendingVariantOwnerReturnSource,
    PendingVariantPayloadValueCondition,
};
use super::owner_variant_utils::{push_unique_source, source_list_contains};
use super::owner_variant_value_condition::PendingVariantValueCondition;

impl PendingVariantOwnerEffects {
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
                extent: entry.extent.clone(),
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

    pub(super) fn resolve_result(&mut self, result: &Place) {
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
                ..
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
                ..
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

    pub(super) fn variant_is_unreachable(&self, result: &Place, variant: &str) -> bool {
        self.unreachable_variants
            .iter()
            .any(|entry| entry.result == *result && entry.variant == variant)
    }

    pub(super) fn push_unique_consumption(&mut self, entry: PendingVariantOwnerConsumption) {
        if self.consumptions.iter().any(|existing| existing == &entry) {
            return;
        }
        self.consumptions.push(entry);
    }

    pub(super) fn push_unique_return(&mut self, entry: PendingVariantOwnerReturn) {
        if self.returns.iter().any(|existing| existing == &entry) {
            return;
        }
        self.returns.push(entry);
    }

    pub(super) fn push_unique_unreachable(&mut self, entry: PendingUnreachableVariant) {
        if self
            .unreachable_variants
            .iter()
            .any(|existing| existing == &entry)
        {
            return;
        }
        self.unreachable_variants.push(entry);
    }

    pub(super) fn push_unique_payload_condition(
        &mut self,
        entry: PendingVariantPayloadValueCondition,
    ) {
        if self
            .payload_conditions
            .iter()
            .any(|existing| existing == &entry)
        {
            return;
        }
        self.payload_conditions.push(entry);
    }

    pub(super) fn push_unique_value_condition(&mut self, entry: PendingVariantValueCondition) {
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
