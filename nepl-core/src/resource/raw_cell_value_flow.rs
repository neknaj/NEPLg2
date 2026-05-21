use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::cell_state::{raw_cell_suffix_after_address, CellTable};
use super::model::Place;
use super::place_utils::{place_suffix_after_prefix, place_with_suffix, raw_memory_cell_place};
use super::type_pattern::type_pattern_matches;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawCellValueFlowKind {
    StoreValue,
    MoveOutLoadedCell,
    DropLoadedCell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellValueFlowEntry {
    cell: Place,
    ty: TypeId,
    kind: RawCellValueFlowKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawCellLoadedValueOrigin {
    value: Place,
    cell: Place,
    ty: TypeId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct RawCellValueFlowFacts {
    entries: Vec<RawCellValueFlowEntry>,
    loaded_values: Vec<RawCellLoadedValueOrigin>,
}

impl RawCellValueFlowFacts {
    pub(super) fn record(&mut self, address: &Place, ty: TypeId, kind: RawCellValueFlowKind) {
        let cell = raw_memory_cell_place(address, ty);
        self.record_cell_flow(&cell, ty, kind);
    }

    pub(super) fn record_loaded_value_origin(
        &mut self,
        address: &Place,
        ty: TypeId,
        value: &Place,
    ) {
        let cell = raw_memory_cell_place(address, ty);
        self.loaded_values.retain(|origin| origin.value != *value);
        self.loaded_values.push(RawCellLoadedValueOrigin {
            value: value.clone(),
            cell,
            ty,
        });
    }

    pub(super) fn transfer_loaded_value_origin(&mut self, source: &Place, target: &Place) {
        if source == target {
            return;
        }
        for origin in &mut self.loaded_values {
            let Some(suffix) = place_suffix_after_prefix(&origin.value, source) else {
                continue;
            };
            origin.value = place_with_suffix(target, &suffix, origin.value.ty);
        }
    }

    pub(super) fn discard_loaded_value_origin(&mut self, place: &Place) {
        self.loaded_values
            .retain(|origin| place_suffix_after_prefix(&origin.value, place).is_none());
    }

    pub(super) fn record_loaded_value_drop(&mut self, dropped: &Place, types: &TypeCtx) -> bool {
        let mut dropped_origins = Vec::new();
        self.loaded_values.retain(|origin| {
            if loaded_value_origin_dropped_by(&origin.value, dropped, types) {
                dropped_origins.push((origin.cell.clone(), origin.ty));
                false
            } else {
                true
            }
        });
        for (cell, ty) in &dropped_origins {
            self.record_cell_flow(cell, *ty, RawCellValueFlowKind::DropLoadedCell);
        }
        !dropped_origins.is_empty()
    }

    fn record_cell_flow(&mut self, cell: &Place, ty: TypeId, kind: RawCellValueFlowKind) {
        match kind {
            RawCellValueFlowKind::StoreValue => {
                self.entries.retain(|entry| {
                    !(entry.cell == *cell && entry.kind == RawCellValueFlowKind::StoreValue)
                });
            }
            RawCellValueFlowKind::MoveOutLoadedCell => {
                self.entries.retain(|entry| entry.cell != *cell);
            }
            RawCellValueFlowKind::DropLoadedCell => {
                self.entries.retain(|entry| {
                    !(entry.cell == *cell
                        && matches!(
                            entry.kind,
                            RawCellValueFlowKind::MoveOutLoadedCell
                                | RawCellValueFlowKind::DropLoadedCell
                        ))
                });
            }
        }
        self.entries.push(RawCellValueFlowEntry {
            cell: cell.clone(),
            ty,
            kind,
        });
    }

    pub(super) fn contains_matching(
        &self,
        cell: &Place,
        ty: TypeId,
        kind: RawCellValueFlowKind,
        types: &TypeCtx,
    ) -> bool {
        self.entries
            .iter()
            .any(|entry| value_flow_entry_matches(entry, cell, ty, kind, types))
    }

    pub(super) fn consume_matching(
        &mut self,
        cell: &Place,
        ty: TypeId,
        kind: RawCellValueFlowKind,
        types: &TypeCtx,
    ) -> bool {
        let Some(index) = self
            .entries
            .iter()
            .position(|entry| value_flow_entry_matches(entry, cell, ty, kind, types))
        else {
            return false;
        };
        self.entries.remove(index);
        true
    }

    pub(super) fn clear_under_address(&mut self, address: &Place) {
        self.entries
            .retain(|entry| raw_cell_suffix_after_address(&entry.cell, address).is_none());
        self.loaded_values
            .retain(|origin| raw_cell_suffix_after_address(&origin.cell, address).is_none());
    }

    pub(super) fn rekey_under_address(&mut self, source: &Place, target: &Place) {
        for entry in &mut self.entries {
            if let Some(suffix) = raw_cell_suffix_after_address(&entry.cell, source) {
                entry.cell = place_with_suffix(target, &suffix, entry.cell.ty);
            }
        }
        for origin in &mut self.loaded_values {
            if let Some(suffix) = raw_cell_suffix_after_address(&origin.cell, source) {
                origin.cell = place_with_suffix(target, &suffix, origin.cell.ty);
            }
        }
    }

    pub(super) fn merge_paths(paths: &[CellTable]) -> Self {
        let Some((first, rest)) = paths.split_first() else {
            return Self::default();
        };
        let entries = first
            .raw_cell_value_flows
            .entries
            .iter()
            .filter(|entry| {
                rest.iter()
                    .all(|path| path.raw_cell_value_flows.entries.contains(entry))
            })
            .cloned()
            .collect();
        let loaded_values = first
            .raw_cell_value_flows
            .loaded_values
            .iter()
            .filter(|origin| {
                rest.iter()
                    .all(|path| path.raw_cell_value_flows.loaded_values.contains(origin))
            })
            .cloned()
            .collect();
        Self {
            entries,
            loaded_values,
        }
    }
}

fn value_flow_entry_matches(
    entry: &RawCellValueFlowEntry,
    cell: &Place,
    ty: TypeId,
    kind: RawCellValueFlowKind,
    types: &TypeCtx,
) -> bool {
    entry.cell == *cell
        && entry.kind == kind
        && (type_pattern_matches(types, entry.ty, ty) || type_pattern_matches(types, ty, entry.ty))
}

fn loaded_value_origin_dropped_by(origin_value: &Place, dropped: &Place, types: &TypeCtx) -> bool {
    origin_value == dropped
        || (place_suffix_after_prefix(origin_value, dropped).is_some()
            && type_pattern_matches(types, dropped.ty, origin_value.ty))
}
