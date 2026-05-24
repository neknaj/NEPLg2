use crate::layout::storage_size_bytes;
use crate::types::{TypeCtx, TypeId};

use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceI32RelationOp};

pub(super) fn known_slot_offset_is_inside_initialized_count(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    offset: usize,
    initialized_count: &Place,
    expected_ty: TypeId,
) -> bool {
    let stride = storage_size_bytes(types, expected_ty);
    if stride == 0 {
        let first_element = Place::i32_constant(0, initialized_count.ty);
        return raw_aliases.i32_relation_truth(
            &first_element,
            ResourceI32RelationOp::Lt,
            initialized_count,
        ) == Some(true);
    }
    if offset % stride != 0 {
        return false;
    }
    let Some(index) = offset.checked_div(stride) else {
        return false;
    };
    let Ok(index) = i32::try_from(index) else {
        return false;
    };
    let index = Place::i32_constant(index, initialized_count.ty);
    raw_aliases.i32_relation_truth(&index, ResourceI32RelationOp::Lt, initialized_count)
        == Some(true)
}

pub(super) fn known_slot_offset_is_definitely_outside_initialized_count(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    offset: usize,
    initialized_count: &Place,
    expected_ty: TypeId,
) -> bool {
    let stride = storage_size_bytes(types, expected_ty);
    if stride == 0 {
        let first_element = Place::i32_constant(0, initialized_count.ty);
        return raw_aliases.i32_relation_truth(
            &first_element,
            ResourceI32RelationOp::Lt,
            initialized_count,
        ) == Some(false);
    }
    if offset % stride != 0 {
        return true;
    }
    let Some(index) = offset.checked_div(stride) else {
        return true;
    };
    let Ok(index) = i32::try_from(index) else {
        return true;
    };
    let index = Place::i32_constant(index, initialized_count.ty);
    raw_aliases.i32_relation_truth(&index, ResourceI32RelationOp::Lt, initialized_count)
        == Some(false)
}
