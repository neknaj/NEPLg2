extern crate alloc;

use alloc::string::String;

use crate::span::Span;

use super::initialized::ResourceCheckEngine;
use super::model::{CellState, Place};
use super::report::{ResourceCheckDiagnostic, ResourceCheckOperation};

impl ResourceCheckEngine<'_> {
    pub(super) fn push_unavailable(
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
