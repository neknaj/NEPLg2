extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::cell_state::{place_suffix_after_address_prefix, CellTable};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::{
    RawCellInitializationParamByteRange, RawCellInitializationReturnByteRange,
};
use super::model::{CellState, Place, PlaceProjection, ResourceLocal};

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

pub(super) fn collect_param_initialized_raw_byte_ranges(
    out: &mut Vec<RawCellInitializationParamByteRange>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
) {
    for range in cells.initialized_raw_byte_ranges() {
        let address_suffixes = collect_param_value_suffixes(raw_aliases, range.address(), params);
        if address_suffixes.is_empty() {
            continue;
        }
        let count_suffixes =
            collect_param_count_suffixes(cells, raw_aliases, range.count(), params);
        for (address_param_index, address_suffix, address_ty) in &address_suffixes {
            for (count_param_index, count_suffix, count_ty) in &count_suffixes {
                push_unique_param_byte_range(
                    out,
                    RawCellInitializationParamByteRange {
                        address_param_index: *address_param_index,
                        address_suffix: address_suffix.clone(),
                        address_ty: *address_ty,
                        count_param_index: *count_param_index,
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
    let count = raw_aliases.canonicalize_scalar(count);
    let mut out = Vec::new();
    for entry in cells.entries() {
        if !matches!(entry.state, CellState::Initialized(_)) {
            continue;
        }
        if raw_aliases.canonicalize_scalar(&entry.place) != count {
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

fn collect_param_value_suffixes(
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
    params: &[ResourceLocal],
) -> Vec<(usize, Vec<PlaceProjection>, TypeId)> {
    let mut out = Vec::new();
    for value_alias in raw_aliases.aliases_for(value) {
        for (param_index, param) in params.iter().enumerate() {
            for param_alias in raw_aliases.aliases_for(&param.place) {
                let Some(suffix) = place_suffix_after_address_prefix(&value_alias, &param_alias)
                else {
                    continue;
                };
                push_unique_param_suffix(&mut out, param_index, suffix, value_alias.ty);
            }
        }
    }
    out
}

fn collect_param_count_suffixes(
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    count: &Place,
    params: &[ResourceLocal],
) -> Vec<(usize, Vec<PlaceProjection>, TypeId)> {
    let count = raw_aliases.canonicalize_scalar(count);
    let mut out = Vec::new();
    for entry in cells.entries() {
        if !matches!(entry.state, CellState::Initialized(_)) {
            continue;
        }
        if raw_aliases.canonicalize_scalar(&entry.place) != count {
            continue;
        }
        for entry_alias in raw_aliases.scalar_aliases_for_value(&entry.place) {
            for (param_index, param) in params.iter().enumerate() {
                for param_alias in raw_aliases.scalar_aliases_for_value(&param.place) {
                    let Some(suffix) =
                        place_suffix_after_address_prefix(&entry_alias, &param_alias)
                    else {
                        continue;
                    };
                    push_unique_param_suffix(&mut out, param_index, suffix, entry_alias.ty);
                }
            }
        }
    }
    out
}

fn push_unique_param_byte_range(
    ranges: &mut Vec<RawCellInitializationParamByteRange>,
    range: RawCellInitializationParamByteRange,
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

fn push_unique_param_suffix(
    suffixes: &mut Vec<(usize, Vec<PlaceProjection>, TypeId)>,
    param_index: usize,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
) {
    if !suffixes
        .iter()
        .any(|(existing_param_index, existing_suffix, existing_ty)| {
            *existing_param_index == param_index && existing_suffix == &suffix && *existing_ty == ty
        })
    {
        suffixes.push((param_index, suffix, ty));
    }
}
