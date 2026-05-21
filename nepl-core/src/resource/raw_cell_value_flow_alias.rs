extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::cell_state::{raw_cell_address_prefix, raw_cell_suffix_after_address};
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection, ResourceOffset};
use super::place_utils::place_with_suffix;
use super::raw_cell_value_flow::{RawCellValueFlowEntry, RawCellValueFlowKind};
use super::type_pattern::type_pattern_matches;

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

pub(super) fn raw_cell_places_equivalent(left: &Place, right: &Place) -> bool {
    let left = place_without_zero_storage_offsets(left);
    let right = place_without_zero_storage_offsets(right);
    left.root == right.root && left.projections == right.projections
}

pub(super) fn value_flow_entry_matches(
    entry: &RawCellValueFlowEntry,
    cell: &Place,
    ty: TypeId,
    kind: RawCellValueFlowKind,
    types: &TypeCtx,
) -> bool {
    raw_cell_places_equivalent(&entry.cell, cell)
        && entry.kind == kind
        && (type_pattern_matches(types, entry.ty, ty) || type_pattern_matches(types, ty, entry.ty))
}

pub(super) fn value_flow_entry_matches_any_cell(
    entry: &RawCellValueFlowEntry,
    cells: &[Place],
    ty: TypeId,
    kind: RawCellValueFlowKind,
    types: &TypeCtx,
) -> bool {
    entry.kind == kind
        && (type_pattern_matches(types, entry.ty, ty) || type_pattern_matches(types, ty, entry.ty))
        && cells
            .iter()
            .any(|cell| raw_cell_places_equivalent(&entry.cell, cell))
}

fn raw_address_alias_candidates(
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

fn place_without_zero_storage_offsets(place: &Place) -> Place {
    let mut out = place.clone();
    out.projections.retain(|projection| {
        !matches!(
            projection,
            PlaceProjection::StorageOffset(ResourceOffset::Known(0))
        )
    });
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
