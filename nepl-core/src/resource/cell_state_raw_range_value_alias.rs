extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::cell_state_raw_range_model::InitializedRawByteRange;
use super::cell_state_raw_range_value::replace_raw_range_count_value_prefix;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::{place_suffix_after_prefix, place_with_suffix, replace_place_prefix};

impl CellTable {
    pub(super) fn copy_initialized_raw_byte_ranges_through_value_aliases(
        &mut self,
        source: &Place,
        target: &Place,
        raw_aliases: &RawCellAddressAliases,
    ) {
        self.copy_initialized_raw_byte_ranges_through_value(source, target);
        let source_address_canonical = raw_aliases.canonicalize(source);
        let mut copied = Vec::new();
        for range in &self.initialized_raw_byte_ranges {
            let address = replace_place_prefix(&range.address, source, target).or_else(|| {
                let canonical = raw_aliases.canonicalize(&range.address);
                let suffix = place_suffix_after_prefix(&canonical, &source_address_canonical)?;
                Some(place_with_suffix(target, &suffix, range.address.ty))
            });
            let count = replace_raw_range_count_value_prefix(&range.count, source, target)
                .or_else(|| range_count_alias_replacement(raw_aliases, range, source, target));
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
        self.extend_initialized_raw_byte_ranges(copied);
    }
}

fn range_count_alias_replacement(
    raw_aliases: &RawCellAddressAliases,
    range: &InitializedRawByteRange,
    source: &Place,
    target: &Place,
) -> Option<Place> {
    if range.count.ty != source.ty || source.ty != target.ty {
        return None;
    }
    if raw_aliases.canonicalize_scalar(&range.count) == raw_aliases.canonicalize_scalar(source) {
        return Some(target.clone());
    }
    let range_aliases = raw_aliases.scalar_aliases_for_value(&range.count);
    let source_aliases = raw_aliases.scalar_aliases_for_value(source);
    scalar_aliases_overlap(&range_aliases, &source_aliases).then(|| target.clone())
}

fn scalar_aliases_overlap(left: &[Place], right: &[Place]) -> bool {
    left.iter().any(|place| right.contains(place))
}
