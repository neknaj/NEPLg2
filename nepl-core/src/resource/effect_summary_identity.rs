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
use super::effect_summary_seed::parameter_summary_seed_places;
use super::function_alias::FunctionAliasTable;
use super::model::{
    Place, RawMemoryOp, ResourceCallTarget, ResourceFunction, ResourceModule, ResourceOp,
    ResourceTerminator,
};
use super::place_utils::{place_suffix_after_prefix, place_with_suffix, push_unique_place};
use super::summary_worklist::SummaryWorklist;
use crate::span::Span;
use crate::types::TypeCtx;

pub(super) fn compute_raw_identity_return_summaries(
    module: &ResourceModule,
    pointer_summaries: &[RawPointerReturnSummary],
    types: Option<&TypeCtx>,
) -> Vec<RawIdentityReturnSummary> {
    let mut worklist = SummaryWorklist::new(module);
    let mut summaries = Vec::new();
    let pointer_summary_index = RawPointerReturnSummaryIndex::new(pointer_summaries);
    while let Some(function_index) = worklist.pop() {
        let summary_index = RawIdentityReturnSummaryIndex::new(&summaries);
        let summary = function_raw_identity_return_summary(
            &module.functions[function_index],
            &summary_index,
            &pointer_summary_index,
            types,
        );
        if update_raw_identity_return_summary(&mut summaries, summary) {
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
    }
    summaries
}

fn function_raw_identity_return_summary(
    function: &ResourceFunction,
    summary_index: &RawIdentityReturnSummaryIndex<'_>,
    pointer_summary_index: &RawPointerReturnSummaryIndex<'_>,
    types: Option<&TypeCtx>,
) -> RawIdentityReturnSummary {
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
    RawIdentityReturnSummary {
        function: function.name.clone(),
        parameter_returns,
        internal_alloc_returns,
    }
}

fn update_raw_identity_return_summary(
    summaries: &mut Vec<RawIdentityReturnSummary>,
    summary: RawIdentityReturnSummary,
) -> bool {
    let has_facts =
        !summary.parameter_returns.is_empty() || !summary.internal_alloc_returns.is_empty();
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

fn push_parameter_identity_returns(
    target: &mut Vec<RawIdentityParameterReturn>,
    parameter_index: usize,
    function: &ResourceFunction,
    parameter: &Place,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    pointer_summaries: &RawPointerReturnSummaryIndex<'_>,
    types: Option<&TypeCtx>,
) {
    for seed in parameter_identity_summary_seed_places(function, parameter, summaries) {
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

fn parameter_identity_summary_seed_places(
    function: &ResourceFunction,
    parameter: &Place,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
) -> Vec<Place> {
    let mut places = parameter_summary_seed_places(function, parameter);
    for block in &function.blocks {
        collect_call_summary_source_seeds(&block.ops, parameter, summaries, &mut places);
    }
    places.sort();
    places
}

fn collect_call_summary_source_seeds(
    ops: &[ResourceOp],
    parameter: &Place,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    places: &mut Vec<Place>,
) {
    for op in ops {
        match op {
            ResourceOp::Call { target, args, .. } => {
                collect_direct_call_summary_source_seeds(
                    target, args, parameter, summaries, places,
                );
            }
            ResourceOp::Branch {
                then_ops, else_ops, ..
            } => {
                collect_call_summary_source_seeds(then_ops, parameter, summaries, places);
                collect_call_summary_source_seeds(else_ops, parameter, summaries, places);
            }
            ResourceOp::Loop {
                condition_ops,
                body_ops,
                ..
            } => {
                collect_call_summary_source_seeds(condition_ops, parameter, summaries, places);
                collect_call_summary_source_seeds(body_ops, parameter, summaries, places);
            }
            ResourceOp::Match { arms, .. } => {
                for arm in arms {
                    collect_call_summary_source_seeds(&arm.ops, parameter, summaries, places);
                }
            }
            ResourceOp::Expr { .. }
            | ResourceOp::DeclareLocal { .. }
            | ResourceOp::Read { .. }
            | ResourceOp::Assign { .. }
            | ResourceOp::Borrow { .. }
            | ResourceOp::Move { .. }
            | ResourceOp::Drop { .. }
            | ResourceOp::EndScope { .. }
            | ResourceOp::CallEffect { .. }
            | ResourceOp::FunctionValue { .. }
            | ResourceOp::IndirectCall { .. }
            | ResourceOp::RawMemory { .. }
            | ResourceOp::RawAddressAlias { .. }
            | ResourceOp::RawAddressView { .. }
            | ResourceOp::StorageOrigin { .. }
            | ResourceOp::Construct { .. } => {}
        }
    }
}

fn collect_direct_call_summary_source_seeds(
    target: &ResourceCallTarget,
    args: &[Place],
    parameter: &Place,
    summaries: &RawIdentityReturnSummaryIndex<'_>,
    places: &mut Vec<Place>,
) {
    let ResourceCallTarget::User { name, .. } = target else {
        return;
    };
    let Some(summary) = summaries.get(name) else {
        return;
    };
    for parameter_return in &summary.parameter_returns {
        let Some(arg) = args.get(parameter_return.parameter_index) else {
            continue;
        };
        let seed = place_with_suffix(
            arg,
            &parameter_return.source_projections,
            parameter_return.source_ty,
        );
        push_parameter_identity_seed(places, parameter, &seed);
    }
}

fn push_parameter_identity_seed(places: &mut Vec<Place>, parameter: &Place, seed: &Place) {
    if place_suffix_after_prefix(seed, parameter).is_some() {
        push_unique_place(places, seed);
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
            for (suffix, ty, origins) in identities.projection_origins_under(place) {
                if !raw_identity_return_projection_requires_summary(types, place, &suffix, ty) {
                    continue;
                }
                push_unique_return_projection(
                    &mut projections,
                    RawIdentityReturnProjection {
                        projections: suffix,
                        ty,
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
        existing.projections == projection.projections && existing.ty == projection.ty
    }) {
        push_unique_origins(&mut existing.origins, &projection.origins);
    } else {
        target.push(projection);
    }
    target.sort_by(|left, right| {
        left.projections
            .cmp(&right.projections)
            .then_with(|| left.ty.cmp(&right.ty))
            .then_with(|| left.origins.cmp(&right.origins))
    });
}
