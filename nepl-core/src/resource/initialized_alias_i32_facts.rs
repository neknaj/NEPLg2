extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_i32_condition_context::I32ConditionQueryContext;
use super::initialized_alias_i32_offset::{
    merge_i32_derived_value, place_has_raw_address_projection, scalar_alias_rank,
};
use super::initialized_alias_relation_op::relation_holds;
use super::model::{I32ValueCondition, Place, PlaceRoot, ResourceI32RelationOp};

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
        let source_origin = self.canonicalize_scalar(source);
        let fact_copies =
            (offset == 0).then(|| self.scalar_fact_copies_for_aliases(source, target));
        let mut sources = self.scalar_fact_recording_sources(source);
        sources.retain(|source| source != target);
        self.clear_scalar_facts(target);
        self.i32_offsets
            .set_offsets_for_target(sources, target, offset);
        if let Some(fact_copies) = fact_copies {
            // offset 0 は算術的な差分情報であると同時に、source の値そのものを
            // target へ渡す事実でもある。ここで条件と定数を直接写しておくと、
            // Result/Option payload を経由する owner-return summary が 0-offset の
            // 長い鎖だけに依存せず、後続の range proof が同じ値条件を参照できる。
            self.copy_scalar_facts_to_fresh_target(source, target, source_origin, fact_copies);
        }
    }

    pub(super) fn i32_value(&self, place: &Place) -> Option<i32> {
        self.direct_i32_value(place)
            .or_else(|| self.i32_value_from_bounded_offsets(place))
    }

    pub(super) fn direct_i32_value(&self, place: &Place) -> Option<i32> {
        if let PlaceRoot::I32Constant(value) = place.root {
            return Some(value);
        }
        self.i32_facts
            .value_for_aliases(&self.scalar_aliases_for(place))
    }

    pub(super) fn direct_i32_value_with_context(
        &self,
        place: &Place,
        context: &mut I32ConditionQueryContext,
    ) -> Option<i32> {
        if let PlaceRoot::I32Constant(value) = place.root {
            return Some(value);
        }
        let aliases = self.scalar_aliases_for_value_with_context(place, context);
        self.i32_facts.value_for_aliases(&aliases)
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
            .or_else(|| self.i32_relation_truth_from_zero_condition(left, op, right))
            .or_else(|| self.i32_offset_relation_truth(left, op, right))
    }

    pub(super) fn i32_relation_truth_with_context(
        &self,
        left: &Place,
        op: ResourceI32RelationOp,
        right: &Place,
        context: &mut I32ConditionQueryContext,
    ) -> Option<bool> {
        if let (Some(left_value), Some(right_value)) = (
            self.i32_value_with_context(left, context),
            self.i32_value_with_context(right, context),
        ) {
            return Some(relation_holds(left_value, op, right_value));
        }
        let left_aliases = self.scalar_aliases_for_value_with_context(left, context);
        let right_aliases = self.scalar_aliases_for_value_with_context(right, context);
        if let Some(truth) =
            self.i32_relations
                .relation_truth_for_aliases(&left_aliases, op, &right_aliases)
        {
            return Some(truth);
        }
        if let Some(truth) =
            self.i32_relation_truth_from_zero_condition_with_context(left, op, right, context)
        {
            return Some(truth);
        }
        self.i32_offset_relation_truth_with_context(left, op, right, context)
    }

    fn i32_relation_truth_from_zero_condition(
        &self,
        left: &Place,
        op: ResourceI32RelationOp,
        right: &Place,
    ) -> Option<bool> {
        let left_value = self.direct_i32_value(left);
        let right_value = self.direct_i32_value(right);
        match (left_value, right_value) {
            (Some(0), None) => self.i32_relation_truth_from_condition_against_zero(right, op, true),
            (None, Some(0)) => self.i32_relation_truth_from_condition_against_zero(left, op, false),
            _ => None,
        }
    }

    fn i32_relation_truth_from_zero_condition_with_context(
        &self,
        left: &Place,
        op: ResourceI32RelationOp,
        right: &Place,
        context: &mut I32ConditionQueryContext,
    ) -> Option<bool> {
        let left_value = self.direct_i32_value_with_context(left, context);
        let right_value = self.direct_i32_value_with_context(right, context);
        match (left_value, right_value) {
            (Some(0), None) => self.i32_relation_truth_from_condition_against_zero_with_context(
                right, op, true, context,
            ),
            (None, Some(0)) => self.i32_relation_truth_from_condition_against_zero_with_context(
                left, op, false, context,
            ),
            _ => None,
        }
    }

    fn i32_relation_truth_from_condition_against_zero(
        &self,
        place: &Place,
        op: ResourceI32RelationOp,
        zero_on_left: bool,
    ) -> Option<bool> {
        let condition = if zero_on_left {
            match op {
                ResourceI32RelationOp::Lt => I32ValueCondition::Positive,
                ResourceI32RelationOp::Le => I32ValueCondition::NonNegative,
                ResourceI32RelationOp::Gt => I32ValueCondition::Negative,
                ResourceI32RelationOp::Ge => I32ValueCondition::NonPositive,
                ResourceI32RelationOp::Eq => I32ValueCondition::EqZero,
                ResourceI32RelationOp::Ne => I32ValueCondition::NeZero,
            }
        } else {
            match op {
                ResourceI32RelationOp::Lt => I32ValueCondition::Negative,
                ResourceI32RelationOp::Le => I32ValueCondition::NonPositive,
                ResourceI32RelationOp::Gt => I32ValueCondition::Positive,
                ResourceI32RelationOp::Ge => I32ValueCondition::NonNegative,
                ResourceI32RelationOp::Eq => I32ValueCondition::EqZero,
                ResourceI32RelationOp::Ne => I32ValueCondition::NeZero,
            }
        };
        self.i32_condition_truth(place, condition)
    }

    fn i32_relation_truth_from_condition_against_zero_with_context(
        &self,
        place: &Place,
        op: ResourceI32RelationOp,
        zero_on_left: bool,
        context: &mut I32ConditionQueryContext,
    ) -> Option<bool> {
        let condition = if zero_on_left {
            match op {
                ResourceI32RelationOp::Lt => I32ValueCondition::Positive,
                ResourceI32RelationOp::Le => I32ValueCondition::NonNegative,
                ResourceI32RelationOp::Gt => I32ValueCondition::Negative,
                ResourceI32RelationOp::Ge => I32ValueCondition::NonPositive,
                ResourceI32RelationOp::Eq => I32ValueCondition::EqZero,
                ResourceI32RelationOp::Ne => I32ValueCondition::NeZero,
            }
        } else {
            match op {
                ResourceI32RelationOp::Lt => I32ValueCondition::Negative,
                ResourceI32RelationOp::Le => I32ValueCondition::NonPositive,
                ResourceI32RelationOp::Gt => I32ValueCondition::Positive,
                ResourceI32RelationOp::Ge => I32ValueCondition::NonNegative,
                ResourceI32RelationOp::Eq => I32ValueCondition::EqZero,
                ResourceI32RelationOp::Ne => I32ValueCondition::NeZero,
            }
        };
        self.i32_condition_truth_inner(place, condition, 0, true, context)
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

    pub(super) fn i32_value_from_bounded_offsets_with_context(
        &self,
        place: &Place,
        context: &mut I32ConditionQueryContext,
    ) -> Option<i32> {
        let aliases = self.scalar_aliases_for_value_with_context(place, context);
        if !self.i32_offsets.has_offset_for_aliases(&aliases)
            && !self.i32_scales.has_scaled_source_for_aliases(&aliases)
        {
            return None;
        }
        let mut value = None;
        for (reachable, offset) in self.i32_offset_reachable_from_with_context(place, context) {
            let Some(reachable_value) = self.direct_i32_value_with_context(&reachable, context)
            else {
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
        if let Some((source, scale)) = self.i32_scaled_source_with_context(place, context) {
            let candidate = self
                .direct_i32_value_with_context(&source, context)
                .and_then(|source_value| {
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
        for candidate in self.i32_scaled_source_candidates(place) {
            match &out {
                Some(existing) if existing != &candidate => return None,
                Some(_) => {}
                None => out = Some(candidate),
            }
        }
        out
    }

    pub(super) fn i32_scaled_source_with_context(
        &self,
        place: &Place,
        context: &mut I32ConditionQueryContext,
    ) -> Option<(Place, usize)> {
        let mut out = None;
        for candidate in self.i32_scaled_source_candidates_with_context(place, context) {
            match &out {
                Some(existing) if existing != &candidate => return None,
                Some(_) => {}
                None => out = Some(candidate),
            }
        }
        out
    }

    pub(super) fn i32_scaled_source_candidates(&self, place: &Place) -> Vec<(Place, usize)> {
        let mut out = Vec::new();
        for (source, scale) in self
            .i32_scales
            .scaled_sources_for_aliases(&self.scalar_aliases_for(place))
        {
            let candidate = (self.canonicalize_scalar(&source), scale);
            if !out.iter().any(|existing| existing == &candidate) {
                out.push(candidate);
            }
        }
        out
    }

    fn i32_scaled_source_candidates_with_context(
        &self,
        place: &Place,
        context: &mut I32ConditionQueryContext,
    ) -> Vec<(Place, usize)> {
        let aliases = self.scalar_aliases_for_value_with_context(place, context);
        let mut out = Vec::new();
        for (source, scale) in self.i32_scales.scaled_sources_for_aliases(&aliases) {
            let candidate = (
                self.canonicalize_scalar_with_context(&source, context),
                scale,
            );
            if !out.iter().any(|existing| existing == &candidate) {
                out.push(candidate);
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
        let mut out = Vec::new();
        for (minuend, subtrahend) in self
            .i32_differences
            .difference_sources_for_aliases(&self.scalar_aliases_for(place))
        {
            let pair = (
                self.canonicalize_scalar(&minuend),
                self.canonicalize_scalar(&subtrahend),
            );
            if !out.iter().any(|existing| existing == &pair) {
                out.push(pair);
            }
        }
        out
    }
}
