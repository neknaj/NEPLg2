extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias_relation::{I32RelationFact, I32RelationFacts};
use super::model::Place;
use super::place_utils::{place_suffix_after_prefix, replace_place_prefix};

impl I32RelationFacts {
    pub(super) fn facts_with_replaced_prefix(&self, source: &Place, target: &Place) -> Self {
        let mut out = I32RelationFacts::default();
        for fact in &self.relations {
            let left = replace_place_prefix(&fact.left, source, target);
            let right = replace_place_prefix(&fact.right, source, target);
            match (left, right) {
                (Some(left), Some(right)) => out.push_relation_fact(I32RelationFact {
                    left,
                    op: fact.op,
                    right,
                }),
                (Some(left), None) => out.push_relation_fact(I32RelationFact {
                    left,
                    op: fact.op,
                    right: fact.right.clone(),
                }),
                (None, Some(right)) => out.push_relation_fact(I32RelationFact {
                    left: fact.left.clone(),
                    op: fact.op,
                    right,
                }),
                (None, None) => {}
            }
        }
        out
    }

    pub(super) fn clear_prefix(&mut self, place: &Place) {
        self.relations.retain(|fact| {
            place_suffix_after_prefix(&fact.left, place).is_none()
                && place_suffix_after_prefix(&fact.right, place).is_none()
        });
    }

    pub(super) fn merge_paths<'a>(
        paths: impl IntoIterator<Item = &'a I32RelationFacts>,
    ) -> I32RelationFacts {
        let paths = paths.into_iter().collect::<Vec<_>>();
        let mut out = I32RelationFacts::default();
        if let Some(first) = paths.first() {
            for fact in &first.relations {
                if paths
                    .iter()
                    .skip(1)
                    .all(|path| path.relations.iter().any(|existing| existing == fact))
                {
                    out.push_relation_fact(fact.clone());
                }
            }
        }
        out
    }
}
