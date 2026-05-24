use super::collection_slot_state_alias::{
    place_covers_slot_with_aliases, storage_aliases_for_place,
};
use super::collection_slot_state_identity::place_covers_slot;
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::push_unique_place;

impl CollectionSlotStateTable {
    pub(super) fn release_storage_if_collection_tracked_with_aliases(
        &mut self,
        storage: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) -> Result<(), CollectionSlotTableRefutation> {
        if !self.storage_release_has_collection_state_with_aliases(storage, raw_aliases) {
            return Ok(());
        }
        self.release_storage_with_aliases(storage, raw_aliases)
    }

    pub(super) fn release_storage_with_aliases(
        &mut self,
        storage: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) -> Result<(), CollectionSlotTableRefutation> {
        self.storage_release_precondition_with_aliases(storage, raw_aliases)?;
        let aliases = storage_aliases_for_place(storage, raw_aliases);
        if self.released_storage.iter().any(|released| {
            aliases
                .iter()
                .any(|storage| place_covers_slot(storage, released))
        }) {
            return Ok(());
        }
        self.slots
            .retain(|entry| !place_covers_slot_with_aliases(&entry.slot, storage, raw_aliases));
        self.initialized_ranges.retain(|entry| {
            !aliases
                .iter()
                .any(|storage| place_covers_slot(&entry.storage, storage))
        });
        self.maybe_initialized_ranges.retain(|entry| {
            !aliases
                .iter()
                .any(|storage| place_covers_slot(&entry.storage, storage))
        });
        self.maybe_released_storage.retain(|released| {
            !aliases
                .iter()
                .any(|storage| place_covers_slot(released, storage))
        });
        for storage in aliases {
            push_unique_place(&mut self.released_storage, &storage);
        }
        Ok(())
    }
}
