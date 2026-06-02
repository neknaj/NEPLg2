extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::cell_state_raw_range::{merge_initialized_raw_byte_range_refs, InitializedRawByteRange};
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

pub(super) fn merge_initialized_raw_byte_range_refs_with_raw_aliases(
    paths: &[&CellTable],
    raw_alias_paths: &[&RawCellAddressAliases],
    merged_raw_aliases: &RawCellAddressAliases,
) -> Vec<InitializedRawByteRange> {
    if paths.len() != raw_alias_paths.len() {
        return merge_initialized_raw_byte_range_refs(paths);
    }
    let Some((first, rest_paths)) = paths.split_first() else {
        return Vec::new();
    };
    let Some((first_aliases, rest_aliases)) = raw_alias_paths.split_first() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for range in &first.initialized_raw_byte_ranges {
        for candidate in
            initialized_raw_byte_range_merge_candidates(range, first_aliases, merged_raw_aliases)
        {
            if rest_paths
                .iter()
                .zip(rest_aliases.iter())
                .all(|(path, aliases)| {
                    path_has_initialized_raw_byte_range(path, &candidate, aliases)
                })
            {
                push_unique_initialized_raw_byte_range(&mut out, candidate);
            }
        }
    }
    out
}

fn initialized_raw_byte_range_merge_candidates(
    range: &InitializedRawByteRange,
    first_aliases: &RawCellAddressAliases,
    merged_aliases: &RawCellAddressAliases,
) -> Vec<InitializedRawByteRange> {
    let mut out = Vec::new();
    push_unique_initialized_raw_byte_range(&mut out, range.clone());
    push_unique_initialized_raw_byte_range(
        &mut out,
        InitializedRawByteRange {
            address: first_aliases.canonicalize(&range.address),
            count: first_aliases.canonicalize_scalar(&range.count),
            unit: range.unit,
            ty: range.ty,
        },
    );
    push_unique_initialized_raw_byte_range(
        &mut out,
        InitializedRawByteRange {
            address: merged_aliases.canonicalize(&range.address),
            count: merged_aliases.canonicalize_scalar(&range.count),
            unit: range.unit,
            ty: range.ty,
        },
    );
    out
}

fn push_unique_initialized_raw_byte_range(
    ranges: &mut Vec<InitializedRawByteRange>,
    range: InitializedRawByteRange,
) {
    if !ranges.contains(&range) {
        ranges.push(range);
    }
}

fn path_has_initialized_raw_byte_range(
    path: &CellTable,
    expected: &InitializedRawByteRange,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let expected_address = raw_aliases.canonicalize(expected.address());
    let expected_count = raw_aliases.canonicalize_scalar(expected.count());
    path.initialized_raw_byte_ranges.iter().any(|range| {
        range.ty == expected.ty
            && range.unit == expected.unit
            && raw_aliases.canonicalize(range.address()) == expected_address
            && initialized_raw_range_count_covers_same_unit(range, &expected_count, raw_aliases)
    })
}

fn initialized_raw_range_count_covers_same_unit(
    range: &InitializedRawByteRange,
    requested: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> bool {
    let available = raw_aliases.canonicalize_scalar(range.count());
    if available == *requested {
        return true;
    }
    let (Some(available), Some(requested)) = (
        raw_aliases.i32_value(&available),
        raw_aliases.i32_value(requested),
    ) else {
        return false;
    };
    requested <= available
}
