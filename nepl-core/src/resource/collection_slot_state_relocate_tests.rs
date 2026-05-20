use alloc::string::String;

use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    CollectionSlotState,
};
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::model::{Place, PlaceProjection, ResourceOffset};
use crate::types::TypeId;

const OWNED: TypeId = TypeId(20);
const OTHER: TypeId = TypeId(21);

fn storage() -> Place {
    Place::local(String::from("buffer"), OWNED)
}

fn new_storage() -> Place {
    Place::local(String::from("next_buffer"), OWNED)
}

fn slot(index: usize, ty: TypeId) -> Place {
    storage().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(index)),
        ty,
    )
}

fn new_slot(index: usize, ty: TypeId) -> Place {
    new_storage().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(index)),
        ty,
    )
}

#[test]
fn storage_relocate_transfers_slot_states_to_new_prefix() {
    let mut table = CollectionSlotStateTable::new();
    let live = slot(0, OWNED);
    let moved = slot(1, OWNED);
    let dropped = slot(2, OWNED);
    table
        .apply_slot_event(
            &live,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
        )
        .expect("live slot should initialize");
    table
        .apply_slot_event(
            &moved,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
        )
        .expect("moved slot should initialize");
    table
        .apply_slot_event(
            &moved,
            CollectionSlotLifecycleEvent::MoveOut { expected_ty: OWNED },
        )
        .expect("moved slot should move");
    table
        .apply_slot_event(
            &dropped,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
        )
        .expect("dropped slot should initialize");
    table
        .apply_slot_event(
            &dropped,
            CollectionSlotLifecycleEvent::DropInitialized { expected_ty: OWNED },
        )
        .expect("dropped slot should drop");

    table
        .relocate_storage(&storage(), &new_storage())
        .expect("storage relocation should transfer slot states");

    assert_eq!(table.state(&live), CollectionSlotState::Released);
    assert_eq!(
        table.state(&new_slot(0, OWNED)),
        CollectionSlotState::Initialized(OWNED)
    );
    assert_eq!(
        table.state(&new_slot(1, OWNED)),
        CollectionSlotState::Moved(OWNED)
    );
    assert_eq!(
        table.state(&new_slot(2, OWNED)),
        CollectionSlotState::Dropped(OWNED)
    );
    assert_eq!(
        table.release_storage(&new_storage()),
        Err(CollectionSlotTableRefutation {
            slot: new_slot(0, OWNED),
            reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc {
                slot_ty: OWNED,
            },
        })
    );
}

#[test]
fn storage_relocate_rejects_occupied_target_storage() {
    let mut table = CollectionSlotStateTable::new();
    let old_slot = slot(0, OWNED);
    let target_slot = new_slot(0, OWNED);
    table
        .apply_slot_event(
            &old_slot,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
        )
        .expect("old slot should initialize");
    table
        .apply_slot_event(
            &target_slot,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OTHER },
        )
        .expect("target slot should initialize");

    assert_eq!(
        table.relocate_storage(&storage(), &new_storage()),
        Err(CollectionSlotTableRefutation {
            slot: target_slot,
            reason: CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::StorageRelocate,
                state: CollectionSlotState::Initialized(OTHER),
            },
        })
    );
}
