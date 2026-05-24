extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::cell_state_raw_range_offset::NormalizedRawOffset;
use super::collection_slot_drop_traversal_known_range::{
    known_slot_offset_is_definitely_outside_initialized_count,
    known_slot_offset_is_inside_initialized_count,
};
use super::collection_slot_drop_traversal_symbolic_range::symbolic_slot_offset_is_inside_initialized_count;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn collection_slot_offset_is_inside_initialized_count(
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
        NormalizedRawOffset::Symbolic { place, known } => {
            symbolic_slot_offset_is_inside_initialized_count(
                types,
                raw_aliases,
                &place,
                1,
                known,
                initialized_count,
                expected_ty,
            )
        }
        NormalizedRawOffset::ScaledSymbolic {
            place,
            scale,
            known,
        } => symbolic_slot_offset_is_inside_initialized_count(
            types,
            raw_aliases,
            &place,
            scale,
            known,
            initialized_count,
            expected_ty,
        ),
    }
}

pub(super) fn collection_slot_offset_is_definitely_outside_initialized_count(
    types: &TypeCtx,
    raw_aliases: &RawCellAddressAliases,
    slot: &Place,
    storage: &Place,
    initialized_count: &Place,
    expected_ty: TypeId,
) -> bool {
    let Some(suffix) = collection_slot_offset_suffix(slot, storage) else {
        return true;
    };
    let Some(offset) = NormalizedRawOffset::from_suffix(&suffix) else {
        return true;
    };
    match offset {
        NormalizedRawOffset::Known(offset) => {
            known_slot_offset_is_definitely_outside_initialized_count(
                types,
                raw_aliases,
                offset,
                initialized_count,
                expected_ty,
            )
        }
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
