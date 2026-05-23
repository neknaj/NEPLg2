extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::{place_suffix_after_prefix, replace_place_prefix};

#[derive(Debug, Clone, PartialEq, Eq)]
struct I32TypeSizeFact {
    place: Place,
    ty: TypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct I32TypeSizeScaleFact {
    source: Place,
    target: Place,
    ty: TypeId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct I32TypeSizeFacts {
    sizes: Vec<I32TypeSizeFact>,
    scales: Vec<I32TypeSizeScaleFact>,
}

impl I32TypeSizeFacts {
    pub(super) fn set_type_size(&mut self, place: &Place, ty: TypeId) {
        self.sizes.retain(|fact| fact.place != *place);
        self.push_size_fact(I32TypeSizeFact {
            place: place.clone(),
            ty,
        });
    }

    pub(super) fn add_type_size_scale(&mut self, source: &Place, target: &Place, ty: TypeId) {
        self.scales.retain(|fact| fact.target != *target);
        self.push_scale_fact(I32TypeSizeScaleFact {
            source: source.clone(),
            target: target.clone(),
            ty,
        });
    }

    fn type_sizes_for_aliases(&self, aliases: &[Place]) -> Vec<TypeId> {
        let mut out = Vec::new();
        for alias in aliases {
            for fact in &self.sizes {
                if fact.place == *alias && !out.contains(&fact.ty) {
                    out.push(fact.ty);
                }
            }
        }
        out
    }

    fn scaled_sources_for_aliases(&self, aliases: &[Place]) -> Vec<(Place, TypeId)> {
        let mut out = Vec::new();
        for alias in aliases {
            for fact in &self.scales {
                if fact.target != *alias {
                    continue;
                }
                let candidate = (fact.source.clone(), fact.ty);
                if !out.iter().any(|existing| existing == &candidate) {
                    out.push(candidate);
                }
            }
        }
        out
    }

    fn all_size_facts(&self) -> Vec<(Place, TypeId)> {
        let mut out = Vec::new();
        for fact in &self.sizes {
            let candidate = (fact.place.clone(), fact.ty);
            if !out.iter().any(|existing| existing == &candidate) {
                out.push(candidate);
            }
        }
        out
    }

    pub(super) fn facts_with_replaced_prefix(&self, source: &Place, target: &Place) -> Self {
        let mut out = I32TypeSizeFacts::default();
        for fact in &self.sizes {
            if let Some(place) = replace_place_prefix(&fact.place, source, target) {
                out.push_size_fact(I32TypeSizeFact { place, ty: fact.ty });
            }
        }
        for fact in &self.scales {
            let source_place = replace_place_prefix(&fact.source, source, target);
            let target_place = replace_place_prefix(&fact.target, source, target);
            match (source_place, target_place) {
                (Some(source), Some(target)) => out.push_scale_fact(I32TypeSizeScaleFact {
                    source,
                    target,
                    ty: fact.ty,
                }),
                (Some(source), None) => out.push_scale_fact(I32TypeSizeScaleFact {
                    source,
                    target: fact.target.clone(),
                    ty: fact.ty,
                }),
                (None, Some(target)) => out.push_scale_fact(I32TypeSizeScaleFact {
                    source: fact.source.clone(),
                    target,
                    ty: fact.ty,
                }),
                (None, None) => {}
            }
        }
        out
    }

    pub(super) fn extend(&mut self, facts: I32TypeSizeFacts) {
        for fact in facts.sizes {
            self.push_size_fact(fact);
        }
        for fact in facts.scales {
            self.push_scale_fact(fact);
        }
    }

    pub(super) fn clear_prefix(&mut self, place: &Place) {
        self.sizes
            .retain(|fact| place_suffix_after_prefix(&fact.place, place).is_none());
        self.scales.retain(|fact| {
            place_suffix_after_prefix(&fact.source, place).is_none()
                && place_suffix_after_prefix(&fact.target, place).is_none()
        });
    }

    pub(super) fn merge_paths<'a>(
        paths: impl IntoIterator<Item = &'a I32TypeSizeFacts>,
    ) -> I32TypeSizeFacts {
        let paths = paths.into_iter().collect::<Vec<_>>();
        let mut out = I32TypeSizeFacts::default();
        if let Some(first) = paths.first() {
            for fact in &first.sizes {
                if paths
                    .iter()
                    .skip(1)
                    .all(|path| path.sizes.iter().any(|existing| existing == fact))
                {
                    out.push_size_fact(fact.clone());
                }
            }
            for fact in &first.scales {
                if paths
                    .iter()
                    .skip(1)
                    .all(|path| path.scales.iter().any(|existing| existing == fact))
                {
                    out.push_scale_fact(fact.clone());
                }
            }
        }
        out
    }

    fn push_size_fact(&mut self, fact: I32TypeSizeFact) {
        if !self.sizes.iter().any(|existing| existing == &fact) {
            self.sizes.push(fact);
        }
    }

    fn push_scale_fact(&mut self, fact: I32TypeSizeScaleFact) {
        if !self.scales.iter().any(|existing| existing == &fact) {
            self.scales.push(fact);
        }
    }
}

impl RawCellAddressAliases {
    pub(super) fn set_i32_type_size(&mut self, place: &Place, ty: TypeId) {
        let place = self.canonicalize_scalar(place);
        self.i32_type_sizes.set_type_size(&place, ty);
    }

    pub(super) fn i32_type_size(&self, place: &Place) -> Option<TypeId> {
        let mut out = None;
        for ty in self
            .i32_type_sizes
            .type_sizes_for_aliases(&self.scalar_aliases_for(place))
        {
            match out {
                Some(existing) if existing != ty => return None,
                Some(_) => {}
                None => out = Some(ty),
            }
        }
        out
    }

    pub(super) fn i32_type_size_fact_places(&self) -> Vec<(Place, TypeId)> {
        self.i32_type_sizes.all_size_facts()
    }

    pub(super) fn add_i32_type_size_scale(&mut self, source: &Place, target: &Place, ty: TypeId) {
        let source = self.canonicalize_scalar(source);
        self.i32_type_sizes.add_type_size_scale(&source, target, ty);
    }

    pub(super) fn i32_type_size_scaled_source(&self, place: &Place) -> Option<(Place, TypeId)> {
        let mut out = None;
        for (source, ty) in self
            .i32_type_sizes
            .scaled_sources_for_aliases(&self.scalar_aliases_for(place))
        {
            let candidate = (self.canonicalize_scalar(&source), ty);
            match &out {
                Some(existing) if existing != &candidate => return None,
                Some(_) => {}
                None => out = Some(candidate),
            }
        }
        out
    }
}
