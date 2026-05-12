use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_i32::condition_implication;
use super::model::{I32ValueCondition, Place, ResourceI32RelationOp};

const I32_CONDITION_DERIVATION_DEPTH: usize = 8;

impl RawCellAddressAliases {
    pub(super) fn i32_condition_truth(
        &self,
        place: &Place,
        condition: I32ValueCondition,
    ) -> Option<bool> {
        self.i32_condition_truth_inner(place, condition, 0, true)
    }

    pub(super) fn i32_condition_is_known_true(
        &self,
        place: &Place,
        condition: I32ValueCondition,
    ) -> bool {
        self.i32_condition_truth_inner(place, condition, 0, false) == Some(true)
    }

    fn i32_condition_truth_inner(
        &self,
        place: &Place,
        condition: I32ValueCondition,
        depth: usize,
        derive_false: bool,
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
            self.i32_scaled_condition_truth(place, condition, depth + 1, derive_false)
        {
            return Some(truth);
        }
        if let Some(truth) =
            self.i32_relation_condition_truth(place, condition, depth + 1, derive_false)
        {
            return Some(truth);
        }
        if !derive_false {
            return None;
        }
        self.i32_implied_condition_truth(place, condition, depth + 1)
    }

    fn i32_scaled_condition_truth(
        &self,
        place: &Place,
        condition: I32ValueCondition,
        depth: usize,
        derive_false: bool,
    ) -> Option<bool> {
        let (source, scale) = self.i32_scaled_source(place)?;
        if scale == 0 {
            return None;
        }
        self.i32_condition_truth_inner(&source, condition, depth, derive_false)
    }

    fn i32_relation_condition_truth(
        &self,
        place: &Place,
        condition: I32ValueCondition,
        depth: usize,
        derive_false: bool,
    ) -> Option<bool> {
        let aliases = self.scalar_aliases_for(place);
        for fact in self.i32_relations.relations_touching_aliases(&aliases) {
            if aliases.iter().any(|alias| alias == &fact.left)
                && self.relation_implies_condition(
                    true,
                    fact.op,
                    &fact.right,
                    condition,
                    depth,
                    derive_false,
                ) == Some(true)
            {
                return Some(true);
            }
            if aliases.iter().any(|alias| alias == &fact.right)
                && self.relation_implies_condition(
                    false,
                    fact.op,
                    &fact.left,
                    condition,
                    depth,
                    derive_false,
                ) == Some(true)
            {
                return Some(true);
            }
        }
        None
    }

    fn i32_implied_condition_truth(
        &self,
        place: &Place,
        condition: I32ValueCondition,
        depth: usize,
    ) -> Option<bool> {
        for &known in i32_condition_contradictors(condition) {
            if self.i32_condition_truth_inner(place, known, depth, false) == Some(true) {
                return Some(false);
            }
        }
        None
    }

    fn relation_implies_condition(
        &self,
        target_is_left: bool,
        relation: ResourceI32RelationOp,
        other: &Place,
        condition: I32ValueCondition,
        depth: usize,
        derive_false: bool,
    ) -> Option<bool> {
        use I32ValueCondition::{Negative, NonNegative, NonPositive, Positive};
        use ResourceI32RelationOp::{Eq, Ge, Gt, Le, Lt};

        if relation == Eq {
            return self.i32_condition_truth_inner(other, condition, depth, derive_false);
        }

        let required = match (target_is_left, relation, condition) {
            (true, Lt, Negative | NonPositive) => NonPositive,
            (true, Le, Negative) => Negative,
            (true, Le, NonPositive) => NonPositive,
            (true, Gt, Positive | NonNegative) => NonNegative,
            (true, Ge, Positive) => Positive,
            (true, Ge, NonNegative) => NonNegative,
            (false, Lt, Positive | NonNegative) => NonNegative,
            (false, Le, Positive) => Positive,
            (false, Le, NonNegative) => NonNegative,
            (false, Gt, Negative | NonPositive) => NonPositive,
            (false, Ge, Negative) => Negative,
            (false, Ge, NonPositive) => NonPositive,
            _ => return None,
        };
        if self.i32_condition_truth_inner(other, required, depth, derive_false) != Some(true) {
            return None;
        }
        if condition_implication(required, condition) == Some(false) {
            return None;
        }
        Some(true)
    }
}

fn i32_condition_contradictors(condition: I32ValueCondition) -> &'static [I32ValueCondition] {
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
