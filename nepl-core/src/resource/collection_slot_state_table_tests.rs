use alloc::string::String;

use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    CollectionSlotState,
};
use super::collection_slot_state_table::{
    CollectionSlotInitializedRangeStateEntry, CollectionSlotStateTable,
    CollectionSlotTableRefutation,
};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection, ResourceI32RelationOp, ResourceOffset};
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

/// collection range を消費した後、同じ storage 配下に残っていた具体 slot 状態も
/// 同時に消えることを確認する。range は initialized_count で管理される正規形なので、
/// 古い具体 slot 状態を残すと後続の storage release が処理済み payload を live と誤判定する。
#[test]
fn clearing_initialized_range_also_clears_payload_slots_described_by_that_range() {
    let (types, owned, _) = test_types();
    let mut table = CollectionSlotStateTable::new();
    let mut raw_aliases = RawCellAddressAliases::default();
    let buffer = storage(owned);
    let initialized_count = Place::local(String::from("len"), owned);
    let slot0 = slot(owned, 0, owned);
    let slot1 = slot(owned, 4, owned);
    raw_aliases.set_i32_value(&initialized_count, 2);

    table
        .apply_slot_event(
            &types,
            &slot0,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        )
        .expect("slot0 should be initialized before the range is consumed");
    table
        .apply_slot_event(
            &types,
            &slot1,
            CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned },
        )
        .expect("slot1 should be initialized before the range is consumed");
    table.mark_initialized_range_with_aliases(&buffer, &initialized_count, owned, 4, &raw_aliases);

    table.clear_initialized_range_with_aliases(
        &types,
        &buffer,
        &initialized_count,
        owned,
        4,
        &raw_aliases,
    );

    assert_eq!(table.state(&slot0), CollectionSlotState::Uninitialized);
    assert_eq!(table.state(&slot1), CollectionSlotState::Uninitialized);
    assert!(table.initialized_ranges().is_empty());
}

/// initialized range の cleanup は storage 全体ではなく、count で覆われる payload slot
/// だけを正規化する。range 外の具体 slot まで消すと、後続の storage release が本来
/// 検出すべき live slot を見失うため、count 境界を必ず確認する。
#[test]
fn clearing_initialized_range_preserves_payload_slots_outside_count() {
    let (types, owned, _) = test_types();
    let mut table = CollectionSlotStateTable::new();
    let mut raw_aliases = RawCellAddressAliases::default();
    let buffer = storage(owned);
    let initialized_count = Place::local(String::from("len"), owned);
    let slot0 = slot(owned, 0, owned);
    let slot1 = slot(owned, 4, owned);
    raw_aliases.set_i32_value(&initialized_count, 1);

    table.set_slot_state(&slot0, CollectionSlotState::Initialized(owned));
    table.set_slot_state(&slot1, CollectionSlotState::Initialized(owned));
    table.mark_initialized_range_with_aliases(&buffer, &initialized_count, owned, 4, &raw_aliases);

    table.clear_initialized_range_with_aliases(
        &types,
        &buffer,
        &initialized_count,
        owned,
        4,
        &raw_aliases,
    );

    assert_eq!(table.state(&slot0), CollectionSlotState::Uninitialized);
    assert_eq!(table.state(&slot1), CollectionSlotState::Initialized(owned));
}

/// path merge 後の maybe range は「この storage の payload slot が path により
/// 初期化済みかもしれない」ことを表す。具体 slot だけを Initialized のまま残すと、
/// maybe range を持つ path が確定 live slot と誤って結合されるため、slot 状態も
/// MaybeInitialized に弱める必要がある。
#[test]
fn maybe_initialized_range_weakens_concrete_initialized_payload_slot() {
    let (_, owned, _) = test_types();
    let mut table = CollectionSlotStateTable::new();
    let raw_aliases = RawCellAddressAliases::default();
    let buffer = storage(owned);
    let initialized_count = Place::local(String::from("len"), owned);
    let slot0 = slot(owned, 0, owned);

    table.set_slot_state(&slot0, CollectionSlotState::Initialized(owned));
    table
        .maybe_initialized_ranges
        .push(CollectionSlotInitializedRangeStateEntry {
            storage: buffer,
            initialized_count,
            value_ty: owned,
            element_stride: 4,
        });

    table.weaken_slots_described_by_maybe_ranges_with_aliases(&raw_aliases);

    assert_eq!(
        table.state(&slot0),
        CollectionSlotState::MaybeInitialized(Some(owned))
    );
}

/// initialized range の count が別名ではなく i32 relation で等しい場合にも、
/// 同じ storage の range としてまとめて消費されることを確認する。
///
/// Vec の `len` と `initialized_len` は return summary 上で別 field として現れるが、
/// transform 後の正常系では同じ `write_i` から来る等値の scalar fact を持つ。
/// DropTraversal が `initialized_len` 側を消費した後に `len` 側の range を残すと、
/// storage release が処理済み payload を live range と誤判定する。
#[test]
fn clearing_initialized_range_uses_i32_relation_for_count_equivalence() {
    let (types, owned, _) = test_types();
    let mut table = CollectionSlotStateTable::new();
    let mut raw_aliases = RawCellAddressAliases::default();
    let buffer = storage(owned);
    let len = Place::local(String::from("len"), owned);
    let initialized_len = Place::local(String::from("initialized_len"), owned);

    raw_aliases.add_i32_relation(&len, ResourceI32RelationOp::Eq, &initialized_len);
    table.mark_initialized_range_with_aliases(&buffer, &len, owned, 4, &raw_aliases);
    table.mark_initialized_range_with_aliases(&buffer, &initialized_len, owned, 4, &raw_aliases);

    table.clear_initialized_range_with_aliases(
        &types,
        &buffer,
        &initialized_len,
        owned,
        4,
        &raw_aliases,
    );

    assert!(table.initialized_ranges().is_empty());
}
