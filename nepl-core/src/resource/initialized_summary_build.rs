extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::cell_state::{
    raw_address_suffix_after_address, raw_cell_suffix_after_address, CellTable,
};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummary;
use super::initialized_summary::{
    RawCellDestructionParamAddress, RawCellInitializationFunctionSummary,
    RawCellInitializationParamCell, RawCellInitializationReturnCell,
};
use super::initialized_summary_variant_build::collect_variant_param_initialized_raw_cells_from_return;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{
    CellState, Place, RawMemoryOp, ResourceCallTarget, ResourceFunction, ResourceLocal,
    ResourceModule, ResourceOp, ResourceTerminator,
};
use super::place_utils::{match_bind_payload_place, place_with_suffix};
use super::raw_realloc::PendingRawReallocs;
use super::report::{ResourceCheckDeferred, ResourceCheckOperation};

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
                || !summary.variant_conditions.is_empty()
                || !summary.param_destructions.is_empty()
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
        variant_conditions: Vec::new(),
        param_destructions: Vec::new(),
    };
    let mut guaranteed_return_cells = None;
    let mut guaranteed_param_cells = None;
    for block in &function.blocks {
        let mut variant_initializations = PendingVariantRawCellInitializations::default();
        check_ops_and_collect_param_destructions(
            &mut out.param_destructions,
            &mut engine,
            &mut cells,
            &mut raw_aliases,
            &mut function_aliases,
            &mut pending_reallocs,
            &mut variant_initializations,
            &function.params,
            raw_init_summaries,
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
                &mut out.variant_conditions,
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

fn check_ops_and_collect_param_destructions(
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

fn collect_param_destructions_from_raw_memory(
    out: &mut Vec<RawCellDestructionParamAddress>,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    operation: &RawMemoryOp,
    args: &[Place],
) {
    match operation {
        RawMemoryOp::Store => collect_param_destructions_for_arg(
            out,
            raw_aliases,
            params,
            args,
            0,
            ResourceCheckOperation::RawMemoryStoreCell,
        ),
        RawMemoryOp::Dealloc => collect_param_destructions_for_arg(
            out,
            raw_aliases,
            params,
            args,
            0,
            ResourceCheckOperation::RawMemoryDeallocCell,
        ),
        RawMemoryOp::Realloc => collect_param_destructions_for_arg(
            out,
            raw_aliases,
            params,
            args,
            0,
            ResourceCheckOperation::RawMemoryReallocCell,
        ),
        RawMemoryOp::Fill => collect_param_destructions_for_arg(
            out,
            raw_aliases,
            params,
            args,
            0,
            ResourceCheckOperation::RawMemoryFillCell,
        ),
        RawMemoryOp::BulkCopy | RawMemoryOp::BulkMove => {
            collect_param_destructions_for_arg(
                out,
                raw_aliases,
                params,
                args,
                0,
                ResourceCheckOperation::RawMemoryBulkDestinationCell,
            );
            collect_param_destructions_for_arg(
                out,
                raw_aliases,
                params,
                args,
                1,
                ResourceCheckOperation::RawMemoryBulkSourceCell,
            );
        }
        RawMemoryOp::Alloc
        | RawMemoryOp::Load
        | RawMemoryOp::MemorySize
        | RawMemoryOp::MemoryGrow
        | RawMemoryOp::Other { .. } => {}
    }
}

fn collect_param_destructions_for_arg(
    out: &mut Vec<RawCellDestructionParamAddress>,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    args: &[Place],
    arg_index: usize,
    operation: ResourceCheckOperation,
) {
    let Some(address) = args.get(arg_index) else {
        return;
    };
    collect_param_destructions_for_address(out, raw_aliases, params, address, operation);
}

fn collect_param_destructions_from_direct_call(
    out: &mut Vec<RawCellDestructionParamAddress>,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    target: &ResourceCallTarget,
    args: &[Place],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
) {
    let ResourceCallTarget::User { name, .. } = target else {
        return;
    };
    let Some(summary) = raw_init_summaries
        .iter()
        .find(|summary| summary.function == name.as_str())
    else {
        return;
    };
    collect_param_destructions_from_summary(out, raw_aliases, params, args, summary);
}

fn collect_param_destructions_from_summary(
    out: &mut Vec<RawCellDestructionParamAddress>,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    args: &[Place],
    summary: &RawCellInitializationFunctionSummary,
) {
    for destruction in &summary.param_destructions {
        let Some(arg) = args.get(destruction.param_index) else {
            continue;
        };
        let address = place_with_suffix(arg, &destruction.suffix, destruction.ty);
        let address = raw_aliases.canonicalize(&address);
        collect_param_destructions_for_address(
            out,
            raw_aliases,
            params,
            &address,
            destruction.operation,
        );
    }
}

fn collect_param_destructions_for_address(
    out: &mut Vec<RawCellDestructionParamAddress>,
    raw_aliases: &RawCellAddressAliases,
    params: &[ResourceLocal],
    address: &Place,
    operation: ResourceCheckOperation,
) {
    let address = raw_aliases.canonicalize(address);
    for address_alias in raw_aliases.aliases_for(&address) {
        for (param_index, param) in params.iter().enumerate() {
            for param_alias in raw_aliases.aliases_for(&param.place) {
                let Some(suffix) = raw_address_suffix_after_address(&address_alias, &param_alias)
                else {
                    continue;
                };
                push_unique_param_destruction(
                    out,
                    RawCellDestructionParamAddress {
                        param_index,
                        suffix,
                        ty: address_alias.ty,
                        operation,
                    },
                );
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

fn push_unique_param_destruction(
    cells: &mut Vec<RawCellDestructionParamAddress>,
    cell: RawCellDestructionParamAddress,
) {
    if !cells.iter().any(|existing| existing == &cell) {
        cells.push(cell);
    }
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
