extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::model::{I32ValueCondition, Place, PlaceProjection, PlaceRoot};
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
    i32_values: Vec<I32ValueFact>,
    i32_conditions: Vec<I32ConditionFact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct I32ValueFact {
    place: Place,
    value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct I32ConditionFact {
    place: Place,
    condition: I32ValueCondition,
}

impl RawCellAddressAliases {
    pub(super) fn mark(&mut self, place: &Place) {
        self.clear(place);
        push_unique_place(&mut self.marked, place);
        self.union_group(core::slice::from_ref(place));
    }

    pub(super) fn copy_alias_or_seed(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let value_copies = self
            .i32_values
            .iter()
            .filter_map(|fact| {
                replace_place_prefix(&fact.place, source, target).map(|place| I32ValueFact {
                    place,
                    value: fact.value,
                })
            })
            .collect::<Vec<_>>();
        let condition_copies = self
            .i32_conditions
            .iter()
            .filter_map(|fact| {
                replace_place_prefix(&fact.place, source, target).map(|place| I32ConditionFact {
                    place,
                    condition: fact.condition,
                })
            })
            .collect::<Vec<_>>();
        let groups = self.groups_with_replaced_prefix_or_singleton(source, target);
        self.clear(target);
        for group in groups {
            self.union_group(&group);
        }
        self.union_group(&[source.clone(), target.clone()]);
        for fact in value_copies {
            self.push_i32_value_fact(fact);
        }
        for fact in condition_copies {
            self.push_i32_condition_fact(fact);
        }
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
        let moved_values = self
            .i32_values
            .iter()
            .filter_map(|fact| {
                replace_place_prefix(&fact.place, source, target).map(|place| I32ValueFact {
                    place,
                    value: fact.value,
                })
            })
            .collect::<Vec<_>>();
        let moved_conditions = self
            .i32_conditions
            .iter()
            .filter_map(|fact| {
                replace_place_prefix(&fact.place, source, target).map(|place| I32ConditionFact {
                    place,
                    condition: fact.condition,
                })
            })
            .collect::<Vec<_>>();
        let groups = self
            .groups_with_replaced_prefix_or_singleton(source, target)
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
        for moved in moved_marks {
            push_unique_place(&mut self.marked, &moved);
        }
        for fact in moved_values {
            self.push_i32_value_fact(fact);
        }
        for fact in moved_conditions {
            self.push_i32_condition_fact(fact);
        }
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
        self.i32_values.retain(|fact| fact.place != *place);
        self.i32_values.push(I32ValueFact {
            place: place.clone(),
            value,
        });
    }

    pub(super) fn add_i32_condition(&mut self, place: &Place, condition: I32ValueCondition) {
        self.push_i32_condition_fact(I32ConditionFact {
            place: place.clone(),
            condition,
        });
    }

    pub(super) fn i32_value(&self, place: &Place) -> Option<i32> {
        let mut value = None;
        for alias in self.aliases_for(place) {
            for fact in &self.i32_values {
                if fact.place != alias {
                    continue;
                }
                match value {
                    Some(existing) if existing != fact.value => return None,
                    Some(_) => {}
                    None => value = Some(fact.value),
                }
            }
        }
        value
    }

    pub(super) fn i32_condition_truth(
        &self,
        place: &Place,
        condition: I32ValueCondition,
    ) -> Option<bool> {
        if let Some(value) = self.i32_value(place) {
            return Some(condition.holds(value));
        }
        let mut truth = None;
        for alias in self.aliases_for(place) {
            for fact in &self.i32_conditions {
                if fact.place != alias {
                    continue;
                }
                let Some(fact_truth) = condition_implication(fact.condition, condition) else {
                    continue;
                };
                match truth {
                    Some(existing) if existing != fact_truth => return None,
                    Some(_) => {}
                    None => truth = Some(fact_truth),
                }
            }
        }
        truth
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
        self.i32_values
            .retain(|fact| place_suffix_after_prefix(&fact.place, place).is_none());
        self.i32_conditions
            .retain(|fact| place_suffix_after_prefix(&fact.place, place).is_none());
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
        if let Some(first) = paths.first() {
            for fact in &first.i32_values {
                if paths
                    .iter()
                    .skip(1)
                    .all(|path| path.i32_values.iter().any(|existing| existing == fact))
                {
                    out.push_i32_value_fact(fact.clone());
                }
            }
            for fact in &first.i32_conditions {
                if paths
                    .iter()
                    .skip(1)
                    .all(|path| path.i32_conditions.iter().any(|existing| existing == fact))
                {
                    out.push_i32_condition_fact(fact.clone());
                }
            }
        }
        out
    }

    fn push_i32_value_fact(&mut self, fact: I32ValueFact) {
        self.i32_values
            .retain(|existing| existing.place != fact.place);
        self.i32_values.push(fact);
    }

    fn push_i32_condition_fact(&mut self, fact: I32ConditionFact) {
        if self.i32_conditions.iter().any(|existing| existing == &fact) {
            return;
        }
        self.i32_conditions.push(fact);
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

fn owner_cell_alias_rank(place: &Place) -> (u8, u8, usize) {
    (
        owner_cell_projection_rank(place),
        canonical_place_rank(place),
        place.projections.len(),
    )
}

fn owner_cell_projection_rank(place: &Place) -> u8 {
    if place.projections.iter().any(|projection| {
        matches!(
            projection,
            super::model::PlaceProjection::Field { .. }
                | super::model::PlaceProjection::TupleField { .. }
                | super::model::PlaceProjection::EnumPayload { .. }
        )
    }) {
        0
    } else if place
        .projections
        .iter()
        .any(|projection| matches!(projection, super::model::PlaceProjection::StorageOffset(_)))
    {
        1
    } else {
        2
    }
}

fn owner_alias_place_has_raw_projection(place: &Place, base: &Place) -> bool {
    place.projections.iter().any(|projection| {
        matches!(
            projection,
            PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
        )
    }) || place_suffix_after_prefix(place, base).is_some_and(|suffix| {
        suffix.iter().any(|projection| {
            matches!(
                projection,
                PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
            )
        })
    })
}

fn condition_implication(known: I32ValueCondition, query: I32ValueCondition) -> Option<bool> {
    use I32ValueCondition::{EqZero, NeZero, Negative, NonNegative, NonPositive, Positive};
    match (known, query) {
        (left, right) if left == right => Some(true),
        (EqZero, NeZero | Positive | Negative) => Some(false),
        (EqZero, NonPositive | NonNegative) => Some(true),
        (NeZero, EqZero) => Some(false),
        (Positive, EqZero | Negative | NonPositive) => Some(false),
        (Positive, NeZero | NonNegative) => Some(true),
        (NonPositive, Positive) => Some(false),
        (Negative, EqZero | Positive | NonNegative) => Some(false),
        (Negative, NeZero | NonPositive) => Some(true),
        (NonNegative, Negative) => Some(false),
        _ => None,
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
