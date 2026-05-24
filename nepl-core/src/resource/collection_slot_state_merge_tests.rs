use alloc::string::String;

use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    CollectionSlotState,
};
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::initialized_alias::RawCellAddressAliases;
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
fn merge_marks_partially_moved_slot_as_maybe_initialized() {
    let (types, owned, other) = test_types();
    let slot0 = slot(owned, 0, owned);
    let mut left = CollectionSlotStateTable::new();
    left.apply_slot_event(
        &types,
        &slot0,
        CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
    )
    .expect("slot should initialize");

    let mut right = left.clone();
    right
        .apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::MoveOut { expected_ty: owned },
        )
        .expect("slot should move on one path");

    let mut merged = CollectionSlotStateTable::merge_paths(&[left, right]);

    assert_eq!(
        merged.state(&slot0),
        CollectionSlotState::MaybeInitialized(Some(owned))
    );
    assert_eq!(
        merged.apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: other },
        ),
        Err(CollectionSlotTableRefutation {
            slot: slot0.clone(),
            reason: CollectionSlotLifecycleRefutation::MaybeLiveSlotOverwrite {
                slot_ty: Some(owned),
            },
        })
    );
    assert_eq!(
        merged.apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::MoveOut { expected_ty: owned },
        ),
        Err(CollectionSlotTableRefutation {
            slot: slot0.clone(),
            reason: CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::MoveOut,
                state: CollectionSlotState::MaybeInitialized(Some(owned)),
            },
        })
    );
    assert_eq!(
        merged.release_storage(&storage(owned)),
        Err(CollectionSlotTableRefutation {
            slot: slot0,
            reason: CollectionSlotLifecycleRefutation::MaybeLiveSlotDuringStorageDealloc {
                slot_ty: Some(owned),
            },
        })
    );
}

#[test]
fn merge_keeps_storage_release_uncertainty() {
    let (types, owned, other) = test_types();
    let slot0 = slot(owned, 0, owned);
    let mut left = CollectionSlotStateTable::new();
    left.release_storage(&storage(owned))
        .expect("one path releases the storage");
    let right = CollectionSlotStateTable::new();

    let mut merged = CollectionSlotStateTable::merge_paths(&[left, right]);

    assert_eq!(merged.state(&slot0), CollectionSlotState::MaybeReleased);
    assert_eq!(
        merged.apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: other },
        ),
        Err(CollectionSlotTableRefutation {
            slot: slot0,
            reason: CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::InitializeEmpty,
                state: CollectionSlotState::MaybeReleased,
            },
        })
    );
    assert_eq!(
        merged.release_storage(&storage(owned)),
        Err(CollectionSlotTableRefutation {
            slot: storage(owned),
            reason: CollectionSlotLifecycleRefutation::Unavailable {
                operation: CollectionSlotLifecycleOp::StorageDealloc,
                state: CollectionSlotState::MaybeReleased,
            },
        })
    );
}

#[test]
fn merge_definitely_vacant_slots_can_be_reinitialized() {
    let (types, owned, other) = test_types();
    let slot0 = slot(owned, 0, owned);
    let mut left = CollectionSlotStateTable::new();
    let mut right = CollectionSlotStateTable::new();
    for path in [&mut left, &mut right] {
        path.apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        )
        .expect("slot should initialize");
    }
    left.apply_slot_event(
        &types,
        &slot0,
        CollectionSlotLifecycleEvent::MoveOut { expected_ty: owned },
    )
    .expect("left path moves");
    right
        .apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::DropInitialized { expected_ty: owned },
        )
        .expect("right path drops");

    let mut merged = CollectionSlotStateTable::merge_paths(&[left, right]);

    assert_eq!(merged.state(&slot0), CollectionSlotState::Uninitialized);
    assert_eq!(
        merged.apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: other },
        ),
        Ok(CollectionSlotState::Initialized(other))
    );
}

#[test]
fn merge_drops_initialized_range_when_a_path_overrides_a_slot_inside_it() {
    let (types, owned, _) = test_types();
    let raw_aliases = RawCellAddressAliases::default();
    let buffer = storage(owned);
    let count = Place::i32_constant(2, owned);
    let slot0 = slot(owned, 0, owned);
    let mut left = CollectionSlotStateTable::new();
    let mut right = CollectionSlotStateTable::new();
    for path in [&mut left, &mut right] {
        path.mark_initialized_range_with_aliases(&buffer, &count, owned, 4, &raw_aliases);
    }
    right.set_slot_state(&slot0, CollectionSlotState::Moved(owned));

    let merged = CollectionSlotStateTable::merge_paths(&[left, right]);

    assert_eq!(
        merged.state_with_aliases_and_ranges(&types, &slot0, &raw_aliases),
        CollectionSlotState::MaybeInitialized(Some(owned))
    );
    assert!(
        !merged.initialized_ranges().is_empty(),
        "common range summary should survive, while explicit slot state shadows overridden slots"
    );
}

#[test]
fn merge_keeps_one_path_initialized_range_as_maybe_live() {
    let (types, owned, _) = test_types();
    let raw_aliases = RawCellAddressAliases::default();
    let buffer = storage(owned);
    let count = Place::i32_constant(1, owned);
    let slot0 = slot(owned, 0, owned);
    let mut left = CollectionSlotStateTable::new();
    let right = CollectionSlotStateTable::new();
    left.mark_initialized_range_with_aliases(&buffer, &count, owned, 4, &raw_aliases);

    let mut merged = CollectionSlotStateTable::merge_paths(&[left, right]);

    assert_eq!(
        merged.state_with_aliases_and_ranges(&types, &slot0, &raw_aliases),
        CollectionSlotState::MaybeInitialized(Some(owned))
    );
    assert_eq!(
        merged.release_storage_with_aliases(&buffer, &raw_aliases),
        Err(CollectionSlotTableRefutation {
            slot: buffer,
            reason: CollectionSlotLifecycleRefutation::MaybeLiveSlotDuringStorageDealloc {
                slot_ty: Some(owned),
            },
        })
    );
}
