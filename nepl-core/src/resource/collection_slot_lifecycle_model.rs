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
    StorageDealloc {
        value_ty: TypeId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CollectionSlotLifecycleOp {
    InitializeEmpty,
    BorrowRead,
    MoveOut,
    ReplaceInitialized,
    DropInitialized,
    DropTraversal,
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
    StorageRelocateRequiresRawMoveProof,
    StorageDeallocRequiresRawReleaseProof {
        operation: CollectionSlotLifecycleOp,
    },
    RangeProofRequired {
        operation: CollectionSlotLifecycleOp,
        slot_ty: Option<TypeId>,
    },
    LiveSlotDuringStorageDealloc {
        slot_ty: TypeId,
    },
    MaybeLiveSlotDuringStorageDealloc {
        slot_ty: Option<TypeId>,
    },
}
