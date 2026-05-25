use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_state_identity::place_covers_slot;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized_alias::RawCellAddressAliases;
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

        out.weaken_slots_described_by_maybe_ranges_with_aliases(&RawCellAddressAliases::default());
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
    let mut ranges = Vec::new();
    for path in paths {
        for entry in &path.initialized_ranges {
            push_unique_range(&mut ranges, entry);
        }
        for entry in &path.maybe_initialized_ranges {
            push_unique_range(&mut ranges, entry);
        }
    }
    out.maybe_initialized_ranges.extend(
        ranges
            .into_iter()
            .filter(|entry| !out.initialized_ranges.contains(entry)),
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
    let range_state = initialized_range_state_for_merge(path, slot);
    match (state, range_state) {
        (CollectionSlotState::Uninitialized, Some(range_state)) => return range_state,
        (CollectionSlotState::Initialized(actual), Some(range_state)) => {
            // range で支えられている個別 slot の確定性は、その range count を
            // 正にする path-local な i32 fact に依存する。合流後はその条件が
            // 弱まるため、個別 slot も range と同じ maybe state として扱う。
            return merge_range_initialized_state(actual, range_state);
        }
        (CollectionSlotState::MaybeInitialized(slot_ty), Some(range_state)) => {
            return merge_range_maybe_initialized_state(slot_ty, range_state);
        }
        (state, _) if !matches!(state, CollectionSlotState::Uninitialized) => return state,
        _ => {}
    }
    CollectionSlotState::Uninitialized
}

fn initialized_range_state_for_merge(
    path: &CollectionSlotStateTable,
    slot: &super::model::Place,
) -> Option<CollectionSlotState> {
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
    let mut maybe_range_ty = None;
    let mut saw_maybe_range = false;
    for range in path
        .maybe_initialized_ranges
        .iter()
        .filter(|range| place_covers_slot(slot, &range.storage))
    {
        saw_maybe_range = true;
        maybe_range_ty = match maybe_range_ty {
            Some(existing) => merge_type(existing, range.value_ty),
            None => Some(range.value_ty),
        };
    }
    if saw_maybe_range {
        return Some(CollectionSlotState::MaybeInitialized(merge_optional_type(
            range_ty,
            maybe_range_ty,
        )));
    }
    if saw_range {
        Some(match range_ty {
            Some(slot_ty) => CollectionSlotState::MaybeInitialized(Some(slot_ty)),
            None => CollectionSlotState::MaybeInitialized(None),
        })
    } else {
        None
    }
}

fn merge_range_initialized_state(
    actual: TypeId,
    range_state: CollectionSlotState,
) -> CollectionSlotState {
    match range_state {
        CollectionSlotState::MaybeInitialized(range_ty) => {
            CollectionSlotState::MaybeInitialized(merge_optional_type_with_type(range_ty, actual))
        }
        other => merge_collection_slot_states(CollectionSlotState::Initialized(actual), other),
    }
}

fn merge_range_maybe_initialized_state(
    slot_ty: Option<TypeId>,
    range_state: CollectionSlotState,
) -> CollectionSlotState {
    match range_state {
        CollectionSlotState::MaybeInitialized(range_ty) => {
            CollectionSlotState::MaybeInitialized(merge_optional_type(slot_ty, range_ty))
        }
        other => {
            merge_collection_slot_states(CollectionSlotState::MaybeInitialized(slot_ty), other)
        }
    }
}

fn push_unique_range(
    out: &mut Vec<super::collection_slot_state_table::CollectionSlotInitializedRangeStateEntry>,
    range: &super::collection_slot_state_table::CollectionSlotInitializedRangeStateEntry,
) {
    if !out.iter().any(|existing| existing == range) {
        out.push(range.clone());
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
