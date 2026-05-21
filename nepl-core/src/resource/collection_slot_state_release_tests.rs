use alloc::string::String;

use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    CollectionSlotState,
};
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::model::{Place, PlaceProjection, ResourceOffset};
use crate::types::{TypeCtx, TypeId};

fn test_types() -> (TypeCtx, TypeId, TypeId) {
    let types = TypeCtx::new();
    let owned = types.i32();
    let other = types.u8();
    (types, owned, other)
}

fn storage(ty: TypeId) -> Place {
    Place::local(String::from("buffer"), ty)
}

fn slot(storage_ty: TypeId, index: usize, ty: TypeId) -> Place {
    storage(storage_ty).with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(index)),
        ty,
    )
}

#[test]
fn storage_release_rejects_live_slot_and_reports_the_slot() {
    let (types, owned, _) = test_types();
    let mut table = CollectionSlotStateTable::new();
    let slot0 = slot(owned, 0, owned);
    table
        .apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        )
        .expect("slot should be initialized before release");

    assert_eq!(
        table.release_storage(&storage(owned)),
        Err(CollectionSlotTableRefutation {
            slot: slot0,
            reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc {
                slot_ty: owned,
            },
        })
    );
}

#[test]
fn storage_release_forgets_vacant_slots_and_blocks_later_init() {
    let (types, owned, other) = test_types();
    let mut table = CollectionSlotStateTable::new();
    let slot0 = slot(owned, 0, owned);
    table
        .apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        )
        .expect("slot should initialize");
    table
        .apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::DropInitialized { expected_ty: owned },
        )
        .expect("slot should be dropped before storage release");

    assert_eq!(table.release_storage(&storage(owned)), Ok(()));
    assert!(table.entries().is_empty());
    assert_eq!(table.state(&slot0), CollectionSlotState::Released);
    assert_eq!(
        table.apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: other },
        ),
        Err(CollectionSlotTableRefutation {
            slot: slot0,
            reason: CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::InitializeEmpty,
                state: CollectionSlotState::Released,
            },
        })
    );
}
