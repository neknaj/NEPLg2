extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_state_alias::place_covers_slot_with_aliases;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

pub(super) fn collection_slot_drop_traversal_slots(
    collection_slots: &CollectionSlotStateTable,
    raw_aliases: &RawCellAddressAliases,
    storage: &Place,
) -> Vec<(Place, CollectionSlotState)> {
    collection_slots
        .entries()
        .iter()
        .filter_map(|entry| {
            if !place_covers_slot_with_aliases(&entry.slot, storage, raw_aliases) {
                return None;
            }
            Some((entry.slot.clone(), entry.state))
        })
        .collect()
}
