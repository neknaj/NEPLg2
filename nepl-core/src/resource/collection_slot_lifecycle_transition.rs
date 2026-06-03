use crate::types::{TypeCtx, TypeId};

use super::collection_slot_lifecycle_model::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    CollectionSlotReplacement, CollectionSlotState,
};
use super::drop_requirement::resource_type_needs_drop_code;
use super::type_pattern::type_pattern_matches;

pub fn apply_collection_slot_lifecycle_event(
    types: &TypeCtx,
    state: CollectionSlotState,
    event: CollectionSlotLifecycleEvent,
) -> Result<CollectionSlotState, CollectionSlotLifecycleRefutation> {
    match event {
        CollectionSlotLifecycleEvent::InitializeEmpty { value_ty } => {
            initialize_vacant_slot(state, value_ty)
        }
        CollectionSlotLifecycleEvent::BorrowRead { expected_ty } => {
            borrow_read_slot_state(types, state, expected_ty)
        }
        CollectionSlotLifecycleEvent::MoveOut { expected_ty } => initialized_slot_type(
            types,
            state,
            expected_ty,
            CollectionSlotLifecycleOp::MoveOut,
        )
        .map(CollectionSlotState::Moved),
        CollectionSlotLifecycleEvent::ReplaceInitialized {
            old_ty,
            new_ty,
            old_owner,
        } => replace_initialized_slot_type(
            types,
            state,
            old_ty,
            old_owner,
            CollectionSlotLifecycleOp::ReplaceInitialized,
        )
        .map(|_| CollectionSlotState::Initialized(new_ty)),
        CollectionSlotLifecycleEvent::DropInitialized { expected_ty } => initialized_slot_type(
            types,
            state,
            expected_ty,
            CollectionSlotLifecycleOp::DropInitialized,
        )
        .map(CollectionSlotState::Dropped),
        CollectionSlotLifecycleEvent::StorageDealloc { .. } => storage_dealloc_slot(state),
    }
}

fn replace_initialized_slot_type(
    types: &TypeCtx,
    state: CollectionSlotState,
    expected_ty: TypeId,
    old_owner: CollectionSlotReplacement,
    operation: CollectionSlotLifecycleOp,
) -> Result<TypeId, CollectionSlotLifecycleRefutation> {
    match state {
        // no-drop payload の DropOld replacement は、旧値の destructor や owner 返却を
        // 必要としない。loop induction などで個別 slot が MaybeInitialized に合流しても、
        // 型が一致し、旧 payload に drop code が不要なら、store 後の initialized state へ
        // 進められる。
        CollectionSlotState::MaybeInitialized(Some(actual))
            if matches!(old_owner, CollectionSlotReplacement::DropOldOwner)
                && collection_slot_payload_types_match(types, actual, expected_ty)
                && !resource_type_needs_drop_code(types, actual) =>
        {
            Ok(actual)
        }
        _ => initialized_slot_type(types, state, expected_ty, operation),
    }
}

fn initialize_vacant_slot(
    state: CollectionSlotState,
    value_ty: TypeId,
) -> Result<CollectionSlotState, CollectionSlotLifecycleRefutation> {
    match state {
        CollectionSlotState::Uninitialized
        | CollectionSlotState::Moved(_)
        | CollectionSlotState::Dropped(_) => Ok(CollectionSlotState::Initialized(value_ty)),
        CollectionSlotState::Initialized(slot_ty) => {
            Err(CollectionSlotLifecycleRefutation::LiveSlotOverwrite { slot_ty })
        }
        CollectionSlotState::MaybeInitialized(slot_ty) => {
            Err(CollectionSlotLifecycleRefutation::MaybeLiveSlotOverwrite { slot_ty })
        }
        CollectionSlotState::Released | CollectionSlotState::MaybeReleased => {
            Err(CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::InitializeEmpty,
                state,
            })
        }
    }
}

fn storage_dealloc_slot(
    state: CollectionSlotState,
) -> Result<CollectionSlotState, CollectionSlotLifecycleRefutation> {
    match state {
        CollectionSlotState::Initialized(slot_ty) => {
            Err(CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc { slot_ty })
        }
        CollectionSlotState::MaybeInitialized(slot_ty) => {
            Err(CollectionSlotLifecycleRefutation::MaybeLiveSlotDuringStorageDealloc { slot_ty })
        }
        CollectionSlotState::Uninitialized
        | CollectionSlotState::Moved(_)
        | CollectionSlotState::Dropped(_)
        | CollectionSlotState::Released => Ok(CollectionSlotState::Released),
        CollectionSlotState::MaybeReleased => Err(CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::StorageDealloc,
            state,
        }),
    }
}

fn initialized_slot_type(
    types: &TypeCtx,
    state: CollectionSlotState,
    expected_ty: TypeId,
    operation: CollectionSlotLifecycleOp,
) -> Result<TypeId, CollectionSlotLifecycleRefutation> {
    match state {
        CollectionSlotState::Initialized(actual)
            if collection_slot_payload_types_match(types, actual, expected_ty) =>
        {
            Ok(actual)
        }
        CollectionSlotState::Initialized(actual) => {
            Err(CollectionSlotLifecycleRefutation::TypeMismatch {
                operation,
                expected: expected_ty,
                actual,
            })
        }
        CollectionSlotState::Uninitialized
        | CollectionSlotState::MaybeInitialized(_)
        | CollectionSlotState::Moved(_)
        | CollectionSlotState::Dropped(_)
        | CollectionSlotState::Released
        | CollectionSlotState::MaybeReleased => {
            Err(CollectionSlotLifecycleRefutation::Unavailable { operation, state })
        }
    }
}

fn borrow_read_slot_state(
    types: &TypeCtx,
    state: CollectionSlotState,
    expected_ty: TypeId,
) -> Result<CollectionSlotState, CollectionSlotLifecycleRefutation> {
    match state {
        CollectionSlotState::Initialized(actual)
            if collection_slot_payload_types_match(types, actual, expected_ty) =>
        {
            Ok(CollectionSlotState::Initialized(actual))
        }
        CollectionSlotState::MaybeInitialized(Some(actual))
            if collection_slot_payload_types_match(types, actual, expected_ty) =>
        {
            Ok(CollectionSlotState::MaybeInitialized(Some(actual)))
        }
        CollectionSlotState::Initialized(actual)
        | CollectionSlotState::MaybeInitialized(Some(actual)) => {
            Err(CollectionSlotLifecycleRefutation::TypeMismatch {
                operation: CollectionSlotLifecycleOp::BorrowRead,
                expected: expected_ty,
                actual,
            })
        }
        CollectionSlotState::Uninitialized
        | CollectionSlotState::MaybeInitialized(None)
        | CollectionSlotState::Moved(_)
        | CollectionSlotState::Dropped(_)
        | CollectionSlotState::Released
        | CollectionSlotState::MaybeReleased => {
            Err(CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::BorrowRead,
                state,
            })
        }
    }
}

fn collection_slot_payload_types_match(types: &TypeCtx, actual: TypeId, expected: TypeId) -> bool {
    actual == expected
        || type_pattern_matches(types, actual, expected)
        || type_pattern_matches(types, expected, actual)
}
