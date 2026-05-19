extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::drop_point_path::ResourceDropPointPath;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_scalar_flow::I32ScalarReturnSummaryIndex;
use super::initialized_summary::{
    RawCellInitializationFunctionSummaryIndex, RawCellInitializationVariantCondition,
    RawCellInitializationVariantParamCell, RawCellInitializationVariantParamRequirement,
};
use super::initialized_summary_byte_range_model::RawCellInitializationVariantParamByteRange;
use super::initialized_summary_param_byte_ranges::collect_param_initialized_raw_byte_ranges;
use super::initialized_summary_param_cells::collect_param_initialized_raw_cells;
use super::initialized_summary_seed::seed_summary_input_place;
use super::initialized_summary_variant_condition::collect_variant_param_condition;
use super::initialized_summary_variant_requirement::collect_variant_param_required_raw_cells;
use super::initialized_summary_variant_type::return_type_may_have_variant_param_summary;
use super::initialized_summary_variant_unique::{
    push_unique_variant_param_byte_range, push_unique_variant_param_cell,
};
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{
    AggregateKind, Place, ResourceBlockId, ResourceConditionFact, ResourceFunction, ResourceLocal,
    ResourceOp,
};
use super::place_utils::reference_target_place;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDeferred;
use super::variant_name::normalize_variant_name;

pub(super) fn collect_variant_param_initialized_raw_cells_from_return(
    out: &mut Vec<RawCellInitializationVariantParamCell>,
    byte_range_out: &mut Vec<RawCellInitializationVariantParamByteRange>,
    requirement_out: &mut Vec<RawCellInitializationVariantParamRequirement>,
    condition_out: &mut Vec<RawCellInitializationVariantCondition>,
    function: &ResourceFunction,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    i32_scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
    return_value: &Place,
) {
    if !return_type_may_have_variant_param_summary(types, return_value.ty) {
        return;
    }
    let mut engine = ResourceCheckEngine {
        function: function.name.as_str(),
        types,
        raw_alias_summaries,
        i32_scalar_summaries,
        raw_init_summaries,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
    };
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut function_aliases = FunctionAliasTable::default();
    let mut pending_reallocs = PendingRawReallocs::default();
    let mut variant_initializations = PendingVariantRawCellInitializations::default();
    for param in &function.params {
        seed_summary_input_place(types, &mut cells, &mut raw_aliases, &param.place);
        if let Some(target_ty) = reference_target_type(types, param.place.ty) {
            let target = reference_target_place(&param.place, target_ty);
            seed_summary_input_place(types, &mut cells, &mut raw_aliases, &target);
        }
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
                    byte_range_out,
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
                    byte_range_out,
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
            ResourceDropPointPath {
                block: ResourceBlockId(usize::MAX),
                steps: Vec::new(),
            },
        );
        engine.auto_drop_points.clear();
    }
}

fn collect_branch_variant_param_initialized_raw_cells(
    out: &mut Vec<RawCellInitializationVariantParamCell>,
    byte_range_out: &mut Vec<RawCellInitializationVariantParamByteRange>,
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
        i32_scalar_summaries: engine.i32_scalar_summaries,
        raw_init_summaries: engine.raw_init_summaries,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
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
        ResourceDropPointPath {
            block: ResourceBlockId(usize::MAX),
            steps: Vec::new(),
        },
    );
    path_engine.auto_drop_points.clear();

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
    let mut path_param_byte_ranges = Vec::new();
    collect_param_initialized_raw_byte_ranges(
        &mut path_param_byte_ranges,
        &path_cells,
        &path_aliases,
        params,
    );
    for range in path_param_byte_ranges {
        push_unique_variant_param_byte_range(
            byte_range_out,
            RawCellInitializationVariantParamByteRange {
                variant: normalize_variant_name(&variant),
                address_param_index: range.address_param_index,
                address_suffix: range.address_suffix,
                address_ty: range.address_ty,
                count: range.count,
                unit: range.unit,
                ty: range.ty,
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

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}
