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
fn table_routes_slot_events_through_lifecycle_boundary() {
    let (types, owned, _) = test_types();
    let mut table = CollectionSlotStateTable::new();
    let slot0 = slot(owned, 0, owned);

    assert_eq!(
        table.apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        ),
        Ok(CollectionSlotState::Initialized(owned))
    );
    assert_eq!(
        table.apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::MoveOut { expected_ty: owned },
        ),
        Ok(CollectionSlotState::Moved(owned))
    );
    assert_eq!(
        table.apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::MoveOut { expected_ty: owned },
        ),
        Err(CollectionSlotTableRefutation {
            slot: slot0,
            reason: CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::MoveOut,
                state: CollectionSlotState::Moved(owned),
            },
        })
    );
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

#[test]
fn slot_identity_is_independent_from_current_payload_type() {
    let (types, owned, other) = test_types();
    let mut table = CollectionSlotStateTable::new();
    let old_slot = slot(owned, 0, owned);
    let new_slot = slot(owned, 0, other);

    table
        .apply_slot_event(
            &types,
            &old_slot,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        )
        .expect("slot starts with the old payload type");
    table
        .apply_slot_event(
            &types,
            &old_slot,
            CollectionSlotLifecycleEvent::ReplaceInitialized {
                old_ty: owned,
                new_ty: other,
                old_owner:
                    super::collection_slot_lifecycle::CollectionSlotReplacement::ReturnOldOwner,
            },
        )
        .expect("replace keeps the same physical slot identity");

    assert_eq!(
        table.state(&old_slot),
        CollectionSlotState::Initialized(other)
    );
    assert_eq!(
        table.state(&new_slot),
        CollectionSlotState::Initialized(other)
    );
}
