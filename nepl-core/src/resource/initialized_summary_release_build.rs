extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::{raw_address_suffix_after_address, CellTable};
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::drop_point_path::ResourceDropPointPath;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_control::{
    initialize_control_output_path_states, invalidate_control_output_path_states,
    invalidate_control_output_state,
    path_alternatives_or_single,
};
use super::initialized_control_slot_transfer::transfer_control_value_slots as transfer_slots;
use super::initialized_path_state::{
    merge_path_alternatives_into, path_states_need_replay, ResourcePathAlternatives,
};
use super::initialized_str_layout::seed_str_storage_layout;
use super::initialized_summary::{
    RawCellInitializationFunctionSummary, RawCellInitializationFunctionSummaryIndex,
    RawCellReleaseParamRequirement, RawCellReleaseRequirementKind,
};
use super::initialized_summary_engine::summary_check_engine;
use super::initialized_summary_indirect_release::{
    collect_unknown_indirect_call_release_requirements, indirect_call_may_release_raw_cells,
};
use super::initialized_summary_raw_release::collect_raw_memory_release_requirements;
use super::initialized_summary_seed::summary_input_type_may_seed_raw_address_alias;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{
    CellState, Place, ResourceBlockId, ResourceCallTarget, ResourceConditionFact, ResourceLocal,
    ResourceMatchArm, ResourceOp,
};
use super::place_utils::{match_bind_payload_place, projected_place_with_concrete_type};
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckOperation;

pub(super) fn collect_param_release_requirements_from_ops(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    engine: &ResourceCheckEngine<'_>,
    cells: &mut CellTable,
    collection_slots: &mut CollectionSlotStateTable,
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    pending_reallocs: &mut PendingRawReallocs,
    variant_initializations: &mut PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
) {
    let mut step_engine = summary_check_engine(engine);
    collect_param_release_requirements_from_ops_with_engine(
        out,
        &mut step_engine,
        cells,
        collection_slots,
        raw_aliases,
        function_aliases,
        pending_reallocs,
        variant_initializations,
        params,
        raw_init_summaries,
        ops,
    );
}

fn collect_param_release_requirements_from_ops_with_engine(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    step_engine: &mut ResourceCheckEngine<'_>,
    cells: &mut CellTable,
    collection_slots: &mut CollectionSlotStateTable,
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    pending_reallocs: &mut PendingRawReallocs,
    variant_initializations: &mut PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
) {
    for op in ops {
        let incoming_path_alternatives = core::mem::take(&mut step_engine.path_alternatives);
        if let Some(alternatives) = incoming_path_alternatives.into_feasible_states() {
            if alternatives.is_empty() {
                step_engine.path_alternatives = ResourcePathAlternatives::from_states(alternatives);
                continue;
            }
            // Release requirement summaries are may-summaries over raw-cell release
            // operations. Once a branch or match has merged its feasible states into the
            // straight-line tables, carrying the exact alternatives into every later op only
            // multiplies the traversal. The merged raw-address alias table conservatively
            // contains aliases from every feasible path, so collecting from the merged state can
            // over-approximate requirements but does not miss a parameter-backed release.
            merge_path_alternatives_into(
                &alternatives,
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
            );
        }
        if let ResourceOp::Branch {
            output,
            condition,
            condition_fact,
            then_ops,
            then_value,
            else_ops,
            else_value,
            span,
        } = op
        {
            collect_branch_release_requirements_and_step(
                out,
                step_engine,
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
                params,
                raw_init_summaries,
                output,
                condition,
                condition_fact.as_ref(),
                then_ops,
                then_value,
                else_ops,
                else_value,
                *span,
            );
            step_engine.auto_drop_points.clear();
            continue;
        }
        if let ResourceOp::Match {
            output,
            scrutinee,
            scrutinee_is_borrow_target,
            arms,
            span,
        } = op
        {
            collect_match_release_requirements_and_step(
                out,
                step_engine,
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
                params,
                raw_init_summaries,
                output,
                scrutinee,
                *scrutinee_is_borrow_target,
                arms,
                *span,
            );
            step_engine.auto_drop_points.clear();
            continue;
        }
        collect_param_release_requirements_from_op(
            out,
            &step_engine,
            cells,
            collection_slots,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
            params,
            raw_init_summaries,
            op,
        );
        step_engine.check_ops(
            cells,
            collection_slots,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
            core::slice::from_ref(op),
            ResourceDropPointPath {
                block: ResourceBlockId(usize::MAX),
                steps: Vec::new(),
            },
        );
        // Auto drop points are diagnostics-oriented state. Release requirement summaries need
        // the proof tables carried above, but do not persist temporary drop insertion candidates.
        step_engine.auto_drop_points.clear();
    }
}

fn collect_match_release_requirements_and_step(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    engine: &mut ResourceCheckEngine<'_>,
    cells: &mut CellTable,
    collection_slots: &mut CollectionSlotStateTable,
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    pending_reallocs: &mut PendingRawReallocs,
    variant_initializations: &mut PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    output: &Place,
    scrutinee: &Place,
    scrutinee_is_borrow_target: bool,
    arms: &[ResourceMatchArm],
    span: crate::span::Span,
) {
    let scrutinee_available = if scrutinee_is_borrow_target {
        engine.ensure_available(
            cells,
            scrutinee,
            ResourceCheckOperation::MatchScrutinee,
            span,
        )
    } else {
        engine.consume_by_value(
            cells,
            scrutinee,
            ResourceCheckOperation::MatchScrutinee,
            span,
        )
    };
    let mut arms_available = true;
    let mut match_paths = Vec::new();

    for arm in arms {
        let mut arm_cells = cells.clone();
        let mut arm_collection_slots = collection_slots.clone();
        let mut arm_aliases = raw_aliases.clone();
        let mut arm_function_aliases = function_aliases.clone();
        let mut arm_pending_reallocs = pending_reallocs.clone();
        let mut arm_variant_initializations = variant_initializations.clone();
        if !arm_variant_initializations.match_arm_reachable(scrutinee, &arm.pattern) {
            continue;
        }
        if let Some(bind_local) = &arm.bind_local {
            arm_cells.mark_initialized(bind_local);
            if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
                engine.copy_raw_alias_and_rekey_cells(
                    &mut arm_cells,
                    &mut arm_aliases,
                    &source,
                    bind_local,
                );
                arm_cells.transfer_raw_cell_loaded_value_origin(&source, bind_local);
                transfer_slots(
                    engine,
                    &mut arm_collection_slots,
                    &source,
                    bind_local,
                    arm.span,
                );
                arm_function_aliases.copy_alias(&source, bind_local);
                arm_pending_reallocs.copy_result(&source, bind_local);
                arm_variant_initializations.copy_result(&source, bind_local);
            } else {
                arm_aliases.clear(bind_local);
                arm_pending_reallocs.clear_result(bind_local);
                arm_variant_initializations.clear_result(bind_local);
            }
        }
        arm_variant_initializations.apply_match_arm(
            engine,
            &mut arm_cells,
            &mut arm_aliases,
            scrutinee,
            &arm.pattern,
            arm.span,
        );
        collect_param_release_requirements_from_ops_with_engine(
            out,
            engine,
            &mut arm_cells,
            &mut arm_collection_slots,
            &mut arm_aliases,
            &mut arm_function_aliases,
            &mut arm_pending_reallocs,
            &mut arm_variant_initializations,
            params,
            raw_init_summaries,
            &arm.ops,
        );
        let arm_path_alternatives = core::mem::take(&mut engine.path_alternatives);
        if engine.place_is_never(&arm.value) {
            continue;
        }
        let arm_states = path_alternatives_or_single(
            arm_path_alternatives,
            arm_cells,
            arm_collection_slots,
            arm_aliases,
            arm_function_aliases,
            arm_pending_reallocs,
            arm_variant_initializations,
        );
        match_paths.extend(engine.transfer_control_value_path_states(
            arm_states,
            &arm.value,
            output,
            ResourceCheckOperation::MatchValue,
            arm.span,
            &mut arms_available,
        ));
    }

    let has_match_paths = !match_paths.is_empty();
    if !has_match_paths {
        arms_available = false;
    }
    if has_match_paths {
        merge_path_alternatives_into(
            &match_paths,
            cells,
            collection_slots,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
        );
        if path_states_need_replay(&match_paths) {
            engine.path_alternatives = ResourcePathAlternatives::from_states(match_paths);
        }
    }
    if scrutinee_available && arms_available {
        cells.set_state(output, CellState::Initialized(output.ty));
        seed_str_storage_layout(engine.types, cells, raw_aliases, output);
        initialize_control_output_path_states(engine.types, &mut engine.path_alternatives, output);
    } else {
        invalidate_control_output_state(
            cells,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
            output,
        );
        invalidate_control_output_path_states(&mut engine.path_alternatives, output);
    }
}

fn collect_branch_release_requirements_and_step(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    engine: &mut ResourceCheckEngine<'_>,
    cells: &mut CellTable,
    collection_slots: &mut CollectionSlotStateTable,
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    pending_reallocs: &mut PendingRawReallocs,
    variant_initializations: &mut PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    output: &Place,
    condition: &Place,
    condition_fact: Option<&ResourceConditionFact>,
    then_ops: &[ResourceOp],
    then_value: &Place,
    else_ops: &[ResourceOp],
    else_value: &Place,
    span: crate::span::Span,
) {
    let condition_available = engine.consume_by_value(
        cells,
        condition,
        ResourceCheckOperation::BranchCondition,
        span,
    );
    cells.discard_raw_cell_loaded_value_origin(condition);

    let mut then_cells = cells.clone();
    let mut else_cells = cells.clone();
    let mut then_collection_slots = collection_slots.clone();
    let mut else_collection_slots = collection_slots.clone();
    let mut then_aliases = raw_aliases.clone();
    let mut else_aliases = raw_aliases.clone();
    let mut then_function_aliases = function_aliases.clone();
    let mut else_function_aliases = function_aliases.clone();
    let mut then_pending_reallocs = pending_reallocs.clone();
    let mut else_pending_reallocs = pending_reallocs.clone();
    let mut then_variant_initializations = variant_initializations.clone();
    let mut else_variant_initializations = variant_initializations.clone();

    engine.apply_branch_condition_fact(
        &mut then_cells,
        &mut then_aliases,
        &mut then_pending_reallocs,
        condition_fact,
        true,
    );
    engine.apply_branch_condition_fact(
        &mut else_cells,
        &mut else_aliases,
        &mut else_pending_reallocs,
        condition_fact,
        false,
    );

    collect_param_release_requirements_from_ops_with_engine(
        out,
        engine,
        &mut then_cells,
        &mut then_collection_slots,
        &mut then_aliases,
        &mut then_function_aliases,
        &mut then_pending_reallocs,
        &mut then_variant_initializations,
        params,
        raw_init_summaries,
        then_ops,
    );
    let then_path_alternatives = core::mem::take(&mut engine.path_alternatives);

    collect_param_release_requirements_from_ops_with_engine(
        out,
        engine,
        &mut else_cells,
        &mut else_collection_slots,
        &mut else_aliases,
        &mut else_function_aliases,
        &mut else_pending_reallocs,
        &mut else_variant_initializations,
        params,
        raw_init_summaries,
        else_ops,
    );
    let else_path_alternatives = core::mem::take(&mut engine.path_alternatives);

    let mut branch_paths = Vec::new();
    let mut paths_available = condition_available;
    if !engine.place_is_never(then_value) {
        let then_states = path_alternatives_or_single(
            then_path_alternatives,
            then_cells,
            then_collection_slots,
            then_aliases,
            then_function_aliases,
            then_pending_reallocs,
            then_variant_initializations,
        );
        branch_paths.extend(engine.transfer_control_value_path_states(
            then_states,
            then_value,
            output,
            ResourceCheckOperation::BranchValue,
            span,
            &mut paths_available,
        ));
    }
    if !engine.place_is_never(else_value) {
        let else_states = path_alternatives_or_single(
            else_path_alternatives,
            else_cells,
            else_collection_slots,
            else_aliases,
            else_function_aliases,
            else_pending_reallocs,
            else_variant_initializations,
        );
        branch_paths.extend(engine.transfer_control_value_path_states(
            else_states,
            else_value,
            output,
            ResourceCheckOperation::BranchValue,
            span,
            &mut paths_available,
        ));
    }

    let has_branch_paths = !branch_paths.is_empty();
    if has_branch_paths {
        merge_path_alternatives_into(
            &branch_paths,
            cells,
            collection_slots,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
        );
        if path_states_need_replay(&branch_paths) {
            engine.path_alternatives = ResourcePathAlternatives::from_states(branch_paths);
        }
    }
    if paths_available && has_branch_paths {
        cells.set_state(output, CellState::Initialized(output.ty));
        seed_str_storage_layout(engine.types, cells, raw_aliases, output);
        initialize_control_output_path_states(engine.types, &mut engine.path_alternatives, output);
    } else {
        invalidate_control_output_state(
            cells,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
            output,
        );
        invalidate_control_output_path_states(&mut engine.path_alternatives, output);
    }
}

fn collect_param_release_requirements_from_op(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    engine: &ResourceCheckEngine<'_>,
    cells: &CellTable,
    collection_slots: &CollectionSlotStateTable,
    raw_aliases: &RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    pending_reallocs: &PendingRawReallocs,
    variant_initializations: &PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    op: &ResourceOp,
) {
    match op {
        ResourceOp::RawMemory {
            operation, args, ..
        } => collect_raw_memory_release_requirements(
            out,
            engine.types,
            operation,
            args,
            raw_aliases,
            params,
        ),
        ResourceOp::Call { target, args, .. } => collect_call_release_requirements(
            out,
            engine,
            target,
            args,
            raw_aliases,
            params,
            raw_init_summaries,
        ),
        ResourceOp::IndirectCall {
            callee,
            params: call_params,
            args,
            effect,
            ..
        } => {
            let functions = function_aliases.function_symbols(callee);
            for function in &functions {
                collect_function_summary_release_requirements(
                    out,
                    engine,
                    args,
                    raw_aliases,
                    params,
                    raw_init_summaries.get(*function),
                );
            }
            if functions.is_empty() && indirect_call_may_release_raw_cells(effect) {
                collect_unknown_indirect_call_release_requirements(
                    out,
                    engine.types,
                    call_params,
                    args,
                    raw_aliases,
                    params,
                );
            }
        }
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            collect_nested_release_requirements(
                out,
                engine,
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
                params,
                raw_init_summaries,
                then_ops,
            );
            collect_nested_release_requirements(
                out,
                engine,
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
                params,
                raw_init_summaries,
                else_ops,
            );
        }
        ResourceOp::Loop {
            condition_ops,
            body_ops,
            ..
        } => {
            collect_nested_release_requirements(
                out,
                engine,
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
                params,
                raw_init_summaries,
                condition_ops,
            );
            collect_nested_release_requirements(
                out,
                engine,
                cells,
                collection_slots,
                raw_aliases,
                function_aliases,
                pending_reallocs,
                variant_initializations,
                params,
                raw_init_summaries,
                body_ops,
            );
        }
        ResourceOp::Match {
            scrutinee, arms, ..
        } => {
            for arm in arms {
                collect_match_arm_release_requirements(
                    out,
                    engine,
                    cells,
                    collection_slots,
                    raw_aliases,
                    function_aliases,
                    pending_reallocs,
                    variant_initializations,
                    params,
                    raw_init_summaries,
                    scrutinee,
                    arm,
                );
            }
        }
        ResourceOp::DeclareLocal { .. }
        | ResourceOp::Read { .. }
        | ResourceOp::Assign { .. }
        | ResourceOp::Borrow { .. }
        | ResourceOp::Move { .. }
        | ResourceOp::Drop { .. }
        | ResourceOp::CallEffect { .. }
        | ResourceOp::FunctionValue { .. }
        | ResourceOp::RawAddressAlias { .. }
        | ResourceOp::RawAddressView { .. }
        | ResourceOp::StorageOrigin { .. }
        | ResourceOp::CollectionSlotLifecycle { .. }
        | ResourceOp::CollectionStorageRelocate { .. }
        | ResourceOp::CollectionSlotDropTraversal { .. }
        | ResourceOp::CollectionSlotTransformRange { .. }
        | ResourceOp::Construct { .. }
        | ResourceOp::Expr { .. }
        | ResourceOp::EndScope { .. } => {}
    }
}

fn collect_match_arm_release_requirements(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    engine: &ResourceCheckEngine<'_>,
    cells: &CellTable,
    collection_slots: &CollectionSlotStateTable,
    raw_aliases: &RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    pending_reallocs: &PendingRawReallocs,
    variant_initializations: &PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    scrutinee: &Place,
    arm: &super::model::ResourceMatchArm,
) {
    let mut arm_cells = cells.clone();
    let mut arm_collection_slots = collection_slots.clone();
    let mut arm_aliases = raw_aliases.clone();
    let mut arm_function_aliases = function_aliases.clone();
    let mut arm_pending_reallocs = pending_reallocs.clone();
    let mut arm_variant_initializations = variant_initializations.clone();
    let mut arm_engine = summary_check_engine(engine);
    if let Some(bind_local) = &arm.bind_local {
        arm_cells.mark_initialized(bind_local);
        if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
            arm_engine.copy_raw_alias_and_rekey_cells(
                &mut arm_cells,
                &mut arm_aliases,
                &source,
                bind_local,
            );
            transfer_slots(
                &mut arm_engine,
                &mut arm_collection_slots,
                &source,
                bind_local,
                arm.span,
            );
            arm_function_aliases.copy_alias(&source, bind_local);
            arm_pending_reallocs.copy_result(&source, bind_local);
            arm_variant_initializations.copy_result(&source, bind_local);
        }
    }
    arm_variant_initializations.apply_match_arm(
        &mut arm_engine,
        &mut arm_cells,
        &mut arm_aliases,
        scrutinee,
        &arm.pattern,
        arm.span,
    );
    collect_param_release_requirements_from_ops(
        out,
        &arm_engine,
        &mut arm_cells,
        &mut arm_collection_slots,
        &mut arm_aliases,
        &mut arm_function_aliases,
        &mut arm_pending_reallocs,
        &mut arm_variant_initializations,
        params,
        raw_init_summaries,
        &arm.ops,
    );
}

fn collect_nested_release_requirements(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    engine: &ResourceCheckEngine<'_>,
    cells: &CellTable,
    collection_slots: &CollectionSlotStateTable,
    raw_aliases: &RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    pending_reallocs: &PendingRawReallocs,
    variant_initializations: &PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
) {
    let mut path_cells = cells.clone();
    let mut path_collection_slots = collection_slots.clone();
    let mut path_aliases = raw_aliases.clone();
    let mut path_function_aliases = function_aliases.clone();
    let mut path_pending_reallocs = pending_reallocs.clone();
    let mut path_variant_initializations = variant_initializations.clone();
    collect_param_release_requirements_from_ops(
        out,
        engine,
        &mut path_cells,
        &mut path_collection_slots,
        &mut path_aliases,
        &mut path_function_aliases,
        &mut path_pending_reallocs,
        &mut path_variant_initializations,
        params,
        raw_init_summaries,
        ops,
    );
}

fn collect_call_release_requirements(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    engine: &ResourceCheckEngine<'_>,
    target: &ResourceCallTarget,
    args: &[Place],
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
) {
    let ResourceCallTarget::User { name, .. } = target else {
        return;
    };
    collect_function_summary_release_requirements(
        out,
        engine,
        args,
        raw_aliases,
        params,
        raw_init_summaries.get(name),
    );
}

fn collect_function_summary_release_requirements(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    summary: Option<&RawCellInitializationFunctionSummary>,
) {
    let Some(summary) = summary else {
        return;
    };
    let param_alias_index = RawCellReleaseParamAliasIndex::new(engine.types, raw_aliases, params);
    for requirement in &summary.param_release_requirements {
        let Some(arg) = args.get(requirement.param_index) else {
            continue;
        };
        let address = projected_place_with_concrete_type(
            engine.types,
            arg,
            &requirement.suffix,
            requirement.ty,
        );
        collect_address_release_requirements_with_param_alias_index(
            out,
            &address,
            requirement.kind,
            raw_aliases,
            &param_alias_index,
        );
    }
}

pub(super) fn collect_address_release_requirements(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    types: &crate::types::TypeCtx,
    address: &Place,
    kind: RawCellReleaseRequirementKind,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
) {
    let param_alias_index = RawCellReleaseParamAliasIndex::new(types, raw_aliases, params);
    collect_address_release_requirements_with_param_alias_index(
        out,
        address,
        kind,
        raw_aliases,
        &param_alias_index,
    );
}

fn collect_address_release_requirements_with_param_alias_index(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    address: &Place,
    kind: RawCellReleaseRequirementKind,
    raw_aliases: &RawCellAddressAliases,
    param_alias_index: &RawCellReleaseParamAliasIndex,
) {
    let address = raw_aliases.canonicalize(address);
    for address_alias in raw_aliases.aliases_for(&address) {
        for param_alias in &param_alias_index.entries {
            let Some(suffix) = raw_address_suffix_after_address(&address_alias, &param_alias.alias)
            else {
                continue;
            };
            push_unique_param_release_requirement(
                out,
                RawCellReleaseParamRequirement {
                    param_index: param_alias.param_index,
                    suffix,
                    ty: address_alias.ty,
                    kind,
                },
            );
        }
    }
}

struct RawCellReleaseParamAliasIndex {
    entries: Vec<RawCellReleaseParamAliasEntry>,
}

struct RawCellReleaseParamAliasEntry {
    param_index: usize,
    alias: Place,
}

impl RawCellReleaseParamAliasIndex {
    fn new(
        types: &crate::types::TypeCtx,
        raw_aliases: &RawCellAddressAliases,
        params: &[ResourceLocal],
    ) -> Self {
        let mut entries = Vec::new();
        for (param_index, param) in params.iter().enumerate() {
            if !summary_input_type_may_seed_raw_address_alias(types, param.place.ty) {
                continue;
            }
            for alias in raw_aliases.aliases_for(&param.place) {
                entries.push(RawCellReleaseParamAliasEntry { param_index, alias });
            }
        }
        Self { entries }
    }
}

fn push_unique_param_release_requirement(
    requirements: &mut Vec<RawCellReleaseParamRequirement>,
    requirement: RawCellReleaseParamRequirement,
) {
    if !requirements.iter().any(|existing| existing == &requirement) {
        requirements.push(requirement);
    }
}
