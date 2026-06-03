extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary_condition::RawCellValueCondition;
use super::initialized_summary_variant_model::{
    RawCellInitializationVariantCondition, RawCellInitializationVariantValueCondition,
};
use super::model::{Place, ResourceConditionFact, ResourceLocal};
use super::place_utils::place_suffix_after_prefix;
use super::summary_projection::summary_suffix_for_params;
use super::variant_name::normalize_variant_name;

pub(super) fn collect_variant_param_value_conditions(
    out: &mut Vec<RawCellInitializationVariantValueCondition>,
    condition_fact: Option<&ResourceConditionFact>,
    truthy_path: bool,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
) {
    let Some((place, condition)) = variant_value_condition(condition_fact, truthy_path) else {
        return;
    };
    for condition_alias in raw_aliases.scalar_aliases_for_value(place) {
        for (param_index, param) in params.iter().enumerate() {
            for param_alias in raw_aliases.scalar_aliases_for_value(&param.place) {
                let Some(suffix) = place_suffix_after_prefix(&condition_alias, &param_alias) else {
                    continue;
                };
                let Some(suffix) = summary_suffix_for_params(params, &suffix) else {
                    continue;
                };
                push_unique_variant_condition(
                    out,
                    RawCellInitializationVariantValueCondition {
                        param_index,
                        suffix,
                        ty: condition_alias.ty,
                        condition,
                    },
                );
            }
        }
    }
}

pub(super) fn push_unique_variant_path_condition(
    conditions: &mut Vec<RawCellInitializationVariantCondition>,
    variant: &str,
    path_conditions: Vec<RawCellInitializationVariantValueCondition>,
) {
    let condition = RawCellInitializationVariantCondition {
        variant: normalize_variant_name(variant),
        conditions: path_conditions,
    };
    if !conditions.iter().any(|existing| existing == &condition) {
        conditions.push(condition);
    }
}

fn variant_value_condition(
    condition_fact: Option<&ResourceConditionFact>,
    truthy_path: bool,
) -> Option<(&Place, RawCellValueCondition)> {
    match (condition_fact?, truthy_path) {
        (ResourceConditionFact::EqZero { place }, true)
        | (ResourceConditionFact::NeZero { place }, false) => {
            Some((place, RawCellValueCondition::EqZero))
        }
        (ResourceConditionFact::EqZero { place }, false)
        | (ResourceConditionFact::NeZero { place }, true) => {
            Some((place, RawCellValueCondition::NeZero))
        }
        (ResourceConditionFact::Positive { place }, true)
        | (ResourceConditionFact::NonPositive { place }, false) => {
            Some((place, RawCellValueCondition::Positive))
        }
        (ResourceConditionFact::Positive { place }, false)
        | (ResourceConditionFact::NonPositive { place }, true) => {
            Some((place, RawCellValueCondition::NonPositive))
        }
        (ResourceConditionFact::Negative { place }, true)
        | (ResourceConditionFact::NonNegative { place }, false) => {
            Some((place, RawCellValueCondition::Negative))
        }
        (ResourceConditionFact::Negative { place }, false)
        | (ResourceConditionFact::NonNegative { place }, true) => {
            Some((place, RawCellValueCondition::NonNegative))
        }
        (ResourceConditionFact::I32Relation { .. }, _)
        | (ResourceConditionFact::Any(_), _)
        | (ResourceConditionFact::All(_), _) => None,
    }
}

fn push_unique_variant_condition(
    conditions: &mut Vec<RawCellInitializationVariantValueCondition>,
    condition: RawCellInitializationVariantValueCondition,
) {
    if !conditions.iter().any(|existing| existing == &condition) {
        conditions.push(condition);
    }
}
