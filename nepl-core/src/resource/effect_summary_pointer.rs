use alloc::vec::Vec;

use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_counts::ResourceEffectCounts;
use super::effect_identity::RawIdentityTable;
use super::effect_pointer_alias::RawPointerAliasTable;
use super::effect_raw_memory_identity::RawMemoryIdentityTable;
use super::effect_summary::{
    RawIdentityReturnSummary, RawIdentityReturnSummaryIndex, RawPointerParameterReturn,
    RawPointerReturnSummary, RawPointerReturnSummaryIndex,
};
use super::effect_summary_seed::parameter_summary_seed_places;
use super::function_alias::FunctionAliasTable;
use super::model::{Place, ResourceFunction, ResourceModule, ResourceTerminator};
use super::place_utils::place_suffix_after_prefix;
use super::summary_worklist::SummaryWorklist;

pub(super) fn compute_raw_pointer_return_summaries(
    module: &ResourceModule,
) -> Vec<RawPointerReturnSummary> {
    let mut worklist = SummaryWorklist::new(module);
    let mut summaries = Vec::new();
    while let Some(function_index) = worklist.pop() {
        let summary_index = RawPointerReturnSummaryIndex::new(&summaries);
        let summary =
            function_raw_pointer_return_summary(&module.functions[function_index], &summary_index);
        if update_raw_pointer_return_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_raw_pointer_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
    }
    summaries
}

fn function_raw_pointer_return_summary(
    function: &ResourceFunction,
    summary_index: &RawPointerReturnSummaryIndex<'_>,
) -> RawPointerReturnSummary {
    let mut parameter_returns = Vec::new();
    for (index, param) in function.params.iter().enumerate() {
        push_parameter_pointer_returns(
            &mut parameter_returns,
            index,
            function,
            &param.place,
            summary_index,
        );
    }
    RawPointerReturnSummary {
        function: function.name.clone(),
        parameter_returns,
    }
}

fn update_raw_pointer_return_summary(
    summaries: &mut Vec<RawPointerReturnSummary>,
    summary: RawPointerReturnSummary,
) -> bool {
    let has_facts = !summary.parameter_returns.is_empty();
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

fn push_parameter_pointer_returns(
    target: &mut Vec<RawPointerParameterReturn>,
    parameter_index: usize,
    function: &ResourceFunction,
    parameter: &Place,
    pointer_summaries: &RawPointerReturnSummaryIndex<'_>,
) {
    for seed in parameter_summary_seed_places(function, parameter) {
        let mut pointer_aliases = RawPointerAliasTable::default();
        pointer_aliases.mark(&seed);
        for returned in function_returned_pointer_alias_projections(
            function,
            &seed,
            pointer_aliases,
            pointer_summaries,
        ) {
            let source_projections =
                place_suffix_after_prefix(&seed, parameter).unwrap_or_default();
            push_unique_parameter_return(
                target,
                RawPointerParameterReturn {
                    parameter_index,
                    source_projections,
                    source_ty: seed.ty,
                    return_projections: returned.0,
                    return_ty: returned.1,
                },
            );
        }
    }
}

fn function_returned_pointer_alias_projections(
    function: &ResourceFunction,
    seed: &Place,
    mut pointer_aliases: RawPointerAliasTable,
    pointer_summaries: &RawPointerReturnSummaryIndex<'_>,
) -> Vec<(Vec<super::model::PlaceProjection>, crate::types::TypeId)> {
    let empty_identity_summaries: &[RawIdentityReturnSummary] = &[];
    let empty_identity_summary_index = RawIdentityReturnSummaryIndex::new(empty_identity_summaries);
    let mut engine = ResourceEffectBoundaryEngine {
        function: function.name.as_str(),
        effect: function.effect,
        summaries: &empty_identity_summary_index,
        pointer_summaries,
        types: None,
        track_alloc_identities: false,
        diagnostics: Vec::new(),
        counts: ResourceEffectCounts::default(),
    };
    let mut identities = RawIdentityTable::default();
    let mut function_aliases = FunctionAliasTable::default();
    let mut raw_memory_identities = RawMemoryIdentityTable::default();
    let mut projections = Vec::new();
    for block in &function.blocks {
        engine.check_ops(
            &mut identities,
            &mut pointer_aliases,
            &mut function_aliases,
            &mut raw_memory_identities,
            &block.ops,
        );
        if let ResourceTerminator::Return {
            value: Some(place), ..
        } = &block.terminator
        {
            for returned in pointer_aliases.projection_aliases_under(place, seed) {
                push_unique_return_projection(&mut projections, returned);
            }
        }
    }
    projections
}

fn push_unique_parameter_return(
    target: &mut Vec<RawPointerParameterReturn>,
    projection: RawPointerParameterReturn,
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
    target: &mut Vec<(Vec<super::model::PlaceProjection>, crate::types::TypeId)>,
    projection: (Vec<super::model::PlaceProjection>, crate::types::TypeId),
) {
    if !target
        .iter()
        .any(|existing| existing.0 == projection.0 && existing.1 == projection.1)
    {
        target.push(projection);
    }
    target.sort();
}
