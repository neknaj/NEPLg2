extern crate alloc;

use alloc::vec::Vec;

use super::model::{Place, PlaceProjection, PlaceRoot};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn prefer_stable_canonical(group: &mut Vec<Place>) {
    let Some((index, _)) = group.iter().enumerate().min_by_key(|(_, place)| {
        (
            canonical_place_projection_rank(place),
            canonical_place_rank(place),
            place.projections.len(),
        )
    }) else {
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
    }) || place_suffix_after_prefix(place, base).is_some_and(|suffix| {
        suffix.iter().any(|projection| {
            matches!(
                projection,
                PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
            )
        })
    })
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
    match &place.root {
        PlaceRoot::Local(_) => 0,
        PlaceRoot::Return => 1,
        PlaceRoot::Storage(_) => 2,
        PlaceRoot::Temporary(_) => 3,
        PlaceRoot::Unknown => 4,
    }
}
