use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_i32_condition_context::{
    i32_condition_contradictors, I32ConditionQueryContext,
};
use super::model::{I32ValueCondition, Place};

const I32_CONDITION_DERIVATION_DEPTH: usize = 8;

impl RawCellAddressAliases {
    pub(super) fn i32_condition_truth(
        &self,
        place: &Place,
        condition: I32ValueCondition,
    ) -> Option<bool> {
        self.i32_condition_truth_inner(
            place,
            condition,
            0,
            true,
            &mut I32ConditionQueryContext::default(),
        )
    }

    pub(super) fn i32_condition_is_known_true(
        &self,
        place: &Place,
        condition: I32ValueCondition,
    ) -> bool {
        self.i32_condition_truth_inner(
            place,
            condition,
            0,
            false,
            &mut I32ConditionQueryContext::default(),
        ) == Some(true)
    }

    pub(super) fn i32_condition_truth_inner(
        &self,
        place: &Place,
        condition: I32ValueCondition,
        depth: usize,
        derive_false: bool,
        context: &mut I32ConditionQueryContext,
    ) -> Option<bool> {
        if let Some(result) = context.memo_result(place, condition, depth, derive_false) {
            return result;
        }
        if !context.push_active(place, condition, derive_false) {
            return None;
        }
        let result = self.i32_condition_truth_inner_unvisited(
            place,
            condition,
            depth,
            derive_false,
            context,
        );
        context.pop_active();
        context.memoize(place, condition, depth, derive_false, result);
        result
    }

    fn i32_condition_truth_inner_unvisited(
        &self,
        place: &Place,
        condition: I32ValueCondition,
        depth: usize,
        derive_false: bool,
        context: &mut I32ConditionQueryContext,
    ) -> Option<bool> {
        if let Some(value) = self.i32_value(place) {
            return Some(condition.holds(value));
        }
        if let Some(truth) = self
            .i32_facts
            .condition_truth_for_aliases(&self.scalar_aliases_for(place), condition)
        {
            return Some(truth);
        }
        if depth >= I32_CONDITION_DERIVATION_DEPTH {
            return None;
        }
        if let Some(truth) =
            self.i32_scaled_condition_truth(place, condition, depth + 1, derive_false, context)
        {
            return Some(truth);
        }
        if let Some(truth) =
            self.i32_relation_condition_truth(place, condition, depth + 1, derive_false, context)
        {
            return Some(truth);
        }
        if !derive_false {
            return None;
        }
        self.i32_implied_condition_truth(place, condition, depth + 1, context)
    }

    fn i32_scaled_condition_truth(
        &self,
        place: &Place,
        condition: I32ValueCondition,
        depth: usize,
        derive_false: bool,
        context: &mut I32ConditionQueryContext,
    ) -> Option<bool> {
        let (source, scale) = self.i32_scaled_source(place)?;
        if scale == 0 {
            return None;
        }
        self.i32_condition_truth_inner(&source, condition, depth, derive_false, context)
    }

    fn i32_implied_condition_truth(
        &self,
        place: &Place,
        condition: I32ValueCondition,
        depth: usize,
        context: &mut I32ConditionQueryContext,
    ) -> Option<bool> {
        for &known in i32_condition_contradictors(condition) {
            if self.i32_condition_truth_inner(place, known, depth, false, context) == Some(true) {
                return Some(false);
            }
        }
        None
    }
}
