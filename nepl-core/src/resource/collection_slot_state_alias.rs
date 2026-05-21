extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_state_identity::place_covers_slot;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::raw_cell_value_flow_alias_candidates::raw_address_alias_candidates;

pub(super) fn storage_aliases_for_place(
    storage: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Vec<Place> {
    raw_address_alias_candidates(storage, raw_aliases)
}

pub(super) fn storage_alias_covering_slot(
    slot: &Place,
    storage: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Option<Place> {
    storage_aliases_for_place(storage, raw_aliases)
        .into_iter()
        .find(|storage| place_covers_slot(slot, storage))
}

pub(super) fn place_covers_slot_with_aliases(
    slot: &Place,
    storage: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    storage_alias_covering_slot(slot, storage, raw_aliases).is_some()
}
