use alloc::vec;
use alloc::vec::Vec;

use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_counts::ResourceEffectCounts;
use super::effect_identity::{push_unique_origins, RawIdentityTable};
use super::effect_pointer_alias::RawPointerAliasTable;
use super::effect_raw_memory_identity::RawMemoryIdentityTable;
use super::effect_return_summary_filter::raw_identity_return_projection_requires_summary;
use super::effect_summary::{
    RawIdentityParameterReturn, RawIdentityReturnProjection, RawIdentityReturnSummary,
    RawIdentityReturnSummaryIndex, RawPointerReturnSummary, RawPointerReturnSummaryIndex,
};
use super::effect_summary_identity_seed::{
    parameter_identity_summary_seed_places, summary_seed_can_carry_raw_identity,
};
use super::effect_summary_projection::summary_projection_is_valid;
use super::function_alias::FunctionAliasTable;
use super::model::{Place, RawMemoryOp, ResourceFunction, ResourceModule, ResourceTerminator};
use super::place_utils::place_suffix_after_prefix;
use super::private_cache_mask::PrivateCacheMaskProofIndex;
use super::private_cache_taint::PrivateCacheRegionTaintTable;
use super::summary_index::SummaryNameIndex;
use super::summary_worklist::SummaryWorklist;
use crate::span::Span;
use crate::types::TypeCtx;

pub(super) fn compute_raw_identity_return_summaries(
    module: &ResourceModule,
    pointer_summaries: &[RawPointerReturnSummary],
    types: Option<&TypeCtx>,
) -> Vec<RawIdentityReturnSummary> {
    let relevant_functions = raw_identity_return_summary_relevant_functions(module, types);
    let mut worklist = SummaryWorklist::new_filtered(module, relevant_functions.clone());
    let mut summaries = Vec::new();
    let pointer_summary_index = RawPointerReturnSummaryIndex::new(pointer_summaries);
    let mut summary_name_index = SummaryNameIndex::from_entries(&summaries);
    while let Some(function_index) = worklist.pop() {
        let summary = {
            let summary_index = summary_name_index.as_summary_index(&summaries);
            function_raw_identity_return_summary(
                &module.functions[function_index],
                &summary_index,
                &pointer_summary_index,
                types,
            )
        };
        if update_raw_identity_return_summary(&mut summaries, &mut summary_name_index, summary) {
            worklist.notify_changed(function_index);
        }
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_raw_identity_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
        std::eprintln!(
            "[compile-stage] resource_raw_identity_summary_relevant_functions={}",
            relevant_functions
                .iter()
                .filter(|is_relevant| **is_relevant)
                .count()
        );
    }
    summaries
}

fn raw_identity_return_summary_relevant_functions(
    module: &ResourceModule,
    types: Option<&TypeCtx>,
) -> Vec<bool> {
    let Some(types) = types else {
        return vec![true; module.functions.len()];
    };
    module
        .functions
        .iter()
        .map(|function| {
            let returned = Place::temporary(super::model::ResourceId(usize::MAX), function.result);
            raw_identity_return_projection_requires_summary(
                Some(types),
                &returned,
                &[],
                function.result,
            )
        })
        .collect()
}

fn function_raw_identity_return_summary(
    function: &ResourceFunction,
    summary_index: &RawIdentityReturnSummaryIndex<'_>,
    pointer_summary_index: &RawPointerReturnSummaryIndex<'_>,
    types: Option<&TypeCtx>,
) -> RawIdentityReturnSummary {
    if types.is_some_and(|types| {
        let returned = Place::temporary(super::model::ResourceId(usize::MAX), function.result);
        !raw_identity_return_projection_requires_summary(
            Some(types),
            &returned,
            &[],
            function.result,
        )
    }) {
        return RawIdentityReturnSummary {
            function: function.name.clone(),
            parameter_returns: Vec::new(),
            internal_alloc_returns: Vec::new(),
        };
    }
    let mut parameter_returns = Vec::new();
    for (index, param) in function.params.iter().enumerate() {
        push_parameter_identity_returns(
            &mut parameter_returns,
            index,
            function,
            &param.place,
            summary_index,
            pointer_summary_index,
            types,
        );
    }
    let internal_alloc_returns = function_returns_internal_alloc_identity_projections(
        function,
        summary_index,
        pointer_summary_index,
        types,
    );
    let mut summary = RawIdentityReturnSummary {
        function: function.name.clone(),
        parameter_returns,
        internal_alloc_returns,
    };
    filter_raw_identity_return_summary(&mut summary, function, types);
    summary
}

pub(super) fn filter_raw_identity_return_summary(
    summary: &mut RawIdentityReturnSummary,
    function: &ResourceFunction,
    types: Option<&TypeCtx>,
) {
    let Some(types) = types else {
        return;
    };
    let return_place = Place::temporary(super::model::ResourceId(usize::MAX), function.result);
    summary.parameter_returns.retain(|item| {
        let Some(parameter) = function.params.get(item.parameter_index) else {
            return false;
        };
        summary_projection_is_valid(
            types,
            &parameter.place,
            &item.source_projections,
            item.source_ty,
        ) && summary_projection_is_valid(
            types,
            &return_place,
            &item.return_projections,
            item.return_ty,
        )
    });
    summary.internal_alloc_returns.retain(|item| {
        summary_projection_is_valid(types, &return_place, &item.projections, item.ty)
    });
}

fn update_raw_identity_return_summary(
    summaries: &mut Vec<RawIdentityReturnSummary>,
    summary_name_index: &mut SummaryNameIndex,
    summary: RawIdentityReturnSummary,
) -> bool {
    let has_facts =
        !summary.parameter_returns.is_empty() || !summary.internal_alloc_returns.is_empty();
    let function = summary.function.clone();
    let position = summary_name_index.position(&function);
    match (has_facts, position) {
        (true, Some(index)) if summaries[index] == summary => false,
        (true, Some(index)) => {
            summaries[index] = summary;
            true
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

fn push_parameter_identity_returns(
    target: &mut Vec<RawIdentityParameterReturn>,
    parameter_index: usize,
    function: &ResourceFunction,
    parameter: &Place,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    pointer_summaries: &RawPointerReturnSummaryIndex<'_>,
    types: Option<&TypeCtx>,
) {
    for seed in parameter_identity_summary_seed_places(function, parameter, summaries, types) {
        if !summary_seed_can_carry_raw_identity(types, parameter, &seed) {
            continue;
        }
        let mut identities = RawIdentityTable::default();
        identities.mark(&seed, RawMemoryOp::Alloc, Span::dummy());
        for returned in function_returned_identity_projections_with_engine(
            function,
            identities,
            summaries,
            pointer_summaries,
            false,
            types,
        ) {
            let source_projections =
                place_suffix_after_prefix(&seed, parameter).unwrap_or_default();
            push_unique_parameter_return(
                target,
                RawIdentityParameterReturn {
                    parameter_index,
                    source_projections,
                    source_ty: seed.ty,
                    return_projections: returned.projections,
                    return_ty: returned.ty,
                },
            );
        }
    }
}

fn function_returns_internal_alloc_identity_projections(
    function: &ResourceFunction,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    pointer_summaries: &RawPointerReturnSummaryIndex<'_>,
    types: Option<&TypeCtx>,
) -> Vec<RawIdentityReturnProjection> {
    let identities = RawIdentityTable::default();
    function_returned_identity_projections_with_engine(
        function,
        identities,
        summaries,
        pointer_summaries,
        true,
        types,
    )
}

fn function_returned_identity_projections_with_engine(
    function: &ResourceFunction,
    mut identities: RawIdentityTable,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    pointer_summaries: &RawPointerReturnSummaryIndex<'_>,
    track_alloc_identities: bool,
    types: Option<&TypeCtx>,
) -> Vec<RawIdentityReturnProjection> {
    let mut engine = ResourceEffectBoundaryEngine {
        function: function.name.as_str(),
        effect: function.effect,
        summaries,
        pointer_summaries,
        types,
        track_alloc_identities,
        propagate_return_provenance: true,
        private_cache_mask_proofs: &PrivateCacheMaskProofIndex::empty(),
        diagnostics: Vec::new(),
        counts: ResourceEffectCounts::default(),
    };
    let mut function_aliases = FunctionAliasTable::default();
    let mut pointer_aliases = RawPointerAliasTable::default();
    let mut raw_memory_identities = RawMemoryIdentityTable::default();
    let mut private_cache_taints = PrivateCacheRegionTaintTable::default();
    let mut projections = Vec::new();
    for block in &function.blocks {
        engine.check_ops(
            &mut identities,
            &mut pointer_aliases,
            &mut function_aliases,
            &mut raw_memory_identities,
            &mut private_cache_taints,
            &block.ops,
        );
        if let ResourceTerminator::Return {
            value: Some(place),
            span,
        } = &block.terminator
        {
            for (suffix, ty, origins) in identities.projection_origins_under(place) {
                if !raw_identity_return_projection_requires_summary(types, place, &suffix, ty) {
                    continue;
                }
                push_unique_return_projection(
                    &mut projections,
                    RawIdentityReturnProjection {
                        projections: suffix,
                        ty,
                        return_span: *span,
                        origins,
                    },
                );
            }
        }
    }
    projections
}

fn push_unique_parameter_return(
    target: &mut Vec<RawIdentityParameterReturn>,
    projection: RawIdentityParameterReturn,
) {
    if !target.iter().any(|existing| existing == &projection) {
        target.push(projection);
    }
    target.sort_by(|left, right| {
        left.parameter_index
            .cmp(&right.parameter_index)
            .then_with(|| left.source_projections.cmp(&right.source_projections))
            .then_with(|| left.source_ty.cmp(&right.source_ty))
            .then_with(|| left.return_projections.cmp(&right.return_projections))
            .then_with(|| left.return_ty.cmp(&right.return_ty))
    });
}

fn push_unique_return_projection(
    target: &mut Vec<RawIdentityReturnProjection>,
    projection: RawIdentityReturnProjection,
) {
    if let Some(existing) = target.iter_mut().find(|existing| {
        existing.projections == projection.projections
            && existing.ty == projection.ty
            && existing.return_span == projection.return_span
    }) {
        push_unique_origins(&mut existing.origins, &projection.origins);
    } else {
        target.push(projection);
    }
    target.sort_by(|left, right| {
        left.projections
            .cmp(&right.projections)
            .then_with(|| left.ty.cmp(&right.ty))
            .then_with(|| left.return_span.file_id.0.cmp(&right.return_span.file_id.0))
            .then_with(|| left.return_span.start.cmp(&right.return_span.start))
            .then_with(|| left.return_span.end.cmp(&right.return_span.end))
            .then_with(|| left.origins.cmp(&right.origins))
    });
}
