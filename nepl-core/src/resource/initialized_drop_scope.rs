use crate::span::Span;
use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::drop_plan::auto_drop_candidates_for_end_scope;
use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{CellState, Place};
use super::raw_realloc::PendingRawReallocs;

pub(super) fn auto_drop_scope_locals(
    types: &TypeCtx,
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    pending_reallocs: &mut PendingRawReallocs,
    variant_initializations: &mut PendingVariantRawCellInitializations,
    locals: &[Place],
    span: Span,
) {
    for candidate in auto_drop_candidates_for_end_scope(types, locals, span) {
        let local = &candidate.place;
        if !matches!(
            cells.availability_state_with_types(types, local),
            CellState::Initialized(_)
        ) {
            continue;
        }
        cells.set_state(local, CellState::Dropped);
        raw_aliases.clear(local);
        function_aliases.clear_alias(local);
        pending_reallocs.clear_result(local);
        variant_initializations.clear_result(local);
    }
}
