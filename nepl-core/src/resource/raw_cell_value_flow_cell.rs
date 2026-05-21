use crate::types::{TypeCtx, TypeId};

use super::cell_state::CellTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::raw_cell_value_flow::RawCellValueFlowKind;

impl CellTable {
    #[cfg(test)]
    pub(super) fn record_raw_cell_value_flow(
        &mut self,
        address: &Place,
        ty: TypeId,
        kind: RawCellValueFlowKind,
    ) {
        self.raw_cell_value_flows.record(address, ty, kind);
    }

    pub(super) fn record_raw_cell_value_flow_with_aliases(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        address: &Place,
        ty: TypeId,
        kind: RawCellValueFlowKind,
    ) {
        self.raw_cell_value_flows
            .record_with_aliases(raw_aliases, address, ty, kind);
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

    pub(super) fn raw_cell_value_flow_available_with_aliases(
        &self,
        raw_aliases: &RawCellAddressAliases,
        cell: &Place,
        ty: TypeId,
        kind: RawCellValueFlowKind,
        types: &TypeCtx,
    ) -> bool {
        self.raw_cell_value_flows
            .contains_matching_with_aliases(raw_aliases, cell, ty, kind, types)
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

    pub(super) fn consume_raw_cell_value_flow_with_aliases(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        cell: &Place,
        ty: TypeId,
        kind: RawCellValueFlowKind,
        types: &TypeCtx,
    ) -> bool {
        self.raw_cell_value_flows
            .consume_matching_with_aliases(raw_aliases, cell, ty, kind, types)
    }

    #[cfg(test)]
    pub(super) fn record_raw_cell_loaded_value_origin(
        &mut self,
        address: &Place,
        ty: TypeId,
        value: &Place,
    ) {
        self.raw_cell_value_flows
            .record_loaded_value_origin(address, ty, value);
    }

    pub(super) fn record_raw_cell_loaded_value_origin_with_aliases(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        address: &Place,
        ty: TypeId,
        value: &Place,
    ) {
        self.raw_cell_value_flows
            .record_loaded_value_origin_with_aliases(raw_aliases, address, ty, value);
    }

    pub(super) fn transfer_raw_cell_loaded_value_origin(&mut self, source: &Place, target: &Place) {
        self.raw_cell_value_flows
            .transfer_loaded_value_origin(source, target);
    }

    pub(super) fn discard_raw_cell_loaded_value_origin(&mut self, place: &Place) {
        self.raw_cell_value_flows.discard_loaded_value_origin(place);
    }

    pub(super) fn record_raw_cell_loaded_value_drop(
        &mut self,
        dropped: &Place,
        types: &TypeCtx,
    ) -> bool {
        self.raw_cell_value_flows
            .record_loaded_value_drop(dropped, types)
    }
}
