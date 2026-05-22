use alloc::vec::Vec;

use super::initialized_alias::ProjectedRawCellAddressAlias;
use super::model::Place;

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
