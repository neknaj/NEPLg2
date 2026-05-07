use crate::types::TypeId;

use super::cell_state::{place_suffix_after_address_prefix, CellTable};
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::cell_state_raw_range_model::InitializedRawByteRange;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

impl CellTable {
    pub(super) fn raw_range_initialized_for_count(
        &self,
        address: &Place,
        count: &Place,
        ty: TypeId,
        raw_aliases: &RawCellAddressAliases,
    ) -> bool {
        let address = raw_aliases.canonicalize(address);
        let count = raw_aliases.canonicalize(count);
        self.initialized_raw_byte_ranges.iter().any(|range| {
            range.ty == ty
                && place_suffix_after_address_prefix(&address, range.address())
                    .is_some_and(|suffix| suffix.is_empty())
                && initialized_range_count_covers(range, &count, raw_aliases)
        })
    }
}

fn initialized_range_count_covers(
    range: &InitializedRawByteRange,
    requested: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let available = raw_aliases.canonicalize(range.count());
    if available == *requested {
        return matches!(
            range.unit(),
            InitializedRawRangeUnit::Bytes | InitializedRawRangeUnit::Elements { stride: 1.. }
        );
    }
    let (Some(available), Some(requested)) = (
        raw_aliases.i32_value(&available),
        raw_aliases.i32_value(requested),
    ) else {
        return false;
    };
    let (Ok(available), Ok(requested)) = (usize::try_from(available), usize::try_from(requested))
    else {
        return false;
    };
    let available_bytes = match range.unit() {
        InitializedRawRangeUnit::Bytes => Some(available),
        InitializedRawRangeUnit::Elements { stride } => available.checked_mul(stride),
    };
    available_bytes.is_some_and(|bytes| requested <= bytes)
}
