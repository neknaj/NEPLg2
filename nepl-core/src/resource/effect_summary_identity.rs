use alloc::vec::Vec;

use super::effect_check::ResourceEffectBoundaryEngine;
use super::effect_counts::ResourceEffectCounts;
use super::effect_identity::{RawIdentityTable, RawPointerAliasTable};
use super::effect_raw_memory_identity::RawMemoryIdentityTable;
use super::effect_summary::{
    RawIdentityParameterReturn, RawIdentityReturnProjection, RawIdentityReturnSummary,
    RawIdentityReturnSummaryIndex, RawPointerReturnSummary, RawPointerReturnSummaryIndex,
};
use super::effect_summary_seed::parameter_summary_seed_places;
use super::function_alias::FunctionAliasTable;
use super::model::{Place, RawMemoryOp, ResourceFunction, ResourceModule, ResourceTerminator};
use super::place_utils::place_suffix_after_prefix;

pub(super) fn compute_raw_identity_return_summaries(
    module: &ResourceModule,
    pointer_summaries: &[RawPointerReturnSummary],
) -> Vec<RawIdentityReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        let summary_index = RawIdentityReturnSummaryIndex::new(&summaries);
        let pointer_summary_index = RawPointerReturnSummaryIndex::new(pointer_summaries);
        for function in &module.functions {
            let mut parameter_returns = Vec::new();
            for (index, param) in function.params.iter().enumerate() {
                push_parameter_identity_returns(
                    &mut parameter_returns,
                    index,
                    function,
                    &param.place,
                    &summary_index,
                    &pointer_summary_index,
                );
            }
            let internal_alloc_returns = function_returns_internal_alloc_identity_projections(
                function,
                &summary_index,
                &pointer_summary_index,
            );
            if !parameter_returns.is_empty() || !internal_alloc_returns.is_empty() {
                next.push(RawIdentityReturnSummary {
                    function: function.name.clone(),
                    parameter_returns,
                    internal_alloc_returns,
                });
            }
        }
        if next == summaries {
            return summaries;
        }
        summaries = next;
    }
    summaries
}

fn push_parameter_identity_returns(
    target: &mut Vec<RawIdentityParameterReturn>,
    parameter_index: usize,
    function: &ResourceFunction,
    parameter: &Place,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    pointer_summaries: &RawPointerReturnSummaryIndex<'_>,
) {
    for seed in parameter_summary_seed_places(function, parameter) {
        let mut identities = RawIdentityTable::default();
        identities.mark(&seed, RawMemoryOp::Alloc);
        for returned in function_returned_identity_projections_with_engine(
            function,
            identities,
            summaries,
            pointer_summaries,
            false,
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
) -> Vec<RawIdentityReturnProjection> {
    let identities = RawIdentityTable::default();
    function_returned_identity_projections_with_engine(
        function,
        identities,
        summaries,
        pointer_summaries,
        true,
    )
}

fn function_returned_identity_projections_with_engine(
    function: &ResourceFunction,
    mut identities: RawIdentityTable,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    pointer_summaries: &RawPointerReturnSummaryIndex<'_>,
    track_alloc_identities: bool,
) -> Vec<RawIdentityReturnProjection> {
    let mut engine = ResourceEffectBoundaryEngine {
        function: function.name.as_str(),
        effect: function.effect,
        summaries,
        pointer_summaries,
        types: None,
        track_alloc_identities,
        diagnostics: Vec::new(),
        counts: ResourceEffectCounts::default(),
    };
    let mut function_aliases = FunctionAliasTable::default();
    let mut pointer_aliases = RawPointerAliasTable::default();
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
            for (suffix, ty, operations) in identities.projection_operations_under(place) {
                push_unique_return_projection(
                    &mut projections,
                    RawIdentityReturnProjection {
                        projections: suffix,
                        ty,
                        operations,
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
        existing.projections == projection.projections && existing.ty == projection.ty
    }) {
        push_unique_operations(&mut existing.operations, &projection.operations);
    } else {
        target.push(projection);
    }
    target.sort_by(|left, right| {
        left.projections
            .cmp(&right.projections)
            .then_with(|| left.ty.cmp(&right.ty))
            .then_with(|| left.operations.cmp(&right.operations))
    });
}

fn push_unique_operations(target: &mut Vec<RawMemoryOp>, source: &[RawMemoryOp]) {
    for operation in source {
        if !target.contains(operation) {
            target.push(*operation);
        }
    }
    target.sort();
}
