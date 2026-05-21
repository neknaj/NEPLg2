use super::model::Place;
use super::place_utils::place_suffix_after_prefix;

pub(super) fn place_covers_slot(slot: &Place, storage: &Place) -> bool {
    place_suffix_after_prefix(slot, storage).is_some()
}

pub(super) fn same_collection_slot_identity(left: &Place, right: &Place) -> bool {
    left.root == right.root && left.projections == right.projections
}
