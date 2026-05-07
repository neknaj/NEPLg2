extern crate alloc;

use alloc::boxed::Box;
use alloc::string::String;

use crate::types::TypeId;

use super::cell_state::CellTable;
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    I32ValueCondition, Place, PlaceProjection, PlaceRoot, ResourceI32RelationOp, ResourceId,
    ResourceOffset,
};

fn local(name: &str) -> Place {
    Place::local(String::from(name), TypeId(1))
}

#[test]
fn element_range_accepts_guarded_scaled_symbolic_offset() {
    let ty = TypeId(1);
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

    assert!(cells.raw_cell_initialized_by_byte_range(&loaded, ty, &raw_aliases));
}
