use alloc::vec::Vec;

use super::collection_slot_lifecycle::{
    apply_collection_slot_lifecycle_event, CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp,
    CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use super::model::Place;
use super::place_utils::{place_suffix_after_prefix, push_unique_place, should_track};
use crate::types::TypeCtx;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionSlotStateEntry {
    pub slot: Place,
    pub state: CollectionSlotState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectionSlotTableRefutation {
    pub slot: Place,
    pub reason: CollectionSlotLifecycleRefutation,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CollectionSlotStateTable {
    pub(super) slots: Vec<CollectionSlotStateEntry>,
    pub(super) released_storage: Vec<Place>,
    pub(super) maybe_released_storage: Vec<Place>,
}

impl CollectionSlotStateTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entries(&self) -> &[CollectionSlotStateEntry] {
        &self.slots
    }

    pub fn released_storage(&self) -> &[Place] {
        &self.released_storage
    }

    pub fn maybe_released_storage(&self) -> &[Place] {
        &self.maybe_released_storage
    }

    pub fn state(&self, slot: &Place) -> CollectionSlotState {
        if self.storage_release_covers_slot(slot) {
            return CollectionSlotState::Released;
        }
        if self.storage_maybe_release_covers_slot(slot) {
            return CollectionSlotState::MaybeReleased;
        }
        self.slots
            .iter()
            .find(|entry| same_collection_slot_identity(&entry.slot, slot))
            .map(|entry| entry.state)
            .unwrap_or(CollectionSlotState::Uninitialized)
    }

    pub fn apply_slot_event(
        &mut self,
        types: &TypeCtx,
        slot: &Place,
        event: CollectionSlotLifecycleEvent,
    ) -> Result<CollectionSlotState, CollectionSlotTableRefutation> {
        if !should_track(slot) {
            return Err(CollectionSlotTableRefutation {
                slot: slot.clone(),
                reason: CollectionSlotLifecycleRefutation::Unavailable {
                    operation: collection_slot_event_operation(event),
                    state: CollectionSlotState::Uninitialized,
                },
            });
        }
        let state = self.state(slot);
        let next =
            apply_collection_slot_lifecycle_event(types, state, event).map_err(|reason| {
                CollectionSlotTableRefutation {
                    slot: slot.clone(),
                    reason,
                }
            })?;
        self.set_slot_state(slot, next);
        Ok(next)
    }

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
            return Err(CollectionSlotTableRefutation {
                slot: storage.clone(),
                reason: CollectionSlotLifecycleRefutation::Unavailable {
                    operation: CollectionSlotLifecycleOp::StorageDealloc,
                    state: CollectionSlotState::Uninitialized,
                },
            });
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
            return Err(CollectionSlotTableRefutation {
                slot: storage.clone(),
                reason: CollectionSlotLifecycleRefutation::Unavailable {
                    operation: CollectionSlotLifecycleOp::StorageDealloc,
                    state: CollectionSlotState::MaybeReleased,
                },
            });
        }
        for entry in self
            .slots
            .iter()
            .filter(|entry| place_covers_slot(&entry.slot, storage))
        {
            match entry.state {
                CollectionSlotState::Initialized(slot_ty) => {
                    return Err(CollectionSlotTableRefutation {
                        slot: entry.slot.clone(),
                        reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc {
                            slot_ty,
                        },
                    });
                }
                CollectionSlotState::MaybeInitialized(slot_ty) => {
                    return Err(CollectionSlotTableRefutation {
                        slot: entry.slot.clone(),
                        reason:
                            CollectionSlotLifecycleRefutation::MaybeLiveSlotDuringStorageDealloc {
                                slot_ty,
                            },
                    });
                }
                CollectionSlotState::MaybeReleased => {
                    return Err(CollectionSlotTableRefutation {
                        slot: entry.slot.clone(),
                        reason: CollectionSlotLifecycleRefutation::Unavailable {
                            operation: CollectionSlotLifecycleOp::StorageDealloc,
                            state: CollectionSlotState::MaybeReleased,
                        },
                    });
                }
                CollectionSlotState::Uninitialized
                | CollectionSlotState::Moved(_)
                | CollectionSlotState::Dropped(_)
                | CollectionSlotState::Released => {}
            }
        }
        Ok(())
    }

    pub(super) fn set_slot_state(&mut self, slot: &Place, state: CollectionSlotState) {
        if matches!(state, CollectionSlotState::Uninitialized) {
            self.slots
                .retain(|entry| !same_collection_slot_identity(&entry.slot, slot));
            return;
        }
        if let Some(entry) = self
            .slots
            .iter_mut()
            .find(|entry| same_collection_slot_identity(&entry.slot, slot))
        {
            entry.slot = slot.clone();
            entry.state = state;
        } else {
            self.slots.push(CollectionSlotStateEntry {
                slot: slot.clone(),
                state,
            });
        }
    }

    fn storage_release_covers_slot(&self, slot: &Place) -> bool {
        self.released_storage
            .iter()
            .any(|storage| place_covers_slot(slot, storage))
    }

    fn storage_maybe_release_covers_slot(&self, slot: &Place) -> bool {
        self.maybe_released_storage
            .iter()
            .any(|storage| place_covers_slot(slot, storage))
    }
}

fn collection_slot_event_operation(
    event: CollectionSlotLifecycleEvent,
) -> CollectionSlotLifecycleOp {
    match event {
        CollectionSlotLifecycleEvent::InitializeEmpty { .. } => {
            CollectionSlotLifecycleOp::InitializeEmpty
        }
        CollectionSlotLifecycleEvent::BorrowRead { .. } => CollectionSlotLifecycleOp::BorrowRead,
        CollectionSlotLifecycleEvent::MoveOut { .. } => CollectionSlotLifecycleOp::MoveOut,
        CollectionSlotLifecycleEvent::ReplaceInitialized { .. } => {
            CollectionSlotLifecycleOp::ReplaceInitialized
        }
        CollectionSlotLifecycleEvent::DropInitialized { .. } => {
            CollectionSlotLifecycleOp::DropInitialized
        }
        CollectionSlotLifecycleEvent::StorageDealloc => CollectionSlotLifecycleOp::StorageDealloc,
    }
}

pub(super) fn place_covers_slot(slot: &Place, storage: &Place) -> bool {
    place_suffix_after_prefix(slot, storage).is_some()
}

pub(super) fn same_collection_slot_identity(left: &Place, right: &Place) -> bool {
    left.root == right.root && left.projections == right.projections
}
