extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_path_state::ResourceCheckState;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::raw_realloc::PendingRawReallocs;

pub(super) fn merge_resource_check_states(
    alternatives: &[ResourceCheckState],
) -> ResourceCheckState {
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

pub(super) fn dedup_resource_check_states(states: &mut Vec<ResourceCheckState>) {
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
