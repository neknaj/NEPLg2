extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::cell_state::{place_suffix_after_address_prefix, CellTable};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary_byte_range_model::RawCellInitializationParamByteRange;
use super::initialized_summary_param_byte_range_count::collect_param_count_sources;
use super::model::{Place, PlaceProjection, ResourceLocal};

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
        let count_sources = collect_param_count_sources(cells, raw_aliases, range.count(), params);
        for (address_param_index, address_suffix, address_ty) in &address_suffixes {
            for count in &count_sources {
                push_unique_param_byte_range(
                    out,
                    RawCellInitializationParamByteRange {
                        address_param_index: *address_param_index,
                        address_suffix: address_suffix.clone(),
                        address_ty: *address_ty,
                        count: count.clone(),
                        unit: range.unit(),
                        ty: range.ty(),
                    },
                );
            }
        }
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

fn push_unique_param_byte_range(
    ranges: &mut Vec<RawCellInitializationParamByteRange>,
    range: RawCellInitializationParamByteRange,
) {
    if !ranges.iter().any(|existing| existing == &range) {
        ranges.push(range);
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
