extern crate alloc;

use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummary;
use super::initialized_summary::RawCellInitializationFunctionSummary;
use super::initialized_summary_cells::{
    collect_param_initialized_raw_cells, collect_return_initialized_raw_cells,
};
use super::initialized_summary_release_build::collect_param_release_requirements_from_ops;
use super::initialized_summary_variant_build::collect_variant_param_initialized_raw_cells_from_return;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{ResourceFunction, ResourceModule, ResourceTerminator};
use super::place_utils::reference_target_place;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDeferred;
use super::summary_worklist::SummaryWorklist;

pub(super) fn compute_raw_cell_initialization_function_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
) -> Vec<RawCellInitializationFunctionSummary> {
    let mut worklist = SummaryWorklist::new(module);
    let mut summaries = Vec::new();
    while let Some(function_index) = worklist.pop() {
        let function = &module.functions[function_index];
        let summary = function_raw_cell_initialization_summary(
            function,
            types,
            raw_alias_summaries,
            &summaries,
        );
        if update_raw_cell_initialization_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_raw_init_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
    }
    summaries
}

fn update_raw_cell_initialization_summary(
    summaries: &mut Vec<RawCellInitializationFunctionSummary>,
    summary: RawCellInitializationFunctionSummary,
) -> bool {
    let has_facts = !summary.return_cells.is_empty()
        || !summary.param_cells.is_empty()
        || !summary.param_release_requirements.is_empty()
        || !summary.variant_param_cells.is_empty()
        || !summary.variant_required_param_cells.is_empty()
        || !summary.variant_conditions.is_empty();
    let position = summaries
        .iter()
        .position(|existing| existing.function == summary.function);
    match (has_facts, position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
        }
        (true, None) => {
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
            true
        }
        (false, None) => false,
    }
}

fn function_raw_cell_initialization_summary(
    function: &ResourceFunction,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    raw_init_summaries: &[RawCellInitializationFunctionSummary],
) -> RawCellInitializationFunctionSummary {
    let engine = ResourceCheckEngine {
        function: function.name.as_str(),
        types,
        raw_alias_summaries,
        raw_init_summaries,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
    };
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut function_aliases = FunctionAliasTable::default();
    let mut pending_reallocs = PendingRawReallocs::default();
    for param in &function.params {
        cells.mark_initialized(&param.place);
        raw_aliases.mark(&param.place);
        if let Some(target_ty) = reference_target_type(types, param.place.ty) {
            let target = reference_target_place(&param.place, target_ty);
            cells.mark_initialized(&target);
            raw_aliases.mark(&target);
        }
    }

    let mut out = RawCellInitializationFunctionSummary {
        function: function.name.clone(),
        return_cells: Vec::new(),
        param_cells: Vec::new(),
        param_release_requirements: Vec::new(),
        variant_param_cells: Vec::new(),
        variant_required_param_cells: Vec::new(),
        variant_conditions: Vec::new(),
    };
    let mut guaranteed_return_cells = None;
    let mut guaranteed_param_cells = None;
    let mut variant_initializations = PendingVariantRawCellInitializations::default();
    for block in &function.blocks {
        collect_param_release_requirements_from_ops(
            &mut out.param_release_requirements,
            &engine,
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

fn reference_target_type(types: &TypeCtx, ty: TypeId) -> Option<TypeId> {
    let resolved = types.resolve_named_type_id(types.resolve_id(ty));
    match types.get_ref(resolved) {
        TypeKind::Reference(target, _) => Some(*target),
        _ => None,
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
