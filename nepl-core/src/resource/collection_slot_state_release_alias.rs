use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use super::collection_slot_state_alias::{
    place_covers_slot_with_aliases, storage_alias_covering_slot, storage_aliases_for_place,
};
use super::collection_slot_state_identity::{place_covers_slot, slot_requires_range_proof};
use super::collection_slot_state_release::{
    collection_slot_state_type, storage_release_slot_precondition, unavailable_storage_dealloc,
};
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::{push_unique_place, should_track};

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

    pub(super) fn storage_release_precondition_with_aliases(
        &self,
        storage: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) -> Result<(), CollectionSlotTableRefutation> {
        if !should_track(storage) {
            return Err(unavailable_storage_dealloc(
                storage,
                CollectionSlotState::Uninitialized,
            ));
        }
        let aliases = storage_aliases_for_place(storage, raw_aliases);
        if self.released_storage.iter().any(|released| {
            aliases
                .iter()
                .any(|storage| place_covers_slot(storage, released))
        }) {
            return Ok(());
        }
        if self.maybe_released_storage.iter().any(|released| {
            aliases
                .iter()
                .any(|storage| place_covers_slot(storage, released))
        }) {
            return Err(unavailable_storage_dealloc(
                storage,
                CollectionSlotState::MaybeReleased,
            ));
        }
        for entry in self
            .slots
            .iter()
            .filter(|entry| place_covers_slot_with_aliases(&entry.slot, storage, raw_aliases))
        {
            let storage_alias = storage_alias_covering_slot(&entry.slot, storage, raw_aliases)
                .unwrap_or_else(|| storage.clone());
            if slot_requires_range_proof(&entry.slot, &storage_alias) {
                return Err(CollectionSlotTableRefutation {
                    slot: entry.slot.clone(),
                    reason: CollectionSlotLifecycleRefutation::RangeProofRequired {
                        operation: CollectionSlotLifecycleOp::StorageDealloc,
                        slot_ty: collection_slot_state_type(entry.state),
                    },
                });
            }
            storage_release_slot_precondition(entry.state).map_err(|reason| {
                CollectionSlotTableRefutation {
                    slot: entry.slot.clone(),
                    reason,
                }
            })?;
        }
        Ok(())
    }

    fn storage_release_has_collection_state_with_aliases(
        &self,
        storage: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) -> bool {
        let aliases = storage_aliases_for_place(storage, raw_aliases);
        self.slots
            .iter()
            .any(|entry| place_covers_slot_with_aliases(&entry.slot, storage, raw_aliases))
            || self.released_storage.iter().any(|released| {
                aliases
                    .iter()
                    .any(|storage| place_covers_slot(storage, released))
            })
            || self.maybe_released_storage.iter().any(|released| {
                aliases
                    .iter()
                    .any(|storage| place_covers_slot(storage, released))
            })
    }
}
