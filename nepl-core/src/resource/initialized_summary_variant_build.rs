extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::cell_state::raw_address_suffix_after_address;
use super::cell_state::CellTable;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummary;
use super::initialized_summary::{
    RawCellInitializationFunctionSummary, RawCellInitializationVariantCondition,
    RawCellInitializationVariantParamCell, RawCellInitializationVariantParamRange,
    RawCellInitializationVariantParamRequirement,
};
use super::initialized_summary_cells::collect_param_initialized_raw_cells;
use super::initialized_summary_variant_condition::collect_variant_param_condition;
use super::initialized_summary_variant_requirement::collect_variant_param_required_raw_cells;
use super::initialized_variant::{normalize_variant_name, PendingVariantRawCellInitializations};
use super::model::{
    AggregateKind, Place, RawMemoryOp, ResourceConditionFact, ResourceFunction, ResourceLocal,
    ResourceOp,
};
use super::place_utils::place_suffix_after_prefix;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDeferred;

pub(super) fn collect_variant_param_initialized_raw_cells_from_return(
    out: &mut Vec<RawCellInitializationVariantParamCell>,
    range_out: &mut Vec<RawCellInitializationVariantParamRange>,
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
                    range_out,
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
                    range_out,
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
    range_out: &mut Vec<RawCellInitializationVariantParamRange>,
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
    collect_variant_param_initialized_raw_ranges(
        range_out,
        &variant,
        path_ops,
        &path_aliases,
        params,
    );
    collect_variant_param_required_raw_cells(
        requirement_out,
        &variant,
        path_ops,
        &path_aliases,
        params,
    );
}

fn collect_variant_param_initialized_raw_ranges(
    out: &mut Vec<RawCellInitializationVariantParamRange>,
    variant: &str,
    ops: &[ResourceOp],
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
) {
    for op in ops {
        let ResourceOp::RawMemory {
            operation: RawMemoryOp::Fill { unit },
            args,
            ..
        } = op
        else {
            continue;
        };
        let (Some(address), Some(count), Some(value)) = (args.first(), args.get(1), args.get(2))
        else {
            continue;
        };
        let address = raw_aliases.canonicalize(address);
        let count = raw_aliases.canonicalize(count);
        let address_params = param_suffixes_for_raw_address(raw_aliases, params, &address);
        let count_params = param_suffixes_for_place(raw_aliases, params, &count);
        for (address_param_index, address_suffix) in &address_params {
            for (count_param_index, count_suffix) in &count_params {
                push_unique_variant_param_range(
                    out,
                    RawCellInitializationVariantParamRange {
                        variant: normalize_variant_name(variant),
                        address_param_index: *address_param_index,
                        address_suffix: address_suffix.clone(),
                        address_ty: address.ty,
                        count_param_index: *count_param_index,
                        count_suffix: count_suffix.clone(),
                        count_ty: count.ty,
                        unit: *unit,
                        ty: value.ty,
                    },
                );
            }
        }
    }
}

fn param_suffixes_for_raw_address(
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    place: &Place,
) -> Vec<(usize, Vec<super::model::PlaceProjection>)> {
    let mut out = Vec::new();
    let aliases = raw_aliases.aliases_for(place);
    for (param_index, param) in params.iter().enumerate() {
        for param_alias in raw_aliases.aliases_for(&param.place) {
            for alias in &aliases {
                let Some(suffix) = raw_address_suffix_after_address(alias, &param_alias) else {
                    continue;
                };
                push_unique_param_suffix(&mut out, param_index, suffix);
            }
        }
    }
    out
}

fn param_suffixes_for_place(
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    place: &Place,
) -> Vec<(usize, Vec<super::model::PlaceProjection>)> {
    let mut out = Vec::new();
    let aliases = raw_aliases.aliases_for(place);
    for (param_index, param) in params.iter().enumerate() {
        for param_alias in raw_aliases.aliases_for(&param.place) {
            for alias in &aliases {
                let Some(suffix) = place_suffix_after_prefix(alias, &param_alias) else {
                    continue;
                };
                push_unique_param_suffix(&mut out, param_index, suffix);
            }
        }
    }
    out
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

fn push_unique_variant_param_range(
    ranges: &mut Vec<RawCellInitializationVariantParamRange>,
    range: RawCellInitializationVariantParamRange,
) {
    if !ranges.iter().any(|existing| existing == &range) {
        ranges.push(range);
    }
}

fn push_unique_param_suffix(
    values: &mut Vec<(usize, Vec<super::model::PlaceProjection>)>,
    param_index: usize,
    suffix: Vec<super::model::PlaceProjection>,
) {
    if !values.iter().any(|(existing_index, existing_suffix)| {
        *existing_index == param_index && *existing_suffix == suffix
    }) {
        values.push((param_index, suffix));
    }
}
