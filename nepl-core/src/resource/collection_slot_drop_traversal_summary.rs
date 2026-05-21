extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::cell_state::CellTable;
use super::collection_slot_drop_traversal_slots::collection_slot_drop_traversal_slots;
use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

impl ResourceCheckEngine<'_> {
    pub(super) fn collection_slot_drop_traversal_certified_slots(
        &self,
        cells: &CellTable,
        collection_slots: &CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        initialized_count: &Place,
        expected_ty: TypeId,
    ) -> Option<Vec<Place>> {
        let storage = raw_aliases.canonicalize_owner_cell_address(storage);
        let initialized_count = raw_aliases.canonicalize_scalar(initialized_count);
        let slots = collection_slot_drop_traversal_slots(collection_slots, raw_aliases, &storage);
        let certified_slots: Vec<_> = slots
            .iter()
            .filter_map(|(slot, state)| match state {
                CollectionSlotState::Initialized(_) => Some(slot.clone()),
                CollectionSlotState::Uninitialized
                | CollectionSlotState::MaybeInitialized(_)
                | CollectionSlotState::Moved(_)
                | CollectionSlotState::Dropped(_)
                | CollectionSlotState::Released
                | CollectionSlotState::MaybeReleased => None,
            })
            .collect();
        if certified_slots.is_empty() {
            return None;
        }
        self.collection_slot_drop_traversal_result(
            &mut cells.clone(),
            &mut collection_slots.clone(),
            raw_aliases,
            &storage,
            &initialized_count,
            expected_ty,
        )
        .is_ok()
        .then_some(certified_slots)
    }
}
