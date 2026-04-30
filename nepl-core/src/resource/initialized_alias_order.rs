extern crate alloc;

use core::cmp::Ordering;

use alloc::vec::Vec;

use super::model::{Place, PlaceProjection, PlaceRoot};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn prefer_stable_canonical(group: &mut Vec<Place>) {
    let Some((index, _)) = group
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| stable_canonical_order(left, right))
    else {
        return;
    };
    if index != 0 {
        let place = group.remove(index);
        group.insert(0, place);
    }
}

pub(super) fn owner_cell_alias_rank(place: &Place) -> (u8, u8, usize) {
    (
        owner_cell_projection_rank(place),
        canonical_place_rank(place),
        place.projections.len(),
    )
}

pub(super) fn owner_alias_place_has_raw_projection(place: &Place, base: &Place) -> bool {
    place.projections.iter().any(|projection| {
        matches!(
            projection,
            PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
        )
    }) || place_suffix_after_prefix(place, base).is_some()
}

fn canonical_place_projection_rank(place: &Place) -> u8 {
    if place
        .projections
        .iter()
        .any(|projection| matches!(projection, PlaceProjection::StorageOffset(_)))
    {
        0
    } else {
        1
    }
}

fn owner_cell_projection_rank(place: &Place) -> u8 {
    if place.projections.iter().any(|projection| {
        matches!(
            projection,
            PlaceProjection::Field { .. }
                | PlaceProjection::TupleField { .. }
                | PlaceProjection::EnumPayload { .. }
        )
    }) {
        0
    } else if place
        .projections
        .iter()
        .any(|projection| matches!(projection, PlaceProjection::StorageOffset(_)))
    {
        1
    } else {
        2
    }
}

fn canonical_place_rank(place: &Place) -> u8 {
    match place.root {
        PlaceRoot::Local(_) => 0,
        PlaceRoot::Return => 1,
        PlaceRoot::Storage(_) => 2,
        PlaceRoot::Temporary(_) => 3,
        PlaceRoot::Unknown => 4,
    }
}

fn stable_canonical_order(left: &Place, right: &Place) -> Ordering {
    (
        canonical_place_projection_rank(left),
        canonical_place_rank(left),
        left.projections.len(),
    )
        .cmp(&(
            canonical_place_projection_rank(right),
            canonical_place_rank(right),
            right.projections.len(),
        ))
        .then_with(|| place_root_order(&left.root, &right.root))
        .then_with(|| projection_slice_order(&left.projections, &right.projections))
        .then_with(|| left.ty.0.cmp(&right.ty.0))
}

fn place_root_order(left: &PlaceRoot, right: &PlaceRoot) -> Ordering {
    match (left, right) {
        (PlaceRoot::Local(left), PlaceRoot::Local(right)) => left.cmp(right),
        (PlaceRoot::Temporary(left), PlaceRoot::Temporary(right)) => left.0.cmp(&right.0),
        (PlaceRoot::Storage(left), PlaceRoot::Storage(right)) => left.0.cmp(&right.0),
        (PlaceRoot::Return, PlaceRoot::Return) | (PlaceRoot::Unknown, PlaceRoot::Unknown) => {
            Ordering::Equal
        }
        _ => canonical_place_root_rank(left).cmp(&canonical_place_root_rank(right)),
    }
}

fn canonical_place_root_rank(root: &PlaceRoot) -> u8 {
    match root {
        PlaceRoot::Local(_) => 0,
        PlaceRoot::Return => 1,
        PlaceRoot::Storage(_) => 2,
        PlaceRoot::Temporary(_) => 3,
        PlaceRoot::Unknown => 4,
    }
}

fn projection_slice_order(left: &[PlaceProjection], right: &[PlaceProjection]) -> Ordering {
    for (left_projection, right_projection) in left.iter().zip(right) {
        let ordering = projection_order(left_projection, right_projection);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    left.len().cmp(&right.len())
}

fn projection_order(left: &PlaceProjection, right: &PlaceProjection) -> Ordering {
    match (left, right) {
        (
            PlaceProjection::Field {
                index: left_index,
                offset_bytes: left_offset,
            },
            PlaceProjection::Field {
                index: right_index,
                offset_bytes: right_offset,
            },
        )
        | (
            PlaceProjection::TupleField {
                index: left_index,
                offset_bytes: left_offset,
            },
            PlaceProjection::TupleField {
                index: right_index,
                offset_bytes: right_offset,
            },
        ) => (left_index, left_offset).cmp(&(right_index, right_offset)),
        (
            PlaceProjection::EnumPayload { variant: left },
            PlaceProjection::EnumPayload { variant: right },
        ) => left.cmp(right),
        (PlaceProjection::StorageOffset(left), PlaceProjection::StorageOffset(right)) => {
            storage_offset_order(left.bytes, right.bytes)
        }
        (PlaceProjection::Deref, PlaceProjection::Deref) => Ordering::Equal,
        _ => projection_kind_rank(left).cmp(&projection_kind_rank(right)),
    }
}

fn storage_offset_order(left: Option<usize>, right: Option<usize>) -> Ordering {
    match (left, right) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(left), Some(right)) => left.cmp(&right),
    }
}

fn projection_kind_rank(projection: &PlaceProjection) -> u8 {
    match projection {
        PlaceProjection::Field { .. } => 0,
        PlaceProjection::TupleField { .. } => 1,
        PlaceProjection::EnumPayload { .. } => 2,
        PlaceProjection::Deref => 3,
        PlaceProjection::StorageOffset(_) => 4,
    }
}
