extern crate alloc;

use alloc::vec::Vec;

use super::model::Place;
use super::place_utils::{place_suffix_after_prefix, replace_place_prefix};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ScaleFact {
    pub(super) source: Place,
    pub(super) target: Place,
    pub(super) scale: usize,
}

#[derive(Debug, Clone, Default)]
pub(super) struct I32ScaleFacts {
    facts: Vec<I32ScaleFact>,
}

impl I32ScaleFacts {
    pub(super) fn add_scale(&mut self, source: &Place, target: &Place, scale: usize) {
        if scale == 0 {
            return;
        }
        self.facts.retain(|fact| fact.target != *target);
        self.push_scale_fact(I32ScaleFact {
            source: source.clone(),
            target: target.clone(),
            scale,
        });
    }

    pub(super) fn scaled_sources_for_aliases(&self, aliases: &[Place]) -> Vec<(Place, usize)> {
        let mut out = Vec::new();
        for alias in aliases {
            for fact in &self.facts {
                if fact.target != *alias {
                    continue;
                }
                let candidate = (fact.source.clone(), fact.scale);
                if !out.iter().any(|existing| existing == &candidate) {
                    out.push(candidate);
                }
            }
        }
        out
    }

    pub(super) fn scaled_targets_for_source_aliases(
        &self,
        aliases: &[Place],
        scale: usize,
    ) -> Vec<Place> {
        let mut out = Vec::new();
        for alias in aliases {
            for fact in &self.facts {
                if fact.source == *alias && fact.scale == scale && !out.contains(&fact.target) {
                    out.push(fact.target.clone());
                }
            }
        }
        out
    }

    pub(super) fn facts_with_replaced_prefix(&self, source: &Place, target: &Place) -> Self {
        let mut out = I32ScaleFacts::default();
        for fact in &self.facts {
            let source_place = replace_place_prefix(&fact.source, source, target);
            let target_place = replace_place_prefix(&fact.target, source, target);
            match (source_place, target_place) {
                (Some(source), Some(target)) => out.push_scale_fact(I32ScaleFact {
                    source,
                    target,
                    scale: fact.scale,
                }),
                (Some(source), None) => out.push_scale_fact(I32ScaleFact {
                    source,
                    target: fact.target.clone(),
                    scale: fact.scale,
                }),
                (None, Some(target)) => out.push_scale_fact(I32ScaleFact {
                    source: fact.source.clone(),
                    target,
                    scale: fact.scale,
                }),
                (None, None) => {}
            }
        }
        out
    }

    pub(super) fn extend(&mut self, facts: I32ScaleFacts) {
        for fact in facts.facts {
            self.push_scale_fact(fact);
        }
    }

    pub(super) fn clear_prefix(&mut self, place: &Place) {
        self.facts.retain(|fact| {
            place_suffix_after_prefix(&fact.source, place).is_none()
                && place_suffix_after_prefix(&fact.target, place).is_none()
        });
    }

    pub(super) fn merge_paths<'a>(
        paths: impl IntoIterator<Item = &'a I32ScaleFacts>,
    ) -> I32ScaleFacts {
        let paths = paths.into_iter().collect::<Vec<_>>();
        let mut out = I32ScaleFacts::default();
        if let Some(first) = paths.first() {
            for fact in &first.facts {
                if paths
                    .iter()
                    .skip(1)
                    .all(|path| path.facts.iter().any(|existing| existing == fact))
                {
                    out.push_scale_fact(fact.clone());
                }
            }
        }
        out
    }

    fn push_scale_fact(&mut self, fact: I32ScaleFact) {
        if self.facts.iter().any(|existing| existing == &fact) {
            return;
        }
        self.facts.push(fact);
    }
}
