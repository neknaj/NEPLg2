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

fn slot(index: usize, ty: TypeId) -> Place {
    storage().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(index)),
        ty,
    )
}

#[test]
fn table_routes_slot_events_through_lifecycle_boundary() {
    let mut table = CollectionSlotStateTable::new();
    let slot0 = slot(0, OWNED);

    assert_eq!(
        table.apply_slot_event(
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
        ),
        Ok(CollectionSlotState::Initialized(OWNED))
    );
    assert_eq!(
        table.apply_slot_event(
            &slot0,
            CollectionSlotLifecycleEvent::MoveOut { expected_ty: OWNED },
        ),
        Ok(CollectionSlotState::Moved(OWNED))
    );
    assert_eq!(
        table.apply_slot_event(
            &slot0,
            CollectionSlotLifecycleEvent::MoveOut { expected_ty: OWNED },
        ),
        Err(CollectionSlotTableRefutation {
            slot: slot0,
            reason: CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::MoveOut,
                state: CollectionSlotState::Moved(OWNED),
            },
        })
    );
}

#[test]
fn storage_release_rejects_live_slot_and_reports_the_slot() {
    let mut table = CollectionSlotStateTable::new();
    let slot0 = slot(0, OWNED);
    table
        .apply_slot_event(
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
        )
        .expect("slot should be initialized before release");

    assert_eq!(
        table.release_storage(&storage()),
        Err(CollectionSlotTableRefutation {
            slot: slot0,
            reason: CollectionSlotLifecycleRefutation::LiveSlotDuringStorageDealloc {
                slot_ty: OWNED,
            },
        })
    );
}

#[test]
fn storage_release_forgets_vacant_slots_and_blocks_later_init() {
    let mut table = CollectionSlotStateTable::new();
    let slot0 = slot(0, OWNED);
    table
        .apply_slot_event(
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
        )
        .expect("slot should initialize");
    table
        .apply_slot_event(
            &slot0,
            CollectionSlotLifecycleEvent::DropInitialized { expected_ty: OWNED },
        )
        .expect("slot should be dropped before storage release");

    assert_eq!(table.release_storage(&storage()), Ok(()));
    assert!(table.entries().is_empty());
    assert_eq!(table.state(&slot0), CollectionSlotState::Released);
    assert_eq!(
        table.apply_slot_event(
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OTHER },
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
