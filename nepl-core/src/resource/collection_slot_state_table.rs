use alloc::vec::Vec;

use super::collection_slot_lifecycle::{
    apply_collection_slot_lifecycle_event, CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp,
    CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use super::collection_slot_state_identity::same_collection_slot_identity;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::should_track;
use super::raw_cell_value_flow_alias::{
    raw_cell_place_alias_candidates, raw_cell_places_equivalent,
};
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

    pub(super) fn state_with_aliases(
        &self,
        slot: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) -> CollectionSlotState {
        let candidates = raw_cell_place_alias_candidates(slot, raw_aliases);
        if candidates
            .iter()
            .any(|candidate| self.storage_release_covers_slot(candidate))
        {
            return CollectionSlotState::Released;
        }
        if candidates
            .iter()
            .any(|candidate| self.storage_maybe_release_covers_slot(candidate))
        {
            return CollectionSlotState::MaybeReleased;
        }
        self.slots
            .iter()
            .find(|entry| slot_matches_alias_candidates(&entry.slot, &candidates, raw_aliases))
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

    pub(super) fn apply_slot_event_with_aliases(
        &mut self,
        types: &TypeCtx,
        slot: &Place,
        raw_aliases: &RawCellAddressAliases,
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
        let state = self.state_with_aliases(slot, raw_aliases);
        let next =
            apply_collection_slot_lifecycle_event(types, state, event).map_err(|reason| {
                CollectionSlotTableRefutation {
                    slot: slot.clone(),
                    reason,
                }
            })?;
        self.set_slot_state_with_aliases(slot, raw_aliases, next);
        Ok(next)
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

    fn set_slot_state_with_aliases(
        &mut self,
        slot: &Place,
        raw_aliases: &RawCellAddressAliases,
        state: CollectionSlotState,
    ) {
        let candidates = raw_cell_place_alias_candidates(slot, raw_aliases);
        if matches!(state, CollectionSlotState::Uninitialized) {
            self.slots.retain(|entry| {
                !slot_matches_alias_candidates(&entry.slot, &candidates, raw_aliases)
            });
            return;
        }
        if let Some(entry) = self
            .slots
            .iter_mut()
            .find(|entry| slot_matches_alias_candidates(&entry.slot, &candidates, raw_aliases))
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

fn slot_matches_alias_candidates(
    slot: &Place,
    candidates: &[Place],
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let slot_candidates = raw_cell_place_alias_candidates(slot, raw_aliases);
    slot_candidates.iter().any(|slot_candidate| {
        candidates.iter().any(|candidate| {
            same_collection_slot_identity(slot_candidate, candidate)
                || raw_cell_places_equivalent(slot_candidate, candidate)
        })
    })
}
