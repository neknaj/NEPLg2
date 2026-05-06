use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_relation_op::relation_negation;
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
        (ResourceConditionFact::I32Relation { left, op, right }, true) => {
            record_i32_relation_fact(raw_aliases, left, *op, right);
        }
        (ResourceConditionFact::I32Relation { left, op, right }, false) => {
            record_i32_relation_fact(raw_aliases, left, relation_negation(*op), right);
        }
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

fn record_i32_relation_fact(
    raw_aliases: &mut RawCellAddressAliases,
    left: &Place,
    op: super::model::ResourceI32RelationOp,
    right: &Place,
) {
    if raw_aliases.i32_relation_truth(left, op, right) != Some(true) {
        raw_aliases.add_i32_relation(left, op, right);
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
        (ResourceConditionFact::I32Relation { .. }, _)
        | (ResourceConditionFact::Any(_), _)
        | (ResourceConditionFact::All(_), _) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeId;
    use alloc::string::String;

    use super::super::model::ResourceI32RelationOp;

    fn local(name: &str) -> Place {
        Place::local(String::from(name), TypeId(1))
    }

    #[test]
    fn record_condition_fact_retains_truthy_i32_relation() {
        let left = local("i");
        let right = local("len");
        let fact = ResourceConditionFact::I32Relation {
            left: left.clone(),
            op: ResourceI32RelationOp::Lt,
            right: right.clone(),
        };
        let mut raw_aliases = RawCellAddressAliases::default();

        record_condition_fact_value_constraints(&mut raw_aliases, &fact, true);

        assert_eq!(
            raw_aliases.i32_relation_truth(&left, ResourceI32RelationOp::Lt, &right),
            Some(true)
        );
        assert_eq!(
            raw_aliases.i32_relation_truth(&left, ResourceI32RelationOp::Ge, &right),
            Some(false)
        );
        assert_eq!(
            raw_aliases.i32_relation_truth(&right, ResourceI32RelationOp::Gt, &left),
            Some(true)
        );
    }

    #[test]
    fn record_condition_fact_retains_false_i32_relation_as_negation() {
        let left = local("i");
        let right = local("len");
        let fact = ResourceConditionFact::I32Relation {
            left: left.clone(),
            op: ResourceI32RelationOp::Lt,
            right: right.clone(),
        };
        let mut raw_aliases = RawCellAddressAliases::default();

        record_condition_fact_value_constraints(&mut raw_aliases, &fact, false);

        assert_eq!(
            raw_aliases.i32_relation_truth(&left, ResourceI32RelationOp::Ge, &right),
            Some(true)
        );
        assert_eq!(
            raw_aliases.i32_relation_truth(&left, ResourceI32RelationOp::Lt, &right),
            Some(false)
        );
    }
}
