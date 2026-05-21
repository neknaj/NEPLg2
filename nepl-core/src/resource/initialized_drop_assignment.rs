extern crate alloc;

use crate::span::Span;
use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::drop_model::{ResourceAutoDrop, ResourceDropPoint};
use super::drop_plan_assignment::auto_drop_candidate_for_assignment_overwrite;
use super::drop_point_path::ResourceDropPointPath;
use super::drop_requirement::ResourceDropRequirement;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_drop_requirement::partial_drop_requirement_for_initialized_descendants;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{CellState, Place};
use super::raw_realloc::PendingRawReallocs;

fn auto_drop_assignment_overwrite_with_record(
    types: &TypeCtx,
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    pending_reallocs: &mut PendingRawReallocs,
    variant_initializations: &mut PendingVariantRawCellInitializations,
    target: &Place,
    span: Span,
) -> Option<ResourceAutoDrop> {
    let candidate = auto_drop_candidate_for_assignment_overwrite(types, target, span)?;
    let requirement = if matches!(
        cells.availability_state_with_types(types, target),
        CellState::Initialized(_)
    ) {
        candidate.requirement.clone()
    } else {
        partial_drop_requirement_for_initialized_descendants(types, cells, target)
    };
    if matches!(requirement, ResourceDropRequirement::StateOnly) {
        return None;
    }
    cells.record_raw_cell_loaded_value_drop(target, types);
    cells.set_state(target, CellState::Dropped);
    raw_aliases.clear(target);
    function_aliases.clear_alias(target);
    pending_reallocs.clear_result(target);
    variant_initializations.clear_result(target);
    Some(ResourceAutoDrop {
        requirement,
        ..candidate
    })
}

impl ResourceCheckEngine<'_> {
    pub(super) fn record_assignment_overwrite_drop(
        &mut self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        function_aliases: &mut FunctionAliasTable,
        pending_reallocs: &mut PendingRawReallocs,
        variant_initializations: &mut PendingVariantRawCellInitializations,
        target: &Place,
        path: ResourceDropPointPath,
        span: Span,
    ) {
        if let Some(auto_drop) = auto_drop_assignment_overwrite_with_record(
            self.types,
            cells,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
            target,
            span,
        ) {
            self.auto_drop_points.push(ResourceDropPoint {
                path,
                span,
                auto_drops: alloc::vec![auto_drop],
            });
        }
    }
}
