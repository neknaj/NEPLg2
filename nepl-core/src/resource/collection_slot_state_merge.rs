use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_state_identity::place_covers_slot;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::place_utils::push_unique_place;
use crate::types::TypeId;

impl CollectionSlotStateTable {
    pub fn merge_paths(paths: &[CollectionSlotStateTable]) -> Self {
        let mut out = Self::new();
        merge_released_storage(paths, &mut out);
        merge_initialized_ranges(paths, &mut out);

        let mut slots = Vec::new();
        for path in paths {
            for entry in &path.slots {
                push_unique_place(&mut slots, &entry.slot);
            }
        }

        for slot in slots {
            let mut states = paths
                .iter()
                .map(|path| state_with_initialized_ranges_for_merge(path, &slot));
            if let Some(mut merged) = states.next() {
                for state in states {
                    merged = merge_collection_slot_states(merged, state);
                }
                out.set_slot_state(&slot, merged);
            }
        }

        out
    }
}

fn merge_initialized_ranges(
    paths: &[CollectionSlotStateTable],
    out: &mut CollectionSlotStateTable,
) {
    let Some((first, rest)) = paths.split_first() else {
        return;
    };
    out.initialized_ranges.extend(
        first
            .initialized_ranges
            .iter()
            .filter(|entry| {
                rest.iter()
                    .all(|path| path.initialized_ranges.contains(entry))
            })
            .cloned(),
    );
}

fn merge_released_storage(paths: &[CollectionSlotStateTable], out: &mut CollectionSlotStateTable) {
    let mut storages = Vec::new();
    for path in paths {
        for storage in &path.released_storage {
            push_unique_place(&mut storages, storage);
        }
        for storage in &path.maybe_released_storage {
            push_unique_place(&mut storages, storage);
        }
    }

    for storage in storages {
        let mut saw_release = false;
        let mut all_definitely_released = !paths.is_empty();
        for path in paths {
            let definitely_released = path
                .released_storage
                .iter()
                .any(|released| place_covers_slot(&storage, released));
            let maybe_released = path
                .maybe_released_storage
                .iter()
                .any(|released| place_covers_slot(&storage, released));

            saw_release |= definitely_released || maybe_released;
            all_definitely_released &= definitely_released;
        }

        if all_definitely_released {
            push_unique_place(&mut out.released_storage, &storage);
        } else if saw_release {
            push_unique_place(&mut out.maybe_released_storage, &storage);
        }
    }
}

fn state_with_initialized_ranges_for_merge(
    path: &CollectionSlotStateTable,
    slot: &super::model::Place,
) -> CollectionSlotState {
    let state = path.state(slot);
    if !matches!(state, CollectionSlotState::Uninitialized) {
        return state;
    }
    let mut range_ty = None;
    let mut saw_range = false;
    for range in path
        .initialized_ranges
        .iter()
        .filter(|range| place_covers_slot(slot, &range.storage))
    {
        saw_range = true;
        range_ty = match range_ty {
            Some(existing) => merge_type(existing, range.value_ty),
            None => Some(range.value_ty),
        };
    }
    if saw_range {
        match range_ty {
            Some(slot_ty) => CollectionSlotState::Initialized(slot_ty),
            None => CollectionSlotState::MaybeInitialized(None),
        }
    } else {
        CollectionSlotState::Uninitialized
    }
}

pub(super) fn merge_collection_slot_states(
    left: CollectionSlotState,
    right: CollectionSlotState,
) -> CollectionSlotState {
    if left == right {
        return left;
    }
    match (left, right) {
        (CollectionSlotState::MaybeReleased, _) | (_, CollectionSlotState::MaybeReleased) => {
            CollectionSlotState::MaybeReleased
        }
        (CollectionSlotState::Released, _) | (_, CollectionSlotState::Released) => {
            CollectionSlotState::MaybeReleased
        }
        (
            CollectionSlotState::MaybeInitialized(left_ty),
            CollectionSlotState::MaybeInitialized(right_ty),
        ) => CollectionSlotState::MaybeInitialized(merge_optional_type(left_ty, right_ty)),
        (
            CollectionSlotState::MaybeInitialized(slot_ty),
            CollectionSlotState::Initialized(actual),
        )
        | (
            CollectionSlotState::Initialized(actual),
            CollectionSlotState::MaybeInitialized(slot_ty),
        ) => CollectionSlotState::MaybeInitialized(merge_optional_type_with_type(slot_ty, actual)),
        (
            CollectionSlotState::MaybeInitialized(slot_ty),
            CollectionSlotState::Uninitialized
            | CollectionSlotState::Moved(_)
            | CollectionSlotState::Dropped(_),
        )
        | (
            CollectionSlotState::Uninitialized
            | CollectionSlotState::Moved(_)
            | CollectionSlotState::Dropped(_),
            CollectionSlotState::MaybeInitialized(slot_ty),
        ) => CollectionSlotState::MaybeInitialized(slot_ty),
        (CollectionSlotState::Initialized(left_ty), CollectionSlotState::Initialized(right_ty)) => {
            CollectionSlotState::MaybeInitialized(merge_type(left_ty, right_ty))
        }
        (
            CollectionSlotState::Initialized(slot_ty),
            CollectionSlotState::Uninitialized
            | CollectionSlotState::Moved(_)
            | CollectionSlotState::Dropped(_),
        )
        | (
            CollectionSlotState::Uninitialized
            | CollectionSlotState::Moved(_)
            | CollectionSlotState::Dropped(_),
            CollectionSlotState::Initialized(slot_ty),
        ) => CollectionSlotState::MaybeInitialized(Some(slot_ty)),
        (
            CollectionSlotState::Uninitialized
            | CollectionSlotState::Moved(_)
            | CollectionSlotState::Dropped(_),
            CollectionSlotState::Uninitialized
            | CollectionSlotState::Moved(_)
            | CollectionSlotState::Dropped(_),
        ) => CollectionSlotState::Uninitialized,
    }
}

fn merge_type(left: TypeId, right: TypeId) -> Option<TypeId> {
    if left == right {
        Some(left)
    } else {
        None
    }
}

fn merge_optional_type(left: Option<TypeId>, right: Option<TypeId>) -> Option<TypeId> {
    match (left, right) {
        (Some(left), Some(right)) => merge_type(left, right),
        _ => None,
    }
}

fn merge_optional_type_with_type(left: Option<TypeId>, right: TypeId) -> Option<TypeId> {
    match left {
        Some(left) => merge_type(left, right),
        None => None,
    }
}
