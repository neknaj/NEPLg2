extern crate alloc;

use alloc::string::String;
use alloc::vec;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::initialized_alias_flow_raw::function_raw_cell_address_return_aliases;
use super::initialized_alias_flow_value_cache::{
    preseed_raw_alias_return_summaries_from_value_cache,
    record_raw_alias_return_summary_value_cache_candidates,
};
use super::initialized_alias_flow_value_projection::function_value_projection_return_aliases;
use super::model::{
    Place, PlaceProjection, ResourceExprKind, ResourceFunction, ResourceModule, ResourceOffset,
};
use super::place_utils::type_preserves_raw_address_alias;
use super::resource_summary_value_cache::{
    ResourceSummaryValueCache, ResourceSummaryValueCacheContext,
};
use super::summary_dependency::ResourceSummaryDependencyGraph;
use super::summary_index::{FunctionSummary, SummaryIndex, SummaryNameIndex};
use super::summary_worklist::SummaryWorklist;

pub(super) use super::initialized_alias_flow_apply::{
    apply_direct_call_raw_alias_summary, apply_indirect_call_raw_alias_summary,
    construct_raw_cell_address_alias_fields,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellAddressReturnSummary {
    pub(super) function: String,
    pub(super) parameters: Vec<Place>,
    pub(super) aliases: Vec<RawCellAddressReturnAlias>,
}

pub(super) type RawCellAddressReturnSummaryIndex<'a> =
    SummaryIndex<'a, RawCellAddressReturnSummary>;

impl FunctionSummary for RawCellAddressReturnSummary {
    fn function_name(&self) -> &str {
        &self.function
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct RawCellAddressReturnAlias {
    pub(super) parameter_index: usize,
    pub(super) parameter_projection: Vec<PlaceProjection>,
    pub(super) parameter_ty: TypeId,
    pub(super) return_projection: Vec<PlaceProjection>,
    pub(super) return_ty: TypeId,
}

pub(super) fn expr_kind_preserves_raw_alias(kind: ResourceExprKind) -> bool {
    matches!(
        kind,
        ResourceExprKind::LocalRead
            | ResourceExprKind::Call
            | ResourceExprKind::IndirectCall
            | ResourceExprKind::Intrinsic
            | ResourceExprKind::Borrow
            | ResourceExprKind::Branch
            | ResourceExprKind::Match
            | ResourceExprKind::Construct
    )
}

pub(super) fn expr_kind_preserves_read_scalar_facts(kind: ResourceExprKind) -> bool {
    matches!(kind, ResourceExprKind::Deref)
}

#[cfg(test)]
pub(super) fn compute_raw_cell_address_return_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
) -> Vec<RawCellAddressReturnSummary> {
    let dependency_graph = ResourceSummaryDependencyGraph::build(module);
    compute_raw_cell_address_return_summaries_with_recomputations(
        module,
        types,
        &dependency_graph,
        None,
        None,
    )
    .0
}

pub(super) fn compute_raw_cell_address_return_summaries_with_recomputations(
    module: &ResourceModule,
    types: &TypeCtx,
    dependency_graph: &ResourceSummaryDependencyGraph,
    mut summary_value_cache: Option<&mut ResourceSummaryValueCache>,
    summary_value_cache_context: Option<&ResourceSummaryValueCacheContext>,
) -> (Vec<RawCellAddressReturnSummary>, usize) {
    let relevant_functions = vec![true; module.functions.len()];
    let mut initially_skipped_functions = vec![false; module.functions.len()];
    let mut preseeded_functions = vec![false; module.functions.len()];
    let mut summaries = Vec::new();
    let mut replay_plan = match (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        (Some(cache), Some(context)) if cache.raw_alias_return_entry_collection_enabled() => {
            Some(cache.begin_raw_alias_summary_replay_plan(
                context,
                types,
                module,
                dependency_graph,
                &relevant_functions,
            ))
        }
        _ => None,
    };
    if let (Some(cache), Some(context)) = (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        if cache.raw_alias_return_entry_collection_enabled() {
            preseed_raw_alias_return_summaries_from_value_cache(
                cache,
                context,
                types,
                module,
                dependency_graph.dependencies(),
                &mut initially_skipped_functions,
                &mut preseeded_functions,
                &mut summaries,
                replay_plan.as_mut(),
            );
        }
    }
    let mut worklist = SummaryWorklist::new_filtered_with_dependency_graph_and_initial_skips(
        module,
        relevant_functions,
        initially_skipped_functions,
        dependency_graph,
    );
    let mut summary_name_index = SummaryNameIndex::from_entries(&summaries);
    while let Some(function_index) = worklist.pop() {
        let function = &module.functions[function_index];
        let summary = {
            let summary_index = summary_name_index.as_summary_index(&summaries);
            function_raw_cell_address_return_summary(function, &summary_index, types)
        };
        if update_raw_cell_address_return_summary(&mut summaries, &mut summary_name_index, summary)
        {
            worklist.notify_changed(function_index);
        }
    }
    if let (Some(cache), Some(context)) = (
        summary_value_cache.as_deref_mut(),
        summary_value_cache_context,
    ) {
        if cache.raw_alias_return_entry_collection_enabled() {
            let candidate_skipped_functions =
                worklist.unrecomputed_initial_skips(&preseeded_functions);
            record_raw_alias_return_summary_value_cache_candidates(
                cache,
                context,
                types,
                module,
                dependency_graph.dependencies(),
                &candidate_skipped_functions,
                &summaries,
                replay_plan.as_mut(),
            );
        }
    }
    if let (Some(cache), Some(plan)) = (summary_value_cache.as_deref_mut(), replay_plan) {
        cache.finish_raw_alias_summary_replay_plan(plan);
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_raw_alias_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
    }
    let recomputations = worklist.recomputations();
    (summaries, recomputations)
}

fn update_raw_cell_address_return_summary(
    summaries: &mut Vec<RawCellAddressReturnSummary>,
    summary_name_index: &mut SummaryNameIndex,
    summary: RawCellAddressReturnSummary,
) -> bool {
    let has_aliases = !summary.aliases.is_empty();
    let function = summary.function.clone();
    let position = summary_name_index.position(&function);
    match (has_aliases, position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            let widened = widen_raw_cell_address_return_summary(&summaries[index], summary);
            if summaries[index] == widened {
                false
            } else {
                summaries[index] = widened;
                true
            }
        }
        (true, None) => {
            summary_name_index.insert_at_end(&function, summaries.len());
            summaries.push(summary);
            true
        }
        (false, Some(index)) => {
            summaries.remove(index);
            summary_name_index.remove_and_shift(&function, index);
            true
        }
        (false, None) => false,
    }
}

fn widen_raw_cell_address_return_summary(
    existing: &RawCellAddressReturnSummary,
    mut summary: RawCellAddressReturnSummary,
) -> RawCellAddressReturnSummary {
    let mut widened = Vec::new();
    for alias in summary.aliases {
        let alias = normalize_raw_cell_address_return_alias(alias);
        let Some(existing_alias) = existing
            .aliases
            .iter()
            .find(|existing_alias| raw_aliases_are_widening_compatible(existing_alias, &alias))
        else {
            push_unique_return_alias(&mut widened, alias);
            continue;
        };
        push_unique_return_alias(
            &mut widened,
            widen_raw_cell_address_return_alias(existing_alias, &alias),
        );
    }
    summary.aliases = widened;
    summary
}

fn function_raw_cell_address_return_summary(
    function: &ResourceFunction,
    summary_index: &RawCellAddressReturnSummaryIndex<'_>,
    types: &TypeCtx,
) -> RawCellAddressReturnSummary {
    let mut aliases = function_value_projection_return_aliases(function, summary_index, types);
    for (index, param) in function.params.iter().enumerate() {
        for alias in function_raw_cell_address_return_aliases(
            function,
            index,
            &param.place,
            summary_index,
            types,
        ) {
            push_unique_return_alias(&mut aliases, alias);
        }
    }
    RawCellAddressReturnSummary {
        function: function.name.clone(),
        parameters: function
            .params
            .iter()
            .map(|param| param.place.clone())
            .collect(),
        aliases,
    }
}

pub(super) fn expr_output_preserves_raw_alias(
    types: &TypeCtx,
    kind: ResourceExprKind,
    output: &Place,
) -> bool {
    matches!(kind, ResourceExprKind::Deref) && type_preserves_raw_address_alias(types, output.ty)
}

pub(super) fn push_unique_return_alias(
    aliases: &mut Vec<RawCellAddressReturnAlias>,
    alias: RawCellAddressReturnAlias,
) {
    let alias = normalize_raw_cell_address_return_alias(alias);
    if aliases
        .iter()
        .any(|existing| raw_alias_subsumes(existing, &alias))
    {
        return;
    }
    aliases.retain(|existing| !raw_alias_subsumes(&alias, existing));
    aliases.push(alias);
}

fn normalize_raw_cell_address_return_alias(
    mut alias: RawCellAddressReturnAlias,
) -> RawCellAddressReturnAlias {
    alias.parameter_projection = normalize_raw_address_projections(alias.parameter_projection);
    alias.return_projection = normalize_raw_address_projections(alias.return_projection);
    alias
}

fn normalize_raw_address_projections(projections: Vec<PlaceProjection>) -> Vec<PlaceProjection> {
    let mut out = Vec::new();
    for projection in projections {
        let PlaceProjection::StorageOffset(offset) = projection else {
            out.push(projection);
            continue;
        };
        if matches!(offset, ResourceOffset::Known(0)) {
            continue;
        }
        match out.last_mut() {
            Some(PlaceProjection::StorageOffset(existing)) => {
                *existing = combine_raw_address_offsets(existing.clone(), offset);
            }
            _ => out.push(PlaceProjection::StorageOffset(offset)),
        }
    }
    out
}

fn combine_raw_address_offsets(left: ResourceOffset, right: ResourceOffset) -> ResourceOffset {
    match (left, right) {
        (offset, ResourceOffset::Known(0)) | (ResourceOffset::Known(0), offset) => offset,
        (ResourceOffset::Known(left), ResourceOffset::Known(right)) => left
            .checked_add(right)
            .map(ResourceOffset::Known)
            .unwrap_or(ResourceOffset::Unknown),
        _ => ResourceOffset::Unknown,
    }
}

fn raw_aliases_are_widening_compatible(
    left: &RawCellAddressReturnAlias,
    right: &RawCellAddressReturnAlias,
) -> bool {
    left.parameter_index == right.parameter_index
        && left.parameter_ty == right.parameter_ty
        && left.return_ty == right.return_ty
        && raw_address_projections_are_widening_compatible(
            &left.parameter_projection,
            &right.parameter_projection,
        )
        && raw_address_projections_are_widening_compatible(
            &left.return_projection,
            &right.return_projection,
        )
}

fn raw_alias_subsumes(
    general: &RawCellAddressReturnAlias,
    specific: &RawCellAddressReturnAlias,
) -> bool {
    general.parameter_index == specific.parameter_index
        && general.parameter_ty == specific.parameter_ty
        && general.return_ty == specific.return_ty
        && raw_address_projections_subsume(
            &general.parameter_projection,
            &specific.parameter_projection,
        )
        && raw_address_projections_subsume(&general.return_projection, &specific.return_projection)
}

fn raw_address_projections_subsume(
    general: &[PlaceProjection],
    specific: &[PlaceProjection],
) -> bool {
    general.len() == specific.len()
        && general
            .iter()
            .zip(specific)
            .all(|(general, specific)| match (general, specific) {
                (
                    PlaceProjection::StorageOffset(ResourceOffset::Unknown),
                    PlaceProjection::StorageOffset(_),
                ) => true,
                (
                    PlaceProjection::StorageOffset(general),
                    PlaceProjection::StorageOffset(specific),
                ) => general == specific,
                _ => general == specific,
            })
}

fn raw_address_projections_are_widening_compatible(
    left: &[PlaceProjection],
    right: &[PlaceProjection],
) -> bool {
    left.len() == right.len()
        && left
            .iter()
            .zip(right)
            .all(|(left, right)| match (left, right) {
                (PlaceProjection::StorageOffset(_), PlaceProjection::StorageOffset(_)) => true,
                _ => left == right,
            })
}

fn widen_raw_cell_address_return_alias(
    existing: &RawCellAddressReturnAlias,
    alias: &RawCellAddressReturnAlias,
) -> RawCellAddressReturnAlias {
    RawCellAddressReturnAlias {
        parameter_index: alias.parameter_index,
        parameter_projection: widen_raw_address_projections(
            &existing.parameter_projection,
            &alias.parameter_projection,
        ),
        parameter_ty: alias.parameter_ty,
        return_projection: widen_raw_address_projections(
            &existing.return_projection,
            &alias.return_projection,
        ),
        return_ty: alias.return_ty,
    }
}

fn widen_raw_address_projections(
    existing: &[PlaceProjection],
    current: &[PlaceProjection],
) -> Vec<PlaceProjection> {
    existing
        .iter()
        .zip(current)
        .map(|(existing, current)| match (existing, current) {
            (PlaceProjection::StorageOffset(existing), PlaceProjection::StorageOffset(current))
                if existing != current =>
            {
                PlaceProjection::StorageOffset(ResourceOffset::Unknown)
            }
            (_, current) => current.clone(),
        })
        .collect()
}

#[cfg(test)]
#[path = "initialized_alias_flow_tests.rs"]
mod tests;
