extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_extent::instantiate_owner_extent_summary;
use super::owner_projection_source::{
    owner_projection_source_consumed_unconditionally, owner_projection_source_returned_by_variant,
};
use super::owner_return_apply_place::{
    owner_projection_source_place_for_arg, summary_projection_place,
};
use super::owner_variant::{
    PendingVariantOwnerConsumption, PendingVariantOwnerEffects,
    PendingVariantOwnerExtentRequirement, PendingVariantOwnerReturn,
    PendingVariantOwnerReturnSource, PendingVariantPayloadValueCondition,
};
use super::owner_variant_value_condition::PendingVariantValueCondition;
use super::summary::{
    OwnerProjectionReturnOwner, OwnerProjectionSource, OwnerReturnSummary,
    OwnerVariantConsumedExtentRequirement,
};
use super::variant_name::normalize_variant_name;

impl PendingVariantOwnerEffects {
    pub(super) fn record_call(
        &mut self,
        types: &TypeCtx,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        type_args: &[TypeId],
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
            let Some(arg) = args.get(entry.parameter_index) else {
                continue;
            };
            let source = OwnerProjectionSource {
                parameter_index: entry.parameter_index,
                suffix: Vec::new(),
                ty: arg.ty,
            };
            if owner_projection_source_consumed_unconditionally(summary, &source)
                && !owner_projection_source_returned_by_variant(summary, &source)
            {
                continue;
            }
            self.push_unique_consumption(PendingVariantOwnerConsumption {
                result: output.clone(),
                variant: normalize_variant_name(&entry.variant),
                arg: raw_aliases.canonicalize(arg),
                suffix: Vec::new(),
                ty: arg.ty,
                extent: pending_variant_extent_requirement_for_source(
                    types,
                    args,
                    &summary.type_params,
                    type_args,
                    &summary.variant_consumed_extent_requirements,
                    &entry.variant,
                    &source,
                ),
            });
        }
        for entry in &summary.variant_consumed_parameter_sources {
            if owner_projection_source_consumed_unconditionally(summary, &entry.source)
                && !owner_projection_source_returned_by_variant(summary, &entry.source)
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
                    types,
                    args,
                    &summary.type_params,
                    type_args,
                    &summary.variant_consumed_extent_requirements,
                    &entry.variant,
                    &entry.source,
                ),
            });
        }
        for entry in &summary.variant_projection_returns {
            let target = summary_projection_place(output, &entry.suffix, entry.ty);
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
                            types,
                            args,
                            &summary.type_params,
                            type_args,
                            &summary.variant_consumed_extent_requirements,
                            &entry.variant,
                            source,
                        ),
                        returned_extent: instantiate_owner_extent_summary(
                            types,
                            &summary.type_params,
                            type_args,
                            args,
                            returned_extent,
                        ),
                    }
                }
                OwnerProjectionReturnOwner::Fresh { extent } => {
                    PendingVariantOwnerReturnSource::Fresh {
                        extent: instantiate_owner_extent_summary(
                            types,
                            &summary.type_params,
                            type_args,
                            args,
                            extent,
                        ),
                    }
                }
                OwnerProjectionReturnOwner::UnknownSource { extent } => {
                    PendingVariantOwnerReturnSource::UnknownSource {
                        extent: instantiate_owner_extent_summary(
                            types,
                            &summary.type_params,
                            type_args,
                            args,
                            extent,
                        ),
                    }
                }
                OwnerProjectionReturnOwner::Maybe => PendingVariantOwnerReturnSource::Maybe,
            };
            self.push_unique_return(PendingVariantOwnerReturn {
                result: output.clone(),
                variant: normalize_variant_name(&entry.variant),
                target_suffix: entry.suffix.clone(),
                target_ty: target.ty,
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
}

fn pending_variant_extent_requirement_for_source(
    types: &TypeCtx,
    args: &[Place],
    summary_type_params: &[TypeId],
    type_args: &[TypeId],
    requirements: &[OwnerVariantConsumedExtentRequirement],
    variant: &str,
    source: &OwnerProjectionSource,
) -> Option<PendingVariantOwnerExtentRequirement> {
    let variant = normalize_variant_name(variant);
    let requirement = requirements
        .iter()
        .find(|requirement| requirement.variant == variant && requirement.owner == *source)?;
    let expected = instantiate_owner_extent_summary(
        types,
        summary_type_params,
        type_args,
        args,
        &requirement.extent,
    );
    if matches!(expected, super::model::OwnerStorageExtent::Unknown) {
        return None;
    }
    Some(PendingVariantOwnerExtentRequirement {
        expected,
        operation: requirement.operation,
    })
}
