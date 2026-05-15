use alloc::vec::Vec;

use super::effect_identity::RawPointerAliasTable;
use super::model::{Place, RawMemoryOp};

#[derive(Debug, Clone, Default)]
pub(super) struct RawMemoryIdentityTable {
    pointer_groups: Vec<RawMemoryIdentityGroup>,
}

#[derive(Debug, Clone)]
struct RawMemoryIdentityGroup {
    places: Vec<Place>,
    operations: Vec<RawMemoryOp>,
}

impl RawMemoryIdentityTable {
    pub(super) fn operations(
        &self,
        pointer_aliases: &RawPointerAliasTable,
        place: &Place,
    ) -> Vec<RawMemoryOp> {
        let group = pointer_aliases.group_for_or_singleton(place);
        let mut operations = Vec::new();
        for stored in &self.pointer_groups {
            if groups_overlap(&stored.places, &group) {
                push_unique_operations(&mut operations, &stored.operations);
            }
        }
        operations
    }

    pub(super) fn mark_many(
        &mut self,
        pointer_aliases: &RawPointerAliasTable,
        place: &Place,
        operations: &[RawMemoryOp],
    ) {
        self.union_group(&pointer_aliases.group_for_or_singleton(place), operations);
    }

    pub(super) fn clear(&mut self, pointer_aliases: &RawPointerAliasTable, place: &Place) {
        let group = pointer_aliases.group_for_or_singleton(place);
        self.pointer_groups
            .retain(|stored| !groups_overlap(&stored.places, &group));
    }

    pub(super) fn merge_paths(paths: &[RawMemoryIdentityTable]) -> Self {
        let mut out = RawMemoryIdentityTable::default();
        for path in paths {
            for group in &path.pointer_groups {
                out.union_group(&group.places, &group.operations);
            }
        }
        out
    }

    pub(super) fn remove_place(&mut self, place: &Place) {
        for group in &mut self.pointer_groups {
            group
                .places
                .retain(|existing| !place_has_prefix(existing, place));
        }
        self.pointer_groups.retain(|group| !group.places.is_empty());
    }

    fn union_group(&mut self, places: &[Place], operations: &[RawMemoryOp]) {
        let mut merged_places = places.to_vec();
        let mut merged_operations = operations.to_vec();
        let mut retained = Vec::new();
        for existing in self.pointer_groups.drain(..) {
            if groups_overlap(&existing.places, &merged_places) {
                push_unique_places(&mut merged_places, &existing.places);
                push_unique_operations(&mut merged_operations, &existing.operations);
            } else {
                retained.push(existing);
            }
        }
        if !merged_places.is_empty() && !merged_operations.is_empty() {
            retained.push(RawMemoryIdentityGroup {
                places: merged_places,
                operations: merged_operations,
            });
        }
        self.pointer_groups = retained;
    }
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
