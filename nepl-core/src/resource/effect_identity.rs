use alloc::vec;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::effect_raw_memory_identity::RawMemoryIdentityTable;
use super::model::{AggregateKind, Place, PlaceProjection, RawMemoryOp};
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
    pub(super) fn operations(&self, place: &Place) -> Vec<RawMemoryOp> {
        let mut operations = Vec::new();
        for group in &self.groups {
            if group.places.iter().any(|existing| existing == place) {
                push_unique_operations(&mut operations, &group.operations);
            }
        }
        operations
    }

    pub(super) fn projection_operations_under(
        &self,
        place: &Place,
    ) -> Vec<(Vec<PlaceProjection>, TypeId, Vec<RawMemoryOp>)> {
        let mut out: Vec<(Vec<PlaceProjection>, TypeId, Vec<RawMemoryOp>)> = Vec::new();
        for group in &self.groups {
            for existing in &group.places {
                let Some(suffix) = place_suffix_after_prefix(existing, place) else {
                    continue;
                };
                if let Some((_, _, operations)) = out.iter_mut().find(|(existing_suffix, ty, _)| {
                    existing_suffix == &suffix && *ty == existing.ty
                }) {
                    push_unique_operations(operations, &group.operations);
                } else {
                    let mut operations = Vec::new();
                    push_unique_operations(&mut operations, &group.operations);
                    out.push((suffix, existing.ty, operations));
                }
            }
        }
        out.sort_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| left.1.cmp(&right.1))
                .then_with(|| left.2.cmp(&right.2))
        });
        out
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

fn place_suffix_after_prefix(place: &Place, prefix: &Place) -> Option<Vec<PlaceProjection>> {
    if !place_has_prefix(place, prefix) {
        return None;
    }
    Some(place.projections[prefix.projections.len()..].to_vec())
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
