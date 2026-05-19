extern crate alloc;

use alloc::string::String;

use crate::types::TypeId;

use super::initialized_alias_origin::RawValueOrigins;
use super::model::{Place, PlaceProjection, ResourceId};

#[test]
fn copy_stable_origin_follows_temporary_source_origin() {
    let ty = TypeId(1);
    let source = Place::local(String::from("data_len"), ty);
    let temporary = Place::temporary(ResourceId(1), ty);
    let raw_cell = Place::temporary(ResourceId(2), ty).with_projection(PlaceProjection::Deref, ty);
    let mut origins = RawValueOrigins::default();

    origins.copy_stable_origin(&source, &temporary);
    origins.copy_stable_origin(&temporary, &raw_cell);

    assert_eq!(origins.origin_for(&raw_cell), source);
}

#[test]
fn origins_for_keeps_intermediate_stable_local_source_before_raw_cell_origin() {
    let ty = TypeId(1);
    let raw_cell =
        Place::local(String::from("used_ptr"), ty).with_projection(PlaceProjection::Deref, ty);
    let used = Place::local(String::from("used"), ty);
    let used_read = Place::temporary(ResourceId(1), ty);
    let mut origins = RawValueOrigins::default();

    origins.copy_stable_origin(&raw_cell, &used);
    origins.copy_stable_origin(&used, &used_read);

    assert_eq!(origins.origin_for(&used_read), raw_cell);
    assert_eq!(origins.origins_for(&used_read), alloc::vec![used, raw_cell]);
}
