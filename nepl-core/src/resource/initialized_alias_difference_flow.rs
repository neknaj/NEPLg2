extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias_difference::{I32DifferenceFact, I32DifferenceFacts};
use super::model::Place;
use super::place_utils::{place_suffix_after_prefix, replace_place_prefix};

impl I32DifferenceFacts {
    pub(super) fn facts_with_replaced_prefix(&self, source: &Place, target: &Place) -> Self {
        let mut out = I32DifferenceFacts::default();
        for fact in &self.facts {
            let minuend = replace_place_prefix(&fact.minuend, source, target);
            let subtrahend = replace_place_prefix(&fact.subtrahend, source, target);
            let difference = replace_place_prefix(&fact.difference, source, target);
            match (minuend, subtrahend, difference) {
                (Some(minuend), Some(subtrahend), Some(difference)) => {
                    out.push_difference_fact(I32DifferenceFact {
                        minuend,
                        subtrahend,
                        difference,
                    });
                }
                (Some(minuend), Some(subtrahend), None) => {
                    out.push_difference_fact(I32DifferenceFact {
                        minuend,
                        subtrahend,
                        difference: fact.difference.clone(),
                    });
                }
                (Some(minuend), None, Some(difference)) => {
                    out.push_difference_fact(I32DifferenceFact {
                        minuend,
                        subtrahend: fact.subtrahend.clone(),
                        difference,
                    });
                }
                (None, Some(subtrahend), Some(difference)) => {
                    out.push_difference_fact(I32DifferenceFact {
                        minuend: fact.minuend.clone(),
                        subtrahend,
                        difference,
                    });
                }
                (Some(minuend), None, None) => {
                    out.push_difference_fact(I32DifferenceFact {
                        minuend,
                        subtrahend: fact.subtrahend.clone(),
                        difference: fact.difference.clone(),
                    });
                }
                (None, Some(subtrahend), None) => {
                    out.push_difference_fact(I32DifferenceFact {
                        minuend: fact.minuend.clone(),
                        subtrahend,
                        difference: fact.difference.clone(),
                    });
                }
                (None, None, Some(difference)) => {
                    out.push_difference_fact(I32DifferenceFact {
                        minuend: fact.minuend.clone(),
                        subtrahend: fact.subtrahend.clone(),
                        difference,
                    });
                }
                (None, None, None) => {}
            }
        }
        out
    }

    pub(super) fn clear_prefix(&mut self, place: &Place) {
        self.facts.retain(|fact| {
            place_suffix_after_prefix(&fact.minuend, place).is_none()
                && place_suffix_after_prefix(&fact.subtrahend, place).is_none()
                && place_suffix_after_prefix(&fact.difference, place).is_none()
        });
    }

    pub(super) fn merge_paths<'a>(
        paths: impl IntoIterator<Item = &'a I32DifferenceFacts>,
    ) -> I32DifferenceFacts {
        let paths = paths.into_iter().collect::<Vec<_>>();
        let mut out = I32DifferenceFacts::default();
        if let Some(first) = paths.first() {
            for fact in &first.facts {
                if paths
                    .iter()
                    .skip(1)
                    .all(|path| path.facts.iter().any(|existing| existing == fact))
                {
                    out.push_difference_fact(fact.clone());
                }
            }
        }
        out
    }
}
