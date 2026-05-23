extern crate alloc;

use alloc::vec::Vec;

use super::model::Place;
use super::place_utils::{place_suffix_after_prefix, replace_place_prefix};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32OffsetFact {
    pub(super) source: Place,
    pub(super) target: Place,
    pub(super) offset: i64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct I32OffsetFacts {
    facts: Vec<I32OffsetFact>,
}

impl I32OffsetFacts {
    pub(super) fn set_offsets_for_target(
        &mut self,
        sources: Vec<Place>,
        target: &Place,
        offset: i64,
    ) {
        self.facts.retain(|fact| fact.target != *target);
        for source in sources {
            self.push_offset_fact(I32OffsetFact {
                source,
                target: target.clone(),
                offset,
            });
        }
    }

    pub(super) fn offset_targets_for_source_aliases(&self, aliases: &[Place]) -> Vec<(Place, i64)> {
        let mut out = Vec::new();
        for alias in aliases {
            for fact in &self.facts {
                if fact.source != *alias {
                    continue;
                }
                let candidate = (fact.target.clone(), fact.offset);
                if !out.iter().any(|existing| existing == &candidate) {
                    out.push(candidate);
                }
            }
        }
        out
    }

    pub(super) fn offset_sources_for_target_aliases(&self, aliases: &[Place]) -> Vec<(Place, i64)> {
        let mut out = Vec::new();
        for alias in aliases {
            for fact in &self.facts {
                if fact.target != *alias {
                    continue;
                }
                let candidate = (fact.source.clone(), fact.offset);
                if !out.iter().any(|existing| existing == &candidate) {
                    out.push(candidate);
                }
            }
        }
        out
    }

    pub(super) fn has_offset_for_aliases(&self, aliases: &[Place]) -> bool {
        aliases.iter().any(|alias| {
            self.facts
                .iter()
                .any(|fact| fact.source == *alias || fact.target == *alias)
        })
    }

    pub(super) fn facts_with_replaced_prefix(&self, source: &Place, target: &Place) -> Self {
        let mut out = I32OffsetFacts::default();
        for fact in &self.facts {
            let source_place = replace_place_prefix(&fact.source, source, target);
            let target_place = replace_place_prefix(&fact.target, source, target);
            match (source_place, target_place) {
                (Some(source), Some(target)) => out.push_offset_fact(I32OffsetFact {
                    source,
                    target,
                    offset: fact.offset,
                }),
                (Some(source), None) => out.push_offset_fact(I32OffsetFact {
                    source,
                    target: fact.target.clone(),
                    offset: fact.offset,
                }),
                (None, Some(target)) => out.push_offset_fact(I32OffsetFact {
                    source: fact.source.clone(),
                    target,
                    offset: fact.offset,
                }),
                (None, None) => {}
            }
        }
        out
    }

    pub(super) fn extend(&mut self, facts: I32OffsetFacts) {
        for fact in facts.facts {
            self.push_offset_fact(fact);
        }
    }

    pub(super) fn clear_prefix(&mut self, place: &Place) {
        self.facts.retain(|fact| {
            place_suffix_after_prefix(&fact.source, place).is_none()
                && place_suffix_after_prefix(&fact.target, place).is_none()
        });
    }

    pub(super) fn merge_paths<'a>(
        paths: impl IntoIterator<Item = &'a I32OffsetFacts>,
    ) -> I32OffsetFacts {
        let paths = paths.into_iter().collect::<Vec<_>>();
        let mut out = I32OffsetFacts::default();
        if let Some(first) = paths.first() {
            for fact in &first.facts {
                if paths
                    .iter()
                    .skip(1)
                    .all(|path| path.facts.iter().any(|existing| existing == fact))
                {
                    out.push_offset_fact(fact.clone());
                }
            }
        }
        out
    }

    fn push_offset_fact(&mut self, fact: I32OffsetFact) {
        if self.facts.iter().any(|existing| existing == &fact) {
            return;
        }
        self.facts.push(fact);
    }
}
