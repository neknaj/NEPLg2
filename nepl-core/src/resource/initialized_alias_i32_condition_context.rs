use alloc::vec::Vec;

use super::model::{I32ValueCondition, Place};

#[derive(Default)]
pub(super) struct I32ConditionQueryContext {
    active: Vec<I32ConditionQuery>,
    memo: Vec<I32ConditionMemo>,
    value_memo: Vec<I32ValueMemo>,
    scalar_alias_memo: Vec<ScalarAliasMemo>,
    offset_source_memo: Vec<I32OffsetSourceMemo>,
    offset_target_memo: Vec<I32OffsetTargetMemo>,
    offset_reachable_memo: Vec<I32OffsetReachableMemo>,
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

#[derive(Clone, PartialEq, Eq)]
struct I32ValueMemo {
    place: Place,
    result: Option<i32>,
}

#[derive(Clone, PartialEq, Eq)]
struct ScalarAliasMemo {
    place: Place,
    aliases: Vec<Place>,
}

#[derive(Clone, PartialEq, Eq)]
struct I32OffsetSourceMemo {
    place: Place,
    sources: Vec<(Place, i64)>,
}

#[derive(Clone, PartialEq, Eq)]
struct I32OffsetTargetMemo {
    place: Place,
    targets: Vec<(Place, i64)>,
}

#[derive(Clone, PartialEq, Eq)]
struct I32OffsetReachableMemo {
    place: Place,
    reachable: Vec<(Place, i64)>,
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

    pub(super) fn value_result(&self, place: &Place) -> Option<Option<i32>> {
        self.value_memo
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.result)
    }

    pub(super) fn memoize_value(&mut self, place: &Place, result: Option<i32>) {
        self.value_memo.push(I32ValueMemo {
            place: place.clone(),
            result,
        });
    }

    pub(super) fn scalar_aliases(&self, place: &Place) -> Option<Vec<Place>> {
        self.scalar_alias_memo
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.aliases.clone())
    }

    pub(super) fn memoize_scalar_aliases(&mut self, place: &Place, aliases: Vec<Place>) {
        self.scalar_alias_memo.push(ScalarAliasMemo {
            place: place.clone(),
            aliases,
        });
    }

    pub(super) fn offset_sources(&self, place: &Place) -> Option<Vec<(Place, i64)>> {
        self.offset_source_memo
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.sources.clone())
    }

    pub(super) fn memoize_offset_sources(&mut self, place: &Place, sources: Vec<(Place, i64)>) {
        self.offset_source_memo.push(I32OffsetSourceMemo {
            place: place.clone(),
            sources,
        });
    }

    pub(super) fn offset_targets(&self, place: &Place) -> Option<Vec<(Place, i64)>> {
        self.offset_target_memo
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.targets.clone())
    }

    pub(super) fn memoize_offset_targets(&mut self, place: &Place, targets: Vec<(Place, i64)>) {
        self.offset_target_memo.push(I32OffsetTargetMemo {
            place: place.clone(),
            targets,
        });
    }

    pub(super) fn offset_reachable(&self, place: &Place) -> Option<Vec<(Place, i64)>> {
        self.offset_reachable_memo
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.reachable.clone())
    }

    pub(super) fn memoize_offset_reachable(&mut self, place: &Place, reachable: Vec<(Place, i64)>) {
        self.offset_reachable_memo.push(I32OffsetReachableMemo {
            place: place.clone(),
            reachable,
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
