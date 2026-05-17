use alloc::vec;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::effect_place_prefix::{
    groups_overlap, place_has_prefix, place_suffix_after_prefix, push_unique_place,
    push_unique_places, replace_place_prefix,
};
use super::model::{Place, PlaceProjection};

#[derive(Debug, Clone, Default)]
pub(super) struct RawPointerAliasTable {
    groups: Vec<Vec<Place>>,
}

impl RawPointerAliasTable {
    pub(super) fn mark(&mut self, place: &Place) {
        self.union_group(core::slice::from_ref(place));
    }

    pub(super) fn merge_paths(paths: &[RawPointerAliasTable]) -> Self {
        let mut out = RawPointerAliasTable::default();
        for path in paths {
            for group in &path.groups {
                out.union_group(group);
            }
        }
        out
    }

    pub(super) fn projection_aliases_under(
        &self,
        place: &Place,
        seed: &Place,
    ) -> Vec<(Vec<PlaceProjection>, TypeId)> {
        let mut out = Vec::new();
        for group in &self.groups {
            if !group.iter().any(|existing| existing == seed) {
                continue;
            }
            for existing in group {
                let Some(suffix) = place_suffix_after_prefix(existing, place) else {
                    continue;
                };
                push_unique_projection_alias(&mut out, suffix, existing.ty);
            }
        }
        out.sort();
        out
    }

    pub(in crate::resource) fn copy_alias(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let groups = self.groups_with_replaced_prefix_or_singleton(source, target, true);
        self.remove_place(target);
        for group in groups {
            self.union_group(&group);
        }
    }

    pub(super) fn group_for_or_singleton(&self, place: &Place) -> Vec<Place> {
        self.groups
            .iter()
            .find(|group| group.iter().any(|existing| existing == place))
            .cloned()
            .unwrap_or_else(|| vec![place.clone()])
    }

    fn remove_place(&mut self, place: &Place) {
        for group in &mut self.groups {
            group.retain(|existing| !place_has_prefix(existing, place));
        }
        self.groups.retain(|group| !group.is_empty());
    }

    fn groups_with_replaced_prefix_or_singleton(
        &self,
        source: &Place,
        target: &Place,
        drop_target_prefix: bool,
    ) -> Vec<Vec<Place>> {
        let mut out = Vec::new();
        for group in &self.groups {
            let mut mapped = Vec::new();
            let mut mapped_descendant = false;
            for place in group {
                if let Some(replacement) = replace_place_prefix(place, source, target) {
                    if place.projections.len() > source.projections.len() {
                        mapped_descendant = true;
                    }
                    push_unique_place(&mut mapped, replacement);
                }
            }
            if mapped.is_empty() {
                continue;
            }

            let mut merged = if drop_target_prefix {
                group
                    .iter()
                    .filter(|place| !place_has_prefix(place, target))
                    .cloned()
                    .collect()
            } else {
                group.clone()
            };
            push_unique_places(&mut merged, &mapped);
            if mapped_descendant {
                push_unique_place(&mut merged, target.clone());
            }
            out.push(merged);
        }

        if out.is_empty() {
            let mut group = Vec::new();
            push_unique_place(&mut group, source.clone());
            push_unique_place(&mut group, target.clone());
            out.push(group);
        }
        out
    }

    fn union_group(&mut self, group: &[Place]) {
        let mut merged = group.to_vec();
        let mut retained = Vec::new();
        for existing in self.groups.drain(..) {
            if groups_overlap(&existing, &merged) {
                push_unique_places(&mut merged, &existing);
            } else {
                retained.push(existing);
            }
        }
        if !merged.is_empty() {
            retained.push(merged);
        }
        self.groups = retained;
    }
}

fn push_unique_projection_alias(
    target: &mut Vec<(Vec<PlaceProjection>, TypeId)>,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
) {
    if !target
        .iter()
        .any(|(existing_suffix, existing_ty)| existing_suffix == &suffix && *existing_ty == ty)
    {
        target.push((suffix, ty));
    }
}
