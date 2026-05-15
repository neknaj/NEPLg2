use alloc::vec;
use alloc::vec::Vec;

use super::effect_raw_memory_identity::RawMemoryIdentityTable;
use super::model::{AggregateKind, Place, RawMemoryOp};
use super::place_utils::construct_aggregate_field_place;

pub(super) fn construct_raw_identity_fields(
    identities: &mut RawIdentityTable,
    output: &Place,
    kind: &AggregateKind,
    inputs: &[Place],
) {
    for (index, input) in inputs.iter().enumerate() {
        let field = construct_aggregate_field_place(output, kind, index, input);
        identities.merge_identity(input, &field);
    }
}

pub(super) fn construct_pointer_alias_fields(
    pointer_aliases: &mut RawPointerAliasTable,
    raw_memory_identities: &mut RawMemoryIdentityTable,
    output: &Place,
    kind: &AggregateKind,
    inputs: &[Place],
) {
    for (index, input) in inputs.iter().enumerate() {
        let field = construct_aggregate_field_place(output, kind, index, input);
        copy_pointer_alias(pointer_aliases, raw_memory_identities, input, &field);
    }
}

pub(super) fn copy_pointer_alias(
    pointer_aliases: &mut RawPointerAliasTable,
    raw_memory_identities: &mut RawMemoryIdentityTable,
    source: &Place,
    target: &Place,
) {
    raw_memory_identities.remove_place(target);
    pointer_aliases.copy_alias(source, target);
}

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

    pub(super) fn aliases(&self, left: &Place, right: &Place) -> bool {
        let left_group = self.group_for_or_singleton(left);
        let right_group = self.group_for_or_singleton(right);
        groups_overlap(&left_group, &right_group)
    }

    fn copy_alias(&mut self, source: &Place, target: &Place) {
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

#[derive(Debug, Clone, Default)]
pub(super) struct RawIdentityTable {
    groups: Vec<RawIdentityGroup>,
}

#[derive(Debug, Clone)]
struct RawIdentityGroup {
    places: Vec<Place>,
    operations: Vec<RawMemoryOp>,
}

impl RawIdentityTable {
    pub(super) fn contains(&self, place: &Place) -> bool {
        self.groups
            .iter()
            .any(|group| group.places.iter().any(|existing| existing == place))
    }

    pub(super) fn operations(&self, place: &Place) -> Vec<RawMemoryOp> {
        let mut operations = Vec::new();
        for group in &self.groups {
            if group.places.iter().any(|existing| existing == place) {
                push_unique_operations(&mut operations, &group.operations);
            }
        }
        operations
    }

    pub(super) fn mark(&mut self, place: &Place, operation: RawMemoryOp) {
        self.mark_many(place, core::slice::from_ref(&operation));
    }

    pub(super) fn mark_many(&mut self, place: &Place, operations: &[RawMemoryOp]) {
        self.union_group(core::slice::from_ref(place), operations);
    }

    pub(super) fn copy_identity(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let groups = self.groups_with_replaced_prefix(source, target, true);
        self.clear(target);
        for group in groups {
            self.union_group(&group.places, &group.operations);
        }
    }

    pub(super) fn merge_identity(&mut self, source: &Place, target: &Place) {
        for group in self.groups_with_replaced_prefix(source, target, false) {
            self.union_group(&group.places, &group.operations);
        }
    }

    pub(super) fn clear(&mut self, place: &Place) {
        for group in &mut self.groups {
            group
                .places
                .retain(|existing| !place_has_prefix(existing, place));
        }
        self.groups.retain(|group| !group.places.is_empty());
    }

    pub(super) fn merge_paths(paths: &[RawIdentityTable]) -> Self {
        let mut out = RawIdentityTable::default();
        for path in paths {
            for group in &path.groups {
                out.union_group(&group.places, &group.operations);
            }
        }
        out
    }

    fn groups_with_replaced_prefix(
        &self,
        source: &Place,
        target: &Place,
        drop_target_prefix: bool,
    ) -> Vec<RawIdentityGroup> {
        let mut out = Vec::new();
        for group in &self.groups {
            let mut mapped = Vec::new();
            let mut mapped_descendant = false;
            for place in &group.places {
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
                    .places
                    .iter()
                    .filter(|place| !place_has_prefix(place, target))
                    .cloned()
                    .collect()
            } else {
                group.places.clone()
            };
            push_unique_places(&mut merged, &mapped);
            if mapped_descendant {
                push_unique_place(&mut merged, target.clone());
            }
            out.push(RawIdentityGroup {
                places: merged,
                operations: group.operations.clone(),
            });
        }
        out
    }

    fn union_group(&mut self, places: &[Place], operations: &[RawMemoryOp]) {
        let mut merged_places = places.to_vec();
        let mut merged_operations = operations.to_vec();
        let mut retained = Vec::new();
        for existing in self.groups.drain(..) {
            if groups_overlap(&existing.places, &merged_places) {
                push_unique_places(&mut merged_places, &existing.places);
                push_unique_operations(&mut merged_operations, &existing.operations);
            } else {
                retained.push(existing);
            }
        }
        if !merged_places.is_empty() && !merged_operations.is_empty() {
            retained.push(RawIdentityGroup {
                places: merged_places,
                operations: merged_operations,
            });
        }
        self.groups = retained;
    }
}

pub(super) fn raw_memory_op_produces_identity(operation: &RawMemoryOp) -> bool {
    matches!(operation, RawMemoryOp::Alloc | RawMemoryOp::Realloc)
}

fn replace_place_prefix(place: &Place, prefix: &Place, replacement: &Place) -> Option<Place> {
    if !place_has_prefix(place, prefix) {
        return None;
    }
    let suffix = place.projections[prefix.projections.len()..].to_vec();
    let mut out = replacement.clone();
    let suffix_is_empty = suffix.is_empty();
    out.projections.extend(suffix);
    if !suffix_is_empty {
        out.ty = place.ty;
    }
    Some(out)
}

fn place_has_prefix(place: &Place, prefix: &Place) -> bool {
    place.root == prefix.root
        && place.projections.len() >= prefix.projections.len()
        && place
            .projections
            .iter()
            .zip(&prefix.projections)
            .all(|(projection, prefix_projection)| projection == prefix_projection)
}

fn groups_overlap(left: &[Place], right: &[Place]) -> bool {
    left.iter().any(|place| right.contains(place))
}

fn push_unique_place(target: &mut Vec<Place>, place: Place) {
    if !target.contains(&place) {
        target.push(place);
    }
}

fn push_unique_places(target: &mut Vec<Place>, source: &[Place]) {
    for place in source {
        if !target.contains(place) {
            target.push(place.clone());
        }
    }
}

fn push_unique_operations(target: &mut Vec<RawMemoryOp>, source: &[RawMemoryOp]) {
    for operation in source {
        if !target.contains(operation) {
            target.push(*operation);
        }
    }
    target.sort();
}
