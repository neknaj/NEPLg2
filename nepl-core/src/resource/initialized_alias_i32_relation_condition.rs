use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_i32::condition_implication;
use super::initialized_alias_i32_condition_context::I32ConditionQueryContext;
use super::model::{I32ValueCondition, Place, ResourceI32RelationOp};

impl RawCellAddressAliases {
    pub(super) fn i32_relation_condition_truth(
        &self,
        place: &Place,
        condition: I32ValueCondition,
        depth: usize,
        derive_false: bool,
        context: &mut I32ConditionQueryContext,
    ) -> Option<bool> {
        let aliases = self.scalar_aliases_for_value_with_context(place, context);
        for fact in self.i32_relations.relations_touching_aliases(&aliases) {
            if aliases.iter().any(|alias| alias == &fact.left)
                && self.relation_implies_condition(
                    true,
                    fact.op,
                    &fact.right,
                    condition,
                    depth,
                    derive_false,
                    context,
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
                    context,
                ) == Some(true)
            {
                return Some(true);
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
        context: &mut I32ConditionQueryContext,
    ) -> Option<bool> {
        use I32ValueCondition::{Negative, NonNegative, NonPositive, Positive};
        use ResourceI32RelationOp::{Eq, Ge, Gt, Le, Lt};

        if relation == Eq {
            return self.i32_condition_truth_inner(other, condition, depth, derive_false, context);
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
        if self.i32_condition_truth_inner(other, required, depth, derive_false, context)
            != Some(true)
        {
            return None;
        }
        if condition_implication(required, condition) == Some(false) {
            return None;
        }
        Some(true)
    }
}
