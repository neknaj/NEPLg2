use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::{raw_cell_address_prefix, CellTable};
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::model::{CellState, Place, PlaceProjection, ResourceId, ResourceOffset};
use super::place_utils::{raw_memory_cell_place, raw_memory_unknown_offset_cell_place};

fn symbolic_cell(base: &Place, id: usize, ty: TypeId) -> Place {
    let offset = Place::temporary(ResourceId(id), ty);
    raw_memory_cell_place(
        &base.clone().with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
                place: Box::new(offset),
            }),
            ty,
        ),
        ty,
    )
}

fn non_copy_owned_type(types: &mut TypeCtx) -> TypeId {
    types.register_named(
        String::from("Owned"),
        TypeKind::Struct {
            name: String::from("Owned"),
            type_params: vec![],
            fields: vec![],
            field_names: vec![],
        },
    )
}

#[test]
fn availability_state_prefers_non_initialized_ancestor_over_exact_initialized_field() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.i32());
    let i32_ty = types.i32();
    let aggregate = Place::local(String::from("aggregate"), i32_ty);
    let field = aggregate.clone().with_projection(
        PlaceProjection::Field {
            index: 0,
            offset_bytes: 0,
        },
        i32_ty,
    );
    let mut cells = CellTable::default();

    cells.mark_initialized(&field);
    cells.set_state(&aggregate, CellState::Moved);

    assert_eq!(
        cells.availability_state_with_types(&types, &field),
        CellState::Moved
    );
}

#[test]
fn copy_store_preserves_unknown_offset_initialized_copy_fact() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.i32());
    let i32_ty = types.i32();
    let base = Place::local(String::from("pref"), i32_ty);
    let mut cells = CellTable::default();

    cells.mark_initialized(&raw_memory_unknown_offset_cell_place(&base, i32_ty));
    let stored = symbolic_cell(&base, 0, i32_ty);
    let stored_address = raw_cell_address_prefix(&stored).expect("raw cell address");
    cells.clear_raw_cells_overwritten_by_store(&stored_address, i32_ty, &types);
    cells.mark_initialized(&stored);

    let loaded = symbolic_cell(&base, 1, i32_ty);
    assert_eq!(
        cells.availability_state_with_types(&types, &loaded),
        CellState::Initialized(i32_ty)
    );
}

#[test]
fn raw_move_clears_overlapping_initialized_raw_byte_range() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.u8());
    let owned_ty = non_copy_owned_type(&mut types);
    let i32_ty = types.i32();
    let base = Place::local(String::from("buf"), i32_ty);
    let count = Place::temporary(ResourceId(0), i32_ty);
    let mut cells = CellTable::default();

    cells.mark_initialized_raw_byte_range(
        &base,
        &count,
        InitializedRawRangeUnit::Bytes,
        types.u8(),
    );
    assert_eq!(cells.initialized_raw_byte_ranges().len(), 1);

    cells.mark_raw_cell_moved(&base, owned_ty);

    assert!(cells.initialized_raw_byte_ranges().is_empty());
    assert_eq!(
        cells.availability_state_with_types(&types, &raw_memory_cell_place(&base, owned_ty)),
        CellState::Moved
    );
}

#[test]
fn raw_move_clears_overlapping_initialized_raw_cell_entry() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.i32());
    let owned_ty = non_copy_owned_type(&mut types);
    let i32_ty = types.i32();
    let base = Place::local(String::from("buf"), i32_ty);
    let aggregate_cell = raw_memory_cell_place(&base, owned_ty);
    let moved_address = base.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(4)),
        i32_ty,
    );
    let moved_cell = raw_memory_cell_place(&moved_address, owned_ty);
    let mut cells = CellTable::default();

    cells.mark_initialized(&aggregate_cell);
    cells.mark_raw_cell_moved(&moved_address, owned_ty);

    assert!(cells.entries().iter().all(|entry| {
        entry.place != aggregate_cell || !matches!(entry.state, CellState::Initialized(_))
    }));
    assert_eq!(
        cells.availability_state_with_types(&types, &moved_cell),
        CellState::Moved
    );
}
