extern crate alloc;

use alloc::string::String;

use crate::span::Span;

use super::cell_state::CellTable;
use super::initialized::ResourceCheckEngine;
use super::model::{CellState, Place};
use super::place_utils::should_track;
use super::report::{ResourceCheckDiagnostic, ResourceCheckOperation};

impl ResourceCheckEngine<'_> {
    pub(super) fn ensure_no_live_non_copy_raw_cells(
        &mut self,
        cells: &CellTable,
        address: &Place,
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        let conflicts = cells.live_non_copy_raw_cells_under(address, self.types);
        for conflict in &conflicts {
            self.push_unavailable(operation, &conflict.place, conflict.state.clone(), span);
        }
        conflicts.is_empty()
    }

    pub(super) fn ensure_args(
        &mut self,
        cells: &mut CellTable,
        args: &[Place],
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        let mut available = true;
        for arg in args {
            available &= self.ensure_available(cells, arg, operation, span);
        }
        available
    }

    pub(super) fn consume_args(
        &mut self,
        cells: &mut CellTable,
        args: &[Place],
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        let mut available = true;
        for arg in args {
            available &= self.consume_by_value(cells, arg, operation, span);
        }
        available
    }

    pub(super) fn consume_by_value(
        &mut self,
        cells: &mut CellTable,
        place: &Place,
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        if !self.ensure_available(cells, place, operation, span) {
            return false;
        }
        if should_track(place) && !self.types.is_copy(place.ty) {
            cells.set_state(place, CellState::Moved);
        }
        true
    }

    pub(super) fn ensure_available(
        &mut self,
        cells: &CellTable,
        place: &Place,
        operation: ResourceCheckOperation,
        span: Span,
    ) -> bool {
        if !should_track(place) {
            return true;
        }
        match cells.availability_state_with_types(self.types, place) {
            CellState::Initialized(_) => true,
            state => {
                self.push_unavailable(operation, place, state, span);
                false
            }
        }
    }

    fn push_unavailable(
        &mut self,
        operation: ResourceCheckOperation,
        place: &Place,
        state: CellState,
        span: Span,
    ) {
        self.diagnostics
            .push(ResourceCheckDiagnostic::CellUnavailable {
                function: String::from(self.function),
                operation,
                place: place.clone(),
                state,
                span,
            });
    }
}
