extern crate alloc;

use alloc::vec::Vec;

use super::host_size_contract::HostSizeKind;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::{place_suffix_after_prefix, push_unique_place, replace_place_prefix};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct HostSizeFact {
    place: Place,
    kind: HostSizeKind,
}

#[derive(Debug, Clone, Default)]
pub(super) struct HostSizeFacts {
    facts: Vec<HostSizeFact>,
}

impl HostSizeFacts {
    pub(super) fn set_kind(&mut self, place: &Place, kind: HostSizeKind) {
        self.facts.retain(|fact| fact.place != *place);
        self.push_fact(HostSizeFact {
            place: place.clone(),
            kind,
        });
    }

    pub(super) fn places_for_kind(&self, kind: HostSizeKind) -> Vec<Place> {
        let mut out = Vec::new();
        for fact in &self.facts {
            if fact.kind == kind {
                push_unique_place(&mut out, &fact.place);
            }
        }
        out
    }

    pub(super) fn facts_with_replaced_prefix(&self, source: &Place, target: &Place) -> Self {
        let mut out = HostSizeFacts::default();
        for fact in &self.facts {
            if let Some(place) = replace_place_prefix(&fact.place, source, target) {
                out.push_fact(HostSizeFact {
                    place,
                    kind: fact.kind,
                });
            }
        }
        out
    }

    pub(super) fn extend(&mut self, facts: HostSizeFacts) {
        for fact in facts.facts {
            self.push_fact(fact);
        }
    }

    pub(super) fn clear_prefix(&mut self, place: &Place) {
        self.facts
            .retain(|fact| place_suffix_after_prefix(&fact.place, place).is_none());
    }

    pub(super) fn merge_paths<'a>(
        paths: impl IntoIterator<Item = &'a HostSizeFacts>,
    ) -> HostSizeFacts {
        let paths = paths.into_iter().collect::<Vec<_>>();
        let mut out = HostSizeFacts::default();
        if let Some(first) = paths.first() {
            for fact in &first.facts {
                if paths
                    .iter()
                    .skip(1)
                    .all(|path| path.facts.iter().any(|existing| existing == fact))
                {
                    out.push_fact(fact.clone());
                }
            }
        }
        out
    }

    fn push_fact(&mut self, fact: HostSizeFact) {
        if self.facts.iter().any(|existing| existing == &fact) {
            return;
        }
        self.facts.push(fact);
    }
}

impl RawCellAddressAliases {
    pub(super) fn set_host_size_kind(&mut self, place: &Place, kind: HostSizeKind) {
        let place = self.canonicalize_scalar(place);
        self.host_size_facts.set_kind(&place, kind);
    }

    pub(super) fn host_size_places(&self, kind: HostSizeKind) -> Vec<Place> {
        let mut out = Vec::new();
        for place in self.host_size_facts.places_for_kind(kind) {
            let place = self.canonicalize_scalar(&place);
            push_unique_place(&mut out, &place);
        }
        out
    }
}
