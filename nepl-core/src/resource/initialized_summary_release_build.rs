extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::{raw_address_suffix_after_address, CellTable};
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::drop_point_path::ResourceDropPointPath;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
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
use super::model::{Place, ResourceBlockId, ResourceCallTarget, ResourceLocal, ResourceOp};
use super::place_utils::{match_bind_payload_place, projected_place_with_concrete_type};
use super::raw_realloc::PendingRawReallocs;

pub(super) fn collect_param_release_requirements_from_ops(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    engine: &ResourceCheckEngine<'_>,
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    pending_reallocs: &mut PendingRawReallocs,
    variant_initializations: &mut PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
) {
    let mut step_engine = summary_check_engine(engine);
    let mut collection_slots = CollectionSlotStateTable::new();
    for op in ops {
        collect_param_release_requirements_from_op(
            out,
            &step_engine,
            cells,
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
            &mut collection_slots,
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
        step_engine.auto_drop_points.clear();
    }
}

fn collect_param_release_requirements_from_op(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    engine: &ResourceCheckEngine<'_>,
    cells: &CellTable,
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
            let functions = function_aliases.functions(callee);
            for function in functions {
                collect_function_summary_release_requirements(
                    out,
                    engine,
                    args,
                    raw_aliases,
                    params,
                    raw_init_summaries.get(function),
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
        | ResourceOp::Construct { .. }
        | ResourceOp::Expr { .. }
        | ResourceOp::EndScope { .. } => {}
    }
}

fn collect_match_arm_release_requirements(
    out: &mut Vec<RawCellReleaseParamRequirement>,
    engine: &ResourceCheckEngine<'_>,
    cells: &CellTable,
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
    raw_aliases: &RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    pending_reallocs: &PendingRawReallocs,
    variant_initializations: &PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    ops: &[ResourceOp],
) {
    let mut path_cells = cells.clone();
    let mut path_aliases = raw_aliases.clone();
    let mut path_function_aliases = function_aliases.clone();
    let mut path_pending_reallocs = pending_reallocs.clone();
    let mut path_variant_initializations = variant_initializations.clone();
    collect_param_release_requirements_from_ops(
        out,
        engine,
        &mut path_cells,
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
        collect_address_release_requirements(
            out,
            engine.types,
            &address,
            requirement.kind,
            raw_aliases,
            params,
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
    let address = raw_aliases.canonicalize(address);
    for address_alias in raw_aliases.aliases_for(&address) {
        for (param_index, param) in params.iter().enumerate() {
            if !summary_input_type_may_seed_raw_address_alias(types, param.place.ty) {
                continue;
            }
            for param_alias in raw_aliases.aliases_for(&param.place) {
                let Some(suffix) = raw_address_suffix_after_address(&address_alias, &param_alias)
                else {
                    continue;
                };
                push_unique_param_release_requirement(
                    out,
                    RawCellReleaseParamRequirement {
                        param_index,
                        suffix,
                        ty: address_alias.ty,
                        kind,
                    },
                );
            }
        }
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
