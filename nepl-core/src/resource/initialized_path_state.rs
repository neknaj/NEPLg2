extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary_engine::summary_check_engine;
use super::model::ResourceOp;
use super::raw_realloc::PendingRawReallocs;
use super::{
    drop_point_path::ResourceDropPointPath,
    initialized_variant::PendingVariantRawCellInitializations,
};

#[derive(Clone)]
pub(super) struct ResourceCheckState {
    pub(super) cells: CellTable,
    pub(super) collection_slots: CollectionSlotStateTable,
    pub(super) raw_aliases: RawCellAddressAliases,
    pub(super) function_aliases: FunctionAliasTable,
    pub(super) pending_reallocs: PendingRawReallocs,
    pub(super) variant_initializations: PendingVariantRawCellInitializations,
}

impl ResourceCheckState {
    pub(super) fn new(
        cells: CellTable,
        collection_slots: CollectionSlotStateTable,
        raw_aliases: RawCellAddressAliases,
        function_aliases: FunctionAliasTable,
        pending_reallocs: PendingRawReallocs,
        variant_initializations: PendingVariantRawCellInitializations,
    ) -> Self {
        Self {
            cells,
            collection_slots,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
        }
    }
}

#[derive(Clone, Default)]
pub(super) enum ResourcePathAlternatives {
    #[default]
    None,
    Feasible(Vec<ResourceCheckState>),
}

impl ResourcePathAlternatives {
    pub(super) fn from_states(states: Vec<ResourceCheckState>) -> Self {
        if states.is_empty() {
            Self::None
        } else {
            Self::Feasible(states)
        }
    }

    pub(super) fn into_states(self) -> Vec<ResourceCheckState> {
        match self {
            ResourcePathAlternatives::None => Vec::new(),
            ResourcePathAlternatives::Feasible(states) => states,
        }
    }
}

impl ResourceCheckEngine<'_> {
    pub(super) fn advance_path_alternatives_after_op(
        &mut self,
        alternatives: &[ResourceCheckState],
        op: &ResourceOp,
        path: ResourceDropPointPath,
    ) -> Vec<ResourceCheckState> {
        let mut out = Vec::new();
        for alternative in alternatives {
            let mut path_engine = summary_check_engine(self);
            let mut state = alternative.clone();
            path_engine.check_op(
                &mut state.cells,
                &mut state.collection_slots,
                &mut state.raw_aliases,
                &mut state.function_aliases,
                &mut state.pending_reallocs,
                &mut state.variant_initializations,
                op,
                path.clone(),
            );
            let path_alternatives =
                core::mem::take(&mut path_engine.path_alternatives).into_states();
            if path_alternatives.is_empty() {
                out.push(state);
            } else {
                out.extend(path_alternatives);
            }
            self.absorb_path_engine_output(path_engine);
        }
        out
    }

    fn absorb_path_engine_output(&mut self, path_engine: ResourceCheckEngine<'_>) {
        for diagnostic in path_engine.diagnostics {
            if !self.diagnostics.contains(&diagnostic) {
                self.diagnostics.push(diagnostic);
            }
        }
        for drop_point in path_engine.auto_drop_points {
            if !self.auto_drop_points.contains(&drop_point) {
                self.auto_drop_points.push(drop_point);
            }
        }
        self.deferred.branch_merges += path_engine.deferred.branch_merges;
        self.deferred.loop_merges += path_engine.deferred.loop_merges;
        self.deferred.match_merges += path_engine.deferred.match_merges;
    }
}

pub(super) fn merge_path_alternatives_into(
    alternatives: &[ResourceCheckState],
    cells: &mut CellTable,
    collection_slots: &mut CollectionSlotStateTable,
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    pending_reallocs: &mut PendingRawReallocs,
    variant_initializations: &mut PendingVariantRawCellInitializations,
) {
    if alternatives.is_empty() {
        return;
    }
    let cell_paths = alternatives
        .iter()
        .map(|state| state.cells.clone())
        .collect::<Vec<_>>();
    let collection_slot_paths = alternatives
        .iter()
        .map(|state| state.collection_slots.clone())
        .collect::<Vec<_>>();
    let alias_paths = alternatives
        .iter()
        .map(|state| state.raw_aliases.clone())
        .collect::<Vec<_>>();
    let function_alias_paths = alternatives
        .iter()
        .map(|state| state.function_aliases.clone())
        .collect::<Vec<_>>();
    let pending_realloc_paths = alternatives
        .iter()
        .map(|state| state.pending_reallocs.clone())
        .collect::<Vec<_>>();
    let variant_initialization_paths = alternatives
        .iter()
        .map(|state| state.variant_initializations.clone())
        .collect::<Vec<_>>();
    let merged_raw_aliases = RawCellAddressAliases::merge_paths(&alias_paths);
    *cells =
        CellTable::merge_paths_with_raw_aliases(&cell_paths, &alias_paths, &merged_raw_aliases);
    *collection_slots = CollectionSlotStateTable::merge_paths(&collection_slot_paths);
    *raw_aliases = merged_raw_aliases;
    *function_aliases = FunctionAliasTable::merge_paths(&function_alias_paths);
    *pending_reallocs = PendingRawReallocs::merge_paths(&pending_realloc_paths);
    *variant_initializations =
        PendingVariantRawCellInitializations::merge_paths(&variant_initialization_paths);
}
