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

fn new_storage(ty: TypeId) -> Place {
    Place::local(String::from("next_buffer"), ty)
}

fn slot(storage_ty: TypeId, index: usize, ty: TypeId) -> Place {
    storage(storage_ty).with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(index)),
        ty,
    )
}

fn new_slot(storage_ty: TypeId, index: usize, ty: TypeId) -> Place {
    new_storage(storage_ty).with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(index)),
        ty,
    )
}

#[test]
fn storage_relocate_transfers_slot_states_to_new_prefix() {
    let (types, owned, _) = test_types();
    let mut table = CollectionSlotStateTable::new();
    let live = slot(owned, 0, owned);
    let moved = slot(owned, 1, owned);
    let dropped = slot(owned, 2, owned);
    table
        .apply_slot_event(
            &types,
            &live,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        )
        .expect("live slot should initialize");
    table
        .apply_slot_event(
            &types,
            &moved,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        )
        .expect("moved slot should initialize");
    table
        .apply_slot_event(
            &types,
            &moved,
            CollectionSlotLifecycleEvent::MoveOut { expected_ty: owned },
        )
        .expect("moved slot should move");
    table
        .apply_slot_event(
            &types,
            &dropped,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        )
        .expect("dropped slot should initialize");
    table
        .apply_slot_event(
            &types,
            &dropped,
            CollectionSlotLifecycleEvent::DropInitialized { expected_ty: owned },
        )
        .expect("dropped slot should drop");

    table
        .relocate_storage(&storage(owned), &new_storage(owned))
        .expect("storage relocation should transfer slot states");

    assert_eq!(table.state(&live), CollectionSlotState::Released);
    assert_eq!(
        table.state(&new_slot(owned, 0, owned)),
        CollectionSlotState::Initialized(owned)
    );
    assert_eq!(
        table.state(&new_slot(owned, 1, owned)),
        CollectionSlotState::Moved(owned)
    );
    assert_eq!(
        table.state(&new_slot(owned, 2, owned)),
        CollectionSlotState::Dropped(owned)
    );
    assert_eq!(
        table.release_storage(&new_storage(owned)),
        Err(CollectionSlotTableRefutation {
            slot: new_slot(owned, 0, owned),
            reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc {
                slot_ty: owned,
            },
        })
    );
}

#[test]
fn storage_relocate_rejects_occupied_target_storage() {
    let (types, owned, other) = test_types();
    let mut table = CollectionSlotStateTable::new();
    let old_slot = slot(owned, 0, owned);
    let target_slot = new_slot(owned, 0, owned);
    table
        .apply_slot_event(
            &types,
            &old_slot,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        )
        .expect("old slot should initialize");
    table
        .apply_slot_event(
            &types,
            &target_slot,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: other },
        )
        .expect("target slot should initialize");

    assert_eq!(
        table.relocate_storage(&storage(owned), &new_storage(owned)),
        Err(CollectionSlotTableRefutation {
            slot: target_slot,
            reason: CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::StorageRelocate,
                state: CollectionSlotState::Initialized(other),
            },
        })
    );
}
