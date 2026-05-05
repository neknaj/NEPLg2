use super::model::{I32ValueCondition, Place};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ValueFact {
    pub(super) place: Place,
    pub(super) value: i32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct I32ConditionFact {
    pub(super) place: Place,
    pub(super) condition: I32ValueCondition,
}

pub(super) fn condition_implication(
    known: I32ValueCondition,
    query: I32ValueCondition,
) -> Option<bool> {
    use I32ValueCondition::{EqZero, NeZero, Negative, NonNegative, NonPositive, Positive};
    match (known, query) {
        (left, right) if left == right => Some(true),
        (EqZero, NeZero | Positive | Negative) => Some(false),
        (EqZero, NonPositive | NonNegative) => Some(true),
        (NeZero, EqZero) => Some(false),
        (Positive, EqZero | Negative | NonPositive) => Some(false),
        (Positive, NeZero | NonNegative) => Some(true),
        (NonPositive, Positive) => Some(false),
        (Negative, EqZero | Positive | NonNegative) => Some(false),
        (Negative, NeZero | NonPositive) => Some(true),
        (NonNegative, Negative) => Some(false),
        _ => None,
    }
}
