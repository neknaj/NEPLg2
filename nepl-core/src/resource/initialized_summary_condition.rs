#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum RawCellValueCondition {
    EqZero,
    NeZero,
    Positive,
    NonPositive,
    Negative,
    NonNegative,
}

impl RawCellValueCondition {
    pub(super) fn holds(self, value: i32) -> bool {
        match self {
            RawCellValueCondition::EqZero => value == 0,
            RawCellValueCondition::NeZero => value != 0,
            RawCellValueCondition::Positive => value > 0,
            RawCellValueCondition::NonPositive => value <= 0,
            RawCellValueCondition::Negative => value < 0,
            RawCellValueCondition::NonNegative => value >= 0,
        }
    }
}
