use alloc::vec::Vec;

use crate::types::TypeId;

use super::cell_state::{place_suffix_after_address_prefix, raw_addresses_overlap, CellTable};
use super::cell_state_raw_range_cover::raw_byte_range_address_covers;
pub(super) use super::cell_state_raw_range_model::{
    InitializedRawByteRange, InitializedRawRangeUnit,
};
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::{place_with_suffix, replace_place_prefix};

impl CellTable {
    pub(super) fn initialized_raw_byte_ranges(&self) -> &[InitializedRawByteRange] {
        &self.initialized_raw_byte_ranges
    }

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
        unit: InitializedRawRangeUnit,
        ty: TypeId,
    ) {
        self.clear_initialized_raw_byte_ranges_under(address);
        self.add_initialized_raw_byte_range(address, count, unit, ty);
    }

    pub(super) fn add_initialized_raw_byte_range(
        &mut self,
        address: &Place,
        count: &Place,
        unit: InitializedRawRangeUnit,
        ty: TypeId,
    ) {
        let range = InitializedRawByteRange {
            address: address.clone(),
            count: count.clone(),
            unit,
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

    pub(super) fn extend_initialized_raw_byte_ranges(
        &mut self,
        ranges: Vec<InitializedRawByteRange>,
    ) {
        for range in ranges {
            if !self.initialized_raw_byte_ranges.contains(&range) {
                self.initialized_raw_byte_ranges.push(range);
            }
        }
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
