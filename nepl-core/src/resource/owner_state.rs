use alloc::vec::Vec;

use super::model::{OwnerState, OwnerStateEntry, Place, PlaceProjection, StorageId};
use super::place_utils::{place_suffix_after_prefix, replace_place_prefix, should_track};

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
            .find(|entry| same_owner_place(&entry.place, place))
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
        if let Some(entry) = self
            .owners
            .iter_mut()
            .find(|entry| same_owner_place(&entry.place, place))
        {
            entry.place = place.clone();
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
                !same_owner_place(&entry.place, prefix)
                    && replace_place_prefix(&entry.place, prefix, prefix).is_some()
            })
            .cloned()
            .collect()
    }

    pub(super) fn sibling_enum_payload_places(
        &self,
        scrutinee: &Place,
        selected_variant: &str,
    ) -> Vec<Place> {
        self.owners
            .iter()
            .filter_map(|entry| {
                let suffix = place_suffix_after_prefix(&entry.place, scrutinee)?;
                let Some(PlaceProjection::EnumPayload { variant }) = suffix.first() else {
                    return None;
                };
                if variant == selected_variant {
                    None
                } else {
                    Some(entry.place.clone())
                }
            })
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

    pub(super) fn has_tracked_state_under(&self, place: &Place) -> bool {
        self.state(place).is_some() || !self.descendant_entries(place).is_empty()
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
                push_unique_owner_place(&mut places, &entry.place);
            }
        }
        for place in places {
            let mut states = paths
                .iter()
                .filter_map(|path| path.state_for_variant_merge(&place));
            if let Some(mut merged) = states.next() {
                for state in states {
                    merged = merge_owner_states(merged, state);
                }
                out.set_state(&place, merged);
            }
        }
        out
    }

    fn state_for_variant_merge(&self, place: &Place) -> Option<OwnerState> {
        if let Some(state) = self.state(place) {
            return Some(state);
        }
        if self.has_sibling_enum_payload_state(place) {
            return None;
        }
        Some(OwnerState::NoFreeObligation)
    }

    fn has_sibling_enum_payload_state(&self, place: &Place) -> bool {
        place
            .projections
            .iter()
            .enumerate()
            .any(|(index, projection)| {
                let PlaceProjection::EnumPayload { variant } = projection else {
                    return false;
                };
                self.owners.iter().any(|entry| {
                    entry.place.root == place.root
                        && entry.place.projections.len() > index
                        && entry.place.projections[..index] == place.projections[..index]
                        && matches!(
                            &entry.place.projections[index],
                            PlaceProjection::EnumPayload {
                                variant: sibling
                            } if sibling != variant
                        )
                })
            })
    }
}

fn same_owner_place(left: &Place, right: &Place) -> bool {
    left.root == right.root && left.projections == right.projections
}

fn push_unique_owner_place(places: &mut Vec<Place>, place: &Place) {
    if !places
        .iter()
        .any(|existing| same_owner_place(existing, place))
    {
        places.push(place.clone());
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
        | (OwnerState::Freed, OwnerState::NoFreeObligation)
        | (OwnerState::NoFreeObligation, OwnerState::Moved)
        | (OwnerState::Moved, OwnerState::NoFreeObligation)
        | (OwnerState::Moved, OwnerState::Freed)
        | (OwnerState::Freed, OwnerState::Moved) => OwnerState::NoFreeObligation,
        (OwnerState::NoFreeObligation, OwnerState::NoFreeObligation) => {
            OwnerState::NoFreeObligation
        }
        (OwnerState::Moved, OwnerState::Moved) => OwnerState::Moved,
        (OwnerState::Freed, OwnerState::Freed) => OwnerState::Freed,
        _ => OwnerState::MaybeFreed,
    }
}
