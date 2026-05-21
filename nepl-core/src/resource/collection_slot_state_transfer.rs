use alloc::vec::Vec;

use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use super::collection_slot_state_identity::place_covers_slot;
use super::collection_slot_state_table::{
    CollectionSlotStateEntry, CollectionSlotStateTable, CollectionSlotTableRefutation,
};
use super::model::Place;
use super::place_utils::{push_unique_place, replace_place_prefix, should_track};

impl CollectionSlotStateTable {
    pub(super) fn transfer_storage_prefix(
        &mut self,
        source: &Place,
        target: &Place,
    ) -> Result<(), CollectionSlotTableRefutation> {
        if source == target {
            return Ok(());
        }
        if !should_track(source) {
            return Ok(());
        }
        if !should_track(target) {
            self.clear_storage_prefix(source);
            return Ok(());
        }
        self.require_transfer_target_vacant(source, target)?;

        let moved_entries = self.entries_under_prefix(source, target)?;
        let released_storage = transfer_storage_markers(&self.released_storage, source, target);
        let maybe_released_storage =
            transfer_storage_markers(&self.maybe_released_storage, source, target);
        self.clear_storage_prefix(source);
        for entry in moved_entries {
            self.set_slot_state(&entry.slot, entry.state);
        }
        self.released_storage = released_storage;
        self.maybe_released_storage = maybe_released_storage;
        Ok(())
    }

    pub(super) fn entries_covered_by_storage(
        &self,
        storage: &Place,
    ) -> Vec<CollectionSlotStateEntry> {
        self.slots
            .iter()
            .filter(|entry| place_covers_slot(&entry.slot, storage))
            .cloned()
            .collect()
    }

    pub(super) fn clear_storage_prefix(&mut self, storage: &Place) {
        self.slots
            .retain(|entry| !place_covers_slot(&entry.slot, storage));
        self.released_storage
            .retain(|released| !place_covers_slot(released, storage));
        self.maybe_released_storage
            .retain(|released| !place_covers_slot(released, storage));
    }

    fn require_transfer_target_vacant(
        &self,
        source: &Place,
        target: &Place,
    ) -> Result<(), CollectionSlotTableRefutation> {
        for entry in self.slots.iter().filter(|entry| {
            place_covers_slot(&entry.slot, target) && !place_covers_slot(&entry.slot, source)
        }) {
            return Err(value_transfer_refutation(&entry.slot, entry.state));
        }
        if let Some(released) = self
            .released_storage
            .iter()
            .find(|released| place_covers_slot(released, target))
        {
            return Err(value_transfer_refutation(
                released,
                CollectionSlotState::Released,
            ));
        }
        if let Some(released) = self
            .maybe_released_storage
            .iter()
            .find(|released| place_covers_slot(released, target))
        {
            return Err(value_transfer_refutation(
                released,
                CollectionSlotState::MaybeReleased,
            ));
        }
        Ok(())
    }

    fn entries_under_prefix(
        &self,
        source: &Place,
        target: &Place,
    ) -> Result<Vec<CollectionSlotStateEntry>, CollectionSlotTableRefutation> {
        let mut entries = Vec::new();
        for entry in self
            .slots
            .iter()
            .filter(|entry| place_covers_slot(&entry.slot, source))
        {
            let Some(slot) = replace_place_prefix(&entry.slot, source, target) else {
                return Err(value_transfer_refutation(
                    &entry.slot,
                    CollectionSlotState::Uninitialized,
                ));
            };
            entries.push(CollectionSlotStateEntry {
                slot,
                state: entry.state,
            });
        }
        Ok(entries)
    }
}

fn transfer_storage_markers(markers: &[Place], source: &Place, target: &Place) -> Vec<Place> {
    let mut out = Vec::new();
    for marker in markers {
        let moved = replace_place_prefix(marker, source, target).unwrap_or_else(|| marker.clone());
        push_unique_place(&mut out, &moved);
    }
    out
}

fn value_transfer_refutation(
    slot: &Place,
    state: CollectionSlotState,
) -> CollectionSlotTableRefutation {
    CollectionSlotTableRefutation {
        slot: slot.clone(),
        reason: CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::ValueTransfer,
            state,
        },
    }
}
