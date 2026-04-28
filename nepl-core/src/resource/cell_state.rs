use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::model::{CellState, CellStateEntry, Place, PlaceProjection};
use super::place_utils::{
    place_suffix_after_prefix, push_unique_place, replace_place_prefix, should_track,
};

#[derive(Debug, Clone, Default)]
pub(super) struct CellTable {
    cells: Vec<CellStateEntry>,
}

impl CellTable {
    pub(super) fn into_entries(self) -> Vec<CellStateEntry> {
        self.cells
    }

    pub(super) fn availability_state(&self, place: &Place) -> CellState {
        if let Some(state) = self.state(place) {
            if !matches!(state, CellState::Initialized(_)) {
                return state;
            }
        }
        for entry in self.ancestor_entries(place) {
            if !matches!(entry.state, CellState::Initialized(_)) {
                return entry.state;
            }
        }
        for entry in self.descendant_entries(place) {
            if !matches!(entry.state, CellState::Initialized(_))
                && cell_descendant_state_flows(place, &entry.place)
            {
                return entry.state;
            }
        }
        if let Some(state @ CellState::Initialized(_)) = self.state(place) {
            return state;
        }
        if self.ancestor_entries(place).iter().any(|entry| {
            matches!(entry.state, CellState::Initialized(_))
                && cell_initialized_state_flows(&entry.place, place)
        }) {
            return CellState::Initialized(place.ty);
        }
        CellState::Uninit
    }

    pub(super) fn mark_initialized(&mut self, place: &Place) {
        self.set_state(place, CellState::Initialized(place.ty));
        self.clear_descendants(place);
    }

    pub(super) fn set_state(&mut self, place: &Place, state: CellState) {
        if !should_track(place) {
            return;
        }
        if let Some(entry) = self.cells.iter_mut().find(|entry| entry.place == *place) {
            entry.state = state;
        } else {
            self.cells.push(CellStateEntry {
                place: place.clone(),
                state,
            });
        }
    }

    pub(super) fn clear_raw_cells_under(&mut self, address: &Place) {
        self.cells
            .retain(|entry| raw_cell_suffix_after_address(&entry.place, address).is_none());
    }

    pub(super) fn extend_entries(&mut self, entries: Vec<CellStateEntry>) {
        for entry in entries {
            self.set_state(&entry.place, entry.state);
        }
    }

    pub(super) fn rekey_raw_cells(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        let mut relocated = Vec::new();
        self.cells.retain(|entry| {
            if raw_cell_suffix_after_address(&entry.place, source).is_none() {
                return true;
            }
            if let Some(place) = replace_place_prefix(&entry.place, source, target) {
                relocated.push(CellStateEntry {
                    place,
                    state: entry.state.clone(),
                });
            }
            false
        });
        self.extend_entries(relocated);
    }

    pub(super) fn live_non_copy_raw_cells_under(
        &self,
        address: &Place,
        types: &TypeCtx,
    ) -> Vec<CellStateEntry> {
        self.cells
            .iter()
            .filter(|entry| {
                raw_cell_suffix_after_address(&entry.place, address).is_some()
                    && raw_cell_state_has_live_non_copy_obligation(entry, types)
            })
            .cloned()
            .collect()
    }

    pub(super) fn copy_initialized_copy_raw_cells(
        &self,
        source: &Place,
        destination: &Place,
        types: &TypeCtx,
    ) -> Vec<CellStateEntry> {
        self.cells
            .iter()
            .filter_map(|entry| {
                raw_cell_suffix_after_address(&entry.place, source)?;
                let CellState::Initialized(ty) = entry.state else {
                    return None;
                };
                if !types.is_copy(ty) {
                    return None;
                }
                replace_place_prefix(&entry.place, source, destination).map(|place| {
                    CellStateEntry {
                        place,
                        state: entry.state.clone(),
                    }
                })
            })
            .collect()
    }

    pub(super) fn merge_paths(paths: &[CellTable]) -> Self {
        let mut out = CellTable::default();
        let mut places = Vec::new();
        for path in paths {
            for entry in &path.cells {
                push_unique_place(&mut places, &entry.place);
            }
        }
        for place in places {
            let mut states = paths.iter().map(|path| path.availability_state(&place));
            if let Some(mut merged) = states.next() {
                for state in states {
                    merged = merge_cell_states(merged, state);
                }
                out.set_state(&place, merged);
            }
        }
        out
    }

    fn state(&self, place: &Place) -> Option<CellState> {
        self.cells
            .iter()
            .find(|entry| entry.place == *place)
            .map(|entry| entry.state.clone())
    }

    fn ancestor_entries(&self, place: &Place) -> Vec<CellStateEntry> {
        self.cells
            .iter()
            .filter(|entry| {
                entry.place != *place && place_suffix_after_prefix(place, &entry.place).is_some()
            })
            .cloned()
            .collect()
    }

    fn descendant_entries(&self, prefix: &Place) -> Vec<CellStateEntry> {
        self.cells
            .iter()
            .filter(|entry| {
                entry.place != *prefix && place_suffix_after_prefix(&entry.place, prefix).is_some()
            })
            .cloned()
            .collect()
    }

    fn clear_descendants(&mut self, prefix: &Place) {
        self.cells.retain(|entry| {
            entry.place == *prefix || place_suffix_after_prefix(&entry.place, prefix).is_none()
        });
    }
}

fn cell_initialized_state_flows(prefix: &Place, place: &Place) -> bool {
    place_suffix_after_prefix(place, prefix)
        .map(|suffix| {
            suffix
                .iter()
                .all(|projection| !matches!(projection, PlaceProjection::Deref))
        })
        .unwrap_or(false)
}

fn cell_descendant_state_flows(prefix: &Place, place: &Place) -> bool {
    place_suffix_after_prefix(place, prefix)
        .map(|suffix| {
            suffix.iter().all(|projection| {
                !matches!(
                    projection,
                    PlaceProjection::Deref | PlaceProjection::StorageOffset(_)
                )
            })
        })
        .unwrap_or(false)
}

fn raw_cell_state_has_live_non_copy_obligation(entry: &CellStateEntry, types: &TypeCtx) -> bool {
    match entry.state {
        CellState::Initialized(ty) => !types.is_copy(ty),
        CellState::MaybeMoved => !types.is_copy(entry.place.ty),
        CellState::Uninit | CellState::Moved | CellState::Dropped => false,
    }
}

fn raw_cell_suffix_after_address(cell: &Place, address: &Place) -> Option<Vec<PlaceProjection>> {
    let suffix = place_suffix_after_address_prefix(cell, address)?;
    if suffix
        .iter()
        .any(|projection| matches!(projection, PlaceProjection::Deref))
    {
        Some(suffix)
    } else {
        None
    }
}

fn place_suffix_after_address_prefix(
    place: &Place,
    prefix: &Place,
) -> Option<Vec<PlaceProjection>> {
    if place.root != prefix.root {
        return None;
    }
    let mut place_index = 0;
    for prefix_projection in &prefix.projections {
        if matches!(
            prefix_projection,
            PlaceProjection::StorageOffset(super::model::ResourceOffset { bytes: None })
        ) {
            if matches!(
                place.projections.get(place_index),
                Some(PlaceProjection::StorageOffset(_))
            ) {
                place_index += 1;
            }
            continue;
        }
        let place_projection = place.projections.get(place_index)?;
        if !address_projection_matches(place_projection, prefix_projection) {
            return None;
        }
        place_index += 1;
    }
    Some(place.projections[place_index..].to_vec())
}

fn address_projection_matches(place: &PlaceProjection, prefix: &PlaceProjection) -> bool {
    match (place, prefix) {
        (PlaceProjection::StorageOffset(left), PlaceProjection::StorageOffset(right)) => {
            left.bytes == right.bytes || left.bytes.is_none() || right.bytes.is_none()
        }
        _ => place == prefix,
    }
}

fn merge_cell_states(left: CellState, right: CellState) -> CellState {
    if left == right {
        return left;
    }
    match (left, right) {
        (CellState::Initialized(left_ty), CellState::Initialized(right_ty))
            if left_ty == right_ty =>
        {
            CellState::Initialized(left_ty)
        }
        (CellState::Uninit, CellState::Uninit) => CellState::Uninit,
        (CellState::Moved, CellState::Moved) => CellState::Moved,
        (CellState::Dropped, CellState::Dropped) => CellState::Dropped,
        _ => CellState::MaybeMoved,
    }
}
