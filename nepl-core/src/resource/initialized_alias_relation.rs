extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias_relation_op::{relation_implication, relation_reverse};
use super::model::{Place, ResourceI32RelationOp};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32RelationFact {
    pub(super) left: Place,
    pub(super) op: ResourceI32RelationOp,
    pub(super) right: Place,
}

#[derive(Debug, Clone, Default)]
pub(super) struct I32RelationFacts {
    pub(super) relations: Vec<I32RelationFact>,
}

impl I32RelationFacts {
    pub(super) fn add_relation(&mut self, left: &Place, op: ResourceI32RelationOp, right: &Place) {
        self.push_relation_fact(I32RelationFact {
            left: left.clone(),
            op,
            right: right.clone(),
        });
    }

    pub(super) fn relation_truth_for_aliases(
        &self,
        left_aliases: &[Place],
        op: ResourceI32RelationOp,
        right_aliases: &[Place],
    ) -> Option<bool> {
        let mut truth = None;
        for fact in &self.relations {
            let fact_truth = if left_aliases.iter().any(|alias| alias == &fact.left)
                && right_aliases.iter().any(|alias| alias == &fact.right)
            {
                relation_implication(fact.op, op)
            } else if left_aliases.iter().any(|alias| alias == &fact.right)
                && right_aliases.iter().any(|alias| alias == &fact.left)
            {
                relation_implication(relation_reverse(fact.op), op)
            } else {
                None
            };
            let Some(fact_truth) = fact_truth else {
                continue;
            };
            match truth {
                Some(existing) if existing != fact_truth => return None,
                Some(_) => {}
                None => truth = Some(fact_truth),
            }
        }
        truth
    }

    pub(super) fn extend(&mut self, facts: I32RelationFacts) {
        for fact in facts.relations {
            self.push_relation_fact(fact);
        }
    }

    pub(super) fn push_relation_fact(&mut self, fact: I32RelationFact) {
        if self.relations.iter().any(|existing| existing == &fact) {
            return;
        }
        self.relations.push(fact);
    }
}
