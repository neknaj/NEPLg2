extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_i32::condition_implication;
use super::initialized_alias_relation_op::relation_holds;
use super::model::{I32ValueCondition, Place, PlaceProjection, PlaceRoot, ResourceI32RelationOp};

const I32_CONDITION_DERIVATION_DEPTH: usize = 8;

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
        let left = self.canonicalize_scalar(left);
        let right = self.canonicalize_scalar(right);
        self.i32_relations.add_relation(&left, op, &right);
    }

    pub(super) fn add_i32_difference(
        &mut self,
        minuend: &Place,
        subtrahend: &Place,
        difference: &Place,
    ) {
        let minuend = self.canonicalize_scalar(minuend);
        let subtrahend = self.canonicalize_scalar(subtrahend);
        self.i32_differences
            .add_difference(&minuend, &subtrahend, difference);
    }

    pub(super) fn add_i32_scale(&mut self, source: &Place, target: &Place, scale: usize) {
        let source = self.canonicalize_scalar(source);
        self.i32_scales.add_scale(&source, target, scale);
    }

    pub(super) fn i32_value(&self, place: &Place) -> Option<i32> {
        if let PlaceRoot::I32Constant(value) = place.root {
            return Some(value);
        }
        self.i32_facts
            .value_for_aliases(&self.scalar_aliases_for(place))
    }

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

    pub(super) fn i32_relation_truth(
        &self,
        left: &Place,
        op: ResourceI32RelationOp,
        right: &Place,
    ) -> Option<bool> {
        if let (Some(left_value), Some(right_value)) = (self.i32_value(left), self.i32_value(right))
        {
            return Some(relation_holds(left_value, op, right_value));
        }
        self.i32_relations.relation_truth_for_aliases(
            &self.scalar_aliases_for(left),
            op,
            &self.scalar_aliases_for(right),
        )
    }

    pub(super) fn i32_scaled_source(&self, place: &Place) -> Option<(Place, usize)> {
        let mut out = None;
        for (source, scale) in self
            .i32_scales
            .scaled_sources_for_aliases(&self.scalar_aliases_for(place))
        {
            let candidate = (self.canonicalize_scalar(&source), scale);
            match &out {
                Some(existing) if existing != &candidate => return None,
                Some(_) => {}
                None => out = Some(candidate),
            }
        }
        out
    }

    pub(super) fn i32_difference_sources(&self, place: &Place) -> Vec<(Place, Place)> {
        self.i32_differences
            .difference_sources_for_aliases(&self.scalar_aliases_for(place))
            .into_iter()
            .map(|(minuend, subtrahend)| {
                (
                    self.canonicalize_scalar(&minuend),
                    self.canonicalize_scalar(&subtrahend),
                )
            })
            .collect()
    }

    pub(super) fn scalar_aliases_for_value(&self, place: &Place) -> Vec<Place> {
        self.scalar_aliases_for(place)
    }
}

fn place_has_raw_address_projection(place: &Place) -> bool {
    place.projections.iter().any(|projection| {
        matches!(
            projection,
            PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
        )
    })
}

fn scalar_alias_rank(place: &Place) -> (u8, u8, usize) {
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
