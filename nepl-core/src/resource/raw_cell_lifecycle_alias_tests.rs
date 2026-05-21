extern crate alloc;

use alloc::{boxed::Box, string::String, vec};

use crate::types::{TypeCtx, TypeKind};

use super::cell_state::CellTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{CellState, Place, PlaceProjection, PlaceRoot, ResourceOffset};
use super::place_utils::raw_memory_cell_place;

#[test]
fn raw_move_marks_alias_cells_moved() {
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
    let base = Place {
        root: PlaceRoot::Local(String::from("storage")),
        projections: vec![],
        ty: types.i32(),
    };
    let raw = Place {
        root: PlaceRoot::Local(String::from("raw")),
        projections: vec![],
        ty: types.i32(),
    };
    let byte_offset = Place {
        root: PlaceRoot::Local(String::from("byte_off")),
        projections: vec![],
        ty: types.i32(),
    };
    let symbolic = base.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
            place: Box::new(byte_offset),
        }),
        types.i32(),
    );
    let raw_cell = raw_memory_cell_place(&raw, owned_ty);
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut cells = CellTable::default();

    raw_aliases.copy_explicit_raw_address_alias(&symbolic, &raw);
    cells.mark_initialized(&raw_cell);
    cells.mark_raw_cell_moved_with_aliases(&raw_aliases, &symbolic, owned_ty);

    assert_eq!(
        cells.availability_state_with_types(&types, &raw_cell),
        CellState::Moved
    );
    assert!(cells.live_non_copy_raw_cells_under(&raw, &types).is_empty());
}
