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
    access_size_bytes: usize,
) -> bool {
    let Some(suffix) = place_suffix_after_address_prefix(address, range.address()) else {
        return false;
    };
    let Some(offset) = NormalizedRawOffset::from_suffix(&suffix) else {
        return false;
    };
    offset_is_in_initialized_range(offset, range, raw_aliases, access_size_bytes)
}

fn offset_is_in_initialized_range(
    offset: NormalizedRawOffset,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
    access_size_bytes: usize,
) -> bool {
    match offset {
        NormalizedRawOffset::Known(offset) => {
            known_offset_is_in_initialized_range(offset, range, raw_aliases, access_size_bytes)
        }
        NormalizedRawOffset::Symbolic { place, known } => symbolic_offset_is_in_initialized_range(
            &place,
            known,
            range,
            raw_aliases,
            access_size_bytes,
        ),
        NormalizedRawOffset::ScaledSymbolic {
            place,
            scale,
            known,
        } => scaled_symbolic_offset_is_in_initialized_range(
            &place,
            scale,
            known,
            range,
            raw_aliases,
            access_size_bytes,
        ),
    }
}

fn known_offset_is_in_initialized_range(
    offset: usize,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
    access_size_bytes: usize,
) -> bool {
    let Some(count) = raw_aliases.i32_value(range.count()) else {
        return false;
    };
    let Ok(count) = usize::try_from(count) else {
        return false;
    };
    let Some(end) = offset.checked_add(access_size_bytes) else {
        return false;
    };
    match range.unit() {
        InitializedRawRangeUnit::Bytes => end <= count,
        InitializedRawRangeUnit::Elements { stride } => {
            stride > 0
                && offset % stride == 0
                && access_size_bytes <= stride
                && offset / stride < count
        }
    }
}

fn symbolic_offset_is_in_initialized_range(
    place: &Place,
    known: usize,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
    access_size_bytes: usize,
) -> bool {
    match range.unit() {
        InitializedRawRangeUnit::Bytes => {
            symbolic_offset_is_in_byte_range(place, known, range, raw_aliases, access_size_bytes)
        }
        InitializedRawRangeUnit::Elements { .. } if known == 0 => {
            let Some((source, scale)) = raw_aliases.i32_scaled_source(place) else {
                return false;
            };
            scaled_symbolic_offset_is_in_initialized_range(
                &source,
                scale,
                known,
                range,
                raw_aliases,
                access_size_bytes,
            )
        }
        InitializedRawRangeUnit::Elements { .. } => false,
    }
}

fn symbolic_offset_is_in_byte_range(
    place: &Place,
    known: usize,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
    access_size_bytes: usize,
) -> bool {
    raw_aliases.i32_condition_truth(place, I32ValueCondition::NonNegative) == Some(true)
        && symbolic_offset_end_is_in_byte_range(
            place,
            known,
            access_size_bytes,
            range.count(),
            raw_aliases,
        )
}

fn scaled_symbolic_offset_is_in_initialized_range(
    place: &Place,
    scale: usize,
    known: usize,
    range: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
    access_size_bytes: usize,
) -> bool {
    let InitializedRawRangeUnit::Elements { stride } = range.unit() else {
        return false;
    };
    known == 0
        && scale == stride
        && access_size_bytes <= stride
        && raw_aliases.i32_condition_truth(place, I32ValueCondition::NonNegative) == Some(true)
        && raw_aliases.i32_relation_truth(place, ResourceI32RelationOp::Lt, range.count())
            == Some(true)
}

fn symbolic_offset_end_is_in_byte_range(
    place: &Place,
    known: usize,
    access_size_bytes: usize,
    count: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let Some(end_offset) = known.checked_add(access_size_bytes) else {
        return false;
    };
    if end_offset == 1
        && raw_aliases.i32_relation_truth(place, ResourceI32RelationOp::Lt, count) == Some(true)
    {
        return true;
    }
    let Ok(required_end) = i64::try_from(end_offset) else {
        return false;
    };
    raw_aliases
        .i32_offset_targets(place)
        .into_iter()
        .any(|(target, offset)| {
            offset >= required_end
                && (raw_aliases.i32_relation_truth(&target, ResourceI32RelationOp::Le, count)
                    == Some(true)
                    || raw_aliases.i32_relation_truth(&target, ResourceI32RelationOp::Lt, count)
                        == Some(true)
                    || raw_aliases.i32_relation_truth(&target, ResourceI32RelationOp::Eq, count)
                        == Some(true))
        })
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum NormalizedRawOffset {
    Known(usize),
    Symbolic {
        place: Place,
        known: usize,
    },
    ScaledSymbolic {
        place: Place,
        scale: usize,
        known: usize,
    },
}

impl NormalizedRawOffset {
    fn from_suffix(suffix: &[PlaceProjection]) -> Option<Self> {
        let mut known = 0usize;
        let mut symbolic = None;
        let mut scaled = None;
        for projection in suffix {
            let PlaceProjection::StorageOffset(offset) = projection else {
                return None;
            };
            match offset {
                ResourceOffset::Known(bytes) => {
                    known = known.checked_add(*bytes)?;
                }
                ResourceOffset::Symbolic { place } => {
                    if symbolic.is_some() || scaled.is_some() {
                        return None;
                    }
                    symbolic = Some((**place).clone());
                }
                ResourceOffset::ScaledSymbolic { place, scale } => {
                    if symbolic.is_some() || scaled.is_some() {
                        return None;
                    }
                    scaled = Some(((**place).clone(), *scale));
                }
                ResourceOffset::Unknown => return None,
            }
        }
        if let Some(place) = symbolic {
            Some(Self::Symbolic { place, known })
        } else if let Some((place, scale)) = scaled {
            Some(Self::ScaledSymbolic {
                place,
                scale,
                known,
            })
        } else {
            Some(Self::Known(known))
        }
    }
}
