extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::{
    place_suffix_after_address_prefix, raw_cell_suffix_after_address, CellTable,
};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::{
    RawCellInitializationParamCell, RawCellInitializationReturnByteRange,
    RawCellInitializationReturnCell,
};
use super::model::{CellState, Place, ResourceLocal};

pub(super) fn collect_return_initialized_raw_cells(
    out: &mut Vec<RawCellInitializationReturnCell>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
) {
    let return_aliases = raw_aliases.aliases_for(value);
    for entry in cells.entries() {
        if !matches!(entry.state, CellState::Initialized(_)) {
            continue;
        }
        let holds_raw_address = raw_aliases.value_is_known_raw_address(&entry.place);
        if let Some(suffix) = raw_cell_suffix_after_address(&entry.place, value) {
            push_unique_return_cell(
                out,
                RawCellInitializationReturnCell {
                    suffix,
                    ty: entry.place.ty,
                    holds_raw_address,
                },
            );
        }
        for cell_alias in raw_aliases.aliases_for(&entry.place) {
            for return_alias in &return_aliases {
                let Some(suffix) = raw_cell_suffix_after_address(&cell_alias, return_alias) else {
                    continue;
                };
                push_unique_return_cell(
                    out,
                    RawCellInitializationReturnCell {
                        suffix,
                        ty: entry.place.ty,
                        holds_raw_address,
                    },
                );
            }
        }
    }
}

pub(super) fn collect_param_initialized_raw_cells(
    out: &mut Vec<RawCellInitializationParamCell>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
) {
    for (param_index, param) in params.iter().enumerate() {
        let param_aliases = raw_aliases.aliases_for(&param.place);
        for entry in cells.entries() {
            if !matches!(entry.state, CellState::Initialized(_)) {
                continue;
            }
            let holds_raw_address = raw_aliases.value_is_known_raw_address(&entry.place);
            for cell_alias in raw_aliases.aliases_for(&entry.place) {
                for param_alias in &param_aliases {
                    let Some(suffix) = raw_cell_suffix_after_address(&cell_alias, param_alias)
                    else {
                        continue;
                    };
                    push_unique_param_cell(
                        out,
                        RawCellInitializationParamCell {
                            param_index,
                            suffix,
                            ty: entry.place.ty,
                            holds_raw_address,
                        },
                    );
                }
            }
        }
    }
}

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
) -> Vec<(Vec<super::model::PlaceProjection>, crate::types::TypeId)> {
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
) -> Vec<(Vec<super::model::PlaceProjection>, crate::types::TypeId)> {
    let count = raw_aliases.canonicalize(count);
    let mut out = Vec::new();
    for entry in cells.entries() {
        if !matches!(entry.state, CellState::Initialized(_)) {
            continue;
        }
        if raw_aliases.canonicalize(&entry.place) != count {
            continue;
        }
        for return_alias in return_aliases {
            let Some(suffix) = place_suffix_after_address_prefix(&entry.place, return_alias) else {
                continue;
            };
            push_unique_return_suffix(&mut out, suffix, entry.place.ty);
        }
    }
    out
}

fn push_unique_return_cell(
    cells: &mut Vec<RawCellInitializationReturnCell>,
    cell: RawCellInitializationReturnCell,
) {
    if !cells.iter().any(|existing| existing == &cell) {
        cells.push(cell);
    }
}

fn push_unique_return_byte_range(
    ranges: &mut Vec<RawCellInitializationReturnByteRange>,
    range: RawCellInitializationReturnByteRange,
) {
    if !ranges.iter().any(|existing| existing == &range) {
        ranges.push(range);
    }
}

fn push_unique_param_cell(
    cells: &mut Vec<RawCellInitializationParamCell>,
    cell: RawCellInitializationParamCell,
) {
    if !cells.iter().any(|existing| existing == &cell) {
        cells.push(cell);
    }
}

fn push_unique_return_suffix(
    suffixes: &mut Vec<(Vec<super::model::PlaceProjection>, crate::types::TypeId)>,
    suffix: Vec<super::model::PlaceProjection>,
    ty: crate::types::TypeId,
) {
    if !suffixes
        .iter()
        .any(|(existing_suffix, existing_ty)| existing_suffix == &suffix && *existing_ty == ty)
    {
        suffixes.push((suffix, ty));
    }
}
