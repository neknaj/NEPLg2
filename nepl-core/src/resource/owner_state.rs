use alloc::vec::Vec;

use super::model::{OwnerState, OwnerStateEntry, Place, StorageId};
use super::place_utils::{
    place_suffix_after_prefix, push_unique_place, replace_place_prefix, should_track,
};

#[derive(Debug, Clone, Default)]
pub(super) struct OwnerTable {
    owners: Vec<OwnerStateEntry>,
    next_storage: usize,
}

impl OwnerTable {
    pub(super) fn into_entries(self) -> Vec<OwnerStateEntry> {
        self.owners
    }

    pub(super) fn state(&self, place: &Place) -> Option<OwnerState> {
        self.owners
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.state.clone())
    }

    pub(super) fn allocate(&mut self, place: &Place) {
        let storage = StorageId(self.next_storage);
        self.next_storage += 1;
        self.set_state(place, OwnerState::Live { storage });
    }

    pub(super) fn set_state(&mut self, place: &Place, state: OwnerState) {
        if !should_track(place) {
            return;
        }
        if let Some(entry) = self.owners.iter_mut().find(|entry| entry.place == *place) {
            entry.state = state;
        } else {
            self.owners.push(OwnerStateEntry {
                place: place.clone(),
                state,
            });
        }
    }

    pub(super) fn live_entries(&self) -> Vec<OwnerStateEntry> {
        self.owners
            .iter()
            .filter(|entry| {
                matches!(
                    entry.state,
                    OwnerState::Live { .. } | OwnerState::MaybeFreed
                )
            })
            .cloned()
            .collect()
    }

    pub(super) fn live_entries_under(&self, prefix: &Place) -> Vec<OwnerStateEntry> {
        self.owners
            .iter()
            .filter(|entry| {
                (entry.place == *prefix
                    || place_suffix_after_prefix(&entry.place, prefix).is_some())
                    && matches!(
                        entry.state,
                        OwnerState::Live { .. } | OwnerState::MaybeFreed
                    )
            })
            .cloned()
            .collect()
    }

    pub(super) fn descendant_entries(&self, prefix: &Place) -> Vec<OwnerStateEntry> {
        self.owners
            .iter()
            .filter(|entry| {
                entry.place != *prefix
                    && replace_place_prefix(&entry.place, prefix, prefix).is_some()
            })
            .cloned()
            .collect()
    }

    pub(super) fn has_transferable_owner(&self, place: &Place) -> bool {
        self.state(place)
            .is_some_and(|state| matches!(state, OwnerState::Live { .. }))
            || self
                .descendant_entries(place)
                .iter()
                .any(|entry| matches!(entry.state, OwnerState::Live { .. }))
    }

    pub(super) fn merge_paths(paths: &[OwnerTable]) -> Self {
        let mut out = OwnerTable::default();
        out.next_storage = paths
            .iter()
            .map(|path| path.next_storage)
            .max()
            .unwrap_or_default();
        let mut places = Vec::new();
        for path in paths {
            for entry in &path.owners {
                push_unique_place(&mut places, &entry.place);
            }
        }
        for place in places {
            let mut merged = OwnerState::NoFreeObligation;
            for path in paths {
                let state = path.state(&place).unwrap_or(OwnerState::NoFreeObligation);
                merged = merge_owner_states(merged, state);
            }
            out.set_state(&place, merged);
        }
        out
    }
}

fn merge_owner_states(left: OwnerState, right: OwnerState) -> OwnerState {
    if left == right {
        return left;
    }
    match (left, right) {
        (
            OwnerState::Live {
                storage: left_storage,
            },
            OwnerState::Live {
                storage: right_storage,
            },
        ) if left_storage == right_storage => OwnerState::Live {
            storage: left_storage,
        },
        (OwnerState::NoFreeObligation, OwnerState::Freed)
        | (OwnerState::Freed, OwnerState::NoFreeObligation) => OwnerState::NoFreeObligation,
        (OwnerState::NoFreeObligation, OwnerState::NoFreeObligation) => {
            OwnerState::NoFreeObligation
        }
        (OwnerState::Moved, OwnerState::Moved) => OwnerState::Moved,
        (OwnerState::Freed, OwnerState::Freed) => OwnerState::Freed,
        _ => OwnerState::MaybeFreed,
    }
}
