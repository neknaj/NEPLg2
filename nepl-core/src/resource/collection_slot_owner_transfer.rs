use crate::types::{TypeCtx, TypeId};

use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotReplacement,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CollectionSlotOwnerTransferObligation {
    StoreValue {
        operation: CollectionSlotLifecycleOp,
        value_ty: TypeId,
    },
    MoveOutValue {
        operation: CollectionSlotLifecycleOp,
        value_ty: TypeId,
    },
    MoveOutAndStoreValue {
        operation: CollectionSlotLifecycleOp,
        old_ty: TypeId,
        new_ty: TypeId,
    },
}

impl CollectionSlotOwnerTransferObligation {
    pub(super) fn primary_refutation(self) -> (CollectionSlotLifecycleOp, TypeId) {
        match self {
            CollectionSlotOwnerTransferObligation::StoreValue {
                operation,
                value_ty,
            }
            | CollectionSlotOwnerTransferObligation::MoveOutValue {
                operation,
                value_ty,
            } => (operation, value_ty),
            CollectionSlotOwnerTransferObligation::MoveOutAndStoreValue {
                operation,
                old_ty,
                new_ty: _,
            } => (operation, old_ty),
        }
    }
}

pub(super) fn collection_slot_owner_transfer_obligation(
    types: &TypeCtx,
    event: CollectionSlotLifecycleEvent,
) -> Option<CollectionSlotOwnerTransferObligation> {
    match event {
        CollectionSlotLifecycleEvent::InitializeEmpty { value_ty } => {
            non_copy_store_obligation(types, CollectionSlotLifecycleOp::InitializeEmpty, value_ty)
        }
        CollectionSlotLifecycleEvent::MoveOut { expected_ty } => {
            non_copy_move_out_obligation(types, CollectionSlotLifecycleOp::MoveOut, expected_ty)
        }
        CollectionSlotLifecycleEvent::ReplaceInitialized {
            old_ty,
            new_ty,
            old_owner: CollectionSlotReplacement::ReturnOldOwner,
        } => non_copy_replace_return_old_obligation(types, old_ty, new_ty),
        CollectionSlotLifecycleEvent::ReplaceInitialized {
            old_ty: _,
            new_ty,
            old_owner: CollectionSlotReplacement::DropOldOwner,
        } => {
            non_copy_store_obligation(types, CollectionSlotLifecycleOp::ReplaceInitialized, new_ty)
        }
        CollectionSlotLifecycleEvent::BorrowRead { .. }
        | CollectionSlotLifecycleEvent::DropInitialized { .. }
        | CollectionSlotLifecycleEvent::StorageDealloc => None,
    }
}

fn non_copy_store_obligation(
    types: &TypeCtx,
    operation: CollectionSlotLifecycleOp,
    slot_ty: TypeId,
) -> Option<CollectionSlotOwnerTransferObligation> {
    if types.is_copy(slot_ty) {
        None
    } else {
        Some(CollectionSlotOwnerTransferObligation::StoreValue {
            operation,
            value_ty: slot_ty,
        })
    }
}

fn non_copy_move_out_obligation(
    types: &TypeCtx,
    operation: CollectionSlotLifecycleOp,
    slot_ty: TypeId,
) -> Option<CollectionSlotOwnerTransferObligation> {
    if types.is_copy(slot_ty) {
        None
    } else {
        Some(CollectionSlotOwnerTransferObligation::MoveOutValue {
            operation,
            value_ty: slot_ty,
        })
    }
}

fn non_copy_replace_return_old_obligation(
    types: &TypeCtx,
    old_ty: TypeId,
    new_ty: TypeId,
) -> Option<CollectionSlotOwnerTransferObligation> {
    match (types.is_copy(old_ty), types.is_copy(new_ty)) {
        (true, true) => None,
        (false, true) => Some(CollectionSlotOwnerTransferObligation::MoveOutValue {
            operation: CollectionSlotLifecycleOp::ReplaceInitialized,
            value_ty: old_ty,
        }),
        (true, false) => Some(CollectionSlotOwnerTransferObligation::StoreValue {
            operation: CollectionSlotLifecycleOp::ReplaceInitialized,
            value_ty: new_ty,
        }),
        (false, false) => Some(
            CollectionSlotOwnerTransferObligation::MoveOutAndStoreValue {
                operation: CollectionSlotLifecycleOp::ReplaceInitialized,
                old_ty,
                new_ty,
            },
        ),
    }
}
