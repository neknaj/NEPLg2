use alloc::vec::Vec;

use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use super::collection_slot_state_table::{
    place_covers_slot, CollectionSlotStateEntry, CollectionSlotStateTable,
    CollectionSlotTableRefutation,
};
use super::model::Place;
use super::place_utils::{push_unique_place, replace_place_prefix, should_track};

impl CollectionSlotStateTable {
    pub fn relocate_storage(
        &mut self,
        old_storage: &Place,
        new_storage: &Place,
    ) -> Result<(), CollectionSlotTableRefutation> {
        if old_storage == new_storage {
            return Ok(());
        }
        self.require_relocatable_storage(old_storage)?;
        self.require_relocate_target_storage(new_storage)?;

        let moved_entries = self.entries_under_storage(old_storage, new_storage)?;
        self.slots
            .retain(|entry| !place_covers_slot(&entry.slot, old_storage));
        for entry in moved_entries {
            self.set_slot_state(&entry.slot, entry.state);
        }
        self.maybe_released_storage
            .retain(|released| !place_covers_slot(released, old_storage));
        self.released_storage
            .retain(|released| !place_covers_slot(released, old_storage));
        push_unique_place(&mut self.released_storage, old_storage);
        Ok(())
    }

    fn require_relocatable_storage(
        &self,
        storage: &Place,
    ) -> Result<(), CollectionSlotTableRefutation> {
        if !should_track(storage) {
            return Err(storage_relocate_refutation(
                storage,
                CollectionSlotState::Uninitialized,
            ));
        }
        if self
            .released_storage
            .iter()
            .any(|released| place_covers_slot(storage, released))
        {
            return Err(storage_relocate_refutation(
                storage,
                CollectionSlotState::Released,
            ));
        }
        if self
            .maybe_released_storage
            .iter()
            .any(|released| place_covers_slot(storage, released))
        {
            return Err(storage_relocate_refutation(
                storage,
                CollectionSlotState::MaybeReleased,
            ));
        }
        Ok(())
    }

    fn require_relocate_target_storage(
        &self,
        storage: &Place,
    ) -> Result<(), CollectionSlotTableRefutation> {
        if !should_track(storage) {
            return Err(storage_relocate_refutation(
                storage,
                CollectionSlotState::Uninitialized,
            ));
        }
        if self
            .released_storage
            .iter()
            .any(|released| place_covers_slot(storage, released))
        {
            return Err(storage_relocate_refutation(
                storage,
                CollectionSlotState::Released,
            ));
        }
        if self
            .maybe_released_storage
            .iter()
            .any(|released| place_covers_slot(storage, released))
        {
            return Err(storage_relocate_refutation(
                storage,
                CollectionSlotState::MaybeReleased,
            ));
        }
        for entry in self
            .slots
            .iter()
            .filter(|entry| place_covers_slot(&entry.slot, storage))
        {
            return Err(storage_relocate_refutation(&entry.slot, entry.state));
        }
        Ok(())
    }

    fn entries_under_storage(
        &self,
        old_storage: &Place,
        new_storage: &Place,
    ) -> Result<Vec<CollectionSlotStateEntry>, CollectionSlotTableRefutation> {
        let mut entries = Vec::new();
        for entry in self
            .slots
            .iter()
            .filter(|entry| place_covers_slot(&entry.slot, old_storage))
        {
            let Some(slot) = replace_place_prefix(&entry.slot, old_storage, new_storage) else {
                return Err(storage_relocate_refutation(
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

fn storage_relocate_refutation(
    slot: &Place,
    state: CollectionSlotState,
) -> CollectionSlotTableRefutation {
    CollectionSlotTableRefutation {
        slot: slot.clone(),
        reason: CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::StorageRelocate,
            state,
        },
    }
}
