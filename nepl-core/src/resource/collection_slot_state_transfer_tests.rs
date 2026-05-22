use alloc::string::String;

use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    CollectionSlotState,
};
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection, ResourceId, ResourceOffset};
use crate::types::{TypeCtx, TypeId};
use alloc::boxed::Box;

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

fn temporary_storage(id: usize, ty: TypeId) -> Place {
    Place::temporary(ResourceId(id), ty)
}

fn owner(name: &str, ty: TypeId) -> Place {
    Place::local(String::from(name), ty)
}

fn field(place: &Place, index: usize, offset_bytes: usize, ty: TypeId) -> Place {
    place.clone().with_projection(
        PlaceProjection::Field {
            index,
            offset_bytes,
        },
        ty,
    )
}

fn symbolic_slot(storage: &Place, offset_place: Place, ty: TypeId) -> Place {
    storage
        .clone()
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
                place: Box::new(offset_place),
            }),
            ty,
        )
        .with_projection(PlaceProjection::Deref, ty)
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

#[test]
fn moved_slot_state_matches_stable_origin_queries() {
    let (types, owned) = test_types();
    let mut table = CollectionSlotStateTable::new();
    let source = source_storage(owned);
    let temporary = temporary_storage(1, owned);
    let slot0 = source_slot(owned, 0, owned);
    let mut aliases = RawCellAddressAliases::default();

    table
        .apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        )
        .expect("slot should initialize under the stable owner");
    aliases.record_raw_address_view_origin(&source, &temporary);
    table
        .transfer_storage_prefix_with_aliases(&source, &temporary, &aliases)
        .expect("slot state follows the moved owner value");

    assert_eq!(
        table.state_with_aliases(&slot0, &aliases),
        CollectionSlotState::Initialized(owned),
        "state lookup must be symmetric: a temporary view with a stable origin is the same collection slot"
    );
}

#[test]
fn storage_transfer_preserves_historical_index_when_new_len_is_offset() {
    let (types, owned) = test_types();
    let source_owner = owner("source_owner", owned);
    let target_owner = owner("target_owner", owned);
    let source_len = field(&source_owner, 0, 0, owned);
    let target_len = field(&target_owner, 0, 0, owned);
    let source_storage = field(&source_owner, 3, 12, owned);
    let target_storage = field(&target_owner, 3, 12, owned);
    let old_tail = symbolic_slot(&source_storage, source_len.clone(), owned);
    let moved_old_tail = symbolic_slot(&target_storage, source_len.clone(), owned);
    let one_past_tail = symbolic_slot(&target_storage, target_len.clone(), owned);
    let mut aliases = RawCellAddressAliases::default();
    let mut table = CollectionSlotStateTable::new();

    table
        .apply_slot_event(
            &types,
            &old_tail,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        )
        .expect("old tail slot should initialize before the owner move");
    aliases.add_i32_offset(&source_len, &target_len, 1);
    table
        .transfer_storage_prefix_with_aliases(&source_storage, &target_storage, &aliases)
        .expect("storage transfer should preserve initialized slots");

    assert_eq!(
        table.state_with_aliases(&moved_old_tail, &aliases),
        CollectionSlotState::Initialized(owned),
        "moving a collection owner must not turn an old-len slot into a new-len slot"
    );
    assert_eq!(
        table.state_with_aliases(&one_past_tail, &aliases),
        CollectionSlotState::Uninitialized,
        "new_len denotes the next vacant slot when new_len = old_len + 1"
    );
}
