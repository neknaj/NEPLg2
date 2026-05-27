use crate::types::{TypeCtx, TypeId};

use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;

/// collection slot lifecycle が payload の所有状態を追跡する必要があるかを判定する。
///
/// collection slot checker は、collection 内の非 Copy payload が MoveOut/Drop 後に
/// 再利用されないことを証明する層である。Copy payload の初期化済み状態や raw memory
/// access は raw initialization と Copy invariant の層で検査するため、slot state まで
/// 持ち込まない。これにより、Copy 専用 helper が非 Copy 用の探索空間へ混入しない。
pub(super) fn collection_slot_payload_type_needs_tracking(types: &TypeCtx, ty: TypeId) -> bool {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    !types.is_copy(resolved)
}

/// lifecycle event が collection slot state の追跡を必要とするかを判定する。
///
/// ReplaceInitialized は古い値の破棄と新しい値の初期化を同時に表すため、どちらか一方が
/// 非 Copy なら追跡対象になる。StorageDealloc も payload 型を持つことで、Copy 用の
/// storage release が非 Copy 用 slot state summary を起動しないようにする。
pub(super) fn collection_slot_lifecycle_event_needs_tracking(
    types: &TypeCtx,
    event: CollectionSlotLifecycleEvent,
) -> bool {
    match event {
        CollectionSlotLifecycleEvent::InitializeEmpty { value_ty }
        | CollectionSlotLifecycleEvent::BorrowRead {
            expected_ty: value_ty,
        }
        | CollectionSlotLifecycleEvent::MoveOut {
            expected_ty: value_ty,
        }
        | CollectionSlotLifecycleEvent::DropInitialized {
            expected_ty: value_ty,
        } => collection_slot_payload_type_needs_tracking(types, value_ty),
        CollectionSlotLifecycleEvent::ReplaceInitialized { old_ty, new_ty, .. } => {
            collection_slot_payload_type_needs_tracking(types, old_ty)
                || collection_slot_payload_type_needs_tracking(types, new_ty)
        }
        CollectionSlotLifecycleEvent::StorageDealloc { value_ty } => {
            collection_slot_payload_type_needs_tracking(types, value_ty)
        }
    }
}
