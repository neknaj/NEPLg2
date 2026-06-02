extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::cell_state::CellTable;
use super::collection_slot_summary_model::CollectionSlotLifecycleFunctionSummaryIndex;
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::{
    RawCellAddressReturnSummary, RawCellAddressReturnSummaryIndex,
};
use super::initialized_scalar_flow::{I32ScalarReturnSummary, I32ScalarReturnSummaryIndex};
use super::initialized_summary::{
    RawCellInitializationFunctionSummary, RawCellInitializationFunctionSummaryIndex,
};
use super::initialized_summary_build_relevance::{
    raw_cell_initialization_summary_relevance_with_graph, reference_target_type,
};
use super::initialized_summary_build_update::update_raw_cell_initialization_summary;
use super::initialized_summary_build_value_cache::{
    preseed_raw_cell_initialization_summaries_from_value_cache,
    record_raw_cell_initialization_summary_value_cache_candidates,
};
use super::initialized_summary_cells::collect_return_initialized_raw_cells;
use super::initialized_summary_param_byte_ranges::collect_param_initialized_raw_byte_ranges;
use super::initialized_summary_param_cells::collect_param_initialized_raw_cells;
use super::initialized_summary_release_build::collect_param_release_requirements_from_ops;
use super::initialized_summary_return_byte_ranges::collect_return_initialized_raw_byte_ranges;
use super::initialized_summary_seed::seed_summary_input_place;
use super::initialized_summary_variant_build::collect_variant_param_initialized_raw_cells_from_return;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{Place, ResourceFunction, ResourceModule, ResourceOp, ResourceTerminator};
use super::owner_summary_type_params::owner_summary_type_params;
use super::place_utils::reference_target_place;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDeferred;
use super::resource_summary_value_cache::{
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};
use super::summary_dependency::ResourceSummaryDependencyGraph;
use super::summary_index::SummaryNameIndex;
use super::summary_worklist::SummaryWorklist;
use super::timing::ResourceFunctionTimer;

#[cfg(test)]
pub(super) fn compute_raw_cell_initialization_function_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    i32_scalar_summaries: &[I32ScalarReturnSummary],
    summary_value_cache: Option<&mut ResourceSummaryValueCache>,
    summary_value_cache_context: Option<&ResourceSummaryValueCacheContext>,
) -> Vec<RawCellInitializationFunctionSummary> {
    let dependency_graph = ResourceSummaryDependencyGraph::build(module);
    compute_raw_cell_initialization_function_summaries_with_recomputations(
        module,
        types,
        raw_alias_summaries,
        i32_scalar_summaries,
        &dependency_graph,
        summary_value_cache,
        summary_value_cache_context,
    )
    .0
}

pub(super) fn compute_raw_cell_initialization_function_summaries_with_recomputations(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    i32_scalar_summaries: &[I32ScalarReturnSummary],
    dependency_graph: &ResourceSummaryDependencyGraph,
    mut summary_value_cache: Option<&mut ResourceSummaryValueCache>,
    summary_value_cache_context: Option<&ResourceSummaryValueCacheContext>,
) -> (Vec<RawCellInitializationFunctionSummary>, usize) {
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(raw_alias_summaries);
    let relevant = raw_cell_initialization_summary_relevance_with_graph(
        module,
        types,
        &raw_alias_summary_index,
        dependency_graph,
    );
    let mut worklist_relevant_functions = relevant.clone();
    let mut preseeded_functions = vec![false; module.functions.len()];
    let mut summaries = Vec::new();
    let mut replay_plan = match (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        (Some(cache), Some(context)) => Some(cache.begin_raw_init_summary_replay_plan(
            context,
            types,
            module,
            dependency_graph,
            &relevant,
        )),
        _ => None,
    };
    if let (Some(cache), Some(context)) = (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        preseed_raw_cell_initialization_summaries_from_value_cache(
            cache,
            context,
            types,
            module,
            &relevant,
            dependency_graph.dependencies(),
            &mut worklist_relevant_functions,
            &mut preseeded_functions,
            &mut summaries,
            replay_plan.as_mut(),
        );
    }
    let mut worklist = SummaryWorklist::new_filtered_with_dependency_graph(
        module,
        worklist_relevant_functions,
        dependency_graph,
    );
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(i32_scalar_summaries);
    let empty_collection_slot_summaries = CollectionSlotLifecycleFunctionSummaryIndex::new(&[]);
    let mut summary_name_index = SummaryNameIndex::from_entries(&summaries);
    while let Some(function_index) = worklist.pop() {
        let function = &module.functions[function_index];
        let function_start = ResourceFunctionTimer::start();
        let summary = {
            let raw_init_summary_index = summary_name_index.as_summary_index(&summaries);
            function_raw_cell_initialization_summary(
                function,
                types,
                &raw_alias_summary_index,
                &i32_scalar_summary_index,
                &raw_init_summary_index,
                &empty_collection_slot_summaries,
            )
        };
        function_start.log("raw_init_summary", function);
        if update_raw_cell_initialization_summary(&mut summaries, &mut summary_name_index, summary)
        {
            worklist.notify_changed(function_index);
        }
    }
    if let (Some(cache), Some(context)) = (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        let candidate_skipped_functions = worklist.unrecomputed_initial_skips(&preseeded_functions);
        record_raw_cell_initialization_summary_value_cache_candidates(
            cache,
            context,
            types,
            module,
            dependency_graph.dependencies(),
            &relevant,
            &candidate_skipped_functions,
            &summaries,
            replay_plan.as_mut(),
        );
    }
    if let (Some(cache), Some(plan)) = (summary_value_cache.as_deref_mut(), replay_plan) {
        cache.finish_raw_init_summary_replay_plan(plan);
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_raw_init_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
    }
    let recomputations = worklist.recomputations();
    (summaries, recomputations)
}

fn function_raw_cell_initialization_summary(
    function: &ResourceFunction,
    types: &TypeCtx,
    raw_alias_summaries: &RawCellAddressReturnSummaryIndex<'_>,
    i32_scalar_summaries: &I32ScalarReturnSummaryIndex<'_>,
    raw_init_summaries: &RawCellInitializationFunctionSummaryIndex<'_>,
    empty_collection_slot_summaries: &CollectionSlotLifecycleFunctionSummaryIndex<'_>,
) -> RawCellInitializationFunctionSummary {
    let engine = ResourceCheckEngine {
        function: function.name.as_str(),
        types,
        raw_alias_summaries,
        i32_scalar_summaries,
        raw_init_summaries,
        collection_slot_summaries: empty_collection_slot_summaries,
        transform_range_certificates: None,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
    };
    let mut cells = CellTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut function_aliases = FunctionAliasTable::default();
    let mut pending_reallocs = PendingRawReallocs::default();
    for param in &function.params {
        seed_summary_input_place(types, &mut cells, &mut raw_aliases, &param.place);
        if let Some(target_ty) = reference_target_type(types, param.place.ty) {
            let target = reference_target_place(&param.place, target_ty);
            seed_summary_input_place(types, &mut cells, &mut raw_aliases, &target);
        }
    }

    let mut out = RawCellInitializationFunctionSummary {
        function: function.name.clone(),
        type_params: owner_summary_type_params(types, function),
        return_cells: Vec::new(),
        return_byte_ranges: Vec::new(),
        param_cells: Vec::new(),
        param_byte_ranges: Vec::new(),
        param_release_requirements: Vec::new(),
        variant_param_cells: Vec::new(),
        variant_param_byte_ranges: Vec::new(),
        variant_required_param_cells: Vec::new(),
        variant_conditions: Vec::new(),
    };
    let mut guaranteed_return_cells = None;
    let mut guaranteed_return_byte_ranges = None;
    let mut guaranteed_param_cells = None;
    let mut guaranteed_param_byte_ranges = None;
    let mut variant_initializations = PendingVariantRawCellInitializations::default();
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    let raw_init_timing = RawInitSummaryTiming::from_env(function);
    for block in &function.blocks {
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        let release_start = raw_init_timing.start("release_requirements");
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
        #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
        raw_init_timing.finish("release_requirements", release_start);
        if let ResourceTerminator::Return { value, .. } = &block.terminator {
            let mut path_return_cells = Vec::new();
            if let Some(value) = value {
                #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
                let return_cells_start = raw_init_timing.start("return_cells");
                collect_return_initialized_raw_cells(
                    &mut path_return_cells,
                    &cells,
                    &raw_aliases,
                    value,
                );
                #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
                raw_init_timing.finish("return_cells", return_cells_start);
            }
            merge_guaranteed_facts(&mut guaranteed_return_cells, path_return_cells);

            let mut path_return_byte_ranges = Vec::new();
            if let Some(value) = value {
                #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
                let return_byte_ranges_start = raw_init_timing.start("return_byte_ranges");
                collect_return_initialized_raw_byte_ranges(
                    &mut path_return_byte_ranges,
                    &cells,
                    &raw_aliases,
                    value,
                );
                #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
                raw_init_timing.finish("return_byte_ranges", return_byte_ranges_start);
            }
            merge_guaranteed_facts(&mut guaranteed_return_byte_ranges, path_return_byte_ranges);

            let mut path_param_cells = Vec::new();
            #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
            let param_cells_start = raw_init_timing.start("param_cells");
            collect_param_initialized_raw_cells(
                &mut path_param_cells,
                &cells,
                &raw_aliases,
                &function.params,
            );
            #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
            raw_init_timing.finish("param_cells", param_cells_start);
            merge_guaranteed_facts(&mut guaranteed_param_cells, path_param_cells);

            let mut path_param_byte_ranges = Vec::new();
            #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
            let param_byte_ranges_start = raw_init_timing.start("param_byte_ranges");
            collect_param_initialized_raw_byte_ranges(
                &mut path_param_byte_ranges,
                &cells,
                &raw_aliases,
                &function.params,
            );
            #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
            raw_init_timing.finish("param_byte_ranges", param_byte_ranges_start);
            merge_guaranteed_facts(&mut guaranteed_param_byte_ranges, path_param_byte_ranges);
        }
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            if ops_have_top_level_branch_output_for_return(&block.ops, value) {
                #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
                let variant_start = raw_init_timing.start("variant_param_cells");
                collect_variant_param_initialized_raw_cells_from_return(
                    &mut out.variant_param_cells,
                    &mut out.variant_param_byte_ranges,
                    &mut out.variant_required_param_cells,
                    &mut out.variant_conditions,
                    function,
                    types,
                    raw_alias_summaries,
                    i32_scalar_summaries,
                    raw_init_summaries,
                    &block.ops,
                    value,
                );
                #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
                raw_init_timing.finish("variant_param_cells", variant_start);
            }
        }
    }
    out.return_cells = guaranteed_return_cells.unwrap_or_default();
    out.return_byte_ranges = guaranteed_return_byte_ranges.unwrap_or_default();
    out.param_cells = guaranteed_param_cells.unwrap_or_default();
    out.param_byte_ranges = guaranteed_param_byte_ranges.unwrap_or_default();
    out
}

fn ops_have_top_level_branch_output_for_return(ops: &[ResourceOp], return_value: &Place) -> bool {
    // variant-param summary の collector は、現時点では return value そのものを
    // output とする top-level Branch だけを facts の抽出対象にしている。
    // その Branch がない block では collector を起動しても block prefix の再生だけが走るため、
    // 観測境界を保ったままここで探索対象から外す。
    ops.iter()
        .any(|op| matches!(op, ResourceOp::Branch { output, .. } if output == return_value))
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

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
struct RawInitSummaryTiming<'a> {
    function_name: &'a str,
    enabled: bool,
}

#[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
impl<'a> RawInitSummaryTiming<'a> {
    fn from_env(function: &'a ResourceFunction) -> Self {
        let enabled = std::env::var_os("NEPL_RESOURCE_RAW_INIT_SUMMARY_TIMING").is_some()
            && std::env::var("NEPL_RESOURCE_OP_TIMING_FUNCTION")
                .map(|filter| function.name.contains(&filter))
                .unwrap_or(true);
        Self {
            function_name: function.name.as_str(),
            enabled,
        }
    }

    fn start(&self, stage: &'static str) -> Option<std::time::Instant> {
        if !self.enabled {
            return None;
        }
        std::eprintln!(
            "[resource-raw-init-summary-timing] start function={} stage={}",
            self.function_name,
            stage
        );
        Some(std::time::Instant::now())
    }

    fn finish(&self, stage: &'static str, start: Option<std::time::Instant>) {
        if let Some(start) = start {
            std::eprintln!(
                "[resource-raw-init-summary-timing] end function={} stage={} elapsed_ms={}",
                self.function_name,
                stage,
                start.elapsed().as_millis()
            );
        }
    }
}

#[cfg(test)]
#[path = "initialized_summary_build_tests.rs"]
mod tests;
