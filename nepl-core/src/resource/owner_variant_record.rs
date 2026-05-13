extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_extent::instantiate_owner_extent_summary;
use super::owner_return_apply_source::{
    owner_projection_source_place_for_arg, summary_projection_place,
};
use super::owner_variant::{
    PendingUnreachableVariant, PendingVariantOwnerConsumption, PendingVariantOwnerEffects,
    PendingVariantOwnerExtentRequirement, PendingVariantOwnerReturn,
    PendingVariantOwnerReturnSource, PendingVariantPayloadValueCondition,
};
use super::owner_variant_condition_truth::owner_value_condition_truth;
use super::owner_variant_value_condition::PendingVariantValueCondition;
use super::summary::{
    OwnerProjectionReturnOwner, OwnerProjectionSource, OwnerReturnSummary, OwnerVariantCondition,
    OwnerVariantConsumedExtentRequirement,
};
use super::variant_name::normalize_variant_name;

impl PendingVariantOwnerEffects {
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
            let source = OwnerProjectionSource {
                parameter_index: entry.parameter_index,
                suffix: Vec::new(),
                ty: arg.ty,
            };
            self.push_unique_consumption(PendingVariantOwnerConsumption {
                result: output.clone(),
                variant: normalize_variant_name(&entry.variant),
                arg: raw_aliases.canonicalize(arg),
                suffix: Vec::new(),
                ty: arg.ty,
                extent: pending_variant_extent_requirement_for_source(
                    args,
                    &summary.variant_consumed_extent_requirements,
                    &entry.variant,
                    &source,
                ),
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
                extent: pending_variant_extent_requirement_for_source(
                    args,
                    &summary.variant_consumed_extent_requirements,
                    &entry.variant,
                    &entry.source,
                ),
            });
        }
        for entry in &summary.variant_projection_returns {
            let source = match &entry.owner {
                OwnerProjectionReturnOwner::Parameter {
                    source,
                    returned_extent,
                } => {
                    let Some(arg) = args.get(source.parameter_index) else {
                        continue;
                    };
                    PendingVariantOwnerReturnSource::Parameter {
                        arg: raw_aliases.canonicalize(arg),
                        source_suffix: source.suffix.clone(),
                        source_ty: summary_projection_place(arg, &source.suffix, source.ty).ty,
                        extent_requirement: pending_variant_extent_requirement_for_source(
                            args,
                            &summary.variant_consumed_extent_requirements,
                            &entry.variant,
                            source,
                        ),
                        returned_extent: instantiate_owner_extent_summary(args, returned_extent),
                    }
                }
                OwnerProjectionReturnOwner::Fresh { extent } => {
                    PendingVariantOwnerReturnSource::Fresh {
                        extent: instantiate_owner_extent_summary(args, extent),
                    }
                }
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
}

fn pending_variant_extent_requirement_for_source(
    args: &[Place],
    requirements: &[OwnerVariantConsumedExtentRequirement],
    variant: &str,
    source: &OwnerProjectionSource,
) -> Option<PendingVariantOwnerExtentRequirement> {
    let variant = normalize_variant_name(variant);
    let requirement = requirements
        .iter()
        .find(|requirement| requirement.variant == variant && requirement.owner == *source)?;
    let expected = instantiate_owner_extent_summary(args, &requirement.extent);
    if matches!(expected, super::model::OwnerStorageExtent::Unknown) {
        return None;
    }
    Some(PendingVariantOwnerExtentRequirement {
        expected,
        operation: requirement.operation,
    })
}
