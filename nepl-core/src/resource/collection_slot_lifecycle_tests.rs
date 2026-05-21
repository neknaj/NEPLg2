use super::collection_slot_lifecycle::*;
use crate::types::{TypeCtx, TypeId};

fn test_types() -> (TypeCtx, TypeId, TypeId) {
    let types = TypeCtx::new();
    let owned = types.i32();
    let other = types.u8();
    (types, owned, other)
}

#[test]
fn initialized_slot_can_be_borrowed_without_state_change() {
    let (types, owned, _) = test_types();
    let state = apply_collection_slot_lifecycle_event(
        &types,
        CollectionSlotState::Initialized(owned),
        CollectionSlotLifecycleEvent::BorrowRead { expected_ty: owned },
    );

    assert_eq!(state, Ok(CollectionSlotState::Initialized(owned)));
}

#[test]
fn move_out_marks_slot_moved_and_rejects_second_move() {
    let (types, owned, _) = test_types();
    let moved = apply_collection_slot_lifecycle_event(
        &types,
        CollectionSlotState::Initialized(owned),
        CollectionSlotLifecycleEvent::MoveOut { expected_ty: owned },
    )
    .expect("first move should consume the slot owner");
    let second = apply_collection_slot_lifecycle_event(
        &types,
        moved,
        CollectionSlotLifecycleEvent::MoveOut { expected_ty: owned },
    );

    assert_eq!(
        second,
        Err(CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::MoveOut,
            state: CollectionSlotState::Moved(owned),
        })
    );
}

#[test]
fn initialize_empty_reuses_moved_slot_but_rejects_live_overwrite() {
    let (types, owned, other) = test_types();
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            &types,
            CollectionSlotState::Moved(owned),
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: other },
        ),
        Ok(CollectionSlotState::Initialized(other))
    );
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            &types,
            CollectionSlotState::Initialized(owned),
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: other },
        ),
        Err(CollectionSlotLifecycleRefutation::LiveSlotOverwrite { slot_ty: owned })
    );
}

#[test]
fn replace_initialized_requires_matching_old_type() {
    let (types, owned, other) = test_types();
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            &types,
            CollectionSlotState::Initialized(owned),
            CollectionSlotLifecycleEvent::ReplaceInitialized {
                old_ty: owned,
                new_ty: other,
                old_owner: CollectionSlotReplacement::ReturnOldOwner,
            },
        ),
        Ok(CollectionSlotState::Initialized(other))
    );
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            &types,
            CollectionSlotState::Initialized(owned),
            CollectionSlotLifecycleEvent::ReplaceInitialized {
                old_ty: other,
                new_ty: other,
                old_owner: CollectionSlotReplacement::DropOldOwner,
            },
        ),
        Err(CollectionSlotLifecycleRefutation::TypeMismatch {
            operation: CollectionSlotLifecycleOp::ReplaceInitialized,
            expected: other,
            actual: owned,
        })
    );
}

#[test]
fn drop_marks_slot_dropped_and_rejects_double_drop() {
    let (types, owned, _) = test_types();
    let dropped = apply_collection_slot_lifecycle_event(
        &types,
        CollectionSlotState::Initialized(owned),
        CollectionSlotLifecycleEvent::DropInitialized { expected_ty: owned },
    )
    .expect("first drop should consume the slot owner");
    let second = apply_collection_slot_lifecycle_event(
        &types,
        dropped,
        CollectionSlotLifecycleEvent::DropInitialized { expected_ty: owned },
    );

    assert_eq!(
        second,
        Err(CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::DropInitialized,
            state: CollectionSlotState::Dropped(owned),
        })
    );
}

#[test]
fn released_slot_rejects_later_initialization() {
    let (types, owned, _) = test_types();
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            &types,
            CollectionSlotState::Released,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        ),
        Err(CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::InitializeEmpty,
            state: CollectionSlotState::Released,
        })
    );
}
