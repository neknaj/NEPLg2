extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::CollectionSlotLifecycleFunctionSummaryIndex;
use super::drop_point_path::ResourceDropPointPath;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_control_slot_transfer::transfer_control_value_slots as transfer_slots;
use super::initialized_scalar_flow::I32ScalarReturnSummaryIndex;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::initialized_summary_byte_range_model::RawCellInitializationVariantParamByteRange;
use super::initialized_summary_param_byte_ranges::collect_param_initialized_raw_byte_ranges;
use super::initialized_summary_param_cells::collect_param_initialized_raw_cells;
use super::initialized_summary_seed::seed_summary_input_place;
use super::initialized_summary_variant_condition::{
    collect_variant_param_value_conditions, push_unique_variant_path_condition,
};
use super::initialized_summary_variant_model::{
    RawCellInitializationVariantCondition, RawCellInitializationVariantParamCell,
    RawCellInitializationVariantParamRequirement,
};
use super::initialized_summary_variant_requirement::collect_variant_param_required_raw_cells;
use super::initialized_summary_variant_type::return_type_may_have_variant_param_summary;
use super::initialized_summary_variant_unique::{
    push_unique_variant_param_byte_range, push_unique_variant_param_cell,
};
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{
    AggregateKind, Place, ResourceBlockId, ResourceConditionFact, ResourceFunction, ResourceLocal,
    ResourceMatchArm, ResourceOp,
};
use super::place_utils::{match_bind_payload_place, reference_target_place};
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDeferred;
use super::variant_name::normalize_variant_name;

#[derive(Clone)]
struct SelectedVariantPathCondition {
    condition_fact: ResourceConditionFact,
    truthy_path: bool,
    raw_aliases: RawCellAddressAliases,
}

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
    let empty_collection_slot_summaries = CollectionSlotLifecycleFunctionSummaryIndex::new(&[]);
    let engine = ResourceCheckEngine {
        function: function.name.as_str(),
        types,
        raw_alias_summaries,
        i32_scalar_summaries,
        raw_init_summaries,
        collection_slot_summaries: &empty_collection_slot_summaries,
        transform_range_certificates: None,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
    };
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let function_aliases = FunctionAliasTable::default();
    let pending_reallocs = PendingRawReallocs::default();
    let variant_initializations = PendingVariantRawCellInitializations::default();
    for param in &function.params {
        seed_summary_input_place(types, &mut cells, &mut raw_aliases, &param.place);
        if let Some(target_ty) = reference_target_type(types, param.place.ty) {
            let target = reference_target_place(&param.place, target_ty);
            seed_summary_input_place(types, &mut cells, &mut raw_aliases, &target);
        }
    }

    collect_variant_param_initialized_raw_cells_from_nested_return(
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
        &[],
        ops,
        return_value,
    );
}

#[allow(clippy::too_many_arguments)]
fn collect_variant_param_initialized_raw_cells_from_nested_return(
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
    path_conditions: &[SelectedVariantPathCondition],
    ops: &[ResourceOp],
    return_value: &Place,
) {
    let mut engine = ResourceCheckEngine {
        function: engine.function,
        types: engine.types,
        raw_alias_summaries: engine.raw_alias_summaries,
        i32_scalar_summaries: engine.i32_scalar_summaries,
        raw_init_summaries: engine.raw_init_summaries,
        collection_slot_summaries: engine.collection_slot_summaries,
        transform_range_certificates: engine.transform_range_certificates.clone(),
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
    };
    let mut cells = cells.clone();
    let mut collection_slots = CollectionSlotStateTable::new();
    let mut raw_aliases = raw_aliases.clone();
    let mut function_aliases = function_aliases.clone();
    let mut pending_reallocs = pending_reallocs.clone();
    let mut variant_initializations = variant_initializations.clone();

    for (index, op) in ops.iter().enumerate() {
        match op {
            ResourceOp::Branch {
                output,
                condition_fact,
                then_ops,
                then_value,
                else_ops,
                else_value,
                ..
            } if output == return_value => {
                collect_branch_variant_path(
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
                    params,
                    path_conditions,
                    condition_fact.as_ref(),
                    true,
                    then_ops,
                    then_value,
                );
                collect_branch_variant_path(
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
                    params,
                    path_conditions,
                    condition_fact.as_ref(),
                    false,
                    else_ops,
                    else_value,
                );
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                span,
                ..
            } if output == return_value => {
                for arm in arms {
                    collect_match_variant_path(
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
                        params,
                        path_conditions,
                        scrutinee,
                        arm,
                        *span,
                    );
                }
            }
            _ => {}
        }
        engine.check_ops(
            &mut cells,
            &mut collection_slots,
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
        engine.path_alternatives = Default::default();
    }
}

#[allow(clippy::too_many_arguments)]
fn collect_branch_variant_path(
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
    path_conditions: &[SelectedVariantPathCondition],
    condition_fact: Option<&ResourceConditionFact>,
    truthy_path: bool,
    path_ops: &[ResourceOp],
    path_value: &Place,
) {
    let mut path_cells = cells.clone();
    let mut path_aliases = raw_aliases.clone();
    let mut path_pending_reallocs = pending_reallocs.clone();
    let mut next_path_conditions = path_conditions.to_vec();
    if let Some(condition_fact) = condition_fact {
        next_path_conditions.push(SelectedVariantPathCondition {
            condition_fact: condition_fact.clone(),
            truthy_path,
            raw_aliases: raw_aliases.clone(),
        });
    }
    let mut path_engine = clone_variant_summary_engine(engine);
    path_engine.apply_branch_condition_fact(
        &mut path_cells,
        &mut path_aliases,
        &mut path_pending_reallocs,
        condition_fact,
        truthy_path,
    );
    collect_variant_param_initialized_raw_cells_from_path(
        out,
        byte_range_out,
        requirement_out,
        condition_out,
        &path_engine,
        &path_cells,
        &path_aliases,
        function_aliases,
        &path_pending_reallocs,
        variant_initializations,
        params,
        &next_path_conditions,
        path_ops,
        path_value,
    );
}

#[allow(clippy::too_many_arguments)]
fn collect_match_variant_path(
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
    path_conditions: &[SelectedVariantPathCondition],
    scrutinee: &Place,
    arm: &ResourceMatchArm,
    span: Span,
) {
    let mut path_engine = clone_variant_summary_engine(engine);
    let mut path_cells = cells.clone();
    let mut path_collection_slots = CollectionSlotStateTable::new();
    let mut path_aliases = raw_aliases.clone();
    let mut path_function_aliases = function_aliases.clone();
    let mut path_pending_reallocs = pending_reallocs.clone();
    let mut path_variant_initializations = variant_initializations.clone();
    if !apply_match_arm_entry(
        &mut path_engine,
        &mut path_cells,
        &mut path_collection_slots,
        &mut path_aliases,
        &mut path_function_aliases,
        &mut path_pending_reallocs,
        &mut path_variant_initializations,
        scrutinee,
        arm,
        span,
    ) {
        return;
    }
    collect_variant_param_initialized_raw_cells_from_path(
        out,
        byte_range_out,
        requirement_out,
        condition_out,
        &path_engine,
        &path_cells,
        &path_aliases,
        &path_function_aliases,
        &path_pending_reallocs,
        &path_variant_initializations,
        params,
        path_conditions,
        &arm.ops,
        &arm.value,
    );
}

#[allow(clippy::too_many_arguments)]
fn collect_variant_param_initialized_raw_cells_from_path(
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
    path_conditions: &[SelectedVariantPathCondition],
    path_ops: &[ResourceOp],
    path_value: &Place,
) {
    let Some(variant) = construct_variant_for_value(path_ops, path_value) else {
        collect_variant_param_initialized_raw_cells_from_nested_return(
            out,
            byte_range_out,
            requirement_out,
            condition_out,
            engine,
            cells,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
            params,
            path_conditions,
            path_ops,
            path_value,
        );
        return;
    };
    let mut path_engine = clone_variant_summary_engine(engine);
    let mut path_cells = cells.clone();
    let mut path_collection_slots = CollectionSlotStateTable::new();
    let mut path_aliases = raw_aliases.clone();
    let mut path_function_aliases = function_aliases.clone();
    let mut path_pending_reallocs = pending_reallocs.clone();
    let mut path_variant_initializations = variant_initializations.clone();
    path_engine.check_ops(
        &mut path_cells,
        &mut path_collection_slots,
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
    path_engine.path_alternatives = Default::default();

    let mut variant_path_conditions = Vec::new();
    for condition in path_conditions {
        collect_variant_param_value_conditions(
            &mut variant_path_conditions,
            Some(&condition.condition_fact),
            condition.truthy_path,
            &condition.raw_aliases,
            params,
        );
    }
    push_unique_variant_path_condition(condition_out, &variant, variant_path_conditions);

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
        path_engine.types,
        path_engine.raw_init_summaries,
        cells,
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

fn clone_variant_summary_engine<'a>(engine: &ResourceCheckEngine<'a>) -> ResourceCheckEngine<'a> {
    ResourceCheckEngine {
        function: engine.function,
        types: engine.types,
        raw_alias_summaries: engine.raw_alias_summaries,
        i32_scalar_summaries: engine.i32_scalar_summaries,
        raw_init_summaries: engine.raw_init_summaries,
        collection_slot_summaries: engine.collection_slot_summaries,
        transform_range_certificates: engine.transform_range_certificates.clone(),
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_match_arm_entry(
    path_engine: &mut ResourceCheckEngine<'_>,
    path_cells: &mut CellTable,
    path_collection_slots: &mut CollectionSlotStateTable,
    path_raw_aliases: &mut RawCellAddressAliases,
    path_function_aliases: &mut FunctionAliasTable,
    path_pending_reallocs: &mut PendingRawReallocs,
    path_variant_initializations: &mut PendingVariantRawCellInitializations,
    scrutinee: &Place,
    arm: &ResourceMatchArm,
    span: Span,
) -> bool {
    if !path_variant_initializations.match_arm_reachable(scrutinee, &arm.pattern) {
        return false;
    }
    if let Some(bind_local) = &arm.bind_local {
        path_cells.mark_initialized(bind_local);
        if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
            path_engine.copy_raw_alias_and_rekey_cells(
                path_cells,
                path_raw_aliases,
                &source,
                bind_local,
            );
            path_raw_aliases.copy_scalar_facts_if_tracked(&source, bind_local);
            path_cells.transfer_raw_cell_loaded_value_origin(&source, bind_local);
            transfer_slots(
                path_engine,
                path_collection_slots,
                &source,
                bind_local,
                span,
            );
            path_function_aliases.copy_alias(&source, bind_local);
            path_pending_reallocs.copy_result(&source, bind_local);
            path_variant_initializations.copy_result(&source, bind_local);
        } else {
            path_raw_aliases.clear(bind_local);
            path_pending_reallocs.clear_result(bind_local);
            path_variant_initializations.clear_result(bind_local);
        }
    }
    path_variant_initializations.apply_match_arm(
        path_engine,
        path_cells,
        path_raw_aliases,
        scrutinee,
        &arm.pattern,
        arm.span,
    );
    true
}

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(*target),
        _ => None,
    }
}
