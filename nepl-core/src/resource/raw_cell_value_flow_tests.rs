extern crate alloc;

use alloc::string::String;
use alloc::vec;

use crate::types::{TypeCtx, TypeKind};

use super::cell_state::CellTable;
use super::model::{Place, PlaceRoot};
use super::place_utils::raw_memory_cell_place;
use super::raw_cell_value_flow::RawCellValueFlowKind;

#[test]
fn raw_value_flow_facts_merge_only_path_common_proofs() {
    let (types, owned_ty) = non_copy_type_context();
    let address = Place {
        root: PlaceRoot::Local(String::from("slot_address")),
        projections: vec![],
        ty: types.i32(),
    };
    let cell = raw_memory_cell_place(&address, owned_ty);
    let mut stored_path = CellTable::default();
    let empty_path = CellTable::default();
    stored_path.record_raw_cell_value_flow(&address, owned_ty, RawCellValueFlowKind::StoreValue);

    let merged = CellTable::merge_paths(&[stored_path.clone(), empty_path]);

    assert!(!merged.raw_cell_value_flow_available(
        &cell,
        owned_ty,
        RawCellValueFlowKind::StoreValue,
        &types
    ));

    let merged = CellTable::merge_paths(&[stored_path.clone(), stored_path]);

    assert!(merged.raw_cell_value_flow_available(
        &cell,
        owned_ty,
        RawCellValueFlowKind::StoreValue,
        &types
    ));
}

#[test]
fn raw_load_invalidates_stale_store_proof_for_same_cell() {
    let (types, owned_ty) = non_copy_type_context();
    let address = Place {
        root: PlaceRoot::Local(String::from("slot_address")),
        projections: vec![],
        ty: types.i32(),
    };
    let cell = raw_memory_cell_place(&address, owned_ty);
    let mut cells = CellTable::default();

    cells.record_raw_cell_value_flow(&address, owned_ty, RawCellValueFlowKind::StoreValue);
    cells.record_raw_cell_value_flow(&address, owned_ty, RawCellValueFlowKind::MoveOutLoadedCell);

    assert!(!cells.raw_cell_value_flow_available(
        &cell,
        owned_ty,
        RawCellValueFlowKind::StoreValue,
        &types
    ));
    assert!(cells.raw_cell_value_flow_available(
        &cell,
        owned_ty,
        RawCellValueFlowKind::MoveOutLoadedCell,
        &types
    ));
}

#[test]
fn dropping_loaded_value_records_drop_proof_and_invalidates_move_out_proof() {
    let (types, owned_ty) = non_copy_type_context();
    let address = Place {
        root: PlaceRoot::Local(String::from("slot_address")),
        projections: vec![],
        ty: types.i32(),
    };
    let value = Place::local(String::from("loaded"), owned_ty);
    let cell = raw_memory_cell_place(&address, owned_ty);
    let mut cells = CellTable::default();

    cells.record_raw_cell_value_flow(&address, owned_ty, RawCellValueFlowKind::MoveOutLoadedCell);
    cells.record_raw_cell_loaded_value_origin(&address, owned_ty, &value);

    assert!(cells.record_raw_cell_loaded_value_drop(&value, &types));
    assert!(!cells.raw_cell_value_flow_available(
        &cell,
        owned_ty,
        RawCellValueFlowKind::MoveOutLoadedCell,
        &types
    ));
    assert!(cells.raw_cell_value_flow_available(
        &cell,
        owned_ty,
        RawCellValueFlowKind::DropLoadedCell,
        &types
    ));
}

#[test]
fn loaded_value_origin_follows_owner_value_transfer() {
    let (types, owned_ty) = non_copy_type_context();
    let address = Place {
        root: PlaceRoot::Local(String::from("slot_address")),
        projections: vec![],
        ty: types.i32(),
    };
    let source = Place::local(String::from("loaded"), owned_ty);
    let target = Place::local(String::from("renamed"), owned_ty);
    let cell = raw_memory_cell_place(&address, owned_ty);
    let mut cells = CellTable::default();

    cells.record_raw_cell_loaded_value_origin(&address, owned_ty, &source);
    cells.transfer_raw_cell_loaded_value_origin(&source, &target);

    assert!(cells.record_raw_cell_loaded_value_drop(&target, &types));
    assert!(cells.raw_cell_value_flow_available(
        &cell,
        owned_ty,
        RawCellValueFlowKind::DropLoadedCell,
        &types
    ));
}

fn non_copy_type_context() -> (TypeCtx, crate::types::TypeId) {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.i32());
    let owned_ty = types.register_named(
        String::from("Owned"),
        TypeKind::Struct {
            name: String::from("Owned"),
            type_params: vec![],
            fields: vec![],
            field_names: vec![],
        },
    );
    (types, owned_ty)
}
