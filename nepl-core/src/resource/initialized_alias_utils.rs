use alloc::vec::Vec;

use crate::types::TypeId;

use super::initialized_alias::ProjectedRawCellAddressAlias;
use super::model::{Place, PlaceProjection};

pub(super) fn groups_overlap(left: &[Place], right: &[Place]) -> bool {
    left.iter().any(|place| right.contains(place))
}

pub(super) fn push_unique_projected_alias(
    aliases: &mut Vec<ProjectedRawCellAddressAlias>,
    alias: ProjectedRawCellAddressAlias,
) {
    if !aliases.iter().any(|existing| existing == &alias) {
        aliases.push(alias);
    }
}

pub(super) fn place_without_suffix(
    place: &Place,
    suffix: &[PlaceProjection],
    ty: TypeId,
) -> Option<Place> {
    if suffix.len() > place.projections.len() {
        return None;
    }
    let prefix_len = place.projections.len() - suffix.len();
    if place.projections[prefix_len..] != *suffix {
        return None;
    }
    let mut out = place.clone();
    out.projections.truncate(prefix_len);
    out.ty = ty;
    Some(out)
}
