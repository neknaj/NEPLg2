use alloc::vec::Vec;

use super::collection_slot_lifecycle::{
    apply_collection_slot_lifecycle_event, CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp,
    CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use super::model::Place;
use super::place_utils::{place_suffix_after_prefix, push_unique_place, should_track};

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
    slots: Vec<CollectionSlotStateEntry>,
    released_storage: Vec<Place>,
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

    pub fn state(&self, slot: &Place) -> CollectionSlotState {
        if self.storage_release_covers_slot(slot) {
            return CollectionSlotState::Released;
        }
        self.slots
            .iter()
            .find(|entry| entry.slot == *slot)
            .map(|entry| entry.state)
            .unwrap_or(CollectionSlotState::Uninitialized)
    }

    pub fn apply_slot_event(
        &mut self,
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
        let next = apply_collection_slot_lifecycle_event(state, event).map_err(|reason| {
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
        for entry in self
            .slots
            .iter()
            .filter(|entry| place_covers_slot(&entry.slot, storage))
        {
            if let CollectionSlotState::Initialized(slot_ty) = entry.state {
                return Err(CollectionSlotTableRefutation {
                    slot: entry.slot.clone(),
                    reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc {
                        slot_ty,
                    },
                });
            }
        }
        self.slots
            .retain(|entry| !place_covers_slot(&entry.slot, storage));
        push_unique_place(&mut self.released_storage, storage);
        Ok(())
    }

    fn set_slot_state(&mut self, slot: &Place, state: CollectionSlotState) {
        if matches!(state, CollectionSlotState::Uninitialized) {
            self.slots.retain(|entry| entry.slot != *slot);
            return;
        }
        if let Some(entry) = self.slots.iter_mut().find(|entry| entry.slot == *slot) {
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

fn place_covers_slot(slot: &Place, storage: &Place) -> bool {
    place_suffix_after_prefix(slot, storage).is_some()
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use crate::types::TypeId;

    use super::*;
    use crate::resource::model::{PlaceProjection, ResourceOffset};

    const OWNED: TypeId = TypeId(20);
    const OTHER: TypeId = TypeId(21);

    fn storage() -> Place {
        Place::local(String::from("buffer"), OWNED)
    }

    fn slot(index: usize, ty: TypeId) -> Place {
        storage().with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::Known(index)),
            ty,
        )
    }

    #[test]
    fn table_routes_slot_events_through_lifecycle_boundary() {
        let mut table = CollectionSlotStateTable::new();
        let slot0 = slot(0, OWNED);

        assert_eq!(
            table.apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
            ),
            Ok(CollectionSlotState::Initialized(OWNED))
        );
        assert_eq!(
            table.apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::MoveOut { expected_ty: OWNED },
            ),
            Ok(CollectionSlotState::Moved(OWNED))
        );
        assert_eq!(
            table.apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::MoveOut { expected_ty: OWNED },
            ),
            Err(CollectionSlotTableRefutation {
                slot: slot0,
                reason: CollectionSlotLifecycleRefutation::Unavailable {
                    operation: CollectionSlotLifecycleOp::MoveOut,
                    state: CollectionSlotState::Moved(OWNED),
                },
            })
        );
    }

    #[test]
    fn storage_release_rejects_live_slot_and_reports_the_slot() {
        let mut table = CollectionSlotStateTable::new();
        let slot0 = slot(0, OWNED);
        table
            .apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
            )
            .expect("slot should be initialized before release");

        assert_eq!(
            table.release_storage(&storage()),
            Err(CollectionSlotTableRefutation {
                slot: slot0,
                reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc {
                    slot_ty: OWNED,
                },
            })
        );
    }

    #[test]
    fn storage_release_forgets_vacant_slots_and_blocks_later_init() {
        let mut table = CollectionSlotStateTable::new();
        let slot0 = slot(0, OWNED);
        table
            .apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
            )
            .expect("slot should initialize");
        table
            .apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::DropInitialized { expected_ty: OWNED },
            )
            .expect("slot should be dropped before storage release");

        assert_eq!(table.release_storage(&storage()), Ok(()));
        assert!(table.entries().is_empty());
        assert_eq!(table.state(&slot0), CollectionSlotState::Released);
        assert_eq!(
            table.apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OTHER },
            ),
            Err(CollectionSlotTableRefutation {
                slot: slot0,
                reason: CollectionSlotLifecycleRefutation::Unavailable {
                    operation: CollectionSlotLifecycleOp::InitializeEmpty,
                    state: CollectionSlotState::Released,
                },
            })
        );
    }
}
