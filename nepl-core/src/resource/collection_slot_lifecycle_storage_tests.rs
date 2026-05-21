use super::collection_slot_lifecycle::{
    apply_collection_slot_lifecycle_event, CollectionSlotLifecycleEvent,
    CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use crate::types::{TypeCtx, TypeId};

fn test_types() -> (TypeCtx, TypeId) {
    let types = TypeCtx::new();
    let owned = types.i32();
    (types, owned)
}

#[test]
fn storage_dealloc_rejects_live_slot_and_releases_vacant_slot() {
    let (types, owned) = test_types();
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            &types,
            CollectionSlotState::Initialized(owned),
            CollectionSlotLifecycleEvent::StorageDealloc,
        ),
        Err(CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc { slot_ty: owned })
    );
    assert_eq!(
        apply_collection_slot_lifecycle_event(
            &types,
            CollectionSlotState::Dropped(owned),
            CollectionSlotLifecycleEvent::StorageDealloc,
        ),
        Ok(CollectionSlotState::Released)
    );
}
