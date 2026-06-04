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

pub(super) fn condition_fact_truth(
    raw_aliases: &RawCellAddressAliases,
    fact: &ResourceConditionFact,
) -> Option<bool> {
    if let Some((place, condition)) = simple_condition_value_constraint(fact, true) {
        return raw_aliases.i32_condition_truth(place, condition);
    }
    match fact {
        ResourceConditionFact::I32Relation { left, op, right } => {
            raw_aliases.i32_relation_truth(left, *op, right)
        }
        ResourceConditionFact::All(facts) => all_condition_facts_truth(raw_aliases, facts),
        ResourceConditionFact::Any(facts) => any_condition_facts_truth(raw_aliases, facts),
        ResourceConditionFact::EqZero { .. }
        | ResourceConditionFact::NeZero { .. }
        | ResourceConditionFact::Positive { .. }
        | ResourceConditionFact::NonPositive { .. }
        | ResourceConditionFact::Negative { .. }
        | ResourceConditionFact::NonNegative { .. } => None,
    }
}

fn all_condition_facts_truth(
    raw_aliases: &RawCellAddressAliases,
    facts: &[ResourceConditionFact],
) -> Option<bool> {
    let mut has_unknown = false;
    for fact in facts {
        match condition_fact_truth(raw_aliases, fact) {
            Some(true) => {}
            Some(false) => return Some(false),
            None => has_unknown = true,
        }
    }
    if has_unknown {
        None
    } else {
        Some(true)
    }
}

fn any_condition_facts_truth(
    raw_aliases: &RawCellAddressAliases,
    facts: &[ResourceConditionFact],
) -> Option<bool> {
    let mut has_unknown = false;
    for fact in facts {
        match condition_fact_truth(raw_aliases, fact) {
            Some(true) => return Some(true),
            Some(false) => {}
            None => has_unknown = true,
        }
    }
    if has_unknown {
        None
    } else {
        Some(false)
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
    use alloc::vec;

    use super::super::model::ResourceI32RelationOp::{Ge, Lt};

    fn local(name: &str) -> Place {
        Place::local(String::from(name), TypeId(1))
    }

    #[test]
    fn record_condition_fact_retains_truthy_i32_relation() {
        let left = local("i");
        let right = local("len");
        let fact = ResourceConditionFact::I32Relation {
            left: left.clone(),
            op: Lt,
            right: right.clone(),
        };
        let mut raw_aliases = RawCellAddressAliases::default();

        record_condition_fact_value_constraints(&mut raw_aliases, &fact, true);

        assert_eq!(
            raw_aliases.i32_relation_truth(&left, Lt, &right),
            Some(true)
        );
        assert_eq!(
            raw_aliases.i32_relation_truth(&left, Ge, &right),
            Some(false)
        );
        assert_eq!(
            raw_aliases.i32_relation_truth(
                &right,
                super::super::model::ResourceI32RelationOp::Gt,
                &left,
            ),
            Some(true)
        );
    }

    #[test]
    fn record_condition_fact_retains_false_i32_relation_as_negation() {
        let left = local("i");
        let right = local("len");
        let fact = ResourceConditionFact::I32Relation {
            left: left.clone(),
            op: Lt,
            right: right.clone(),
        };
        let mut raw_aliases = RawCellAddressAliases::default();

        record_condition_fact_value_constraints(&mut raw_aliases, &fact, false);

        assert_eq!(
            raw_aliases.i32_relation_truth(&left, Ge, &right),
            Some(true)
        );
        assert_eq!(
            raw_aliases.i32_relation_truth(&left, Lt, &right),
            Some(false)
        );
    }

    #[test]
    fn condition_fact_truth_reads_known_i32_value_conditions() {
        let place = local("len");
        let mut raw_aliases = RawCellAddressAliases::default();
        raw_aliases.add_i32_condition(&place, I32ValueCondition::Positive);

        assert_eq!(
            condition_fact_truth(
                &raw_aliases,
                &ResourceConditionFact::Positive {
                    place: place.clone(),
                },
            ),
            Some(true)
        );
        assert_eq!(
            condition_fact_truth(&raw_aliases, &ResourceConditionFact::Negative { place }),
            Some(false)
        );
    }

    #[test]
    fn condition_fact_truth_reads_known_i32_relations() {
        let left = local("i");
        let right = local("len");
        let mut raw_aliases = RawCellAddressAliases::default();
        raw_aliases.add_i32_relation(&left, Lt, &right);

        assert_eq!(
            condition_fact_truth(
                &raw_aliases,
                &ResourceConditionFact::I32Relation {
                    left: left.clone(),
                    op: Lt,
                    right: right.clone(),
                },
            ),
            Some(true)
        );
        assert_eq!(
            condition_fact_truth(
                &raw_aliases,
                &ResourceConditionFact::I32Relation {
                    left,
                    op: Ge,
                    right,
                },
            ),
            Some(false)
        );
    }

    #[test]
    fn condition_fact_truth_keeps_unknown_for_partially_known_composites() {
        let left = local("i");
        let right = local("len");
        let unknown = local("unknown");
        let mut raw_aliases = RawCellAddressAliases::default();
        raw_aliases.add_i32_relation(&left, Lt, &right);

        let fact = ResourceConditionFact::All(vec![
            ResourceConditionFact::I32Relation {
                left,
                op: Lt,
                right,
            },
            ResourceConditionFact::Positive { place: unknown },
        ]);

        assert_eq!(condition_fact_truth(&raw_aliases, &fact), None);
    }

    #[test]
    fn condition_fact_truth_resolves_composite_short_circuits() {
        let left = local("i");
        let right = local("len");
        let unknown = local("unknown");
        let mut raw_aliases = RawCellAddressAliases::default();
        raw_aliases.add_i32_relation(&left, Lt, &right);

        let any = ResourceConditionFact::Any(vec![
            ResourceConditionFact::Positive {
                place: unknown.clone(),
            },
            ResourceConditionFact::I32Relation {
                left: left.clone(),
                op: Lt,
                right: right.clone(),
            },
        ]);
        let all = ResourceConditionFact::All(vec![
            ResourceConditionFact::Positive { place: unknown },
            ResourceConditionFact::I32Relation {
                left,
                op: Ge,
                right,
            },
        ]);

        assert_eq!(condition_fact_truth(&raw_aliases, &any), Some(true));
        assert_eq!(condition_fact_truth(&raw_aliases, &all), Some(false));
    }
}
