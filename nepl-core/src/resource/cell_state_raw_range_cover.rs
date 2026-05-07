extern crate alloc;

use super::cell_state::place_suffix_after_address_prefix;
use super::cell_state_raw_range_model::{InitializedRawByteRange, InitializedRawRangeUnit};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    I32ValueCondition, Place, PlaceProjection, ResourceI32RelationOp, ResourceOffset,
};

pub(super) fn raw_byte_range_address_covers(
    range: &InitializedRawByteRange,
    address: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let Some(suffix) = place_suffix_after_address_prefix(address, range.address()) else {
        return false;
    };
    match suffix.as_slice() {
        [] => raw_range_count_is_positive(range, raw_aliases),
        [PlaceProjection::StorageOffset(ResourceOffset::Known(offset))] => {
            known_offset_is_in_initialized_range(*offset, range, raw_aliases)
        }
        [PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place })] => {
            if symbolic_offset_is_in_byte_range(place, range, raw_aliases) {
                return true;
            }
            let Some((source, scale)) = raw_aliases.i32_scaled_source(place) else {
                return false;
            };
            scaled_symbolic_offset_is_in_initialized_range(&source, scale, range, raw_aliases)
        }
        [PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic { place, scale })] => {
            scaled_symbolic_offset_is_in_initialized_range(place, *scale, range, raw_aliases)
        }
        _ => false,
    }
}

fn raw_range_count_is_positive(
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    raw_aliases
        .i32_value(range.count())
        .is_some_and(|count| count > 0)
        || raw_aliases.i32_condition_truth(range.count(), I32ValueCondition::Positive) == Some(true)
}

fn known_offset_is_in_initialized_range(
    offset: usize,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let Some(count) = raw_aliases.i32_value(range.count()) else {
        return false;
    };
    let Ok(count) = usize::try_from(count) else {
        return false;
    };
    match range.unit() {
        InitializedRawRangeUnit::Bytes => offset < count,
        InitializedRawRangeUnit::Elements { stride } => {
            stride > 0 && offset % stride == 0 && offset / stride < count
        }
    }
}

fn symbolic_offset_is_in_byte_range(
    place: &Place,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    if range.unit() != InitializedRawRangeUnit::Bytes {
        return false;
    }
    raw_aliases.i32_condition_truth(place, I32ValueCondition::NonNegative) == Some(true)
        && raw_aliases.i32_relation_truth(place, ResourceI32RelationOp::Lt, range.count())
            == Some(true)
}

fn scaled_symbolic_offset_is_in_initialized_range(
    place: &Place,
    scale: usize,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let InitializedRawRangeUnit::Elements { stride } = range.unit() else {
        return false;
    };
    scale == stride
        && raw_aliases.i32_condition_truth(place, I32ValueCondition::NonNegative) == Some(true)
        && raw_aliases.i32_relation_truth(place, ResourceI32RelationOp::Lt, range.count())
            == Some(true)
}
