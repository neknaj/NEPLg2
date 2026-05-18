use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::resource::model::I32ValueCondition;
use crate::resource::summary::{
    OwnerExtentSummary, OwnerParameterReturnExtent, OwnerProjectionSource, OwnerReturnSummary,
    OwnerValueCondition, OwnerVariantCondition,
};
use crate::types::TypeId;

use super::*;

fn source(parameter_index: usize) -> OwnerProjectionSource {
    OwnerProjectionSource {
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
        host_memory_span_requirements: Vec::new(),
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
        host_memory_span_requirements: Vec::new(),
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
