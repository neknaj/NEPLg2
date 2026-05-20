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
fn byte_range_accepts_guarded_symbolic_offset_after_count_is_loaded_to_local() {
    let types = TypeCtx::new();
    let ty = types.u8();
    let count_ty = types.i32();
    let address = local("p");
    let used_ptr = local("used_ptr");
    let used_cell = raw_memory_cell_place(&used_ptr, count_ty);
    let used_value = Place::temporary(ResourceId(1), count_ty);
    let used = local("used");
    let source = local("i");
    let offset = Place::temporary(ResourceId(2), count_ty);
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
fn raw_range_count_copy_rejects_mismatched_value_type() {
    let types = TypeCtx::new();
    let ty = types.u8();
    let count_ty = types.i32();
    let address = local("p");
    let used_ptr = local("used_ptr");
    let used_cell = raw_memory_cell_place(&used_ptr, count_ty);
    let bad_used_value = Place::temporary(ResourceId(1), ty);
    let used = local("used");
    let mut cells = CellTable::default();
    let raw_aliases = RawCellAddressAliases::default();

    cells.mark_initialized_raw_byte_range(&address, &used_cell, InitializedRawRangeUnit::Bytes, ty);
    cells.copy_initialized_raw_byte_ranges_through_value_aliases(
        &used_cell,
        &bad_used_value,
        &raw_aliases,
    );
    cells.copy_initialized_raw_byte_ranges_through_value_aliases(
        &bad_used_value,
        &used,
        &raw_aliases,
    );

    assert!(cells
        .initialized_raw_byte_ranges()
        .iter()
        .all(|range| range.count() != &used));
}
