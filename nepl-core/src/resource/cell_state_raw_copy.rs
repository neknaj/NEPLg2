use alloc::vec::Vec;

use crate::layout::storage_size_bytes;
use crate::types::{TypeCtx, TypeId};

use super::cell_state::{raw_cell_suffix_after_address, CellTable};
use super::cell_state_raw_range_offset::NormalizedRawOffset;
use super::i32_extent_proof::scalar_place_covers_count;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{CellState, CellStateEntry, Place, PlaceProjection};
use super::place_utils::place_with_suffix;

impl CellTable {
    pub(super) fn copy_initialized_copy_raw_cells_covered_by_count(
        &self,
        source: &Place,
        destination: &Place,
        count: &Place,
        raw_aliases: &RawCellAddressAliases,
        types: &TypeCtx,
    ) -> Vec<CellStateEntry> {
        self.entries()
            .iter()
            .filter_map(|entry| {
                let suffix = raw_cell_suffix_after_address(&entry.place, source)?;
                let CellState::Initialized(ty) = entry.state else {
                    return None;
                };
                if !types.is_copy(ty) {
                    return None;
                }
                if !raw_cell_suffix_is_covered_by_byte_count(
                    &suffix,
                    entry.place.ty,
                    count,
                    raw_aliases,
                    types,
                ) {
                    return None;
                }
                Some(CellStateEntry {
                    place: place_with_suffix(destination, &suffix, entry.place.ty),
                    state: entry.state.clone(),
                })
            })
            .collect()
    }
}

fn raw_cell_suffix_is_covered_by_byte_count(
    suffix: &[PlaceProjection],
    cell_ty: TypeId,
    count: &Place,
    raw_aliases: &RawCellAddressAliases,
    types: &TypeCtx,
) -> bool {
    let Some(offset) = raw_cell_suffix_known_byte_offset(suffix) else {
        return false;
    };
    let Some(size) = storage_size_bytes(types, cell_ty).checked_add(offset) else {
        return false;
    };
    let Ok(size) = i32::try_from(size) else {
        return false;
    };
    let required = Place::i32_constant(size, count.ty);
    scalar_place_covers_count(raw_aliases, count, &required)
}

fn raw_cell_suffix_known_byte_offset(suffix: &[PlaceProjection]) -> Option<usize> {
    let deref_index = suffix
        .iter()
        .position(|projection| matches!(projection, PlaceProjection::Deref))?;
    let address_offset = match NormalizedRawOffset::from_suffix(&suffix[..deref_index])? {
        NormalizedRawOffset::Known(offset) => offset,
        NormalizedRawOffset::Symbolic { .. } | NormalizedRawOffset::ScaledSymbolic { .. } => {
            return None;
        }
    };
    suffix[deref_index + 1..]
        .iter()
        .try_fold(address_offset, |offset, projection| match projection {
            PlaceProjection::Field { offset_bytes, .. }
            | PlaceProjection::TupleField { offset_bytes, .. } => offset.checked_add(*offset_bytes),
            PlaceProjection::Deref
            | PlaceProjection::StorageOffset(_)
            | PlaceProjection::EnumPayload { .. } => None,
        })
}
