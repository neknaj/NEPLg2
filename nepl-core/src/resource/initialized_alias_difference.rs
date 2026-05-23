extern crate alloc;

use alloc::vec::Vec;

use super::model::Place;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32DifferenceFact {
    pub(super) minuend: Place,
    pub(super) subtrahend: Place,
    pub(super) difference: Place,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct I32DifferenceFacts {
    pub(super) facts: Vec<I32DifferenceFact>,
}

impl I32DifferenceFacts {
    pub(super) fn add_difference(
        &mut self,
        minuend: &Place,
        subtrahend: &Place,
        difference: &Place,
    ) {
        self.push_difference_fact(I32DifferenceFact {
            minuend: minuend.clone(),
            subtrahend: subtrahend.clone(),
            difference: difference.clone(),
        });
    }

    pub(super) fn difference_sources_for_aliases(
        &self,
        difference_aliases: &[Place],
    ) -> Vec<(Place, Place)> {
        let mut out = Vec::new();
        for fact in &self.facts {
            if !difference_aliases
                .iter()
                .any(|alias| alias == &fact.difference)
            {
                continue;
            }
            if !out
                .iter()
                .any(|existing| existing == &(fact.minuend.clone(), fact.subtrahend.clone()))
            {
                out.push((fact.minuend.clone(), fact.subtrahend.clone()));
            }
        }
        out
    }

    pub(super) fn extend(&mut self, facts: I32DifferenceFacts) {
        for fact in facts.facts {
            self.push_difference_fact(fact);
        }
    }

    pub(super) fn push_difference_fact(&mut self, fact: I32DifferenceFact) {
        if self.facts.iter().any(|existing| existing == &fact) {
            return;
        }
        self.facts.push(fact);
    }
}
