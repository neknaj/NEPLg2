extern crate alloc;

use alloc::vec::Vec;

use super::owner_extent::merge_owner_extent_summaries;
use super::summary::{
    OwnerConsumedExtentRequirement, OwnerExtentSummary, OwnerParameterReturnExtent,
    OwnerProjectionReturnOwner, OwnerProjectionReturnSummary, OwnerReturnSummary,
    OwnerValueCondition, OwnerVariantCondition, OwnerVariantConsumedExtentRequirement,
    OwnerVariantProjectionReturn,
};

pub(super) fn update_owner_return_summary(
    summaries: &mut Vec<OwnerReturnSummary>,
    mut summary: OwnerReturnSummary,
) -> bool {
    canonicalize_owner_return_summary(&mut summary);
    let position = summaries
        .iter()
        .position(|existing| existing.function == summary.function);
    match (owner_return_summary_has_facts(&summary), position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
        }
        (true, None) => {
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
            true
        }
        (false, None) => false,
    }
}

fn canonicalize_owner_return_summary(summary: &mut OwnerReturnSummary) {
    canonicalize_ord_vec(&mut summary.parameter_indices);
    canonicalize_ord_vec(&mut summary.parameter_sources);
    canonicalize_parameter_return_extents(&mut summary.parameter_return_extents);
    canonicalize_ord_vec(&mut summary.consumed_parameter_indices);
    canonicalize_ord_vec(&mut summary.consumed_parameter_sources);
    canonicalize_consumed_extent_requirements(&mut summary.consumed_extent_requirements);
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
        | OwnerExtentSummary::PayloadBytesParameter(_)
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

fn owner_return_summary_has_facts(summary: &OwnerReturnSummary) -> bool {
    summary.returns_fresh_owner
        || summary.returns_maybe_owner
        || !summary.non_owning_raw_view_returns.is_empty()
        || !summary.parameter_indices.is_empty()
        || !summary.parameter_sources.is_empty()
        || !summary.parameter_return_extents.is_empty()
        || !summary.consumed_parameter_indices.is_empty()
        || !summary.consumed_parameter_sources.is_empty()
        || !summary.consumed_extent_requirements.is_empty()
        || !summary.variant_consumed_parameter_indices.is_empty()
        || !summary.variant_consumed_parameter_sources.is_empty()
        || !summary.variant_consumed_extent_requirements.is_empty()
        || !summary.variant_projection_returns.is_empty()
        || !summary.resolved_parameter_variants.is_empty()
        || !summary.variant_conditions.is_empty()
        || !summary.variant_payload_conditions.is_empty()
        || !summary.projection_returns.is_empty()
        || !summary.projection_markers.is_empty()
        || !summary.storage_origin_markers.is_empty()
}

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec;

    use crate::types::TypeId;

    use super::*;
    use crate::resource::model::I32ValueCondition;

    fn source(parameter_index: usize) -> super::super::summary::OwnerProjectionSource {
        super::super::summary::OwnerProjectionSource {
            parameter_index,
            suffix: Vec::new(),
            ty: TypeId(parameter_index),
        }
    }

    #[test]
    fn update_owner_return_summary_ignores_fact_order() {
        let first = OwnerReturnSummary {
            function: String::from("f"),
            parameter_indices: vec![1, 0],
            parameter_sources: Vec::new(),
            parameter_return_extents: vec![
                OwnerParameterReturnExtent {
                    source: source(1),
                    extent: OwnerExtentSummary::Unknown,
                },
                OwnerParameterReturnExtent {
                    source: source(0),
                    extent: OwnerExtentSummary::Unknown,
                },
            ],
            consumed_parameter_indices: Vec::new(),
            consumed_parameter_sources: Vec::new(),
            consumed_extent_requirements: Vec::new(),
            variant_consumed_parameter_indices: Vec::new(),
            variant_consumed_parameter_sources: Vec::new(),
            variant_consumed_extent_requirements: Vec::new(),
            variant_projection_returns: Vec::new(),
            resolved_parameter_variants: Vec::new(),
            variant_conditions: Vec::new(),
            variant_payload_conditions: Vec::new(),
            non_owning_raw_view_returns: Vec::new(),
            returns_fresh_owner: false,
            returns_fresh_owner_extent: OwnerExtentSummary::Unknown,
            returns_maybe_owner: false,
            projection_returns: Vec::new(),
            projection_markers: Vec::new(),
            storage_origin_markers: Vec::new(),
        };
        let mut second = first.clone();
        second.parameter_indices = vec![0, 1, 0];
        second.parameter_return_extents.reverse();

        let mut summaries = Vec::new();
        assert!(update_owner_return_summary(&mut summaries, first));
        assert!(!update_owner_return_summary(&mut summaries, second));
    }

    #[test]
    fn update_owner_return_summary_canonicalizes_nested_conditions() {
        let mut first = empty_summary("g");
        first.variant_conditions.push(OwnerVariantCondition {
            variant: String::from("Result::Ok"),
            condition: OwnerValueCondition::All(vec![
                OwnerValueCondition::Param {
                    source: source(0),
                    condition: I32ValueCondition::Positive,
                },
                OwnerValueCondition::Always,
            ]),
        });

        let mut second = empty_summary("g");
        second.variant_conditions.push(OwnerVariantCondition {
            variant: String::from("Result::Ok"),
            condition: OwnerValueCondition::Param {
                source: source(0),
                condition: I32ValueCondition::Positive,
            },
        });

        let mut summaries = Vec::new();
        assert!(update_owner_return_summary(&mut summaries, first));
        assert!(!update_owner_return_summary(&mut summaries, second));
    }

    fn empty_summary(function: &str) -> OwnerReturnSummary {
        OwnerReturnSummary {
            function: String::from(function),
            parameter_indices: Vec::new(),
            parameter_sources: Vec::new(),
            parameter_return_extents: Vec::new(),
            consumed_parameter_indices: Vec::new(),
            consumed_parameter_sources: Vec::new(),
            consumed_extent_requirements: Vec::new(),
            variant_consumed_parameter_indices: Vec::new(),
            variant_consumed_parameter_sources: Vec::new(),
            variant_consumed_extent_requirements: Vec::new(),
            variant_projection_returns: Vec::new(),
            resolved_parameter_variants: Vec::new(),
            variant_conditions: Vec::new(),
            variant_payload_conditions: Vec::new(),
            non_owning_raw_view_returns: Vec::new(),
            returns_fresh_owner: false,
            returns_fresh_owner_extent: OwnerExtentSummary::Unknown,
            returns_maybe_owner: false,
            projection_returns: Vec::new(),
            projection_markers: Vec::new(),
            storage_origin_markers: Vec::new(),
        }
    }
}
