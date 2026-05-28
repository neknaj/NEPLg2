extern crate alloc;

use alloc::vec;
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

const MAX_PATH_SENSITIVE_ALTERNATIVES: usize = 4;

#[derive(Clone, PartialEq, Eq)]
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
    pub(super) fn from_states(mut states: Vec<ResourceCheckState>) -> Self {
        if states.len() > MAX_PATH_SENSITIVE_ALTERNATIVES {
            dedup_resource_check_states(&mut states);
        }
        // 空の候補集合は、既存の path-sensitive replay が全候補を棄却した
        // 結果として使われる。単に path-sensitive refinement が存在しない
        // 場合は、呼び出し元で `None` のままにして直線状態を保持する。
        if states.len() > MAX_PATH_SENSITIVE_ALTERNATIVES {
            // 分岐ごとの replay は診断位置を細かく保つための精密化であり、
            // 安全性そのものは merge lattice で保守的に表現できる。上限を
            // 超えた場合は全候補を一つの merged state に畳み、探索空間が
            // 後続 operation ごとに指数的に増え続けることを防ぐ。
            states = vec![merge_resource_check_states(&states)];
        }
        Self::Feasible(states)
    }

    pub(super) fn into_feasible_states(self) -> Option<Vec<ResourceCheckState>> {
        match self {
            ResourcePathAlternatives::None => None,
            ResourcePathAlternatives::Feasible(states) => Some(states),
        }
    }

    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    pub(super) fn len(&self) -> usize {
        match self {
            ResourcePathAlternatives::None => 0,
            ResourcePathAlternatives::Feasible(states) => states.len(),
        }
    }
}

impl ResourceCheckEngine<'_> {
    pub(super) fn advance_path_alternatives_after_op(
        &mut self,
        alternatives: Vec<ResourceCheckState>,
        op: &ResourceOp,
        path: ResourceDropPointPath,
    ) -> Vec<ResourceCheckState> {
        let mut out = Vec::new();
        for mut state in alternatives {
            let mut path_engine = summary_check_engine(self);
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
            match core::mem::take(&mut path_engine.path_alternatives).into_feasible_states() {
                Some(path_alternatives) => out.extend(path_alternatives),
                None => out.push(state),
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

fn merge_resource_check_states(alternatives: &[ResourceCheckState]) -> ResourceCheckState {
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
    ResourceCheckState {
        cells: CellTable::merge_paths_with_raw_aliases(
            &cell_paths,
            &alias_paths,
            &merged_raw_aliases,
        ),
        collection_slots: CollectionSlotStateTable::merge_paths(&collection_slot_paths),
        raw_aliases: merged_raw_aliases,
        function_aliases: FunctionAliasTable::merge_paths(&function_alias_paths),
        pending_reallocs: PendingRawReallocs::merge_paths(&pending_realloc_paths),
        variant_initializations: PendingVariantRawCellInitializations::merge_paths(
            &variant_initialization_paths,
        ),
    }
}

fn dedup_resource_check_states(states: &mut Vec<ResourceCheckState>) {
    let mut unique = Vec::new();
    for state in states.drain(..) {
        if !unique.iter().any(|existing| existing == &state) {
            unique.push(state);
        }
    }
    *states = unique;
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
    let merged = merge_resource_check_states(alternatives);
    *cells = merged.cells;
    *collection_slots = merged.collection_slots;
    *raw_aliases = merged.raw_aliases;
    *function_aliases = merged.function_aliases;
    *pending_reallocs = merged.pending_reallocs;
    *variant_initializations = merged.variant_initializations;
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::String;
    use alloc::vec;

    use crate::resource::model::Place;
    use crate::types::TypeId;

    use super::*;

    fn empty_resource_check_state() -> ResourceCheckState {
        ResourceCheckState::new(
            CellTable::default(),
            CollectionSlotStateTable::new(),
            RawCellAddressAliases::default(),
            FunctionAliasTable::default(),
            PendingRawReallocs::default(),
            PendingVariantRawCellInitializations::default(),
        )
    }

    fn resource_check_state_with_function_alias(index: usize) -> ResourceCheckState {
        let mut function_aliases = FunctionAliasTable::default();
        let name = format!("f{index}");
        function_aliases.set_alias(
            &Place::local(String::from("callback"), TypeId(0)),
            String::from(name),
        );
        ResourceCheckState::new(
            CellTable::default(),
            CollectionSlotStateTable::new(),
            RawCellAddressAliases::default(),
            function_aliases,
            PendingRawReallocs::default(),
            PendingVariantRawCellInitializations::default(),
        )
    }

    #[test]
    fn path_alternatives_merge_to_single_state_after_precision_budget() {
        let alternatives = (0..=MAX_PATH_SENSITIVE_ALTERNATIVES)
            .map(resource_check_state_with_function_alias)
            .collect::<Vec<_>>();

        let ResourcePathAlternatives::Feasible(states) =
            ResourcePathAlternatives::from_states(alternatives)
        else {
            panic!("from_states should keep feasible path alternatives");
        };

        assert_eq!(states.len(), 1);
    }

    #[test]
    fn path_alternatives_drop_duplicate_states_before_precision_budgeting() {
        let state = empty_resource_check_state();
        let alternatives = vec![state; MAX_PATH_SENSITIVE_ALTERNATIVES + 1];

        let ResourcePathAlternatives::Feasible(states) =
            ResourcePathAlternatives::from_states(alternatives)
        else {
            panic!("from_states should keep feasible path alternatives");
        };

        assert_eq!(states.len(), 1);
    }
}
