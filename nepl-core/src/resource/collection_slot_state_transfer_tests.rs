use alloc::string::String;

use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    CollectionSlotState,
};
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::model::{Place, PlaceProjection, ResourceOffset};
use crate::types::{TypeCtx, TypeId};

fn test_types() -> (TypeCtx, TypeId) {
    let types = TypeCtx::new();
    let owned = types.i32();
    (types, owned)
}

fn source_storage(ty: TypeId) -> Place {
    Place::local(String::from("source"), ty)
}

fn target_storage(ty: TypeId) -> Place {
    Place::local(String::from("target"), ty)
}

fn source_slot(storage_ty: TypeId, index: usize, ty: TypeId) -> Place {
    source_storage(storage_ty).with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(index)),
        ty,
    )
}

fn target_slot(storage_ty: TypeId, index: usize, ty: TypeId) -> Place {
    target_storage(storage_ty).with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(index)),
        ty,
    )
}

#[test]
fn transfer_storage_prefix_moves_release_marker_to_target() {
    let (types, owned) = test_types();
    let mut table = CollectionSlotStateTable::new();
    let slot0 = source_slot(owned, 0, owned);
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
        .expect("slot should be vacant before storage release");
    table
        .release_storage(&source_storage(owned))
        .expect("released storage marker should be recorded");

    table
        .transfer_storage_prefix(&source_storage(owned), &target_storage(owned))
        .expect("released marker should transfer with moved storage owner");

    assert_eq!(table.state(&slot0), CollectionSlotState::Uninitialized);
    assert_eq!(
        table.state(&target_slot(owned, 0, owned)),
        CollectionSlotState::Released
    );
    assert_eq!(
        table.apply_slot_event(
            &types,
            &target_slot(owned, 0, owned),
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        ),
        Err(CollectionSlotTableRefutation {
            slot: target_slot(owned, 0, owned),
            reason: CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::InitializeEmpty,
                state: CollectionSlotState::Released,
            },
        })
    );
}
