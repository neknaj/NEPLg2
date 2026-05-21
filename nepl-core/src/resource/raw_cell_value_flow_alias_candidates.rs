extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::{raw_cell_address_prefix, raw_cell_suffix_after_address};
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::place_utils::place_with_suffix;
use super::raw_cell_value_flow_alias::{
    place_without_zero_storage_offsets, raw_cell_places_equivalent,
};

pub(super) fn raw_cell_alias_candidates(
    cell: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Vec<Place> {
    let mut out = Vec::new();
    push_unique_equivalent_place(&mut out, cell);
    let Some(address) = raw_cell_address_prefix(cell) else {
        return out;
    };
    let Some(suffix) = raw_cell_suffix_after_address(cell, &address) else {
        return out;
    };
    for address in raw_address_alias_candidates(&address, raw_aliases) {
        let candidate = place_with_suffix(&address, &suffix, cell.ty);
        push_unique_equivalent_place(&mut out, &candidate);
    }
    out
}

pub(super) fn raw_address_alias_candidates(
    address: &Place,
    raw_aliases: &RawCellAddressAliases,
) -> Vec<Place> {
    let mut out = Vec::new();
    let normalized = place_without_zero_storage_offsets(address);
    for seed in [address.clone(), normalized] {
        push_unique_equivalent_place(&mut out, &seed);
        for alias in raw_aliases.raw_address_aliases_for_value(&seed) {
            push_unique_equivalent_place(&mut out, &alias);
            let normalized_alias = place_without_zero_storage_offsets(&alias);
            push_unique_equivalent_place(&mut out, &normalized_alias);
        }
    }
    out
}

fn push_unique_equivalent_place(out: &mut Vec<Place>, place: &Place) {
    if out
        .iter()
        .all(|existing| !raw_cell_places_equivalent(existing, place))
    {
        out.push(place.clone());
    }
}
