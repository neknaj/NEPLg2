use alloc::vec::Vec;

use super::model::{Place, PlaceProjection};

pub(super) fn replace_place_prefix(
    place: &Place,
    prefix: &Place,
    replacement: &Place,
) -> Option<Place> {
    if !place_has_prefix(place, prefix) {
        return None;
    }
    let suffix = place.projections[prefix.projections.len()..].to_vec();
    let mut out = replacement.clone();
    let suffix_is_empty = suffix.is_empty();
    out.projections.extend(suffix);
    if !suffix_is_empty {
        out.ty = place.ty;
    }
    Some(out)
}

pub(super) fn place_suffix_after_prefix(
    place: &Place,
    prefix: &Place,
) -> Option<Vec<PlaceProjection>> {
    if !place_has_prefix(place, prefix) {
        return None;
    }
    Some(place.projections[prefix.projections.len()..].to_vec())
}

pub(super) fn place_has_prefix(place: &Place, prefix: &Place) -> bool {
    place.root == prefix.root
        && place.projections.len() >= prefix.projections.len()
        && place
            .projections
            .iter()
            .zip(&prefix.projections)
            .all(|(projection, prefix_projection)| projection == prefix_projection)
}

pub(super) fn groups_overlap(left: &[Place], right: &[Place]) -> bool {
    left.iter().any(|place| right.contains(place))
}

pub(super) fn push_unique_place(target: &mut Vec<Place>, place: Place) {
    if !target.contains(&place) {
        target.push(place);
    }
}

pub(super) fn push_unique_places(target: &mut Vec<Place>, source: &[Place]) {
    for place in source {
        if !target.contains(place) {
            target.push(place.clone());
        }
    }
}
