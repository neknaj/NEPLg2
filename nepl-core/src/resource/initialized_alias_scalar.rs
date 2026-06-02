extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias_i32::{condition_implication, I32ConditionFact, I32ValueFact};
use super::model::{I32ValueCondition, Place};
use super::place_utils::{place_suffix_after_prefix, replace_place_prefix};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct I32AliasFacts {
    values: Vec<I32ValueFact>,
    conditions: Vec<I32ConditionFact>,
}

impl I32AliasFacts {
    pub(super) fn has_condition_sources(&self) -> bool {
        !self.values.is_empty() || !self.conditions.is_empty()
    }

    pub(super) fn has_condition_sources_for_aliases(&self, aliases: &[Place]) -> bool {
        aliases.iter().any(|alias| {
            self.values.iter().any(|fact| fact.place == *alias)
                || self.conditions.iter().any(|fact| fact.place == *alias)
        })
    }

    pub(super) fn set_value(&mut self, place: &Place, value: i32) {
        self.values.retain(|fact| fact.place != *place);
        self.conditions.retain(|fact| fact.place != *place);
        self.values.push(I32ValueFact {
            place: place.clone(),
            value,
        });
        for condition in conditions_implied_by_i32_value(value) {
            self.push_condition_fact(I32ConditionFact {
                place: place.clone(),
                condition: *condition,
            });
        }
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
        let mut saw_direct_implication = false;
        for alias in aliases {
            for fact in &self.conditions {
                if fact.place != *alias {
                    continue;
                }
                let Some(fact_truth) = condition_implication(fact.condition, condition) else {
                    continue;
                };
                saw_direct_implication = true;
                match truth {
                    Some(existing) if existing != fact_truth => return None,
                    Some(_) => {}
                    None => truth = Some(fact_truth),
                }
            }
        }
        if saw_direct_implication {
            return truth;
        }
        self.combined_condition_truth_for_aliases(aliases, condition)
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

    fn combined_condition_truth_for_aliases(
        &self,
        aliases: &[Place],
        condition: I32ValueCondition,
    ) -> Option<bool> {
        use I32ValueCondition::{EqZero, NeZero, Negative, NonNegative, NonPositive, Positive};

        let derived = if self.aliases_have_condition(aliases, NonNegative)
            && self.aliases_have_condition(aliases, NonPositive)
        {
            EqZero
        } else if self.aliases_have_condition(aliases, NonNegative)
            && self.aliases_have_condition(aliases, NeZero)
        {
            Positive
        } else if self.aliases_have_condition(aliases, NonPositive)
            && self.aliases_have_condition(aliases, NeZero)
        {
            Negative
        } else {
            return None;
        };
        condition_implication(derived, condition)
    }

    fn aliases_have_condition(&self, aliases: &[Place], condition: I32ValueCondition) -> bool {
        aliases.iter().any(|alias| {
            self.conditions
                .iter()
                .any(|fact| fact.place == *alias && fact.condition == condition)
        })
    }
}

fn conditions_implied_by_i32_value(value: i32) -> &'static [I32ValueCondition] {
    use I32ValueCondition::{EqZero, NeZero, Negative, NonNegative, NonPositive, Positive};
    if value == 0 {
        &[EqZero, NonNegative, NonPositive]
    } else if value > 0 {
        &[NeZero, Positive, NonNegative]
    } else {
        &[NeZero, Negative, NonPositive]
    }
}
