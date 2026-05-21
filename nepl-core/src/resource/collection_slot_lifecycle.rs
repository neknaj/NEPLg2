use crate::types::TypeId;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionSlotState {
    Uninitialized,
    Initialized(TypeId),
    MaybeInitialized(Option<TypeId>),
    Moved(TypeId),
    Dropped(TypeId),
    Released,
    MaybeReleased,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionSlotReplacement {
    ReturnOldOwner,
    DropOldOwner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionSlotLifecycleEvent {
    InitializeEmpty {
        value_ty: TypeId,
    },
    BorrowRead {
        expected_ty: TypeId,
    },
    MoveOut {
        expected_ty: TypeId,
    },
    ReplaceInitialized {
        old_ty: TypeId,
        new_ty: TypeId,
        old_owner: CollectionSlotReplacement,
    },
    DropInitialized {
        expected_ty: TypeId,
    },
    StorageDealloc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionSlotLifecycleOp {
    InitializeEmpty,
    BorrowRead,
    MoveOut,
    ReplaceInitialized,
    DropInitialized,
    StorageDealloc,
    StorageRelocate,
    ValueTransfer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionSlotLifecycleRefutation {
    Unavailable {
        operation: CollectionSlotLifecycleOp,
        state: CollectionSlotState,
    },
    TypeMismatch {
        operation: CollectionSlotLifecycleOp,
        expected: TypeId,
        actual: TypeId,
    },
    LiveSlotOverwrite {
        slot_ty: TypeId,
    },
    MaybeLiveSlotOverwrite {
        slot_ty: Option<TypeId>,
    },
    OwnerTransferRequiresValueProof {
        operation: CollectionSlotLifecycleOp,
        slot_ty: TypeId,
    },
    DropRequiresElaboration {
        operation: CollectionSlotLifecycleOp,
        slot_ty: TypeId,
    },
    LiveSlotDuringStorageDealloc {
        slot_ty: TypeId,
    },
    MaybeLiveSlotDuringStorageDealloc {
        slot_ty: Option<TypeId>,
    },
}

pub fn apply_collection_slot_lifecycle_event(
    state: CollectionSlotState,
    event: CollectionSlotLifecycleEvent,
) -> Result<CollectionSlotState, CollectionSlotLifecycleRefutation> {
    match event {
        CollectionSlotLifecycleEvent::InitializeEmpty { value_ty } => {
            initialize_vacant_slot(state, value_ty)
        }
        CollectionSlotLifecycleEvent::BorrowRead { expected_ty } => {
            initialized_slot_type(state, expected_ty, CollectionSlotLifecycleOp::BorrowRead)
                .map(CollectionSlotState::Initialized)
        }
        CollectionSlotLifecycleEvent::MoveOut { expected_ty } => {
            initialized_slot_type(state, expected_ty, CollectionSlotLifecycleOp::MoveOut)
                .map(CollectionSlotState::Moved)
        }
        CollectionSlotLifecycleEvent::ReplaceInitialized {
            old_ty,
            new_ty,
            old_owner,
        } => {
            match old_owner {
                CollectionSlotReplacement::ReturnOldOwner
                | CollectionSlotReplacement::DropOldOwner => {}
            }
            initialized_slot_type(state, old_ty, CollectionSlotLifecycleOp::ReplaceInitialized)
                .map(|_| CollectionSlotState::Initialized(new_ty))
        }
        CollectionSlotLifecycleEvent::DropInitialized { expected_ty } => initialized_slot_type(
            state,
            expected_ty,
            CollectionSlotLifecycleOp::DropInitialized,
        )
        .map(CollectionSlotState::Dropped),
        CollectionSlotLifecycleEvent::StorageDealloc => match state {
            CollectionSlotState::Initialized(slot_ty) => {
                Err(CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc { slot_ty })
            }
            CollectionSlotState::MaybeInitialized(slot_ty) => Err(
                CollectionSlotLifecycleRefutation::MaybeLiveSlotDuringStorageDealloc { slot_ty },
            ),
            CollectionSlotState::Uninitialized
            | CollectionSlotState::Moved(_)
            | CollectionSlotState::Dropped(_)
            | CollectionSlotState::Released => Ok(CollectionSlotState::Released),
            CollectionSlotState::MaybeReleased => {
                Err(CollectionSlotLifecycleRefutation::Unavailable {
                    operation: CollectionSlotLifecycleOp::StorageDealloc,
                    state,
                })
            }
        },
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
        CollectionSlotState::Released => Err(CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::InitializeEmpty,
            state,
        }),
        CollectionSlotState::MaybeReleased => Err(CollectionSlotLifecycleRefutation::Unavailable {
            operation: CollectionSlotLifecycleOp::InitializeEmpty,
            state,
        }),
    }
}

fn initialized_slot_type(
    state: CollectionSlotState,
    expected_ty: TypeId,
    operation: CollectionSlotLifecycleOp,
) -> Result<TypeId, CollectionSlotLifecycleRefutation> {
    match state {
        CollectionSlotState::Initialized(actual) if actual == expected_ty => Ok(actual),
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
