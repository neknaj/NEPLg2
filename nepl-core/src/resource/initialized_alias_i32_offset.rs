extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_i32_condition_context::I32ConditionQueryContext;
use super::model::{Place, PlaceProjection, PlaceRoot, ResourceI32RelationOp};

const I32_OFFSET_RELATION_DERIVATION_DEPTH: usize = 8;
const I32_OFFSET_RELATION_MAX_STATES: usize = 128;

impl RawCellAddressAliases {
    pub(super) fn i32_offset_relation_truth(
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

    pub(super) fn i32_offset_relation_truth_with_context(
        &self,
        left: &Place,
        op: ResourceI32RelationOp,
        right: &Place,
        context: &mut I32ConditionQueryContext,
    ) -> Option<bool> {
        let left_reachable = self.i32_offset_reachable_from_with_context(left, context);
        let right_reachable = self.i32_offset_reachable_from_with_context(right, context);
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

    pub(super) fn i32_offset_reachable_from(&self, start: &Place) -> Vec<(Place, i64)> {
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

    pub(super) fn i32_offset_reachable_from_with_context(
        &self,
        start: &Place,
        context: &mut I32ConditionQueryContext,
    ) -> Vec<(Place, i64)> {
        if let Some(reachable) = context.offset_reachable(start) {
            return reachable;
        }
        let mut out = Vec::new();
        let mut queue = Vec::new();
        for alias in self.scalar_aliases_for_value_with_context(start, context) {
            push_i32_offset_reachable_state(
                &mut out,
                &mut queue,
                self.canonicalize_scalar_with_context(&alias, context),
                0,
                0,
            );
        }
        push_i32_offset_reachable_state(
            &mut out,
            &mut queue,
            self.canonicalize_scalar_with_context(start, context),
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
            for (target, step) in self.i32_offset_targets_with_context(&place, context) {
                let Some(next_offset) = offset.checked_add(step) else {
                    continue;
                };
                push_i32_offset_reachable_state(
                    &mut out,
                    &mut queue,
                    self.canonicalize_scalar_with_context(&target, context),
                    next_offset,
                    depth + 1,
                );
            }
            for (source, step) in self.i32_offset_sources_with_context(&place, context) {
                let Some(next_offset) = offset.checked_sub(step) else {
                    continue;
                };
                push_i32_offset_reachable_state(
                    &mut out,
                    &mut queue,
                    self.canonicalize_scalar_with_context(&source, context),
                    next_offset,
                    depth + 1,
                );
            }
        }
        context.memoize_offset_reachable(start, out.clone());
        out
    }

    pub(super) fn i32_offset_sources_with_context(
        &self,
        target: &Place,
        context: &mut I32ConditionQueryContext,
    ) -> Vec<(Place, i64)> {
        let aliases = self.scalar_aliases_for_value_with_context(target, context);
        self.i32_offset_sources_for_target_aliases_with_context(target, &aliases, context)
    }

    /// 事前に計算済みの target scalar aliases から offset source を引く。
    ///
    /// `aliases` は `target` に対する完全な scalar alias 集合でなければならない。
    /// 結果は `target` 単位で memoize されるため、部分集合を渡すと後続の同じ
    /// target query まで狭まり、offset proof を失う。
    pub(super) fn i32_offset_sources_for_target_aliases_with_context(
        &self,
        target: &Place,
        aliases: &[Place],
        context: &mut I32ConditionQueryContext,
    ) -> Vec<(Place, i64)> {
        if let Some(sources) = context.offset_sources(target) {
            return sources;
        }
        let sources: Vec<(Place, i64)> = self
            .i32_offsets
            .offset_sources_for_target_aliases(aliases)
            .into_iter()
            .map(|(source, offset)| {
                (
                    self.canonicalize_scalar_with_context(&source, context),
                    offset,
                )
            })
            .collect();
        context.memoize_offset_sources(target, sources.clone());
        sources
    }

    pub(super) fn i32_offset_targets_with_context(
        &self,
        source: &Place,
        context: &mut I32ConditionQueryContext,
    ) -> Vec<(Place, i64)> {
        if let Some(targets) = context.offset_targets(source) {
            return targets;
        }
        let aliases = self.scalar_aliases_for_value_with_context(source, context);
        let targets: Vec<(Place, i64)> = self
            .i32_offsets
            .offset_targets_for_source_aliases(&aliases)
            .into_iter()
            .map(|(target, offset)| {
                (
                    self.canonicalize_scalar_with_context(&target, context),
                    offset,
                )
            })
            .collect();
        context.memoize_offset_targets(source, targets.clone());
        targets
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

pub(super) fn merge_i32_derived_value(
    value: &mut Option<i32>,
    candidate: Option<i32>,
) -> Option<()> {
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

pub(super) fn place_has_raw_address_projection(place: &Place) -> bool {
    place.projections.iter().any(|projection| {
        matches!(
            projection,
            PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
        )
    })
}

pub(super) fn scalar_alias_rank(place: &Place) -> (u8, u8, usize) {
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
