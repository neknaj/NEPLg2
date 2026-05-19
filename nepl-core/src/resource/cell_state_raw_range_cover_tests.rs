extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;

use crate::types::{TypeCtx, TypeId};

use super::cell_state::CellTable;
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    I32ValueCondition, Place, PlaceProjection, PlaceRoot, ResourceI32RelationOp, ResourceId,
    ResourceOffset,
};
use super::place_utils::raw_memory_cell_place;

fn local(name: &str) -> Place {
    Place::local(String::from(name), TypeId(1))
}

#[test]
fn element_range_accepts_guarded_scaled_symbolic_offset() {
    let types = TypeCtx::new();
    let ty = types.i32();
    let address = local("p");
    let len = local("len");
    let source = local("i");
    let offset = Place::temporary(ResourceId(1), ty);
    let loaded = Place {
        root: PlaceRoot::Local(String::from("p")),
        projections: alloc::vec![PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
            place: Box::new(offset.clone()),
        })],
        ty,
    };
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();

    cells.mark_initialized_raw_byte_range(
        &address,
        &len,
        InitializedRawRangeUnit::Elements { stride: 4 },
        ty,
    );
    raw_aliases.add_i32_scale(&source, &offset, 4);
    raw_aliases.add_i32_condition(&source, I32ValueCondition::NonNegative);
    raw_aliases.add_i32_relation(&source, ResourceI32RelationOp::Lt, &len);

    assert!(cells.raw_cell_initialized_by_byte_range(&loaded, ty, &raw_aliases, &types));
}

#[test]
fn byte_range_accepts_guarded_symbolic_offset_copied_from_local() {
    let types = TypeCtx::new();
    let ty = types.u8();
    let address = local("p");
    let len = local("len");
    let source = local("i");
    let offset = Place::temporary(ResourceId(1), ty);
    let loaded = Place {
        root: PlaceRoot::Local(String::from("p")),
        projections: alloc::vec![PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
            place: Box::new(offset.clone()),
        })],
        ty,
    };
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();

    cells.mark_initialized_raw_byte_range(&address, &len, InitializedRawRangeUnit::Bytes, ty);
    raw_aliases.add_i32_condition(&source, I32ValueCondition::NonNegative);
    raw_aliases.add_i32_relation(&source, ResourceI32RelationOp::Lt, &len);
    raw_aliases.copy_scalar_facts_if_tracked(&source, &offset);

    assert!(cells.raw_cell_initialized_by_byte_range(&loaded, ty, &raw_aliases, &types));
}

#[test]
fn byte_range_accepts_guarded_symbolic_offset_after_count_is_loaded_to_local() {
    let types = TypeCtx::new();
    let ty = types.u8();
    let address = local("p");
    let used_ptr = local("used_ptr");
    let used_cell = raw_memory_cell_place(&used_ptr, ty);
    let used_value = Place::temporary(ResourceId(1), ty);
    let used = local("used");
    let source = local("i");
    let offset = Place::temporary(ResourceId(2), ty);
    let loaded = Place {
        root: PlaceRoot::Local(String::from("p")),
        projections: alloc::vec![PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
            place: Box::new(offset.clone()),
        })],
        ty,
    };
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();

    cells.mark_initialized_raw_byte_range(&address, &used_cell, InitializedRawRangeUnit::Bytes, ty);
    cells.copy_initialized_raw_byte_ranges_through_value_aliases(
        &used_cell,
        &used_value,
        &raw_aliases,
    );
    cells.copy_initialized_raw_byte_ranges_through_value_aliases(&used_value, &used, &raw_aliases);
    raw_aliases.add_i32_condition(&source, I32ValueCondition::NonNegative);
    raw_aliases.add_i32_relation(&source, ResourceI32RelationOp::Lt, &used);
    raw_aliases.copy_scalar_facts_if_tracked(&source, &offset);

    assert!(cells.raw_cell_initialized_by_byte_range(&loaded, ty, &raw_aliases, &types));
}

#[test]
fn byte_range_accepts_i32_load_when_larger_affine_guard_covers_access_end() {
    let types = TypeCtx::new();
    let ty = types.i32();
    let address = local("p");
    let used = local("used");
    let off = local("off");
    let header_end = local("header_end");
    let loaded = Place {
        root: PlaceRoot::Local(String::from("p")),
        projections: alloc::vec![
            PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
                place: Box::new(off.clone()),
            }),
            PlaceProjection::StorageOffset(ResourceOffset::Known(16)),
        ],
        ty,
    };
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();

    cells.mark_initialized_raw_byte_range(&address, &used, InitializedRawRangeUnit::Bytes, ty);
    raw_aliases.add_i32_condition(&off, I32ValueCondition::NonNegative);
    raw_aliases.add_i32_offset(&off, &header_end, 24);
    raw_aliases.add_i32_relation(&header_end, ResourceI32RelationOp::Le, &used);

    assert!(cells.raw_cell_initialized_by_byte_range(&loaded, ty, &raw_aliases, &types));
}

#[test]
fn byte_range_accepts_i32_load_after_affine_guard_source_is_copied_to_temporary() {
    let types = TypeCtx::new();
    let ty = types.i32();
    let address = local("p");
    let used_ptr = local("used_ptr");
    let used_cell = raw_memory_cell_place(&used_ptr, ty);
    let off = local("off");
    let off_read = Place::temporary(ResourceId(1), ty);
    let access_end = Place::temporary(ResourceId(2), ty);
    let loaded = Place {
        root: PlaceRoot::Local(String::from("p")),
        projections: alloc::vec![PlaceProjection::StorageOffset(ResourceOffset::Symbolic {
            place: Box::new(off_read.clone()),
        })],
        ty,
    };
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();

    cells.mark_initialized_raw_byte_range(&address, &used_cell, InitializedRawRangeUnit::Bytes, ty);
    raw_aliases.add_i32_condition(&off, I32ValueCondition::NonNegative);
    raw_aliases.add_i32_offset(&off, &access_end, 4);
    raw_aliases.add_i32_relation(&access_end, ResourceI32RelationOp::Le, &used_cell);
    raw_aliases.copy_alias_if_tracked(&off, &off_read);

    assert!(cells.raw_cell_initialized_by_byte_range(&loaded, ty, &raw_aliases, &types));
}
