use super::initialized_alias::RawCellAddressAliases;
use super::model::{I32ValueCondition, Place, ResourceConditionFact};

pub(super) fn record_condition_fact_value_constraints(
    raw_aliases: &mut RawCellAddressAliases,
    fact: &ResourceConditionFact,
    truthy_path: bool,
) {
    if let Some((place, condition)) = simple_condition_value_constraint(fact, truthy_path) {
        raw_aliases.add_i32_condition(place, condition);
        return;
    }
    match (fact, truthy_path) {
        (ResourceConditionFact::All(facts), true) | (ResourceConditionFact::Any(facts), false) => {
            for fact in facts {
                record_condition_fact_value_constraints(raw_aliases, fact, truthy_path);
            }
        }
        (ResourceConditionFact::All(_), false)
        | (ResourceConditionFact::Any(_), true)
        | (ResourceConditionFact::EqZero { .. }, _)
        | (ResourceConditionFact::NeZero { .. }, _)
        | (ResourceConditionFact::Positive { .. }, _)
        | (ResourceConditionFact::NonPositive { .. }, _)
        | (ResourceConditionFact::Negative { .. }, _)
        | (ResourceConditionFact::NonNegative { .. }, _) => {}
    }
}

pub(super) fn simple_condition_value_constraint(
    fact: &ResourceConditionFact,
    truthy_path: bool,
) -> Option<(&Place, I32ValueCondition)> {
    match (fact, truthy_path) {
        (ResourceConditionFact::EqZero { place }, true)
        | (ResourceConditionFact::NeZero { place }, false) => {
            Some((place, I32ValueCondition::EqZero))
        }
        (ResourceConditionFact::EqZero { place }, false)
        | (ResourceConditionFact::NeZero { place }, true) => {
            Some((place, I32ValueCondition::NeZero))
        }
        (ResourceConditionFact::Positive { place }, true)
        | (ResourceConditionFact::NonPositive { place }, false) => {
            Some((place, I32ValueCondition::Positive))
        }
        (ResourceConditionFact::Positive { place }, false)
        | (ResourceConditionFact::NonPositive { place }, true) => {
            Some((place, I32ValueCondition::NonPositive))
        }
        (ResourceConditionFact::Negative { place }, true)
        | (ResourceConditionFact::NonNegative { place }, false) => {
            Some((place, I32ValueCondition::Negative))
        }
        (ResourceConditionFact::Negative { place }, false)
        | (ResourceConditionFact::NonNegative { place }, true) => {
            Some((place, I32ValueCondition::NonNegative))
        }
        (ResourceConditionFact::Any(_), _) | (ResourceConditionFact::All(_), _) => None,
    }
}
