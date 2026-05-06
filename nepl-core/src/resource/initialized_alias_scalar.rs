extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias_i32::{condition_implication, I32ConditionFact, I32ValueFact};
use super::model::{I32ValueCondition, Place};
use super::place_utils::{place_suffix_after_prefix, replace_place_prefix};

#[derive(Debug, Clone, Default)]
pub(super) struct I32AliasFacts {
    values: Vec<I32ValueFact>,
    conditions: Vec<I32ConditionFact>,
}

impl I32AliasFacts {
    pub(super) fn set_value(&mut self, place: &Place, value: i32) {
        self.values.retain(|fact| fact.place != *place);
        self.values.push(I32ValueFact {
            place: place.clone(),
            value,
        });
    }

    pub(super) fn add_condition(&mut self, place: &Place, condition: I32ValueCondition) {
        self.push_condition_fact(I32ConditionFact {
            place: place.clone(),
            condition,
        });
    }

    pub(super) fn value_for_aliases(&self, aliases: &[Place]) -> Option<i32> {
        let mut value = None;
        for alias in aliases {
            for fact in &self.values {
                if fact.place != *alias {
                    continue;
                }
                match value {
                    Some(existing) if existing != fact.value => return None,
                    Some(_) => {}
                    None => value = Some(fact.value),
                }
            }
        }
        value
    }

    pub(super) fn condition_truth_for_aliases(
        &self,
        aliases: &[Place],
        condition: I32ValueCondition,
    ) -> Option<bool> {
        let mut truth = None;
        for alias in aliases {
            for fact in &self.conditions {
                if fact.place != *alias {
                    continue;
                }
                let Some(fact_truth) = condition_implication(fact.condition, condition) else {
                    continue;
                };
                match truth {
                    Some(existing) if existing != fact_truth => return None,
                    Some(_) => {}
                    None => truth = Some(fact_truth),
                }
            }
        }
        truth
    }

    pub(super) fn facts_with_replaced_prefix(&self, source: &Place, target: &Place) -> Self {
        let mut out = I32AliasFacts::default();
        for fact in &self.values {
            if let Some(place) = replace_place_prefix(&fact.place, source, target) {
                out.push_value_fact(I32ValueFact {
                    place,
                    value: fact.value,
                });
            }
        }
        for fact in &self.conditions {
            if let Some(place) = replace_place_prefix(&fact.place, source, target) {
                out.push_condition_fact(I32ConditionFact {
                    place,
                    condition: fact.condition,
                });
            }
        }
        out
    }

    pub(super) fn extend(&mut self, facts: I32AliasFacts) {
        for fact in facts.values {
            self.push_value_fact(fact);
        }
        for fact in facts.conditions {
            self.push_condition_fact(fact);
        }
    }

    pub(super) fn clear_prefix(&mut self, place: &Place) {
        self.values
            .retain(|fact| place_suffix_after_prefix(&fact.place, place).is_none());
        self.conditions
            .retain(|fact| place_suffix_after_prefix(&fact.place, place).is_none());
    }

    pub(super) fn merge_paths<'a>(
        paths: impl IntoIterator<Item = &'a I32AliasFacts>,
    ) -> I32AliasFacts {
        let paths = paths.into_iter().collect::<Vec<_>>();
        let mut out = I32AliasFacts::default();
        if let Some(first) = paths.first() {
            for fact in &first.values {
                if paths
                    .iter()
                    .skip(1)
                    .all(|path| path.values.iter().any(|existing| existing == fact))
                {
                    out.push_value_fact(fact.clone());
                }
            }
            for fact in &first.conditions {
                if paths
                    .iter()
                    .skip(1)
                    .all(|path| path.conditions.iter().any(|existing| existing == fact))
                {
                    out.push_condition_fact(fact.clone());
                }
            }
        }
        out
    }

    fn push_value_fact(&mut self, fact: I32ValueFact) {
        self.values.retain(|existing| existing.place != fact.place);
        self.values.push(fact);
    }

    fn push_condition_fact(&mut self, fact: I32ConditionFact) {
        if self.conditions.iter().any(|existing| existing == &fact) {
            return;
        }
        self.conditions.push(fact);
    }
}
