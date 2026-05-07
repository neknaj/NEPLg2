extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::cell_state_raw_range_model::InitializedRawByteRange;
use super::model::{Place, PlaceProjection};
use super::place_utils::{place_suffix_after_prefix, place_with_suffix, replace_place_prefix};

impl CellTable {
    pub(super) fn clear_initialized_raw_byte_ranges_through_value(&mut self, place: &Place) {
        self.initialized_raw_byte_ranges.retain(|range| {
            place_suffix_after_prefix(&range.address, place).is_none()
                && place_suffix_after_prefix(&range.count, place).is_none()
        });
    }

    pub(super) fn copy_initialized_raw_byte_ranges_through_value(
        &mut self,
        source: &Place,
        target: &Place,
    ) {
        let mut copied = Vec::new();
        for range in &self.initialized_raw_byte_ranges {
            let address = replace_place_prefix(&range.address, source, target);
            let count = replace_raw_range_count_value_prefix(&range.count, source, target);
            if address.is_none() && count.is_none() {
                continue;
            }
            copied.push(InitializedRawByteRange {
                address: address.unwrap_or_else(|| range.address.clone()),
                count: count.unwrap_or_else(|| range.count.clone()),
                unit: range.unit,
                ty: range.ty,
            });
        }
        for range in copied {
            if !self.initialized_raw_byte_ranges.contains(&range) {
                self.initialized_raw_byte_ranges.push(range);
            }
        }
    }
}

pub(super) fn replace_raw_range_count_value_prefix(
    place: &Place,
    source: &Place,
    target: &Place,
) -> Option<Place> {
    let suffix = place_suffix_after_prefix(place, source)?;
    if suffix.iter().any(|projection| {
        matches!(
            projection,
            PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
        )
    }) {
        return None;
    }
    Some(place_with_suffix(target, &suffix, place.ty))
}
