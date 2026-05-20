use super::collection_slot_lifecycle::*;
use crate::types::TypeId;

const OWNED: TypeId = TypeId(10);
const OTHER: TypeId = TypeId(11);

#[test]
fn initialized_slot_can_be_borrowed_without_state_change() {
    let state = apply_collection_slot_lifecycle_event(
        CollectionSlotState::Initialized(OWNED),
        CollectionSlotLifecycleEvent::BorrowRead { expected_ty: OWNED },
    );

    assert_eq!(state, Ok(CollectionSlotState::Initialized(OWNED)));
}

#[test]
fn move_out_marks_slot_moved_and_rejects_second_move() {
    let moved = apply_collection_slot_lifecycle_event(
        CollectionSlotState::Initialized(OWNED),
        CollectionSlotLifecycleEvent::MoveOut { expected_ty: OWNED },
    )
    .expect("first move should consume the slot owner");
    let second = apply_collection_slot_lifecycle_event(
        moved,
        CollectionSlotLifecycleEvent::MoveOut { expected_ty: OWNED },
    );

    assert_eq!(
        second,
        Err(CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::MoveOut,
            state: CollectionSlotState::Moved(OWNED),
        })
    );
}

#[test]
fn initialize_empty_reuses_moved_slot_but_rejects_live_overwrite() {
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            CollectionSlotState::Moved(OWNED),
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OTHER },
        ),
        Ok(CollectionSlotState::Initialized(OTHER))
    );
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            CollectionSlotState::Initialized(OWNED),
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OTHER },
        ),
        Err(CollectionSlotLifecycleRefutation::LiveSlotOverwrite { slot_ty: OWNED })
    );
}

#[test]
fn replace_initialized_requires_matching_old_type() {
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            CollectionSlotState::Initialized(OWNED),
            CollectionSlotLifecycleEvent::ReplaceInitialized {
                old_ty: OWNED,
                new_ty: OTHER,
                old_owner: CollectionSlotReplacement::ReturnOldOwner,
            },
        ),
        Ok(CollectionSlotState::Initialized(OTHER))
    );
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            CollectionSlotState::Initialized(OWNED),
            CollectionSlotLifecycleEvent::ReplaceInitialized {
                old_ty: OTHER,
                new_ty: OTHER,
                old_owner: CollectionSlotReplacement::DropOldOwner,
            },
        ),
        Err(CollectionSlotLifecycleRefutation::TypeMismatch {
            operation: CollectionSlotLifecycleOp::ReplaceInitialized,
            expected: OTHER,
            actual: OWNED,
        })
    );
}

#[test]
fn drop_marks_slot_dropped_and_rejects_double_drop() {
    let dropped = apply_collection_slot_lifecycle_event(
        CollectionSlotState::Initialized(OWNED),
        CollectionSlotLifecycleEvent::DropInitialized { expected_ty: OWNED },
    )
    .expect("first drop should consume the slot owner");
    let second = apply_collection_slot_lifecycle_event(
        dropped,
        CollectionSlotLifecycleEvent::DropInitialized { expected_ty: OWNED },
    );

    assert_eq!(
        second,
        Err(CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::DropInitialized,
            state: CollectionSlotState::Dropped(OWNED),
        })
    );
}

#[test]
fn storage_dealloc_rejects_live_slot_and_releases_vacant_slot() {
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            CollectionSlotState::Initialized(OWNED),
            CollectionSlotLifecycleEvent::StorageDealloc,
        ),
        Err(CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc { slot_ty: OWNED })
    );
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            CollectionSlotState::Dropped(OWNED),
            CollectionSlotLifecycleEvent::StorageDealloc,
        ),
        Ok(CollectionSlotState::Released)
    );
}

#[test]
fn released_slot_rejects_later_initialization() {
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            CollectionSlotState::Released,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
        ),
        Err(CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::InitializeEmpty,
            state: CollectionSlotState::Released,
        })
    );
}
