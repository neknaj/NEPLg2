use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::span::Span;
use crate::types::TypeId;

use super::effect_place_prefix::{
    groups_overlap, place_has_prefix, place_suffix_after_prefix, push_unique_place,
    push_unique_places, replace_place_prefix,
};
use super::effect_pointer_alias::RawPointerAliasTable;
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
pub(super) struct RawIdentityTable {
    groups: Vec<RawIdentityGroup>,
}

#[derive(Debug, Clone)]
struct RawIdentityGroup {
    places: Vec<Place>,
    origins: Vec<RawIdentityOrigin>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct RawIdentityOrigin {
    pub(super) operation: RawMemoryOp,
    pub(super) span: Span,
}

impl RawIdentityOrigin {
    pub(super) fn new(operation: RawMemoryOp, span: Span) -> Self {
        Self { operation, span }
    }
}

impl Ord for RawIdentityOrigin {
    fn cmp(&self, other: &Self) -> Ordering {
        self.operation
            .cmp(&other.operation)
            .then_with(|| self.span.file_id.0.cmp(&other.span.file_id.0))
            .then_with(|| self.span.start.cmp(&other.span.start))
            .then_with(|| self.span.end.cmp(&other.span.end))
    }
}

impl PartialOrd for RawIdentityOrigin {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl RawIdentityTable {
    pub(super) fn origins(&self, place: &Place) -> Vec<RawIdentityOrigin> {
        let mut origins = Vec::new();
        for group in &self.groups {
            if group.places.iter().any(|existing| existing == place) {
                push_unique_origins(&mut origins, &group.origins);
            }
        }
        origins
    }

    pub(super) fn operations(&self, place: &Place) -> Vec<RawMemoryOp> {
        let mut operations = Vec::new();
        for group in &self.groups {
            if group.places.iter().any(|existing| existing == place) {
                push_unique_operations_from_origins(&mut operations, &group.origins);
            }
        }
        operations
    }

    pub(super) fn projection_origins_under(
        &self,
        place: &Place,
    ) -> Vec<(Vec<PlaceProjection>, TypeId, Vec<RawIdentityOrigin>)> {
        let mut out: Vec<(Vec<PlaceProjection>, TypeId, Vec<RawIdentityOrigin>)> = Vec::new();
        for group in &self.groups {
            for existing in &group.places {
                let Some(suffix) = place_suffix_after_prefix(existing, place) else {
                    continue;
                };
                if let Some((_, _, origins)) = out.iter_mut().find(|(existing_suffix, ty, _)| {
                    existing_suffix == &suffix && *ty == existing.ty
                }) {
                    push_unique_origins(origins, &group.origins);
                } else {
                    let mut origins = Vec::new();
                    push_unique_origins(&mut origins, &group.origins);
                    out.push((suffix, existing.ty, origins));
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

    pub(super) fn mark(&mut self, place: &Place, operation: RawMemoryOp, span: Span) {
        self.mark_many(
            place,
            core::slice::from_ref(&RawIdentityOrigin::new(operation, span)),
        );
    }

    pub(super) fn mark_many(&mut self, place: &Place, origins: &[RawIdentityOrigin]) {
        self.union_group(core::slice::from_ref(place), origins);
    }

    pub(super) fn copy_identity(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let groups = self.groups_with_replaced_prefix(source, target, true);
        self.clear(target);
        for group in groups {
            self.union_group(&group.places, &group.origins);
        }
    }

    pub(super) fn merge_identity(&mut self, source: &Place, target: &Place) {
        for group in self.groups_with_replaced_prefix(source, target, false) {
            self.union_group(&group.places, &group.origins);
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
                out.union_group(&group.places, &group.origins);
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
            for place in &group.places {
                if let Some(replacement) = replace_place_prefix(place, source, target) {
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
            out.push(RawIdentityGroup {
                places: merged,
                origins: group.origins.clone(),
            });
        }
        out
    }

    fn union_group(&mut self, places: &[Place], origins: &[RawIdentityOrigin]) {
        let mut merged_places = places.to_vec();
        let mut merged_origins = origins.to_vec();
        let mut retained = Vec::new();
        for existing in self.groups.drain(..) {
            if groups_overlap(&existing.places, &merged_places) {
                push_unique_places(&mut merged_places, &existing.places);
                push_unique_origins(&mut merged_origins, &existing.origins);
            } else {
                retained.push(existing);
            }
        }
        if !merged_places.is_empty() && !merged_origins.is_empty() {
            retained.push(RawIdentityGroup {
                places: merged_places,
                origins: merged_origins,
            });
        }
        self.groups = retained;
    }
}

pub(super) fn raw_memory_op_produces_identity(operation: &RawMemoryOp) -> bool {
    matches!(operation, RawMemoryOp::Alloc | RawMemoryOp::Realloc)
}

pub(super) fn push_unique_origins(
    target: &mut Vec<RawIdentityOrigin>,
    source: &[RawIdentityOrigin],
) {
    for origin in source {
        if !target.contains(origin) {
            target.push(*origin);
        }
    }
    target.sort();
}

fn push_unique_operations_from_origins(
    target: &mut Vec<RawMemoryOp>,
    source: &[RawIdentityOrigin],
) {
    for origin in source {
        if !target.contains(&origin.operation) {
            target.push(origin.operation);
        }
    }
    target.sort();
}
