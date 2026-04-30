extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummary;
use super::initialized_summary::{
    RawCellInitializationFunctionSummary, RawCellInitializationVariantCondition,
    RawCellInitializationVariantParamCell, RawCellInitializationVariantParamRequirement,
};
use super::initialized_summary_cells::collect_param_initialized_raw_cells;
use super::initialized_summary_variant_condition::collect_variant_param_condition;
use super::initialized_summary_variant_requirement::collect_variant_param_required_raw_cells;
use super::initialized_variant::{normalize_variant_name, PendingVariantRawCellInitializations};
use super::model::{
    AggregateKind, Place, ResourceConditionFact, ResourceFunction, ResourceLocal, ResourceOp,
};
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDeferred;

pub(super) fn collect_variant_param_initialized_raw_cells_from_return(
    out: &mut Vec<RawCellInitializationVariantParamCell>,
    requirement_out: &mut Vec<RawCellInitializationVariantParamRequirement>,
    condition_out: &mut Vec<RawCellInitializationVariantCondition>,
    function: &ResourceFunction,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
    ops: &[ResourceOp],
    return_value: &Place,
) {
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
    let mut variant_initializations = PendingVariantRawCellInitializations::default();
    for param in &function.params {
        cells.mark_initialized(&param.place);
        raw_aliases.mark(&param.place);
    }

    for (index, op) in ops.iter().enumerate() {
        if let ResourceOp::Branch {
            output,
            condition_fact,
            then_ops,
            then_value,
            else_ops,
            else_value,
            ..
        } = op
        {
            if output == return_value {
                collect_branch_variant_param_initialized_raw_cells(
                    out,
                    requirement_out,
                    condition_out,
                    &engine,
                    &cells,
                    &raw_aliases,
                    &function_aliases,
                    &pending_reallocs,
                    &variant_initializations,
                    &function.params,
                    condition_fact.as_ref(),
                    true,
                    then_ops,
                    then_value,
                );
                collect_branch_variant_param_initialized_raw_cells(
                    out,
                    requirement_out,
                    condition_out,
                    &engine,
                    &cells,
                    &raw_aliases,
                    &function_aliases,
                    &pending_reallocs,
                    &variant_initializations,
                    &function.params,
                    condition_fact.as_ref(),
                    false,
                    else_ops,
                    else_value,
                );
            }
        }
        engine.check_ops(
            &mut cells,
            &mut raw_aliases,
            &mut function_aliases,
            &mut pending_reallocs,
            &mut variant_initializations,
            &ops[index..=index],
        );
    }
}

fn collect_branch_variant_param_initialized_raw_cells(
    out: &mut Vec<RawCellInitializationVariantParamCell>,
    requirement_out: &mut Vec<RawCellInitializationVariantParamRequirement>,
    condition_out: &mut Vec<RawCellInitializationVariantCondition>,
    engine: &ResourceCheckEngine<'_>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    pending_reallocs: &PendingRawReallocs,
    variant_initializations: &PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    condition_fact: Option<&ResourceConditionFact>,
    truthy_path: bool,
    path_ops: &[ResourceOp],
    path_value: &Place,
) {
    let Some(variant) = construct_variant_for_value(path_ops, path_value) else {
        return;
    };
    let mut path_engine = ResourceCheckEngine {
        function: engine.function,
        types: engine.types,
        raw_alias_summaries: engine.raw_alias_summaries,
        raw_init_summaries: engine.raw_init_summaries,
        diagnostics: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
    };
    let mut path_cells = cells.clone();
    let mut path_aliases = raw_aliases.clone();
    let mut path_function_aliases = function_aliases.clone();
    let mut path_pending_reallocs = pending_reallocs.clone();
    let mut path_variant_initializations = variant_initializations.clone();
    collect_variant_param_condition(
        condition_out,
        &variant,
        condition_fact,
        truthy_path,
        &path_aliases,
        params,
    );
    path_engine.check_ops(
        &mut path_cells,
        &mut path_aliases,
        &mut path_function_aliases,
        &mut path_pending_reallocs,
        &mut path_variant_initializations,
        path_ops,
    );

    let mut path_param_cells = Vec::new();
    collect_param_initialized_raw_cells(&mut path_param_cells, &path_cells, &path_aliases, params);
    for cell in path_param_cells {
        push_unique_variant_param_cell(
            out,
            RawCellInitializationVariantParamCell {
                variant: normalize_variant_name(&variant),
                param_index: cell.param_index,
                suffix: cell.suffix,
                ty: cell.ty,
                holds_raw_address: cell.holds_raw_address,
            },
        );
    }
    collect_variant_param_required_raw_cells(
        requirement_out,
        &variant,
        path_ops,
        &path_aliases,
        params,
    );
}

fn construct_variant_for_value(ops: &[ResourceOp], value: &Place) -> Option<String> {
    for op in ops.iter().rev() {
        let ResourceOp::Construct {
            output,
            kind: AggregateKind::Enum { variant, .. },
            ..
        } = op
        else {
            continue;
        };
        if output == value {
            return Some(variant.clone());
        }
    }
    None
}

fn push_unique_variant_param_cell(
    cells: &mut Vec<RawCellInitializationVariantParamCell>,
    cell: RawCellInitializationVariantParamCell,
) {
    if !cells.iter().any(|existing| existing == &cell) {
        cells.push(cell);
    }
}
