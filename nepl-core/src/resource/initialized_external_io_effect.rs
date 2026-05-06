extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeId;

use super::cell_state::{raw_cell_suffix_after_address, CellTable};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::CellState;
use super::model::{Place, PlaceProjection, ResourceOffset};
use super::place_utils::{
    push_unique_place, raw_memory_cell_place, raw_memory_unknown_offset_cell_place,
};
use super::report::ResourceCheckOperation;

impl ResourceCheckEngine<'_> {
    pub(super) fn ensure_iov_descriptor_cells_available(
        &mut self,
        cells: &CellTable,
        raw_aliases: &RawCellAddressAliases,
        iovs: Option<&Place>,
        span: Span,
    ) -> bool {
        let Some(iovs) = iovs else {
            return true;
        };
        let iovs = raw_aliases.canonicalize(iovs);
        let mut available = true;
        for buffer_cell in iov_buffer_pointer_cells(raw_aliases, &iovs, self.types.i32()) {
            available &= self.ensure_available(
                cells,
                &buffer_cell,
                ResourceCheckOperation::RawMemoryLoadCell,
                span,
            );
            if let Some(length_cell) = iov_length_cell(&buffer_cell, self.types.i32()) {
                available &= self.ensure_available(
                    cells,
                    &length_cell,
                    ResourceCheckOperation::RawMemoryLoadCell,
                    span,
                );
            }
        }
        available
    }

    pub(super) fn ensure_iov_write_buffers_available(
        &mut self,
        cells: &CellTable,
        raw_aliases: &RawCellAddressAliases,
        iovs: Option<&Place>,
        span: Span,
    ) -> bool {
        let Some(iovs) = iovs else {
            return true;
        };
        let iovs = raw_aliases.canonicalize(iovs);
        let iov_aliases = raw_aliases.aliases_for(&iovs);
        let mut available =
            self.ensure_iov_descriptor_cells_available(cells, raw_aliases, Some(&iovs), span);
        for buffer_cell in iov_buffer_pointer_cells(raw_aliases, &iovs, self.types.i32()) {
            for buffer in raw_aliases.aliases_for(&buffer_cell) {
                if buffer == buffer_cell || raw_cell_is_under_any_address(&buffer, &iov_aliases) {
                    continue;
                }
                available &=
                    self.ensure_iov_payload_buffer_available(cells, raw_aliases, &buffer, span);
            }
        }
        available
    }

    fn ensure_iov_payload_buffer_available(
        &mut self,
        cells: &CellTable,
        raw_aliases: &RawCellAddressAliases,
        buffer: &Place,
        span: Span,
    ) -> bool {
        let candidates = raw_aliases.aliases_for(buffer);
        if candidates
            .iter()
            .any(|candidate| cells.raw_cell_is_untracked_external(candidate))
        {
            return true;
        }
        if candidates.iter().any(|candidate| {
            let cell = raw_memory_unknown_offset_cell_place(candidate, self.types.i32());
            matches!(
                cells.availability_state_with_types(self.types, &cell),
                CellState::Initialized(_)
            )
        }) {
            return true;
        }
        let cell = raw_memory_unknown_offset_cell_place(buffer, self.types.i32());
        self.ensure_available(
            cells,
            &cell,
            ResourceCheckOperation::RawMemoryLoadCell,
            span,
        )
    }

    pub(super) fn apply_fd_read_initialized_effect(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        args: &[Place],
    ) {
        if let Some(nread) = args.get(3) {
            self.mark_raw_cell_initialized(cells, raw_aliases, nread, self.types.i32());
        }

        self.apply_iov_read_buffers_initialized(cells, raw_aliases, args.get(1));
    }

    pub(super) fn apply_iov_read_buffers_initialized(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        iovs: Option<&Place>,
    ) {
        let Some(iovs) = iovs else {
            return;
        };
        let iovs = raw_aliases.canonicalize(iovs);
        let iov_buffer_cell = raw_memory_cell_place(&iovs, self.types.i32());
        for buffer in raw_aliases.aliases_for(&iov_buffer_cell) {
            if buffer == iov_buffer_cell {
                continue;
            }
            self.mark_unknown_offset_raw_cell_initialized(
                cells,
                raw_aliases,
                &buffer,
                self.types.i32(),
            );
        }
    }

    pub(super) fn mark_raw_cell_initialized(
        &self,
        cells: &mut CellTable,
        raw_aliases: &RawCellAddressAliases,
        address: &Place,
        ty: TypeId,
    ) {
        let address = raw_aliases.canonicalize(address);
        for alias in raw_aliases.aliases_for(&address) {
            let cell = raw_memory_cell_place(&alias, ty);
            cells.mark_initialized(&cell);
        }
    }

    pub(super) fn mark_unknown_offset_raw_cell_initialized_for_arg(
        &self,
        cells: &mut CellTable,
        raw_aliases: &RawCellAddressAliases,
        address: Option<&Place>,
        ty: TypeId,
    ) {
        if let Some(address) = address {
            self.mark_unknown_offset_raw_cell_initialized(cells, raw_aliases, address, ty);
        }
    }

    fn mark_unknown_offset_raw_cell_initialized(
        &self,
        cells: &mut CellTable,
        raw_aliases: &RawCellAddressAliases,
        address: &Place,
        ty: TypeId,
    ) {
        let address = raw_aliases.canonicalize(address);
        for alias in raw_aliases.aliases_for(&address) {
            let cell = raw_memory_unknown_offset_cell_place(&alias, ty);
            cells.mark_initialized(&cell);
        }
    }
}

fn iov_buffer_pointer_cells(
    raw_aliases: &RawCellAddressAliases,
    iovs: &Place,
    raw_ty: TypeId,
) -> Vec<Place> {
    let mut cells = Vec::new();
    push_unique_place(&mut cells, &raw_memory_cell_place(iovs, raw_ty));
    for place in raw_aliases.tracked_places() {
        if !raw_aliases.value_is_known_raw_address(&place) {
            continue;
        }
        let Some(suffix) = raw_cell_suffix_after_address(&place, iovs) else {
            continue;
        };
        if iov_buffer_cell_suffix_offset(&suffix).is_some() {
            push_unique_place(&mut cells, &place);
        }
    }
    cells
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

fn iov_length_cell(buffer_cell: &Place, raw_ty: TypeId) -> Option<Place> {
    let mut address = raw_cell_address(buffer_cell, raw_ty)?;
    add_static_offset(&mut address, 4);
    Some(raw_memory_cell_place(&address, raw_ty))
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
            ResourceOffset::Symbolic { .. } | ResourceOffset::Unknown,
        )) => {}
        _ => place
            .projections
            .push(PlaceProjection::StorageOffset(ResourceOffset::Known(bytes))),
    }
}

fn raw_cell_is_under_any_address(cell: &Place, addresses: &[Place]) -> bool {
    addresses
        .iter()
        .any(|address| raw_cell_suffix_after_address(cell, address).is_some())
}
