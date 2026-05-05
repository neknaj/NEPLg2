extern crate alloc;

use alloc::vec::Vec;

use super::model::{PlaceProjection, ResourceOffset};

/// Keep small fixed-layout offsets exact, then add a dynamic summary for
/// pointer-arithmetic patterns that would otherwise make fixed points unbounded.
pub(super) const MAX_EXACT_PROJECTION_FACTS_PER_SHAPE: usize = 16;

pub(super) fn normalize_storage_offsets(projections: Vec<PlaceProjection>) -> Vec<PlaceProjection> {
    let mut out = Vec::new();
    for projection in projections {
        match (out.last_mut(), projection) {
            (
                Some(PlaceProjection::StorageOffset(existing)),
                PlaceProjection::StorageOffset(next),
            ) => {
                *existing = combine_storage_offsets(*existing, next);
            }
            (_, projection) => out.push(projection),
        }
    }
    out
}

pub(super) fn widen_projection(
    left: &[PlaceProjection],
    right: &[PlaceProjection],
) -> Option<Vec<PlaceProjection>> {
    if left.len() != right.len() {
        return None;
    }
    let mut out = Vec::new();
    for (left, right) in left.iter().zip(right.iter()) {
        match (left, right) {
            (PlaceProjection::StorageOffset(left), PlaceProjection::StorageOffset(right)) => {
                out.push(PlaceProjection::StorageOffset(widen_storage_offsets(
                    *left, *right,
                )));
            }
            _ if left == right => out.push(left.clone()),
            _ => return None,
        }
    }
    Some(out)
}

fn combine_storage_offsets(left: ResourceOffset, right: ResourceOffset) -> ResourceOffset {
    match (left, right) {
        (ResourceOffset::Exact(left), ResourceOffset::Exact(right)) => left
            .checked_add(right)
            .map_or(ResourceOffset::Dynamic, ResourceOffset::Exact),
        (ResourceOffset::Dynamic, _) | (_, ResourceOffset::Dynamic) => ResourceOffset::Dynamic,
    }
}

fn widen_storage_offsets(left: ResourceOffset, right: ResourceOffset) -> ResourceOffset {
    if left == right {
        left
    } else {
        ResourceOffset::Dynamic
    }
}
