extern crate alloc;

use alloc::string::String;
use alloc::vec;

use crate::types::{TypeCtx, TypeKind};

use super::cell_state::CellTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection, PlaceRoot, ResourceOffset};
use super::place_utils::raw_memory_cell_place;
use super::raw_cell_value_flow::RawCellValueFlowKind;

#[test]
fn raw_value_flow_alias_matching_treats_zero_offset_as_same_cell_only() {
    let (types, owned_ty) = non_copy_type_context();
    let base = Place {
        root: PlaceRoot::Local(String::from("slot_address")),
        projections: vec![],
        ty: types.i32(),
    };
    let raw_view = Place {
        root: PlaceRoot::Local(String::from("raw_view")),
        projections: vec![],
        ty: types.i32(),
    };
    let zero_offset = base.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
        types.i32(),
    );
    let non_zero_offset = base.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        types.i32(),
    );
    let zero_cell = raw_memory_cell_place(&zero_offset, owned_ty);
    let non_zero_cell = raw_memory_cell_place(&non_zero_offset, owned_ty);
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut cells = CellTable::default();

    raw_aliases.copy_explicit_raw_address_alias(&base, &raw_view);
    cells.record_raw_cell_value_flow(&raw_view, owned_ty, RawCellValueFlowKind::StoreValue);

    assert!(cells.raw_cell_value_flow_available_with_aliases(
        &raw_aliases,
        &zero_cell,
        owned_ty,
        RawCellValueFlowKind::StoreValue,
        &types
    ));
    assert!(!cells.raw_cell_value_flow_available_with_aliases(
        &raw_aliases,
        &non_zero_cell,
        owned_ty,
        RawCellValueFlowKind::StoreValue,
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
