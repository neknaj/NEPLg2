extern crate alloc;

use alloc::vec::Vec;

use super::model::{Place, PlaceRoot};
use super::place_utils::{
    place_suffix_after_prefix, place_with_suffix, push_unique_place, replace_place_prefix,
};

#[derive(Debug, Clone, Default)]
pub(super) struct RawCellAddressAliases {
    groups: Vec<Vec<Place>>,
}

impl RawCellAddressAliases {
    pub(super) fn mark(&mut self, place: &Place) {
        self.clear(place);
        self.union_group(core::slice::from_ref(place));
    }

    pub(super) fn copy_alias_or_seed(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let groups = self.groups_with_replaced_prefix_or_singleton(source, target);
        self.clear(target);
        for group in groups {
            self.union_group(&group);
        }
    }

    pub(super) fn aliases(&self, left: &Place, right: &Place) -> bool {
        self.alias_groups_for(left)
            .iter()
            .any(|group| group.iter().any(|place| place == right))
            || self
                .alias_groups_for(right)
                .iter()
                .any(|group| group.iter().any(|place| place == left))
    }

    pub(super) fn canonicalize(&self, place: &Place) -> Place {
        for group in &self.groups {
            for alias in group {
                if let Some(suffix) = place_suffix_after_prefix(place, alias) {
                    return place_with_suffix(&group[0], &suffix, place.ty);
                }
            }
        }
        place.clone()
    }

    pub(super) fn contains_exact(&self, place: &Place) -> bool {
        self.groups
            .iter()
            .any(|group| group.iter().any(|alias| alias == place))
    }

    pub(super) fn clear(&mut self, place: &Place) {
        for group in &mut self.groups {
            group.retain(|existing| place_suffix_after_prefix(existing, place).is_none());
        }
        self.groups.retain(|group| !group.is_empty());
    }

    pub(super) fn merge_paths(paths: &[RawCellAddressAliases]) -> Self {
        let mut out = RawCellAddressAliases::default();
        for path in paths {
            for group in &path.groups {
                out.union_group(group);
            }
        }
        out
    }

    fn alias_groups_for(&self, place: &Place) -> Vec<Vec<Place>> {
        let mut out = Vec::new();
        for group in &self.groups {
            let mut mapped = Vec::new();
            for alias in group {
                if let Some(suffix) = place_suffix_after_prefix(place, alias) {
                    for group_alias in group {
                        push_unique_place(
                            &mut mapped,
                            &place_with_suffix(group_alias, &suffix, place.ty),
                        );
                    }
                    break;
                }
            }
            if !mapped.is_empty() {
                out.push(mapped);
            }
        }
        out
    }

    fn groups_with_replaced_prefix_or_singleton(
        &self,
        source: &Place,
        target: &Place,
    ) -> Vec<Vec<Place>> {
        let mut out = Vec::new();
        for group in &self.groups {
            let mut mapped = Vec::new();
            for place in group {
                if let Some(replacement) = replace_place_prefix(place, source, target) {
                    push_unique_place(&mut mapped, &replacement);
                }
            }
            if mapped.is_empty() {
                for alias in group {
                    if let Some(suffix) = place_suffix_after_prefix(source, alias) {
                        for group_alias in group {
                            push_unique_place(
                                &mut mapped,
                                &place_with_suffix(group_alias, &suffix, source.ty),
                            );
                        }
                        push_unique_place(&mut mapped, target);
                        break;
                    }
                }
            }
            if mapped.is_empty() {
                continue;
            }

            let mut merged: Vec<Place> = group
                .iter()
                .filter(|place| place_suffix_after_prefix(place, target).is_none())
                .cloned()
                .collect();
            for place in &mapped {
                push_unique_place(&mut merged, place);
            }
            out.push(merged);
        }

        if out.is_empty() {
            let mut group = Vec::new();
            push_unique_place(&mut group, source);
            push_unique_place(&mut group, target);
            out.push(group);
        }
        out
    }

    fn union_group(&mut self, group: &[Place]) {
        let mut merged = group.to_vec();
        let mut retained = Vec::new();
        for existing in self.groups.drain(..) {
            if groups_overlap(&existing, &merged) {
                for place in &existing {
                    push_unique_place(&mut merged, place);
                }
            } else {
                retained.push(existing);
            }
        }
        if !merged.is_empty() {
            prefer_stable_canonical(&mut merged);
            retained.push(merged);
        }
        self.groups = retained;
    }
}

fn groups_overlap(left: &[Place], right: &[Place]) -> bool {
    left.iter().any(|place| right.contains(place))
}

fn prefer_stable_canonical(group: &mut Vec<Place>) {
    let Some((index, _)) = group.iter().enumerate().min_by_key(|(_, place)| {
        (
            canonical_place_projection_rank(place),
            canonical_place_rank(place),
            place.projections.len(),
        )
    }) else {
        return;
    };
    if index != 0 {
        let place = group.remove(index);
        group.insert(0, place);
    }
}

fn canonical_place_projection_rank(place: &Place) -> u8 {
    if place
        .projections
        .iter()
        .any(|projection| matches!(projection, super::model::PlaceProjection::StorageOffset(_)))
    {
        0
    } else {
        1
    }
}

fn canonical_place_rank(place: &Place) -> u8 {
    match place.root {
        PlaceRoot::Local(_) => 0,
        PlaceRoot::Return => 1,
        PlaceRoot::Storage(_) => 2,
        PlaceRoot::Temporary(_) => 3,
        PlaceRoot::Unknown => 4,
    }
}
