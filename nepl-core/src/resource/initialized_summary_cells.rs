extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::{raw_cell_suffix_after_address, CellTable};
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::{RawCellInitializationParamCell, RawCellInitializationReturnCell};
use super::model::{CellState, Place, ResourceLocal};

pub(super) fn collect_return_initialized_raw_cells(
    out: &mut Vec<RawCellInitializationReturnCell>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
) {
    let return_aliases = raw_aliases.aliases_for(value);
    for entry in cells.entries() {
        if !matches!(entry.state, CellState::Initialized(_)) {
            continue;
        }
        let holds_raw_address = raw_aliases.value_is_known_raw_address(&entry.place);
        if let Some(suffix) = raw_cell_suffix_after_address(&entry.place, value) {
            push_unique_return_cell(
                out,
                RawCellInitializationReturnCell {
                    suffix,
                    ty: entry.place.ty,
                    holds_raw_address,
                },
            );
        }
        for cell_alias in raw_aliases.aliases_for(&entry.place) {
            for return_alias in &return_aliases {
                let Some(suffix) = raw_cell_suffix_after_address(&cell_alias, return_alias) else {
                    continue;
                };
                push_unique_return_cell(
                    out,
                    RawCellInitializationReturnCell {
                        suffix,
                        ty: entry.place.ty,
                        holds_raw_address,
                    },
                );
            }
        }
    }
}

pub(super) fn collect_param_initialized_raw_cells(
    out: &mut Vec<RawCellInitializationParamCell>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
) {
    for (param_index, param) in params.iter().enumerate() {
        let param_aliases = raw_aliases.aliases_for(&param.place);
        for entry in cells.entries() {
            if !matches!(entry.state, CellState::Initialized(_)) {
                continue;
            }
            let holds_raw_address = raw_aliases.value_is_known_raw_address(&entry.place);
            for cell_alias in raw_aliases.aliases_for(&entry.place) {
                for param_alias in &param_aliases {
                    let Some(suffix) = raw_cell_suffix_after_address(&cell_alias, param_alias)
                    else {
                        continue;
                    };
                    push_unique_param_cell(
                        out,
                        RawCellInitializationParamCell {
                            param_index,
                            suffix,
                            ty: entry.place.ty,
                            holds_raw_address,
                        },
                    );
                }
            }
        }
    }
}

fn push_unique_return_cell(
    cells: &mut Vec<RawCellInitializationReturnCell>,
    cell: RawCellInitializationReturnCell,
) {
    if !cells.iter().any(|existing| existing == &cell) {
        cells.push(cell);
    }
}

fn push_unique_param_cell(
    cells: &mut Vec<RawCellInitializationParamCell>,
    cell: RawCellInitializationParamCell,
) {
    if !cells.iter().any(|existing| existing == &cell) {
        cells.push(cell);
    }
}
