extern crate alloc;

use alloc::vec::Vec;

use crate::layout::storage_size_bytes;
use crate::types::{TypeCtx, TypeId};

use super::cell_state_raw_range_offset::NormalizedRawOffset;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection, ResourceI32RelationOp};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn collection_slot_known_offset_is_inside_initialized_count(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    slot: &Place,
    storage: &Place,
    initialized_count: &Place,
    expected_ty: TypeId,
) -> bool {
    let Some(suffix) = collection_slot_offset_suffix(slot, storage) else {
        return false;
    };
    let Some(offset) = NormalizedRawOffset::from_suffix(&suffix) else {
        return false;
    };
    match offset {
        NormalizedRawOffset::Known(offset) => known_slot_offset_is_inside_initialized_count(
            types,
            raw_aliases,
            offset,
            initialized_count,
            expected_ty,
        ),
        NormalizedRawOffset::Symbolic { .. } | NormalizedRawOffset::ScaledSymbolic { .. } => false,
    }
}

fn collection_slot_offset_suffix(slot: &Place, storage: &Place) -> Option<Vec<PlaceProjection>> {
    let mut suffix = place_suffix_after_prefix(slot, storage)?;
    match suffix.last() {
        Some(PlaceProjection::Deref) => {
            suffix.pop();
            Some(suffix)
        }
        _ => Some(suffix),
    }
}

fn known_slot_offset_is_inside_initialized_count(
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
