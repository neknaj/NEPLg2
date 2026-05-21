use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::cell_state::{raw_cell_suffix_after_address, CellTable};
use super::model::Place;
use super::place_utils::{place_with_suffix, raw_memory_cell_place};
use super::type_pattern::type_pattern_matches;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum RawCellValueFlowKind {
    StoreValue,
    MoveOutLoadedCell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellValueFlowEntry {
    cell: Place,
    ty: TypeId,
    kind: RawCellValueFlowKind,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct RawCellValueFlowFacts {
    entries: Vec<RawCellValueFlowEntry>,
}

impl CellTable {
    pub(super) fn record_raw_cell_value_flow(
        &mut self,
        address: &Place,
        ty: TypeId,
        kind: RawCellValueFlowKind,
    ) {
        self.raw_cell_value_flows.record(address, ty, kind);
    }

    pub(super) fn raw_cell_value_flow_available(
        &self,
        cell: &Place,
        ty: TypeId,
        kind: RawCellValueFlowKind,
        types: &TypeCtx,
    ) -> bool {
        self.raw_cell_value_flows
            .contains_matching(cell, ty, kind, types)
    }

    pub(super) fn consume_raw_cell_value_flow(
        &mut self,
        cell: &Place,
        ty: TypeId,
        kind: RawCellValueFlowKind,
        types: &TypeCtx,
    ) -> bool {
        self.raw_cell_value_flows
            .consume_matching(cell, ty, kind, types)
    }
}

impl RawCellValueFlowFacts {
    fn record(&mut self, address: &Place, ty: TypeId, kind: RawCellValueFlowKind) {
        let cell = raw_memory_cell_place(address, ty);
        match kind {
            RawCellValueFlowKind::StoreValue => {
                self.entries.retain(|entry| {
                    !(entry.cell == cell && entry.kind == RawCellValueFlowKind::StoreValue)
                });
            }
            RawCellValueFlowKind::MoveOutLoadedCell => {
                self.entries.retain(|entry| entry.cell != cell);
            }
        }
        self.entries.push(RawCellValueFlowEntry { cell, ty, kind });
    }

    fn contains_matching(
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

    fn consume_matching(
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
    }

    pub(super) fn rekey_under_address(&mut self, source: &Place, target: &Place) {
        for entry in &mut self.entries {
            if let Some(suffix) = raw_cell_suffix_after_address(&entry.cell, source) {
                entry.cell = place_with_suffix(target, &suffix, entry.cell.ty);
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
        Self { entries }
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

#[cfg(test)]
mod tests {
    use alloc::string::String;
    use alloc::vec;

    use crate::types::{TypeCtx, TypeKind};

    use super::super::model::PlaceRoot;
    use super::*;

    #[test]
    fn raw_value_flow_facts_merge_only_path_common_proofs() {
        let (types, owned_ty) = non_copy_type_context();
        let address = Place {
            root: PlaceRoot::Local(String::from("slot_address")),
            projections: vec![],
            ty: types.i32(),
        };
        let cell = raw_memory_cell_place(&address, owned_ty);
        let mut stored_path = CellTable::default();
        let empty_path = CellTable::default();
        stored_path.record_raw_cell_value_flow(
            &address,
            owned_ty,
            RawCellValueFlowKind::StoreValue,
        );

        let merged = CellTable::merge_paths(&[stored_path.clone(), empty_path]);

        assert!(!merged.raw_cell_value_flow_available(
            &cell,
            owned_ty,
            RawCellValueFlowKind::StoreValue,
            &types
        ));

        let merged = CellTable::merge_paths(&[stored_path.clone(), stored_path]);

        assert!(merged.raw_cell_value_flow_available(
            &cell,
            owned_ty,
            RawCellValueFlowKind::StoreValue,
            &types
        ));
    }

    #[test]
    fn raw_load_invalidates_stale_store_proof_for_same_cell() {
        let (types, owned_ty) = non_copy_type_context();
        let address = Place {
            root: PlaceRoot::Local(String::from("slot_address")),
            projections: vec![],
            ty: types.i32(),
        };
        let cell = raw_memory_cell_place(&address, owned_ty);
        let mut cells = CellTable::default();

        cells.record_raw_cell_value_flow(&address, owned_ty, RawCellValueFlowKind::StoreValue);
        cells.record_raw_cell_value_flow(
            &address,
            owned_ty,
            RawCellValueFlowKind::MoveOutLoadedCell,
        );

        assert!(!cells.raw_cell_value_flow_available(
            &cell,
            owned_ty,
            RawCellValueFlowKind::StoreValue,
            &types
        ));
        assert!(cells.raw_cell_value_flow_available(
            &cell,
            owned_ty,
            RawCellValueFlowKind::MoveOutLoadedCell,
            &types
        ));
    }

    fn non_copy_type_context() -> (TypeCtx, crate::types::TypeId) {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.i32());
        let owned_ty = types.register_named(
            String::from("Owned"),
            TypeKind::Struct {
                name: String::from("Owned"),
                type_params: vec![],
                fields: vec![],
                field_names: vec![],
            },
        );
        (types, owned_ty)
    }
}
