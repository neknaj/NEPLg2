extern crate alloc;

use alloc::vec::Vec;

use super::cell_state::CellTable;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_summary::{
    RawCellDestructionParamAddress, RawCellInitializationFunctionSummary,
};
use super::initialized_summary_destruction_address::{
    collect_param_destructions_from_direct_call, collect_param_destructions_from_raw_memory,
    collect_param_destructions_from_summary,
};
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{ResourceLocal, ResourceOp};
use super::place_utils::match_bind_payload_place;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDeferred;

pub(super) fn check_ops_and_collect_param_destructions(
    out: &mut Vec<RawCellDestructionParamAddress>,
    engine: &mut ResourceCheckEngine<'_>,
    cells: &mut CellTable,
    raw_aliases: &mut RawCellAddressAliases,
    function_aliases: &mut FunctionAliasTable,
    pending_reallocs: &mut PendingRawReallocs,
    variant_initializations: &mut PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
    ops: &[ResourceOp],
) {
    for op in ops {
        collect_param_destructions_from_op(
            out,
            engine,
            cells,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
            params,
            raw_init_summaries,
            op,
        );
        engine.check_ops(
            cells,
            raw_aliases,
            function_aliases,
            pending_reallocs,
            variant_initializations,
            core::slice::from_ref(op),
        );
    }
}

fn collect_param_destructions_from_op(
    out: &mut Vec<RawCellDestructionParamAddress>,
    engine: &ResourceCheckEngine<'_>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    pending_reallocs: &PendingRawReallocs,
    variant_initializations: &PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
    op: &ResourceOp,
) {
    match op {
        ResourceOp::RawMemory {
            operation, args, ..
        } => collect_param_destructions_from_raw_memory(out, raw_aliases, params, operation, args),
        ResourceOp::Call { target, args, .. } => {
            collect_param_destructions_from_direct_call(
                out,
                raw_aliases,
                params,
                target,
                args,
                raw_init_summaries,
            );
        }
        ResourceOp::IndirectCall { callee, args, .. } => {
            for function in function_aliases.functions(callee) {
                if let Some(summary) = raw_init_summaries
                    .iter()
                    .find(|summary| summary.function == function.as_str())
                {
                    collect_param_destructions_from_summary(
                        out,
                        raw_aliases,
                        params,
                        args,
                        summary,
                    );
                }
            }
        }
        ResourceOp::Branch {
            then_ops, else_ops, ..
        } => {
            collect_param_destructions_from_path(
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
            collect_param_destructions_from_path(
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
            let mut path_engine = clone_summary_engine(engine);
            let mut path_cells = cells.clone();
            let mut path_aliases = raw_aliases.clone();
            let mut path_function_aliases = function_aliases.clone();
            let mut path_pending_reallocs = pending_reallocs.clone();
            let mut path_variant_initializations = variant_initializations.clone();
            check_ops_and_collect_param_destructions(
                out,
                &mut path_engine,
                &mut path_cells,
                &mut path_aliases,
                &mut path_function_aliases,
                &mut path_pending_reallocs,
                &mut path_variant_initializations,
                params,
                raw_init_summaries,
                condition_ops,
            );
            check_ops_and_collect_param_destructions(
                out,
                &mut path_engine,
                &mut path_cells,
                &mut path_aliases,
                &mut path_function_aliases,
                &mut path_pending_reallocs,
                &mut path_variant_initializations,
                params,
                raw_init_summaries,
                body_ops,
            );
        }
        ResourceOp::Match {
            scrutinee, arms, ..
        } => {
            for arm in arms {
                let mut path_engine = clone_summary_engine(engine);
                let mut path_cells = cells.clone();
                let mut path_aliases = raw_aliases.clone();
                let mut path_function_aliases = function_aliases.clone();
                let mut path_pending_reallocs = pending_reallocs.clone();
                let mut path_variant_initializations = variant_initializations.clone();
                if !path_variant_initializations.match_arm_reachable(scrutinee, &arm.pattern) {
                    continue;
                }
                if let Some(bind_local) = &arm.bind_local {
                    path_cells.mark_initialized(bind_local);
                    if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
                        path_engine.copy_raw_alias_and_rekey_cells(
                            &mut path_cells,
                            &mut path_aliases,
                            &source,
                            bind_local,
                        );
                        path_function_aliases.copy_alias(&source, bind_local);
                        path_pending_reallocs.copy_result(&source, bind_local);
                        path_variant_initializations.copy_result(&source, bind_local);
                    } else {
                        path_aliases.clear(bind_local);
                        path_pending_reallocs.clear_result(bind_local);
                        path_variant_initializations.clear_result(bind_local);
                    }
                }
                path_variant_initializations.apply_match_arm(
                    &mut path_engine,
                    &mut path_cells,
                    &mut path_aliases,
                    scrutinee,
                    &arm.pattern,
                    arm.span,
                );
                check_ops_and_collect_param_destructions(
                    out,
                    &mut path_engine,
                    &mut path_cells,
                    &mut path_aliases,
                    &mut path_function_aliases,
                    &mut path_pending_reallocs,
                    &mut path_variant_initializations,
                    params,
                    raw_init_summaries,
                    &arm.ops,
                );
            }
        }
        _ => {}
    }
}

fn collect_param_destructions_from_path(
    out: &mut Vec<RawCellDestructionParamAddress>,
    engine: &ResourceCheckEngine<'_>,
    cells: &CellTable,
    raw_aliases: &RawCellAddressAliases,
    function_aliases: &FunctionAliasTable,
    pending_reallocs: &PendingRawReallocs,
    variant_initializations: &PendingVariantRawCellInitializations,
    params: &[ResourceLocal],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
    ops: &[ResourceOp],
) {
    let mut path_engine = clone_summary_engine(engine);
    let mut path_cells = cells.clone();
    let mut path_aliases = raw_aliases.clone();
    let mut path_function_aliases = function_aliases.clone();
    let mut path_pending_reallocs = pending_reallocs.clone();
    let mut path_variant_initializations = variant_initializations.clone();
    check_ops_and_collect_param_destructions(
        out,
        &mut path_engine,
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

fn clone_summary_engine<'a>(engine: &ResourceCheckEngine<'a>) -> ResourceCheckEngine<'a> {
    ResourceCheckEngine {
        function: engine.function,
        types: engine.types,
        raw_alias_summaries: engine.raw_alias_summaries,
        raw_init_summaries: engine.raw_init_summaries,
        diagnostics: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
    }
}
