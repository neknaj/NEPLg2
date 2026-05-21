use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceRoot, ResourceConditionFact, ResourceI32RelationOp};

pub(super) fn initialized_range_loop_bound(
    raw_aliases: &RawCellAddressAliases,
    fact: Option<&ResourceConditionFact>,
) -> Option<(Place, Place)> {
    let fact = fact?;
    let mut out = LoopBoundSearch::None;
    collect_initialized_range_loop_bound(raw_aliases, fact, &mut out);
    match out {
        LoopBoundSearch::One(bound) => Some(bound),
        LoopBoundSearch::None | LoopBoundSearch::Ambiguous => None,
    }
}

fn collect_initialized_range_loop_bound(
    raw_aliases: &RawCellAddressAliases,
    fact: &ResourceConditionFact,
    out: &mut LoopBoundSearch,
) {
    match fact {
        ResourceConditionFact::I32Relation { left, op, right } => match op {
            ResourceI32RelationOp::Lt => push_loop_bound(
                out,
                loop_counter_place(raw_aliases, left),
                raw_aliases.canonicalize_scalar(right),
            ),
            ResourceI32RelationOp::Gt => push_loop_bound(
                out,
                loop_counter_place(raw_aliases, right),
                raw_aliases.canonicalize_scalar(left),
            ),
            ResourceI32RelationOp::Eq
            | ResourceI32RelationOp::Ne
            | ResourceI32RelationOp::Le
            | ResourceI32RelationOp::Ge => {}
        },
        ResourceConditionFact::All(facts) => {
            for fact in facts {
                collect_initialized_range_loop_bound(raw_aliases, fact, out);
            }
        }
        ResourceConditionFact::Any(_)
        | ResourceConditionFact::EqZero { .. }
        | ResourceConditionFact::NeZero { .. }
        | ResourceConditionFact::Positive { .. }
        | ResourceConditionFact::NonPositive { .. }
        | ResourceConditionFact::Negative { .. }
        | ResourceConditionFact::NonNegative { .. } => {}
    }
}

enum LoopBoundSearch {
    None,
    One((Place, Place)),
    Ambiguous,
}

fn push_loop_bound(out: &mut LoopBoundSearch, index: Place, initialized_count: Place) {
    let candidate = (index, initialized_count);
    match out {
        LoopBoundSearch::None => *out = LoopBoundSearch::One(candidate),
        LoopBoundSearch::One(existing) if existing == &candidate => {}
        LoopBoundSearch::One(_) | LoopBoundSearch::Ambiguous => {
            *out = LoopBoundSearch::Ambiguous;
        }
    }
}

fn loop_counter_place(raw_aliases: &RawCellAddressAliases, place: &Place) -> Place {
    raw_aliases
        .scalar_aliases_for_value(place)
        .into_iter()
        .filter(loop_counter_candidate)
        .min_by_key(loop_counter_rank)
        .unwrap_or_else(|| place.clone())
}

fn loop_counter_candidate(place: &Place) -> bool {
    matches!(
        place.root,
        PlaceRoot::Local(_) | PlaceRoot::Return | PlaceRoot::Storage(_)
    )
}

fn loop_counter_rank(place: &Place) -> (u8, usize) {
    (
        match place.root {
            PlaceRoot::Local(_) => 0,
            PlaceRoot::Return => 1,
            PlaceRoot::Storage(_) => 2,
            PlaceRoot::Temporary(_) | PlaceRoot::I32Constant(_) | PlaceRoot::Unknown => 3,
        },
        place.projections.len(),
    )
}
