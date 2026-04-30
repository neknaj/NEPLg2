use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, OwnerStateEntry, Place, PlaceProjection, StorageId};
use super::place_utils::{place_suffix_after_prefix, replace_place_prefix, should_track};

#[derive(Debug, Clone, Default)]
pub(super) struct OwnerTable {
    owners: Vec<OwnerStateEntry>,
    next_storage: usize,
}

impl OwnerTable {
    pub(super) fn entries(&self) -> &[OwnerStateEntry] {
        &self.owners
    }

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
                    OwnerState::Live { .. }
                        | OwnerState::Reserved { .. }
                        | OwnerState::MaybeFreed { .. }
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
                        OwnerState::Live { .. }
                            | OwnerState::Reserved { .. }
                            | OwnerState::MaybeFreed { .. }
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
        self.state(place).is_some_and(|state| {
            matches!(
                state,
                OwnerState::Live { .. } | OwnerState::MaybeFreed { .. }
            )
        }) || self.descendant_entries(place).iter().any(|entry| {
            matches!(
                entry.state,
                OwnerState::Live { .. } | OwnerState::MaybeFreed { .. }
            )
        })
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

    pub(super) fn merge_paths_with_raw_aliases(
        paths: &[OwnerTable],
        raw_aliases: &RawCellAddressAliases,
    ) -> Self {
        let canonical_paths = paths
            .iter()
            .map(|path| path.canonicalize_raw_owner_aliases(raw_aliases))
            .collect::<Vec<_>>();
        Self::merge_paths(&canonical_paths)
    }

    fn canonicalize_raw_owner_aliases(&self, raw_aliases: &RawCellAddressAliases) -> Self {
        let mut out = OwnerTable::default();
        out.next_storage = self.next_storage;
        for entry in &self.owners {
            let place = if place_has_raw_owner_projection(&entry.place) {
                raw_aliases.canonicalize_owner_cell_address(&entry.place)
            } else {
                entry.place.clone()
            };
            let state = match out.state(&place) {
                Some(existing) => merge_owner_states(existing, entry.state.clone()),
                None => entry.state.clone(),
            };
            out.set_state(&place, state);
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

fn place_has_raw_owner_projection(place: &Place) -> bool {
    place.projections.iter().any(|projection| {
        matches!(
            projection,
            PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
        )
    })
}

fn merge_owner_states(left: OwnerState, right: OwnerState) -> OwnerState {
    if left == right {
        return left;
    }
    match (left, right) {
        (
            OwnerState::Reserved {
                storage: left_storage,
            },
            OwnerState::Reserved {
                storage: right_storage,
            },
        ) => OwnerState::Reserved {
            storage: merge_maybe_storage(left_storage, right_storage),
        },
        (OwnerState::Reserved { storage }, _) | (_, OwnerState::Reserved { storage }) => {
            OwnerState::Reserved { storage }
        }
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
        (OwnerState::Live { storage }, OwnerState::NoFreeObligation)
        | (OwnerState::NoFreeObligation, OwnerState::Live { storage })
        | (OwnerState::Live { storage }, OwnerState::Moved)
        | (OwnerState::Moved, OwnerState::Live { storage })
        | (OwnerState::Live { storage }, OwnerState::Freed)
        | (OwnerState::Freed, OwnerState::Live { storage }) => OwnerState::MaybeFreed {
            storage: Some(storage),
        },
        (
            OwnerState::MaybeFreed {
                storage: left_storage,
            },
            OwnerState::MaybeFreed {
                storage: right_storage,
            },
        ) => OwnerState::MaybeFreed {
            storage: merge_maybe_storage(left_storage, right_storage),
        },
        (
            OwnerState::MaybeFreed {
                storage: maybe_storage,
            },
            OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed,
        )
        | (
            OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed,
            OwnerState::MaybeFreed {
                storage: maybe_storage,
            },
        ) => OwnerState::MaybeFreed {
            storage: maybe_storage,
        },
        (
            OwnerState::MaybeFreed {
                storage: maybe_storage,
            },
            OwnerState::Live { storage },
        )
        | (
            OwnerState::Live { storage },
            OwnerState::MaybeFreed {
                storage: maybe_storage,
            },
        ) => OwnerState::MaybeFreed {
            storage: merge_maybe_storage(maybe_storage, Some(storage)),
        },
        (OwnerState::NoFreeObligation, OwnerState::NoFreeObligation) => {
            OwnerState::NoFreeObligation
        }
        (OwnerState::Moved, OwnerState::Moved) => OwnerState::Moved,
        (OwnerState::Freed, OwnerState::Freed) => OwnerState::Freed,
        _ => OwnerState::MaybeFreed { storage: None },
    }
}

fn merge_maybe_storage(left: Option<StorageId>, right: Option<StorageId>) -> Option<StorageId> {
    match (left, right) {
        (Some(left), Some(right)) if left == right => Some(left),
        _ => None,
    }
}
