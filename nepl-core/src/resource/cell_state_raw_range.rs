use alloc::vec::Vec;

use crate::types::TypeId;

use super::cell_state::{place_suffix_after_address_prefix, raw_addresses_overlap, CellTable};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    I32ValueCondition, Place, PlaceProjection, ResourceI32RelationOp, ResourceOffset,
};
use super::place_utils::replace_place_prefix;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InitializedRawByteRange {
    address: Place,
    count: Place,
    ty: TypeId,
}

impl CellTable {
    pub(super) fn raw_cell_initialized_by_byte_range(
        &self,
        address: &Place,
        ty: TypeId,
        raw_aliases: &RawCellAddressAliases,
    ) -> bool {
        self.initialized_raw_byte_ranges.iter().any(|range| {
            range.ty == ty && raw_byte_range_address_covers(range, address, raw_aliases)
        })
    }

    pub(super) fn mark_initialized_raw_byte_range(
        &mut self,
        address: &Place,
        count: &Place,
        ty: TypeId,
    ) {
        self.clear_initialized_raw_byte_ranges_under(address);
        let range = InitializedRawByteRange {
            address: address.clone(),
            count: count.clone(),
            ty,
        };
        if !self
            .initialized_raw_byte_ranges
            .iter()
            .any(|existing| existing == &range)
        {
            self.initialized_raw_byte_ranges.push(range);
        }
    }

    pub(super) fn clear_initialized_raw_byte_ranges_under(&mut self, address: &Place) {
        self.initialized_raw_byte_ranges
            .retain(|range| !raw_addresses_overlap(&range.address, address));
    }
}

pub(super) fn rekey_initialized_raw_byte_ranges(
    ranges: &mut Vec<InitializedRawByteRange>,
    source: &Place,
    target: &Place,
) {
    for range in ranges {
        if let Some(address) = replace_place_prefix(&range.address, source, target) {
            range.address = address;
        }
        if let Some(count) = replace_place_prefix(&range.count, source, target) {
            range.count = count;
        }
    }
}

pub(super) fn merge_initialized_raw_byte_ranges(
    paths: &[CellTable],
) -> Vec<InitializedRawByteRange> {
    let Some((first, rest)) = paths.split_first() else {
        return Vec::new();
    };
    first
        .initialized_raw_byte_ranges
        .iter()
        .filter(|range| {
            rest.iter()
                .all(|path| path.initialized_raw_byte_ranges.contains(range))
        })
        .cloned()
        .collect()
}

fn raw_byte_range_address_covers(
    range: &InitializedRawByteRange,
    address: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let Some(suffix) = place_suffix_after_address_prefix(address, &range.address) else {
        return false;
    };
    match suffix.as_slice() {
        [] => raw_aliases
            .i32_value(&range.count)
            .is_some_and(|count| count > 0),
        [PlaceProjection::StorageOffset(ResourceOffset::Known(offset))] => {
            let Ok(offset) = i64::try_from(*offset) else {
                return false;
            };
            raw_aliases
                .i32_value(&range.count)
                .is_some_and(|count| count >= 0 && offset < i64::from(count))
        }
        [PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place })] => {
            raw_aliases.i32_condition_truth(place, I32ValueCondition::NonNegative) == Some(true)
                && raw_aliases.i32_relation_truth(place, ResourceI32RelationOp::Lt, &range.count)
                    == Some(true)
        }
        _ => false,
    }
}
