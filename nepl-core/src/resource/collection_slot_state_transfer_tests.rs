use alloc::string::String;

use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    CollectionSlotState,
};
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::model::{Place, PlaceProjection, ResourceOffset};
use crate::types::TypeId;

const OWNED: TypeId = TypeId(20);

fn source_storage() -> Place {
    Place::local(String::from("source"), OWNED)
}

fn target_storage() -> Place {
    Place::local(String::from("target"), OWNED)
}

fn source_slot(index: usize, ty: TypeId) -> Place {
    source_storage().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(index)),
        ty,
    )
}

fn target_slot(index: usize, ty: TypeId) -> Place {
    target_storage().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(index)),
        ty,
    )
}

#[test]
fn transfer_storage_prefix_moves_release_marker_to_target() {
    let mut table = CollectionSlotStateTable::new();
    let slot0 = source_slot(0, OWNED);
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
        .expect("slot should be vacant before storage release");
    table
        .release_storage(&source_storage())
        .expect("released storage marker should be recorded");

    table
        .transfer_storage_prefix(&source_storage(), &target_storage())
        .expect("released marker should transfer with moved storage owner");

    assert_eq!(table.state(&slot0), CollectionSlotState::Uninitialized);
    assert_eq!(
        table.state(&target_slot(0, OWNED)),
        CollectionSlotState::Released
    );
    assert_eq!(
        table.apply_slot_event(
            &target_slot(0, OWNED),
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: OWNED },
        ),
        Err(CollectionSlotTableRefutation {
            slot: target_slot(0, OWNED),
            reason: CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::InitializeEmpty,
                state: CollectionSlotState::Released,
            },
        })
    );
}
