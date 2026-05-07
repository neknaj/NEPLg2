extern crate alloc;

use crate::types::TypeId;
use alloc::vec::Vec;

use super::cell_state::{place_suffix_after_address_prefix, CellTable};
use super::cell_state_raw_range::InitializedRawRangeUnit;
use super::cell_state_raw_range_model::InitializedRawByteRange;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection, ResourceOffset};

impl CellTable {
    pub(super) fn mark_initialized_raw_byte_range_extending_appended_difference(
        &mut self,
        appended_address: &Place,
        appended_count: &Place,
        unit: InitializedRawRangeUnit,
        ty: TypeId,
        raw_aliases: &RawCellAddressAliases,
    ) {
        let composed = self.initialized_raw_byte_ranges_by_appended_difference(
            appended_address,
            appended_count,
            unit,
            ty,
            raw_aliases,
        );
        self.mark_initialized_raw_byte_range(appended_address, appended_count, unit, ty);
        self.extend_initialized_raw_byte_ranges(composed);
    }

    fn initialized_raw_byte_ranges_by_appended_difference(
        &self,
        appended_address: &Place,
        appended_count: &Place,
        unit: InitializedRawRangeUnit,
        ty: TypeId,
        raw_aliases: &RawCellAddressAliases,
    ) -> Vec<InitializedRawByteRange> {
        let appended_address = raw_aliases.canonicalize(appended_address);
        let mut composed = Vec::new();
        let difference_sources = raw_aliases.i32_difference_sources(appended_count);
        for (total_count, prefix_count) in difference_sources {
            let prefix_count = raw_aliases.canonicalize_scalar(&prefix_count);
            for prefix in &self.initialized_raw_byte_ranges {
                if prefix.unit() != unit || prefix.ty() != ty {
                    continue;
                }
                let prefix_address = raw_aliases.canonicalize(prefix.address());
                let candidate_count = raw_aliases.canonicalize_scalar(prefix.count());
                let count_matches = candidate_count == prefix_count;
                let starts = count_matches
                    && appended_address_starts_at_count(
                        &appended_address,
                        &prefix_address,
                        &prefix_count,
                        raw_aliases,
                    );
                if !count_matches {
                    continue;
                }
                if !starts {
                    continue;
                }
                composed.push(InitializedRawByteRange {
                    address: prefix_address,
                    count: total_count.clone(),
                    unit,
                    ty,
                });
            }
        }
        composed
    }
}

fn appended_address_starts_at_count(
    appended_address: &Place,
    prefix_address: &Place,
    prefix_count: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let Some(suffix) = place_suffix_after_address_prefix(appended_address, prefix_address) else {
        return false;
    };
    match suffix.as_slice() {
        [PlaceProjection::StorageOffset(ResourceOffset::Symbolic { place })] => {
            raw_aliases.canonicalize_scalar(place) == *prefix_count
        }
        [PlaceProjection::StorageOffset(ResourceOffset::Known(offset))] => {
            raw_aliases
                .i32_value(prefix_count)
                .and_then(|value| usize::try_from(value).ok())
                == Some(*offset)
        }
        _ => false,
    }
}
