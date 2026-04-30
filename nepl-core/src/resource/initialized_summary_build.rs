extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::cell_state::{raw_cell_suffix_after_address, CellTable};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummary;
use super::initialized_summary::{
    RawCellInitializationFunctionSummary, RawCellInitializationParamCell,
    RawCellInitializationReturnCell,
};
use super::initialized_summary_variant_build::collect_variant_param_initialized_raw_cells_from_return;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{
    CellState, Place, ResourceFunction, ResourceLocal, ResourceModule, ResourceTerminator,
};
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDeferred;

pub(super) fn compute_raw_cell_initialization_function_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
) -> Vec<RawCellInitializationFunctionSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let summary = function_raw_cell_initialization_summary(
                function,
                types,
                raw_alias_summaries,
                &summaries,
            );
            if !summary.return_cells.is_empty()
                || !summary.param_cells.is_empty()
                || !summary.variant_param_cells.is_empty()
                || !summary.variant_required_param_cells.is_empty()
            {
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

fn function_raw_cell_initialization_summary(
    function: &ResourceFunction,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
) -> RawCellInitializationFunctionSummary {
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
    let mut pending_reallocs = PendingRawReallocs::default();
    for param in &function.params {
        cells.mark_initialized(&param.place);
        raw_aliases.mark(&param.place);
    }

    let mut out = RawCellInitializationFunctionSummary {
        function: function.name.clone(),
        return_cells: Vec::new(),
        param_cells: Vec::new(),
        variant_param_cells: Vec::new(),
        variant_required_param_cells: Vec::new(),
    };
    let mut guaranteed_return_cells = None;
    let mut guaranteed_param_cells = None;
    for block in &function.blocks {
        engine.check_ops(
            &mut cells,
            &mut raw_aliases,
            &mut function_aliases,
            &mut pending_reallocs,
            &mut PendingVariantRawCellInitializations::default(),
            &block.ops,
        );
        if let ResourceTerminator::Return { value, .. } = &block.terminator {
            let mut path_return_cells = Vec::new();
            if let Some(value) = value {
                collect_return_initialized_raw_cells(
                    &mut path_return_cells,
                    &cells,
                    &raw_aliases,
                    value,
                );
            }
            merge_guaranteed_facts(&mut guaranteed_return_cells, path_return_cells);

            let mut path_param_cells = Vec::new();
            collect_param_initialized_raw_cells(
                &mut path_param_cells,
                &cells,
                &raw_aliases,
                &function.params,
            );
            merge_guaranteed_facts(&mut guaranteed_param_cells, path_param_cells);
        }
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            collect_variant_param_initialized_raw_cells_from_return(
                &mut out.variant_param_cells,
                &mut out.variant_required_param_cells,
                function,
                types,
                raw_alias_summaries,
                raw_init_summaries,
                &block.ops,
                value,
            );
        }
    }
    out.return_cells = guaranteed_return_cells.unwrap_or_default();
    out.param_cells = guaranteed_param_cells.unwrap_or_default();
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
        let holds_raw_address = raw_aliases.value_is_known_raw_address(&entry.place);
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

fn merge_guaranteed_facts<T: Clone + Eq>(guaranteed: &mut Option<Vec<T>>, path: Vec<T>) {
    match guaranteed {
        Some(existing) => {
            existing.retain(|fact| path.contains(fact));
        }
        None => {
            *guaranteed = Some(path);
        }
    }
}
