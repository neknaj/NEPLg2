use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use super::collection_slot_state_identity::{place_covers_slot, slot_requires_range_proof};
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::model::Place;
use super::place_utils::{push_unique_place, should_track};

impl CollectionSlotStateTable {
    pub fn release_storage(
        &mut self,
        storage: &Place,
    ) -> Result<(), CollectionSlotTableRefutation> {
        self.storage_release_precondition(storage)?;
        if self
            .released_storage
            .iter()
            .any(|released| place_covers_slot(storage, released))
        {
            return Ok(());
        }
        self.slots
            .retain(|entry| !place_covers_slot(&entry.slot, storage));
        self.initialized_ranges
            .retain(|entry| !place_covers_slot(&entry.storage, storage));
        self.maybe_initialized_ranges
            .retain(|entry| !place_covers_slot(&entry.storage, storage));
        self.maybe_released_storage
            .retain(|released| !place_covers_slot(released, storage));
        push_unique_place(&mut self.released_storage, storage);
        Ok(())
    }

    pub fn storage_release_precondition(
        &self,
        storage: &Place,
    ) -> Result<(), CollectionSlotTableRefutation> {
        if !should_track(storage) {
            return Err(unavailable_storage_dealloc(
                storage,
                CollectionSlotState::Uninitialized,
            ));
        }
        if self
            .released_storage
            .iter()
            .any(|released| place_covers_slot(storage, released))
        {
            return Ok(());
        }
        if self
            .maybe_released_storage
            .iter()
            .any(|released| place_covers_slot(storage, released))
        {
            return Err(unavailable_storage_dealloc(
                storage,
                CollectionSlotState::MaybeReleased,
            ));
        }
        for entry in self
            .slots
            .iter()
            .filter(|entry| place_covers_slot(&entry.slot, storage))
        {
            if slot_requires_range_proof(&entry.slot, storage) {
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
        for entry in self
            .initialized_ranges()
            .iter()
            .filter(|entry| place_covers_slot(&entry.storage, storage))
        {
            return Err(CollectionSlotTableRefutation {
                slot: entry.storage.clone(),
                reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc {
                    slot_ty: entry.value_ty,
                },
            });
        }
        for entry in self
            .maybe_initialized_ranges()
            .iter()
            .filter(|entry| place_covers_slot(&entry.storage, storage))
        {
            return Err(CollectionSlotTableRefutation {
                slot: entry.storage.clone(),
                reason: CollectionSlotLifecycleRefutation::MaybeLiveSlotDuringStorageDealloc {
                    slot_ty: Some(entry.value_ty),
                },
            });
        }
        Ok(())
    }

    pub(super) fn storage_release_covers_slot(&self, slot: &Place) -> bool {
        self.released_storage
            .iter()
            .any(|storage| place_covers_slot(slot, storage))
    }

    pub(super) fn storage_maybe_release_covers_slot(&self, slot: &Place) -> bool {
        self.maybe_released_storage
            .iter()
            .any(|storage| place_covers_slot(slot, storage))
    }
}

pub(super) fn collection_slot_state_type(
    state: CollectionSlotState,
) -> Option<crate::types::TypeId> {
    match state {
        CollectionSlotState::Initialized(slot_ty)
        | CollectionSlotState::Moved(slot_ty)
        | CollectionSlotState::Dropped(slot_ty) => Some(slot_ty),
        CollectionSlotState::MaybeInitialized(slot_ty) => slot_ty,
        CollectionSlotState::Uninitialized
        | CollectionSlotState::Released
        | CollectionSlotState::MaybeReleased => None,
    }
}

pub(super) fn storage_release_slot_precondition(
    state: CollectionSlotState,
) -> Result<(), CollectionSlotLifecycleRefutation> {
    match state {
        CollectionSlotState::Initialized(slot_ty) => {
            Err(CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc { slot_ty })
        }
        CollectionSlotState::MaybeInitialized(slot_ty) => {
            Err(CollectionSlotLifecycleRefutation::MaybeLiveSlotDuringStorageDealloc { slot_ty })
        }
        CollectionSlotState::MaybeReleased => Err(CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::StorageDealloc,
            state: CollectionSlotState::MaybeReleased,
        }),
        CollectionSlotState::Uninitialized
        | CollectionSlotState::Moved(_)
        | CollectionSlotState::Dropped(_)
        | CollectionSlotState::Released => Ok(()),
    }
}

pub(super) fn unavailable_storage_dealloc(
    storage: &Place,
    state: CollectionSlotState,
) -> CollectionSlotTableRefutation {
    CollectionSlotTableRefutation {
        slot: storage.clone(),
        reason: CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::StorageDealloc,
            state,
        },
    }
}
