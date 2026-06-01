use alloc::collections::BTreeMap;
use alloc::vec::Vec;

use super::initialized_alias_i32_condition_memo::{
    I32OffsetReachableMemo, I32OffsetSourceMemo, I32OffsetTargetMemo, ScalarAliasMemo,
};
use super::model::{I32ValueCondition, Place};

#[derive(Default)]
pub(super) struct I32ConditionQueryContext {
    active: Vec<I32ConditionQuery>,
    memo: BTreeMap<I32ConditionMemoKey, Option<bool>>,
    value_memo: BTreeMap<Place, Option<i32>>,
    pub(super) scalar_alias_memo: ScalarAliasMemo,
    pub(super) offset_source_memo: I32OffsetSourceMemo,
    pub(super) offset_target_memo: I32OffsetTargetMemo,
    pub(super) offset_reachable_memo: I32OffsetReachableMemo,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct I32ConditionQuery {
    place: Place,
    condition: I32ValueCondition,
    derive_false: bool,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct I32ConditionMemoKey {
    place: Place,
    condition: I32ValueCondition,
    depth: usize,
    derive_false: bool,
}

impl I32ConditionQueryContext {
    pub(super) fn push_active(
        &mut self,
        place: &Place,
        condition: I32ValueCondition,
        derive_false: bool,
    ) -> bool {
        let query = I32ConditionQuery {
            place: place.clone(),
            condition,
            derive_false,
        };
        if self.active.iter().any(|entry| entry == &query) {
            return false;
        }
        self.active.push(query);
        true
    }

    pub(super) fn pop_active(&mut self) {
        self.active.pop();
    }

    pub(super) fn memo_result(
        &self,
        place: &Place,
        condition: I32ValueCondition,
        depth: usize,
        derive_false: bool,
    ) -> Option<Option<bool>> {
        self.memo
            .get(&I32ConditionMemoKey {
                place: place.clone(),
                condition,
                depth,
                derive_false,
            })
            .copied()
    }

    pub(super) fn memoize(
        &mut self,
        place: &Place,
        condition: I32ValueCondition,
        depth: usize,
        derive_false: bool,
        result: Option<bool>,
    ) {
        self.memo.insert(
            I32ConditionMemoKey {
                place: place.clone(),
                condition,
                depth,
                derive_false,
            },
            result,
        );
    }

    pub(super) fn value_result(&self, place: &Place) -> Option<Option<i32>> {
        self.value_memo.get(place).copied()
    }

    pub(super) fn memoize_value(&mut self, place: &Place, result: Option<i32>) {
        self.value_memo.insert(place.clone(), result);
    }
}

pub(super) fn i32_condition_contradictors(
    condition: I32ValueCondition,
) -> &'static [I32ValueCondition] {
    use I32ValueCondition::{EqZero, NeZero, Negative, NonNegative, NonPositive, Positive};

    match condition {
        EqZero => &[NeZero, Positive, Negative],
        NeZero => &[EqZero],
        Positive => &[EqZero, NonPositive, Negative],
        NonPositive => &[Positive],
        Negative => &[EqZero, Positive, NonNegative],
        NonNegative => &[Negative],
    }
}
