use super::model::{I32ValueCondition, Place};

#[derive(Default)]
pub(super) struct I32ConditionQueryContext {
    active: alloc::vec::Vec<I32ConditionQuery>,
    memo: alloc::vec::Vec<I32ConditionMemo>,
}

#[derive(Clone, PartialEq, Eq)]
struct I32ConditionQuery {
    place: Place,
    condition: I32ValueCondition,
    derive_false: bool,
}

#[derive(Clone, PartialEq, Eq)]
struct I32ConditionMemo {
    query: I32ConditionQuery,
    depth: usize,
    result: Option<bool>,
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
            .iter()
            .find(|entry| {
                entry.depth == depth
                    && entry.query.place == *place
                    && entry.query.condition == condition
                    && entry.query.derive_false == derive_false
            })
            .map(|entry| entry.result)
    }

    pub(super) fn memoize(
        &mut self,
        place: &Place,
        condition: I32ValueCondition,
        depth: usize,
        derive_false: bool,
        result: Option<bool>,
    ) {
        self.memo.push(I32ConditionMemo {
            query: I32ConditionQuery {
                place: place.clone(),
                condition,
                derive_false,
            },
            depth,
            result,
        });
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
