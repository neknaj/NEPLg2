use super::model::{Place, PlaceProjection, ResourceOffset};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn place_covers_slot(slot: &Place, storage: &Place) -> bool {
    place_suffix_after_prefix(slot, storage).is_some()
}

pub(super) fn slot_requires_range_proof(slot: &Place, storage: &Place) -> bool {
    place_suffix_after_prefix(slot, storage)
        .map(|suffix| suffix.iter().any(projection_requires_range_proof))
        .unwrap_or(false)
}

pub(super) fn same_collection_slot_identity(left: &Place, right: &Place) -> bool {
    left.root == right.root && left.projections == right.projections
}

fn projection_requires_range_proof(projection: &PlaceProjection) -> bool {
    matches!(
        projection,
        PlaceProjection::StorageOffset(
            ResourceOffset::Symbolic { .. }
                | ResourceOffset::ScaledSymbolic { .. }
                | ResourceOffset::Unknown
        )
    )
}
