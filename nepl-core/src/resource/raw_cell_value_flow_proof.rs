use crate::types::{TypeCtx, TypeId};

use super::cell_state::CellTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::raw_cell_value_flow::RawCellValueFlowKind;

pub(super) fn raw_cell_value_flow_available(
    cells: &CellTable,
    raw_aliases: Option<&RawCellAddressAliases>,
    target: &Place,
    ty: TypeId,
    kind: RawCellValueFlowKind,
    types: &TypeCtx,
) -> bool {
    if let Some(raw_aliases) = raw_aliases {
        cells.raw_cell_value_flow_available_with_aliases(raw_aliases, target, ty, kind, types)
    } else {
        cells.raw_cell_value_flow_available(target, ty, kind, types)
    }
}

pub(super) fn consume_raw_cell_value_flow(
    cells: &mut CellTable,
    raw_aliases: Option<&RawCellAddressAliases>,
    target: &Place,
    ty: TypeId,
    kind: RawCellValueFlowKind,
    types: &TypeCtx,
) -> bool {
    if let Some(raw_aliases) = raw_aliases {
        cells.consume_raw_cell_value_flow_with_aliases(raw_aliases, target, ty, kind, types)
    } else {
        cells.consume_raw_cell_value_flow(target, ty, kind, types)
    }
}
