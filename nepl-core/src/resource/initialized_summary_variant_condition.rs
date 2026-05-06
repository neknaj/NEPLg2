extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::RawCellInitializationVariantCondition;
use super::initialized_summary_condition::RawCellValueCondition;
use super::initialized_variant::normalize_variant_name;
use super::model::{Place, ResourceConditionFact, ResourceLocal};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn collect_variant_param_condition(
    out: &mut Vec<RawCellInitializationVariantCondition>,
    variant: &str,
    condition_fact: Option<&ResourceConditionFact>,
    truthy_path: bool,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
) {
    let Some((place, condition)) = variant_value_condition(condition_fact, truthy_path) else {
        return;
    };
    for condition_alias in raw_aliases.aliases_for(place) {
        for (param_index, param) in params.iter().enumerate() {
            for param_alias in raw_aliases.aliases_for(&param.place) {
                let Some(suffix) = place_suffix_after_prefix(&condition_alias, &param_alias) else {
                    continue;
                };
                push_unique_variant_condition(
                    out,
                    RawCellInitializationVariantCondition {
                        variant: normalize_variant_name(variant),
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
    conditions: &mut Vec<RawCellInitializationVariantCondition>,
    condition: RawCellInitializationVariantCondition,
) {
    if !conditions.iter().any(|existing| existing == &condition) {
        conditions.push(condition);
    }
}
