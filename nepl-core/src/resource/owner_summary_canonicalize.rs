extern crate alloc;

use alloc::vec::Vec;

use super::owner_extent::merge_owner_extent_summaries;
use super::summary::{
    OwnerConsumedExtentRequirement, OwnerExtentSummary, OwnerParameterReturnExtent,
    OwnerProjectionReturnOwner, OwnerProjectionReturnSummary, OwnerReturnSummary,
    OwnerValueCondition, OwnerVariantCondition, OwnerVariantConsumedExtentRequirement,
    OwnerVariantProjectionReturn,
};

pub(super) fn canonicalize_owner_return_summary(summary: &mut OwnerReturnSummary) {
    canonicalize_ord_vec(&mut summary.parameter_indices);
    canonicalize_ord_vec(&mut summary.parameter_sources);
    canonicalize_parameter_return_extents(&mut summary.parameter_return_extents);
    canonicalize_ord_vec(&mut summary.consumed_parameter_indices);
    canonicalize_ord_vec(&mut summary.consumed_parameter_sources);
    canonicalize_consumed_extent_requirements(&mut summary.consumed_extent_requirements);
    canonicalize_ord_vec(&mut summary.memory_span_requirements);
    canonicalize_ord_vec(&mut summary.host_size_returns);
    canonicalize_ord_vec(&mut summary.type_size_returns);
    canonicalize_ord_vec(&mut summary.variant_consumed_parameter_indices);
    canonicalize_ord_vec(&mut summary.variant_consumed_parameter_sources);
    canonicalize_variant_consumed_extent_requirements(
        &mut summary.variant_consumed_extent_requirements,
    );
    canonicalize_variant_projection_returns(&mut summary.variant_projection_returns);
    canonicalize_ord_vec(&mut summary.resolved_parameter_variants);
    canonicalize_variant_conditions(&mut summary.variant_conditions);
    canonicalize_ord_vec(&mut summary.variant_payload_conditions);
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
            .iter_mut()
            .find(|existing| existing.source == extent.source)
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
        if let Some(existing) = merged.iter_mut().find(|existing| {
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
        if let Some(existing) = merged.iter_mut().find(|existing| {
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
