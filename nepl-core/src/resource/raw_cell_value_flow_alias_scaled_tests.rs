extern crate alloc;

use alloc::{boxed::Box, string::String, vec};

use crate::types::{TypeCtx, TypeKind};

use super::cell_state::CellTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection, PlaceRoot, ResourceOffset};
use super::place_utils::raw_memory_cell_place;
use super::raw_cell_value_flow::RawCellValueFlowKind;

#[test]
fn raw_value_flow_alias_matching_normalizes_scaled_symbolic_offsets() {
    let (types, owned_ty) = non_copy_type_context();
    let base = Place {
        root: PlaceRoot::Local(String::from("storage")),
        projections: vec![],
        ty: types.i32(),
    };
    let index = Place {
        root: PlaceRoot::Local(String::from("i")),
        projections: vec![],
        ty: types.i32(),
    };
    let byte_offset = Place {
        root: PlaceRoot::Local(String::from("byte_off")),
        projections: vec![],
        ty: types.i32(),
    };
    let symbolic_address = base.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
            place: Box::new(byte_offset.clone()),
        }),
        types.i32(),
    );
    let scaled_address = base.with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
            place: Box::new(index.clone()),
            scale: 4,
        }),
        types.i32(),
    );
    let scaled_cell = raw_memory_cell_place(&scaled_address, owned_ty);
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut cells = CellTable::default();

    raw_aliases.add_i32_scale(&index, &byte_offset, 4);
    cells.record_raw_cell_value_flow_with_aliases(
        &raw_aliases,
        &symbolic_address,
        owned_ty,
        RawCellValueFlowKind::DropLoadedCell,
    );

    assert!(cells.raw_cell_value_flow_available_with_aliases(
        &raw_aliases,
        &scaled_cell,
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
