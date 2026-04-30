extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::cell_state::{raw_cell_suffix_after_address, CellTable};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummary;
use super::model::{
    CellState, Place, PlaceProjection, ResourceCallTarget, ResourceFunction, ResourceModule,
    ResourceTerminator,
};
use super::place_utils::place_with_suffix;
use super::report::ResourceCheckDeferred;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellInitializationReturnSummary {
    pub(super) function: String,
    cells: Vec<RawCellInitializationReturnCell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawCellInitializationReturnCell {
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
    holds_raw_address: bool,
}

pub(super) fn compute_raw_cell_initialization_return_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
) -> Vec<RawCellInitializationReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let summary = function_raw_cell_initialization_return_summary(
                function,
                types,
                raw_alias_summaries,
                &summaries,
            );
            if !summary.cells.is_empty() {
                next.push(summary);
            }
        }
        if next == summaries {
            return summaries;
        }
        summaries = next;
    }
    summaries
}

fn function_raw_cell_initialization_return_summary(
    function: &ResourceFunction,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    raw_init_summaries: &[RawCellInitializationReturnSummary],
) -> RawCellInitializationReturnSummary {
    let mut engine = ResourceCheckEngine {
        function: function.name.as_str(),
        types,
        raw_alias_summaries,
        raw_init_summaries,
        diagnostics: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
    };
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut function_aliases = FunctionAliasTable::default();
    for param in &function.params {
        cells.mark_initialized(&param.place);
        cells.mark_external_raw_storage_root(&param.place);
        raw_aliases.mark(&param.place);
    }

    let mut out = RawCellInitializationReturnSummary {
        function: function.name.clone(),
        cells: Vec::new(),
    };
    for block in &function.blocks {
        engine.check_ops(
            &mut cells,
            &mut raw_aliases,
            &mut function_aliases,
            &block.ops,
        );
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            collect_return_initialized_raw_cells(&mut out.cells, &cells, &raw_aliases, value);
        }
    }
    out
}

fn collect_return_initialized_raw_cells(
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
        let holds_raw_address = raw_cell_value_is_known_raw_address(raw_aliases, &entry.place);
        for cell_alias in raw_aliases.aliases_for(&entry.place) {
            for return_alias in &return_aliases {
                let Some(suffix) = raw_cell_suffix_after_address(&cell_alias, return_alias) else {
                    continue;
                };
                push_unique_raw_cell_initialization_return(
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

fn push_unique_raw_cell_initialization_return(
    cells: &mut Vec<RawCellInitializationReturnCell>,
    cell: RawCellInitializationReturnCell,
) {
    if !cells.iter().any(|existing| existing == &cell) {
        cells.push(cell);
    }
}

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_call_return_raw_cell_initialization(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        output: &Place,
        target: &ResourceCallTarget,
    ) {
        let ResourceCallTarget::User { name, .. } = target else {
            return;
        };
        let Some(summary) = self
            .raw_init_summaries
            .iter()
            .find(|summary| summary.function == name.as_str())
        else {
            return;
        };
        self.apply_raw_cell_initialization_return_summary(cells, raw_aliases, output, summary);
    }

    pub(super) fn apply_indirect_call_return_raw_cell_initialization(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        output: &Place,
        function_aliases: &FunctionAliasTable,
        callee: &Place,
    ) {
        for function in function_aliases.functions(callee) {
            let Some(summary) = self
                .raw_init_summaries
                .iter()
                .find(|summary| summary.function == function.as_str())
            else {
                continue;
            };
            self.apply_raw_cell_initialization_return_summary(cells, raw_aliases, output, summary);
        }
    }

    fn apply_raw_cell_initialization_return_summary(
        &self,
        cells: &mut CellTable,
        raw_aliases: &mut RawCellAddressAliases,
        output: &Place,
        summary: &RawCellInitializationReturnSummary,
    ) {
        mark_known_raw_address(raw_aliases, output);
        for cell in &summary.cells {
            let place = place_with_suffix(output, &cell.suffix, cell.ty);
            cells.mark_initialized(&place);
            if cell.holds_raw_address {
                mark_known_raw_address(raw_aliases, &place);
            }
        }
    }
}

pub(super) fn raw_cell_value_is_known_raw_address(
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
) -> bool {
    raw_aliases.contains_exact(place) || raw_aliases.aliases_for(place).len() > 1
}

pub(super) fn mark_known_raw_address(raw_aliases: &mut RawCellAddressAliases, place: &Place) {
    if !raw_aliases.contains_exact(place) {
        raw_aliases.mark(place);
    }
}
