extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::cell_state::raw_cell_suffix_after_address;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, PlaceProjection, ResourceOffset};
use super::place_utils::{push_unique_place, raw_memory_cell_place};

pub(super) fn iov_buffer_pointer_cells(
    raw_aliases: &RawCellAddressAliases,
    iovs: &Place,
    raw_ty: TypeId,
) -> Vec<Place> {
    let mut cells = Vec::new();
    let iov_aliases = raw_aliases.aliases_for(iovs);
    for place in raw_aliases.tracked_places() {
        if !raw_aliases.value_is_known_raw_address(&place) {
            continue;
        }
        for iov_alias in &iov_aliases {
            let Some(suffix) = raw_cell_suffix_after_address(&place, iov_alias) else {
                continue;
            };
            if iov_buffer_cell_suffix_offset(&suffix).is_some() {
                push_unique_place(&mut cells, &place);
                break;
            }
        }
    }
    if cells.is_empty() {
        push_unique_place(&mut cells, &raw_memory_cell_place(iovs, raw_ty));
    }
    cells
}

pub(super) fn iov_length_cell(buffer_cell: &Place, raw_ty: TypeId) -> Option<Place> {
    let mut address = raw_cell_address(buffer_cell, raw_ty)?;
    add_static_offset(&mut address, 4);
    Some(raw_memory_cell_place(&address, raw_ty))
}

pub(super) fn raw_cell_is_under_any_address(cell: &Place, addresses: &[Place]) -> bool {
    addresses
        .iter()
        .any(|address| raw_cell_suffix_after_address(cell, address).is_some())
}

fn iov_buffer_cell_suffix_offset(suffix: &[PlaceProjection]) -> Option<usize> {
    match suffix {
        [PlaceProjection::Deref] => Some(0),
        [PlaceProjection::StorageOffset(ResourceOffset::Known(bytes)), PlaceProjection::Deref]
            if bytes % 8 == 0 =>
        {
            Some(*bytes)
        }
        _ => None,
    }
}

fn raw_cell_address(cell: &Place, raw_ty: TypeId) -> Option<Place> {
    let Some(PlaceProjection::Deref) = cell.projections.last() else {
        return None;
    };
    let mut address = cell.clone();
    address.projections.pop();
    address.ty = raw_ty;
    Some(address)
}

fn add_static_offset(place: &mut Place, bytes: usize) {
    if bytes == 0 {
        return;
    }
    match place.projections.last_mut() {
        Some(PlaceProjection::StorageOffset(ResourceOffset::Known(existing))) => {
            *existing = existing.saturating_add(bytes);
        }
        Some(PlaceProjection::StorageOffset(
            ResourceOffset::Symbolic { .. }
            | ResourceOffset::ScaledSymbolic { .. }
            | ResourceOffset::Unknown,
        )) => {}
        _ => place
            .projections
            .push(PlaceProjection::StorageOffset(ResourceOffset::Known(bytes))),
    }
}
