extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use super::owner_extent::merge_owner_extent_summaries;
use super::summary::{
    OwnerConsumedExtentRequirement, OwnerExtentSummary, OwnerHostSizeReturn,
    OwnerParameterReturnExtent,
    OwnerProjectionReturnOwner, OwnerProjectionReturnSummary, OwnerReturnSummary,
    OwnerTypeSizeReturn, OwnerValueCondition, OwnerVariantCondition,
    OwnerVariantConsumedExtentRequirement, OwnerVariantParameterIndex,
    OwnerVariantPayloadCondition, OwnerVariantProjectionReturn, OwnerVariantProjectionSource,
};

pub(super) fn canonicalize_owner_return_summary(summary: &mut OwnerReturnSummary) {
    canonicalize_ord_vec(&mut summary.parameter_indices);
    canonicalize_ord_vec(&mut summary.parameter_sources);
    canonicalize_parameter_return_extents(&mut summary.parameter_return_extents);
    canonicalize_ord_vec(&mut summary.consumed_parameter_indices);
    canonicalize_ord_vec(&mut summary.consumed_parameter_sources);
    canonicalize_consumed_extent_requirements(&mut summary.consumed_extent_requirements);
    canonicalize_ord_vec(&mut summary.memory_span_requirements);
    canonicalize_variant_summary_channels(
        &mut summary.variant_consumed_parameter_indices,
        &mut summary.variant_consumed_parameter_sources,
        &mut summary.variant_consumed_extent_requirements,
        &mut summary.variant_conditions,
        &mut summary.variant_payload_conditions,
        &mut summary.host_size_returns,
        &mut summary.type_size_returns,
        &mut summary.variant_projection_returns,
    );
    canonicalize_ord_vec(&mut summary.resolved_parameter_variants);
    canonicalize_ord_vec(&mut summary.non_owning_raw_view_returns);
    canonicalize_owner_extent_summary(&mut summary.returns_fresh_owner_extent);
    canonicalize_projection_return_summaries(&mut summary.projection_returns);
    canonicalize_ord_vec(&mut summary.projection_markers);
    canonicalize_ord_vec(&mut summary.storage_origin_markers);
}

fn canonicalize_ord_vec<T: Ord>(items: &mut Vec<T>) {
    items.sort_unstable();
    items.dedup();
}

fn canonicalize_parameter_return_extents(extents: &mut Vec<OwnerParameterReturnExtent>) {
    for extent in extents.iter_mut() {
        canonicalize_owner_extent_summary(&mut extent.extent);
    }
    extents.sort_unstable_by(|left, right| left.source.cmp(&right.source));
    let mut merged: Vec<OwnerParameterReturnExtent> = Vec::new();
    for extent in extents.drain(..) {
        if let Some(existing) = merged
            .last_mut()
            .filter(|existing| existing.source == extent.source)
        {
            existing.extent = merge_owner_extent_summaries(existing.extent.clone(), extent.extent);
            canonicalize_owner_extent_summary(&mut existing.extent);
        } else {
            merged.push(extent);
        }
    }
    *extents = merged;
}

fn canonicalize_consumed_extent_requirements(
    requirements: &mut Vec<OwnerConsumedExtentRequirement>,
) {
    for requirement in requirements.iter_mut() {
        canonicalize_owner_extent_summary(&mut requirement.extent);
    }
    requirements.sort_unstable_by(|left, right| {
        left.owner
            .cmp(&right.owner)
            .then_with(|| left.operation.cmp(&right.operation))
    });
    let mut merged: Vec<OwnerConsumedExtentRequirement> = Vec::new();
    for requirement in requirements.drain(..) {
        if let Some(existing) = merged.last_mut().filter(|existing| {
            existing.owner == requirement.owner && existing.operation == requirement.operation
        }) {
            existing.extent =
                merge_owner_extent_summaries(existing.extent.clone(), requirement.extent);
            canonicalize_owner_extent_summary(&mut existing.extent);
        } else {
            merged.push(requirement);
        }
    }
    *requirements = merged;
}

fn canonicalize_variant_consumed_extent_requirements(
    requirements: &mut Vec<OwnerVariantConsumedExtentRequirement>,
) {
    for requirement in requirements.iter_mut() {
        canonicalize_owner_extent_summary(&mut requirement.extent);
    }
    requirements.sort_unstable_by(|left, right| {
        left.variant
            .cmp(&right.variant)
            .then_with(|| left.owner.cmp(&right.owner))
            .then_with(|| left.operation.cmp(&right.operation))
    });
    let mut merged: Vec<OwnerVariantConsumedExtentRequirement> = Vec::new();
    for requirement in requirements.drain(..) {
        if let Some(existing) = merged.last_mut().filter(|existing| {
            existing.variant == requirement.variant
                && existing.owner == requirement.owner
                && existing.operation == requirement.operation
        }) {
            existing.extent =
                merge_owner_extent_summaries(existing.extent.clone(), requirement.extent);
            canonicalize_owner_extent_summary(&mut existing.extent);
        } else {
            merged.push(requirement);
        }
    }
    *requirements = merged;
}

fn canonicalize_variant_projection_returns(returns: &mut Vec<OwnerVariantProjectionReturn>) {
    for entry in returns.iter_mut() {
        canonicalize_projection_return_owner(&mut entry.owner);
    }
    canonicalize_ord_vec(returns);
}

fn canonicalize_projection_return_owner(owner: &mut OwnerProjectionReturnOwner) {
    match owner {
        OwnerProjectionReturnOwner::Parameter {
            returned_extent, ..
        } => canonicalize_owner_extent_summary(returned_extent),
        OwnerProjectionReturnOwner::Fresh { extent } => {
            canonicalize_owner_extent_summary(extent);
        }
        OwnerProjectionReturnOwner::UnknownSource { extent } => {
            canonicalize_owner_extent_summary(extent);
        }
        OwnerProjectionReturnOwner::Maybe => {}
    }
}

fn canonicalize_variant_conditions(conditions: &mut Vec<OwnerVariantCondition>) {
    for condition in conditions.iter_mut() {
        condition.condition = normalize_owner_value_condition(condition.condition.clone());
    }
    canonicalize_ord_vec(conditions);
    let mut grouped: Vec<OwnerVariantCondition> = Vec::new();
    let mut current_variant = None;
    let mut current_conditions = Vec::new();
    for condition in conditions.drain(..) {
        if current_variant
            .as_ref()
            .is_some_and(|variant| *variant != condition.variant)
        {
            if let Some(variant) = current_variant.take() {
                push_grouped_variant_condition(
                    &mut grouped,
                    variant,
                    core::mem::take(&mut current_conditions),
                );
            }
        }
        current_variant = Some(condition.variant);
        current_conditions.push(condition.condition);
    }
    if let Some(variant) = current_variant {
        push_grouped_variant_condition(&mut grouped, variant, current_conditions);
    }
    *conditions = grouped;
}

pub(super) fn canonicalize_variant_summary_channels(
    indices: &mut Vec<OwnerVariantParameterIndex>,
    sources: &mut Vec<OwnerVariantProjectionSource>,
    extents: &mut Vec<OwnerVariantConsumedExtentRequirement>,
    conditions: &mut Vec<OwnerVariantCondition>,
    payload_conditions: &mut Vec<OwnerVariantPayloadCondition>,
    host_sizes: &mut Vec<OwnerHostSizeReturn>,
    type_sizes: &mut Vec<OwnerTypeSizeReturn>,
    returns: &mut Vec<OwnerVariantProjectionReturn>,
) {
    canonicalize_ord_vec(indices);
    canonicalize_ord_vec(sources);
    canonicalize_variant_consumed_extent_requirements(extents);
    canonicalize_variant_conditions(conditions);
    canonicalize_ord_vec(payload_conditions);
    canonicalize_ord_vec(host_sizes);
    canonicalize_ord_vec(type_sizes);
    canonicalize_variant_projection_returns(returns);
}

fn push_grouped_variant_condition(
    grouped: &mut Vec<OwnerVariantCondition>,
    variant: String,
    conditions: Vec<OwnerValueCondition>,
) {
    grouped.push(OwnerVariantCondition {
        variant,
        condition: normalize_owner_value_condition(OwnerValueCondition::Any(conditions)),
    });
}

fn canonicalize_projection_return_summaries(summaries: &mut Vec<OwnerProjectionReturnSummary>) {
    for summary in summaries.iter_mut() {
        canonicalize_ord_vec(&mut summary.parameter_indices);
        canonicalize_ord_vec(&mut summary.parameter_sources);
        canonicalize_parameter_return_extents(&mut summary.parameter_return_extents);
        canonicalize_owner_extent_summary(&mut summary.returns_fresh_owner_extent);
    }
    canonicalize_ord_vec(summaries);
}

fn canonicalize_owner_extent_summary(extent: &mut OwnerExtentSummary) {
    match extent {
        OwnerExtentSummary::Unknown
        | OwnerExtentSummary::RegionTokenSize
        | OwnerExtentSummary::PayloadBytesParameter(_)
        | OwnerExtentSummary::PayloadBytesParameterScaled { .. }
        | OwnerExtentSummary::PayloadBytesParameterTypeSize { .. }
        | OwnerExtentSummary::PayloadBytesI32Constant { .. } => {}
    }
}

#[derive(Clone, Copy)]
enum OwnerValueConditionListKind {
    Any,
    All,
}

fn normalize_owner_value_condition(condition: OwnerValueCondition) -> OwnerValueCondition {
    match condition {
        OwnerValueCondition::Always | OwnerValueCondition::Param { .. } => condition,
        OwnerValueCondition::Any(conditions) => {
            normalize_owner_value_condition_list(OwnerValueConditionListKind::Any, conditions)
        }
        OwnerValueCondition::All(conditions) => {
            normalize_owner_value_condition_list(OwnerValueConditionListKind::All, conditions)
        }
    }
}

fn normalize_owner_value_condition_list(
    kind: OwnerValueConditionListKind,
    conditions: Vec<OwnerValueCondition>,
) -> OwnerValueCondition {
    let mut normalized = Vec::new();
    for condition in conditions {
        match (kind, normalize_owner_value_condition(condition)) {
            (OwnerValueConditionListKind::Any, OwnerValueCondition::Always) => {
                return OwnerValueCondition::Always;
            }
            (OwnerValueConditionListKind::All, OwnerValueCondition::Always) => {}
            (OwnerValueConditionListKind::Any, OwnerValueCondition::Any(conditions))
            | (OwnerValueConditionListKind::All, OwnerValueCondition::All(conditions)) => {
                normalized.extend(conditions);
            }
            (_, condition) => normalized.push(condition),
        }
    }
    canonicalize_ord_vec(&mut normalized);
    match (kind, normalized.len()) {
        (OwnerValueConditionListKind::All, 0) => OwnerValueCondition::Always,
        (OwnerValueConditionListKind::Any, 0) => OwnerValueCondition::Any(normalized),
        (_, 1) => normalized.pop().unwrap_or(OwnerValueCondition::Always),
        (OwnerValueConditionListKind::Any, _) => OwnerValueCondition::Any(normalized),
        (OwnerValueConditionListKind::All, _) => OwnerValueCondition::All(normalized),
    }
}
