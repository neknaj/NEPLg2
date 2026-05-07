extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::cell_state::{place_suffix_after_address_prefix, CellTable};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::RawCellInitializationReturnByteRange;
use super::model::{CellState, Place, PlaceProjection};

pub(super) fn collect_return_initialized_raw_byte_ranges(
    out: &mut Vec<RawCellInitializationReturnByteRange>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
) {
    let return_aliases = raw_aliases.aliases_for(value);
    for range in cells.initialized_raw_byte_ranges() {
        let address_suffixes =
            collect_return_value_suffixes(raw_aliases, range.address(), &return_aliases);
        if address_suffixes.is_empty() {
            continue;
        }
        let count_suffixes =
            collect_return_count_suffixes(cells, raw_aliases, range.count(), &return_aliases);
        for (address_suffix, address_ty) in &address_suffixes {
            for (count_suffix, count_ty) in &count_suffixes {
                push_unique_return_byte_range(
                    out,
                    RawCellInitializationReturnByteRange {
                        address_suffix: address_suffix.clone(),
                        address_ty: *address_ty,
                        count_suffix: count_suffix.clone(),
                        count_ty: *count_ty,
                        unit: range.unit(),
                        ty: range.ty(),
                    },
                );
            }
        }
    }
}

fn collect_return_value_suffixes(
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
    return_aliases: &[Place],
) -> Vec<(Vec<PlaceProjection>, TypeId)> {
    let mut out = Vec::new();
    for value_alias in raw_aliases.aliases_for(value) {
        for return_alias in return_aliases {
            let Some(suffix) = place_suffix_after_address_prefix(&value_alias, return_alias) else {
                continue;
            };
            push_unique_return_suffix(&mut out, suffix, value_alias.ty);
        }
    }
    out
}

fn collect_return_count_suffixes(
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    count: &Place,
    return_aliases: &[Place],
) -> Vec<(Vec<PlaceProjection>, TypeId)> {
    let count = raw_aliases.canonicalize(count);
    let mut out = Vec::new();
    for entry in cells.entries() {
        if !matches!(entry.state, CellState::Initialized(_)) {
            continue;
        }
        if raw_aliases.canonicalize(&entry.place) != count {
            continue;
        }
        for entry_alias in raw_aliases.aliases_for(&entry.place) {
            for return_alias in return_aliases {
                let Some(suffix) = place_suffix_after_address_prefix(&entry_alias, return_alias)
                else {
                    continue;
                };
                push_unique_return_suffix(&mut out, suffix, entry_alias.ty);
            }
        }
    }
    out
}

fn push_unique_return_byte_range(
    ranges: &mut Vec<RawCellInitializationReturnByteRange>,
    range: RawCellInitializationReturnByteRange,
) {
    if !ranges.iter().any(|existing| existing == &range) {
        ranges.push(range);
    }
}

fn push_unique_return_suffix(
    suffixes: &mut Vec<(Vec<PlaceProjection>, TypeId)>,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
) {
    if !suffixes
        .iter()
        .any(|(existing_suffix, existing_ty)| existing_suffix == &suffix && *existing_ty == ty)
    {
        suffixes.push((suffix, ty));
    }
}
