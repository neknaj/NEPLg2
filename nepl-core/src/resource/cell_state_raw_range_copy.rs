use alloc::vec::Vec;

use super::cell_state::{place_suffix_after_address_prefix, CellTable};
use super::cell_state_raw_range_model::{InitializedRawByteRange, InitializedRawRangeUnit};
use super::cell_state_raw_range_offset::NormalizedRawOffset;
use super::i32_extent_proof::{
    copied_element_count_from_byte_count, place_covers_scaled_count, scalar_place_covers_count,
};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection};
use super::place_utils::place_with_suffix;

impl CellTable {
    pub(super) fn copy_initialized_raw_byte_ranges_under(
        &self,
        source: &Place,
        target: &Place,
    ) -> Vec<InitializedRawByteRange> {
        self.initialized_raw_byte_ranges
            .iter()
            .filter_map(|range| {
                let suffix = place_suffix_after_address_prefix(&range.address, source)?;
                Some(InitializedRawByteRange {
                    address: place_with_suffix(target, &suffix, range.address.ty),
                    count: range.count.clone(),
                    unit: range.unit,
                    ty: range.ty,
                })
            })
            .collect()
    }

    pub(super) fn copy_initialized_raw_byte_ranges_for_bulk_copy(
        &self,
        source: &Place,
        target: &Place,
        count: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) -> Vec<InitializedRawByteRange> {
        self.initialized_raw_byte_ranges
            .iter()
            .filter_map(|range| {
                let suffix = place_suffix_after_address_prefix(&range.address, source)?;
                let offset = known_range_start_offset(&suffix)?;
                let address = place_with_suffix(target, &suffix, range.address.ty);
                initialized_range_transferred_by_bulk_copy(
                    range,
                    address,
                    offset,
                    count,
                    raw_aliases,
                )
            })
            .collect()
    }
}

fn known_range_start_offset(suffix: &[PlaceProjection]) -> Option<usize> {
    match NormalizedRawOffset::from_suffix(suffix)? {
        NormalizedRawOffset::Known(offset) => Some(offset),
        NormalizedRawOffset::Symbolic { .. } | NormalizedRawOffset::ScaledSymbolic { .. } => None,
    }
}

fn initialized_range_transferred_by_bulk_copy(
    range: &InitializedRawByteRange,
    address: Place,
    offset: usize,
    count: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Option<InitializedRawByteRange> {
    if copy_count_covers_full_range(range, offset, count, raw_aliases) {
        return Some(InitializedRawByteRange {
            address,
            count: raw_aliases.canonicalize_scalar(range.count()),
            unit: range.unit,
            ty: range.ty,
        });
    }
    if offset != 0 {
        return None;
    }
    match range.unit {
        InitializedRawRangeUnit::Bytes => {
            if scalar_place_covers_count(raw_aliases, range.count(), count) {
                Some(InitializedRawByteRange {
                    address,
                    count: raw_aliases.canonicalize_scalar(count),
                    unit: range.unit,
                    ty: range.ty,
                })
            } else {
                None
            }
        }
        InitializedRawRangeUnit::Elements { stride } => {
            let element_count = copied_element_count_from_byte_count(raw_aliases, count, stride)?;
            if scalar_place_covers_count(raw_aliases, range.count(), &element_count) {
                Some(InitializedRawByteRange {
                    address,
                    count: element_count,
                    unit: range.unit,
                    ty: range.ty,
                })
            } else {
                None
            }
        }
    }
}

fn copy_count_covers_full_range(
    range: &InitializedRawByteRange,
    offset: usize,
    count: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    if offset == 0 {
        return place_covers_range_bytes(count, range, raw_aliases);
    }
    if let Some(range_bytes) = initialized_range_byte_count_value(range, raw_aliases) {
        let Some(required) = offset.checked_add(range_bytes) else {
            return false;
        };
        let Ok(required) = i32::try_from(required) else {
            return false;
        };
        return scalar_place_covers_count(
            raw_aliases,
            count,
            &Place::i32_constant(required, count.ty),
        );
    }
    false
}

fn place_covers_range_bytes(
    available: &Place,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    match range.unit {
        InitializedRawRangeUnit::Bytes => {
            scalar_place_covers_count(raw_aliases, available, range.count())
        }
        InitializedRawRangeUnit::Elements { stride } => {
            place_covers_scaled_count(raw_aliases, available, range.count(), stride)
        }
    }
}

fn initialized_range_byte_count_value(
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> Option<usize> {
    let count = usize::try_from(raw_aliases.i32_value(range.count())?).ok()?;
    match range.unit {
        InitializedRawRangeUnit::Bytes => Some(count),
        InitializedRawRangeUnit::Elements { stride } => count.checked_mul(stride),
    }
}
