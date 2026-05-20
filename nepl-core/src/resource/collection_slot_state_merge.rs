use alloc::vec::Vec;

use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_state_table::{place_covers_slot, CollectionSlotStateTable};
use super::place_utils::push_unique_place;
use crate::types::TypeId;

impl CollectionSlotStateTable {
    pub fn merge_paths(paths: &[CollectionSlotStateTable]) -> Self {
        let mut out = Self::new();
        merge_released_storage(paths, &mut out);

        let mut slots = Vec::new();
        for path in paths {
            for entry in &path.slots {
                push_unique_place(&mut slots, &entry.slot);
            }
        }

        for slot in slots {
            let mut states = paths.iter().map(|path| path.state(&slot));
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

fn merge_collection_slot_states(
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

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use crate::resource::collection_slot_lifecycle::{
        CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    };
    use crate::resource::collection_slot_state_table::CollectionSlotTableRefutation;
    use crate::resource::model::{Place, PlaceProjection, ResourceOffset};

    use super::*;

    const OWNED: TypeId = TypeId(30);
    const OTHER: TypeId = TypeId(31);

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
    fn merge_marks_partially_moved_slot_as_maybe_initialized() {
        let slot0 = slot(0, OWNED);
        let mut left = CollectionSlotStateTable::new();
        left.apply_slot_event(
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
        )
        .expect("slot should initialize");

        let mut right = left.clone();
        right
            .apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::MoveOut { expected_ty: OWNED },
            )
            .expect("slot should move on one path");

        let mut merged = CollectionSlotStateTable::merge_paths(&[left, right]);

        assert_eq!(
            merged.state(&slot0),
            CollectionSlotState::MaybeInitialized(Some(OWNED))
        );
        assert_eq!(
            merged.apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OTHER },
            ),
            Err(CollectionSlotTableRefutation {
                slot: slot0.clone(),
                reason: CollectionSlotLifecycleRefutation::MaybeLiveSlotOverwrite {
                    slot_ty: Some(OWNED),
                },
            })
        );
        assert_eq!(
            merged.apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::MoveOut { expected_ty: OWNED },
            ),
            Err(CollectionSlotTableRefutation {
                slot: slot0.clone(),
                reason: CollectionSlotLifecycleRefutation::Unavailable {
                    operation: CollectionSlotLifecycleOp::MoveOut,
                    state: CollectionSlotState::MaybeInitialized(Some(OWNED)),
                },
            })
        );
        assert_eq!(
            merged.release_storage(&storage()),
            Err(CollectionSlotTableRefutation {
                slot: slot0,
                reason: CollectionSlotLifecycleRefutation::MaybeLiveSlotDuringStorageDealloc {
                    slot_ty: Some(OWNED),
                },
            })
        );
    }

    #[test]
    fn merge_keeps_storage_release_uncertainty() {
        let slot0 = slot(0, OWNED);
        let mut left = CollectionSlotStateTable::new();
        left.release_storage(&storage())
            .expect("one path releases the storage");
        let right = CollectionSlotStateTable::new();

        let mut merged = CollectionSlotStateTable::merge_paths(&[left, right]);

        assert_eq!(merged.state(&slot0), CollectionSlotState::MaybeReleased);
        assert_eq!(
            merged.apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OTHER },
            ),
            Err(CollectionSlotTableRefutation {
                slot: slot0,
                reason: CollectionSlotLifecycleRefutation::Unavailable {
                    operation: CollectionSlotLifecycleOp::InitializeEmpty,
                    state: CollectionSlotState::MaybeReleased,
                },
            })
        );
        assert_eq!(
            merged.release_storage(&storage()),
            Err(CollectionSlotTableRefutation {
                slot: storage(),
                reason: CollectionSlotLifecycleRefutation::Unavailable {
                    operation: CollectionSlotLifecycleOp::StorageDealloc,
                    state: CollectionSlotState::MaybeReleased,
                },
            })
        );
    }

    #[test]
    fn merge_definitely_vacant_slots_can_be_reinitialized() {
        let slot0 = slot(0, OWNED);
        let mut left = CollectionSlotStateTable::new();
        let mut right = CollectionSlotStateTable::new();
        for path in [&mut left, &mut right] {
            path.apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
            )
            .expect("slot should initialize");
        }
        left.apply_slot_event(
            &slot0,
            CollectionSlotLifecycleEvent::MoveOut { expected_ty: OWNED },
        )
        .expect("left path moves");
        right
            .apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::DropInitialized { expected_ty: OWNED },
            )
            .expect("right path drops");

        let mut merged = CollectionSlotStateTable::merge_paths(&[left, right]);

        assert_eq!(merged.state(&slot0), CollectionSlotState::Uninitialized);
        assert_eq!(
            merged.apply_slot_event(
                &slot0,
                CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OTHER },
            ),
            Ok(CollectionSlotState::Initialized(OTHER))
        );
    }
}
