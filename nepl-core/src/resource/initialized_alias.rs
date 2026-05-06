extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::initialized_alias_origin::RawValueOrigins;
use super::initialized_alias_rank::{
    owner_alias_place_has_raw_projection, owner_cell_alias_rank, prefer_stable_canonical,
};
use super::initialized_alias_scalar::I32AliasFacts;
use super::model::{I32ValueCondition, Place};
use super::place_utils::{
    place_suffix_after_prefix, place_with_suffix, push_unique_place, replace_place_prefix,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ProjectedRawCellAddressAlias {
    pub(super) left_projection: Vec<super::model::PlaceProjection>,
    pub(super) left_ty: TypeId,
    pub(super) right_projection: Vec<super::model::PlaceProjection>,
    pub(super) right_ty: TypeId,
}

#[derive(Debug, Clone, Default)]
pub(super) struct RawCellAddressAliases {
    groups: Vec<Vec<Place>>,
    marked: Vec<Place>,
    value_origins: RawValueOrigins,
    i32_facts: I32AliasFacts,
}

impl RawCellAddressAliases {
    pub(super) fn mark(&mut self, place: &Place) {
        self.clear(place);
        push_unique_place(&mut self.marked, place);
        self.union_group(core::slice::from_ref(place));
    }

    /// Copies an existing raw-address alias group and scalar facts without treating ordinary
    /// i32 value copies as raw-address aliases.
    pub(super) fn copy_alias_if_tracked(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let fact_copies = self.i32_facts.facts_with_replaced_prefix(source, target);
        let groups = self.groups_with_replaced_prefix(source, target);
        self.clear(target);
        for group in groups {
            self.union_group(&group);
        }
        self.i32_facts.extend(fact_copies);
        self.value_origins.copy_stable_origin(source, target);
    }

    /// Records an explicit raw-address relation while preserving any already-tracked aliases and
    /// scalar facts reachable from the source.
    pub(super) fn copy_explicit_raw_address_alias(&mut self, source: &Place, target: &Place) {
        if source == target {
            self.mark(target);
            return;
        }
        self.copy_alias_if_tracked(source, target);
        let mut group = Vec::new();
        push_unique_place(&mut group, source);
        push_unique_place(&mut group, target);
        push_unique_place(&mut group, &self.value_origins.origin_for(source));
        push_unique_place(&mut group, &self.value_origins.origin_for(target));
        self.union_group(&group);
    }

    pub(super) fn move_owner_aliases(&mut self, source: &Place, target: &Place) {
        if source == target {
            push_unique_place(&mut self.marked, target);
            self.union_group(core::slice::from_ref(target));
            return;
        }
        let moved_marks = self
            .marked
            .iter()
            .filter(|marked| owner_alias_place_has_raw_projection(marked, source))
            .filter_map(|marked| replace_place_prefix(marked, source, target))
            .collect::<Vec<_>>();
        let moved_facts = self.i32_facts.facts_with_replaced_prefix(source, target);
        let groups = self
            .groups_with_replaced_prefix(source, target)
            .into_iter()
            .filter(|group| {
                group
                    .iter()
                    .any(|place| owner_alias_place_has_raw_projection(place, target))
            })
            .collect::<Vec<_>>();
        self.clear(target);
        for group in groups {
            self.union_group(&group);
        }
        self.clear(source);
        push_unique_place(&mut self.marked, target);
        self.union_group(core::slice::from_ref(target));
        for moved in moved_marks {
            push_unique_place(&mut self.marked, &moved);
            self.union_group(core::slice::from_ref(&moved));
        }
        self.i32_facts.extend(moved_facts);
    }

    pub(super) fn canonicalize(&self, place: &Place) -> Place {
        if let Some(canonical) = self.canonicalize_group_member(place) {
            return canonical;
        }
        let origin = self.value_origins.origin_for(place);
        if origin != *place {
            if let Some(canonical) = self.canonicalize_group_member(&origin) {
                return canonical;
            }
            return origin;
        }
        place.clone()
    }

    pub(super) fn canonicalize_owner_cell_address(&self, place: &Place) -> Place {
        self.aliases_for(place)
            .into_iter()
            .min_by_key(owner_cell_alias_rank)
            .unwrap_or_else(|| place.clone())
    }

    pub(super) fn prefer_canonical(&mut self, place: &Place) {
        for group in &mut self.groups {
            let Some(index) = group.iter().position(|alias| alias == place) else {
                continue;
            };
            if index != 0 {
                let alias = group.remove(index);
                group.insert(0, alias);
            }
        }
    }

    pub(super) fn contains_exact(&self, place: &Place) -> bool {
        self.groups
            .iter()
            .any(|group| group.iter().any(|alias| alias == place))
    }

    pub(super) fn value_is_known_raw_address(&self, place: &Place) -> bool {
        self.contains_exact(place) || self.aliases_for(place).len() > 1
    }

    pub(super) fn set_i32_value(&mut self, place: &Place, value: i32) {
        self.i32_facts.set_value(place, value);
    }

    pub(super) fn add_i32_condition(&mut self, place: &Place, condition: I32ValueCondition) {
        self.i32_facts.add_condition(place, condition);
    }

    pub(super) fn i32_value(&self, place: &Place) -> Option<i32> {
        self.i32_facts.value_for_aliases(&self.aliases_for(place))
    }

    pub(super) fn i32_condition_truth(
        &self,
        place: &Place,
        condition: I32ValueCondition,
    ) -> Option<bool> {
        if let Some(value) = self.i32_value(place) {
            return Some(condition.holds(value));
        }
        self.i32_facts
            .condition_truth_for_aliases(&self.aliases_for(place), condition)
    }

    pub(super) fn contains_marked_alias(&self, place: &Place) -> bool {
        self.aliases_for(place)
            .iter()
            .any(|alias| self.marked.iter().any(|marked| marked == alias))
    }

    pub(super) fn aliases_for(&self, place: &Place) -> Vec<Place> {
        let mut out = Vec::new();
        for group in self.alias_groups_for(place) {
            for alias in group {
                push_unique_place(&mut out, &alias);
            }
        }
        if out.is_empty() {
            push_unique_place(&mut out, place);
        }
        out
    }

    pub(super) fn tracked_places(&self) -> Vec<Place> {
        let mut out = Vec::new();
        for group in &self.groups {
            for place in group {
                push_unique_place(&mut out, place);
            }
        }
        for place in &self.marked {
            push_unique_place(&mut out, place);
        }
        out
    }

    pub(super) fn prefix_aliases_for(&self, place: &Place) -> Vec<Place> {
        let mut out = Vec::new();
        for group in &self.groups {
            for group_place in group {
                let Some(suffix) = place_suffix_after_prefix(group_place, place) else {
                    continue;
                };
                for alias in group {
                    if let Some(prefix) = place_without_suffix(alias, &suffix, place.ty) {
                        push_unique_place(&mut out, &prefix);
                    }
                }
            }
        }
        out
    }

    pub(super) fn projected_aliases_between(
        &self,
        left_base: &Place,
        right_base: &Place,
    ) -> Vec<ProjectedRawCellAddressAlias> {
        let mut out = Vec::new();
        for group in &self.groups {
            let left_places = group
                .iter()
                .filter_map(|place| {
                    place_suffix_after_prefix(place, left_base).map(|suffix| (suffix, place.ty))
                })
                .collect::<Vec<_>>();
            if left_places.is_empty() {
                continue;
            }
            let right_places = group
                .iter()
                .filter_map(|place| {
                    place_suffix_after_prefix(place, right_base).map(|suffix| (suffix, place.ty))
                })
                .collect::<Vec<_>>();
            for (left_projection, left_ty) in &left_places {
                for (right_projection, right_ty) in &right_places {
                    push_unique_projected_alias(
                        &mut out,
                        ProjectedRawCellAddressAlias {
                            left_projection: left_projection.clone(),
                            left_ty: *left_ty,
                            right_projection: right_projection.clone(),
                            right_ty: *right_ty,
                        },
                    );
                }
            }
        }
        out
    }

    pub(super) fn clear(&mut self, place: &Place) {
        for group in &mut self.groups {
            group.retain(|existing| place_suffix_after_prefix(existing, place).is_none());
        }
        self.groups.retain(|group| !group.is_empty());
        self.marked
            .retain(|existing| place_suffix_after_prefix(existing, place).is_none());
        self.value_origins.clear_prefix(place);
        self.i32_facts.clear_prefix(place);
    }

    pub(super) fn merge_paths(paths: &[RawCellAddressAliases]) -> Self {
        let mut out = RawCellAddressAliases::default();
        for path in paths {
            for group in &path.groups {
                out.union_group(group);
            }
            for marked in &path.marked {
                push_unique_place(&mut out.marked, marked);
            }
        }
        out.value_origins =
            RawValueOrigins::merge_paths(paths.iter().map(|path| &path.value_origins));
        out.i32_facts = I32AliasFacts::merge_paths(paths.iter().map(|path| &path.i32_facts));
        out
    }

    fn canonicalize_group_member(&self, place: &Place) -> Option<Place> {
        for group in &self.groups {
            for alias in group {
                if let Some(suffix) = place_suffix_after_prefix(place, alias) {
                    return Some(place_with_suffix(&group[0], &suffix, place.ty));
                }
            }
        }
        None
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

    fn groups_with_replaced_prefix(&self, source: &Place, target: &Place) -> Vec<Vec<Place>> {
        let mut out = Vec::new();
        for group in &self.groups {
            let mut mapped = Vec::new();
            let mut replaces_group_member = false;
            for place in group {
                if let Some(replacement) = replace_place_prefix(place, source, target) {
                    push_unique_place(&mut mapped, &replacement);
                    replaces_group_member = true;
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

            if !replaces_group_member {
                out.push(mapped);
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

        out
    }

    fn union_group(&mut self, group: &[Place]) {
        let mut merged = Vec::new();
        let mut retained = Vec::new();
        for existing in self.groups.drain(..) {
            if groups_overlap(&existing, group) {
                for place in &existing {
                    push_unique_place(&mut merged, place);
                }
            } else {
                retained.push(existing);
            }
        }
        for place in group {
            push_unique_place(&mut merged, place);
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

fn push_unique_projected_alias(
    aliases: &mut Vec<ProjectedRawCellAddressAlias>,
    alias: ProjectedRawCellAddressAlias,
) {
    if !aliases.iter().any(|existing| existing == &alias) {
        aliases.push(alias);
    }
}

fn place_without_suffix(
    place: &Place,
    suffix: &[super::model::PlaceProjection],
    ty: TypeId,
) -> Option<Place> {
    if suffix.len() > place.projections.len() {
        return None;
    }
    let prefix_len = place.projections.len() - suffix.len();
    if place.projections[prefix_len..] != *suffix {
        return None;
    }
    let mut out = place.clone();
    out.projections.truncate(prefix_len);
    out.ty = ty;
    Some(out)
}
