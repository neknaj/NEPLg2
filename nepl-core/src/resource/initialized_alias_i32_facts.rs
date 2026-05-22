extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_relation_op::relation_holds;
use super::model::{I32ValueCondition, Place, PlaceProjection, PlaceRoot, ResourceI32RelationOp};
use super::place_utils::push_unique_place;

const I32_OFFSET_RELATION_DERIVATION_DEPTH: usize = 8;
const I32_OFFSET_RELATION_MAX_STATES: usize = 128;

impl RawCellAddressAliases {
    pub(super) fn canonicalize_scalar(&self, place: &Place) -> Place {
        self.scalar_aliases_for(place)
            .into_iter()
            .filter(|alias| !place_has_raw_address_projection(alias))
            .min_by_key(scalar_alias_rank)
            .unwrap_or_else(|| place.clone())
    }

    pub(super) fn set_i32_value(&mut self, place: &Place, value: i32) {
        let place = self.canonicalize_scalar(place);
        self.i32_facts.set_value(&place, value);
    }

    pub(super) fn add_i32_condition(&mut self, place: &Place, condition: I32ValueCondition) {
        let place = self.canonicalize_scalar(place);
        self.i32_facts.add_condition(&place, condition);
    }

    pub(super) fn add_i32_relation(
        &mut self,
        left: &Place,
        op: ResourceI32RelationOp,
        right: &Place,
    ) {
        let left_aliases = self.scalar_fact_recording_sources(left);
        let right_aliases = self.scalar_fact_recording_sources(right);
        for left in &left_aliases {
            for right in &right_aliases {
                self.i32_relations.add_relation(left, op, right);
            }
        }
    }

    pub(super) fn add_i32_difference(
        &mut self,
        minuend: &Place,
        subtrahend: &Place,
        difference: &Place,
    ) {
        let minuend_aliases = self.scalar_fact_recording_sources(minuend);
        let subtrahend_aliases = self.scalar_fact_recording_sources(subtrahend);
        for minuend in &minuend_aliases {
            for subtrahend in &subtrahend_aliases {
                self.i32_differences
                    .add_difference(minuend, subtrahend, difference);
            }
        }
    }

    pub(super) fn add_i32_scale(&mut self, source: &Place, target: &Place, scale: usize) {
        let mut sources = self.scalar_fact_recording_sources(source);
        sources.retain(|source| source != target);
        self.clear_scalar_facts(target);
        self.i32_scales
            .set_scales_for_target(sources, target, scale);
    }

    pub(super) fn add_i32_offset(&mut self, source: &Place, target: &Place, offset: i64) {
        let mut sources = self.scalar_fact_recording_sources(source);
        sources.retain(|source| source != target);
        self.clear_scalar_facts(target);
        self.i32_offsets
            .set_offsets_for_target(sources, target, offset);
    }

    pub(super) fn i32_value(&self, place: &Place) -> Option<i32> {
        self.direct_i32_value(place)
    }

    fn direct_i32_value(&self, place: &Place) -> Option<i32> {
        if let PlaceRoot::I32Constant(value) = place.root {
            return Some(value);
        }
        self.i32_facts
            .value_for_aliases(&self.scalar_aliases_for(place))
    }

    pub(super) fn i32_relation_truth(
        &self,
        left: &Place,
        op: ResourceI32RelationOp,
        right: &Place,
    ) -> Option<bool> {
        if let (Some(left_value), Some(right_value)) = (
            self.i32_relation_value(left),
            self.i32_relation_value(right),
        ) {
            return Some(relation_holds(left_value, op, right_value));
        }
        self.i32_relations
            .relation_truth_for_aliases(
                &self.scalar_aliases_for(left),
                op,
                &self.scalar_aliases_for(right),
            )
            .or_else(|| self.i32_offset_relation_truth(left, op, right))
    }

    fn i32_relation_value(&self, place: &Place) -> Option<i32> {
        self.direct_i32_value(place)
            .or_else(|| self.i32_value_from_bounded_offsets(place))
    }

    fn i32_value_from_bounded_offsets(&self, place: &Place) -> Option<i32> {
        let aliases = self.scalar_aliases_for(place);
        if !self.i32_offsets.has_offset_for_aliases(&aliases)
            && !self.i32_scales.has_scaled_source_for_aliases(&aliases)
        {
            return None;
        }
        let mut value = None;
        for (reachable, offset) in self.i32_offset_reachable_from(place) {
            let Some(reachable_value) = self.direct_i32_value(&reachable) else {
                continue;
            };
            let Some(candidate) = i64::from(reachable_value)
                .checked_sub(offset)
                .and_then(|value| i32::try_from(value).ok())
            else {
                continue;
            };
            merge_i32_derived_value(&mut value, Some(candidate))?;
        }
        if let Some((source, scale)) = self.i32_scaled_source(place) {
            let candidate = self.direct_i32_value(&source).and_then(|source_value| {
                i64::from(source_value)
                    .checked_mul(i64::try_from(scale).ok()?)
                    .and_then(|value| i32::try_from(value).ok())
            });
            merge_i32_derived_value(&mut value, candidate)?;
        }
        value
    }

    pub(super) fn i32_scaled_source(&self, place: &Place) -> Option<(Place, usize)> {
        let mut out = None;
        for (source, scale) in self
            .i32_scales
            .scaled_sources_for_aliases(&self.scalar_aliases_for(place))
        {
            let candidate = (self.canonicalize_scalar(&source), scale);
            match &out {
                Some(existing) if existing != &candidate => return None,
                Some(_) => {}
                None => out = Some(candidate),
            }
        }
        out
    }

    pub(super) fn i32_scaled_targets(&self, source: &Place, scale: usize) -> Vec<Place> {
        self.i32_scales
            .scaled_targets_for_source_aliases(&self.scalar_aliases_for(source), scale)
            .into_iter()
            .map(|target| self.canonicalize_scalar(&target))
            .collect()
    }

    pub(super) fn i32_offset_targets(&self, source: &Place) -> Vec<(Place, i64)> {
        self.i32_offsets
            .offset_targets_for_source_aliases(&self.scalar_aliases_for(source))
            .into_iter()
            .map(|(target, offset)| (self.canonicalize_scalar(&target), offset))
            .collect()
    }

    pub(super) fn i32_offset_sources(&self, target: &Place) -> Vec<(Place, i64)> {
        self.i32_offsets
            .offset_sources_for_target_aliases(&self.scalar_aliases_for(target))
            .into_iter()
            .map(|(source, offset)| (self.canonicalize_scalar(&source), offset))
            .collect()
    }

    pub(super) fn i32_difference_sources(&self, place: &Place) -> Vec<(Place, Place)> {
        self.i32_differences
            .difference_sources_for_aliases(&self.scalar_aliases_for(place))
            .into_iter()
            .map(|(minuend, subtrahend)| {
                (
                    self.canonicalize_scalar(&minuend),
                    self.canonicalize_scalar(&subtrahend),
                )
            })
            .collect()
    }

    pub(super) fn scalar_aliases_for_value(&self, place: &Place) -> Vec<Place> {
        self.scalar_aliases_for(place)
    }

    fn scalar_fact_recording_sources(&self, place: &Place) -> Vec<Place> {
        let mut sources = Vec::new();
        push_unique_place(&mut sources, &self.canonicalize_scalar(place));
        for alias in self.scalar_aliases_for(place) {
            push_unique_place(&mut sources, &alias);
        }
        sources
    }

    fn i32_offset_relation_truth(
        &self,
        left: &Place,
        op: ResourceI32RelationOp,
        right: &Place,
    ) -> Option<bool> {
        let left_reachable = self.i32_offset_reachable_from(left);
        let right_reachable = self.i32_offset_reachable_from(right);
        let mut truth = None;
        for (left_place, left_offset) in &left_reachable {
            for (right_place, right_offset) in &right_reachable {
                if left_place != right_place {
                    continue;
                }
                let Some(left_value) = left_offset.checked_neg() else {
                    continue;
                };
                let Some(right_value) = right_offset.checked_neg() else {
                    continue;
                };
                merge_i32_offset_relation_truth(
                    &mut truth,
                    relation_holds_i64(left_value, op, right_value),
                )?;
            }
        }
        truth
    }

    fn i32_offset_reachable_from(&self, start: &Place) -> Vec<(Place, i64)> {
        let mut out = Vec::new();
        let mut queue = Vec::new();
        for alias in self.scalar_aliases_for(start) {
            push_i32_offset_reachable_state(
                &mut out,
                &mut queue,
                self.canonicalize_scalar(&alias),
                0,
                0,
            );
        }
        push_i32_offset_reachable_state(
            &mut out,
            &mut queue,
            self.canonicalize_scalar(start),
            0,
            0,
        );

        let mut index = 0;
        while index < queue.len() {
            let (place, offset, depth) = queue[index].clone();
            index += 1;
            if depth >= I32_OFFSET_RELATION_DERIVATION_DEPTH {
                continue;
            }
            for (target, step) in self.i32_offset_targets(&place) {
                let Some(next_offset) = offset.checked_add(step) else {
                    continue;
                };
                push_i32_offset_reachable_state(
                    &mut out,
                    &mut queue,
                    self.canonicalize_scalar(&target),
                    next_offset,
                    depth + 1,
                );
            }
            for (source, step) in self.i32_offset_sources(&place) {
                let Some(next_offset) = offset.checked_sub(step) else {
                    continue;
                };
                push_i32_offset_reachable_state(
                    &mut out,
                    &mut queue,
                    self.canonicalize_scalar(&source),
                    next_offset,
                    depth + 1,
                );
            }
        }
        out
    }
}

fn push_i32_offset_reachable_state(
    out: &mut Vec<(Place, i64)>,
    queue: &mut Vec<(Place, i64, usize)>,
    place: Place,
    offset: i64,
    depth: usize,
) {
    if out
        .iter()
        .any(|(existing, existing_offset)| existing == &place && *existing_offset == offset)
    {
        return;
    }
    if out.len() >= I32_OFFSET_RELATION_MAX_STATES {
        return;
    }
    out.push((place.clone(), offset));
    queue.push((place, offset, depth));
}

fn merge_i32_offset_relation_truth(truth: &mut Option<bool>, candidate: bool) -> Option<()> {
    match truth {
        Some(existing) if *existing != candidate => None,
        Some(_) => Some(()),
        None => {
            *truth = Some(candidate);
            Some(())
        }
    }
}

fn merge_i32_derived_value(value: &mut Option<i32>, candidate: Option<i32>) -> Option<()> {
    let Some(candidate) = candidate else {
        return Some(());
    };
    match value {
        Some(existing) if *existing != candidate => None,
        Some(_) => Some(()),
        None => {
            *value = Some(candidate);
            Some(())
        }
    }
}

fn relation_holds_i64(left: i64, op: ResourceI32RelationOp, right: i64) -> bool {
    match op {
        ResourceI32RelationOp::Eq => left == right,
        ResourceI32RelationOp::Ne => left != right,
        ResourceI32RelationOp::Lt => left < right,
        ResourceI32RelationOp::Le => left <= right,
        ResourceI32RelationOp::Gt => left > right,
        ResourceI32RelationOp::Ge => left >= right,
    }
}

fn place_has_raw_address_projection(place: &Place) -> bool {
    place.projections.iter().any(|projection| {
        matches!(
            projection,
            PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
        )
    })
}

fn scalar_alias_rank(place: &Place) -> (u8, u8, usize) {
    (
        if place_has_raw_address_projection(place) {
            1
        } else {
            0
        },
        scalar_place_rank(place),
        place.projections.len(),
    )
}

fn scalar_place_rank(place: &Place) -> u8 {
    match &place.root {
        PlaceRoot::Local(_) => 0,
        PlaceRoot::I32Constant(_) => 0,
        PlaceRoot::Return => 1,
        PlaceRoot::Storage(_) => 2,
        PlaceRoot::Temporary(_) => 3,
        PlaceRoot::Unknown => 4,
    }
}
