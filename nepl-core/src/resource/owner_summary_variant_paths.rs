use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;

use super::condition_fact::simple_condition_value_constraint;
use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    AggregateKind, I32ValueCondition, Place, PlaceProjection, ResourceConditionFact,
    ResourceMatchArm, ResourceOp,
};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_summary::consumed_owner_parameters;
use super::owner_summary_record::OwnerParameterStorageSource;
use super::owner_summary_variant_return::{
    record_variant_projection_returns, returned_owner_returns_for_value,
};
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::{
    construct_aggregate_field_place, match_bind_payload_place, place_suffix_after_prefix,
    place_with_suffix,
};
use super::raw_realloc::PendingRawReallocs;
use super::report::{ResourceOwnerCheckDeferred, ResourceOwnerOperation};
use super::storage_origin::StorageOriginTable;
use super::summary::{
    OwnerProjectionSource, OwnerValueCondition, OwnerVariantCondition, OwnerVariantParameterIndex,
    OwnerVariantPayloadCondition, OwnerVariantProjectionReturn, OwnerVariantProjectionSource,
};

pub(super) fn collect_variant_consumed_owner_parameters_from_nested_return(
    index_out: &mut Vec<OwnerVariantParameterIndex>,
    source_out: &mut Vec<OwnerVariantProjectionSource>,
    condition_out: &mut Vec<OwnerVariantCondition>,
    payload_condition_out: &mut Vec<OwnerVariantPayloadCondition>,
    engine: &ResourceOwnerCheckEngine<'_>,
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    raw_views: &RawAddressViewTable,
    storage_origins: &StorageOriginTable,
    function_aliases: &FunctionAliasTable,
    pending_reallocs: &PendingRawReallocs,
    variant_owner_effects: &PendingVariantOwnerEffects,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    ops: &[ResourceOp],
    return_value: &Place,
    return_out: &mut Vec<OwnerVariantProjectionReturn>,
) {
    let mut engine = ResourceOwnerCheckEngine {
        function: engine.function,
        types: engine.types,
        raw_alias_summaries: engine.raw_alias_summaries,
        summaries: engine.summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
    };
    let mut owners = owners.clone();
    let mut raw_aliases = raw_aliases.clone();
    let mut raw_views = raw_views.clone();
    let mut storage_origins = storage_origins.clone();
    let mut function_aliases = function_aliases.clone();
    let mut pending_reallocs = pending_reallocs.clone();
    let mut variant_owner_effects = variant_owner_effects.clone();
    for (index, op) in ops.iter().enumerate() {
        match op {
            ResourceOp::Branch {
                output,
                condition_fact,
                then_ops,
                then_value,
                else_ops,
                else_value,
                span,
                ..
            } if output == return_value => {
                let mut then_owners = owners.clone();
                let mut then_raw_aliases = raw_aliases.clone();
                let mut then_storage_origins = storage_origins.clone();
                let mut then_pending_reallocs = pending_reallocs.clone();
                engine.apply_branch_condition_fact(
                    &mut then_owners,
                    &mut then_raw_aliases,
                    &mut then_storage_origins,
                    &mut then_pending_reallocs,
                    condition_fact.as_ref(),
                    true,
                    *span,
                );
                collect_variant_consumed_owner_parameters_from_path(
                    index_out,
                    source_out,
                    condition_out,
                    payload_condition_out,
                    &engine,
                    &then_owners,
                    &then_raw_aliases,
                    &raw_views,
                    &then_storage_origins,
                    &function_aliases,
                    &then_pending_reallocs,
                    &variant_owner_effects,
                    parameter_storage_sources,
                    then_ops,
                    then_value,
                    condition_fact.as_ref().map(|fact| (fact, true)),
                    None,
                    return_out,
                );
                let mut else_owners = owners.clone();
                let mut else_raw_aliases = raw_aliases.clone();
                let mut else_storage_origins = storage_origins.clone();
                let mut else_pending_reallocs = pending_reallocs.clone();
                engine.apply_branch_condition_fact(
                    &mut else_owners,
                    &mut else_raw_aliases,
                    &mut else_storage_origins,
                    &mut else_pending_reallocs,
                    condition_fact.as_ref(),
                    false,
                    *span,
                );
                collect_variant_consumed_owner_parameters_from_path(
                    index_out,
                    source_out,
                    condition_out,
                    payload_condition_out,
                    &engine,
                    &else_owners,
                    &else_raw_aliases,
                    &raw_views,
                    &else_storage_origins,
                    &function_aliases,
                    &else_pending_reallocs,
                    &variant_owner_effects,
                    parameter_storage_sources,
                    else_ops,
                    else_value,
                    condition_fact.as_ref().map(|fact| (fact, false)),
                    None,
                    return_out,
                );
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                span,
            } if output == return_value => {
                for arm in arms {
                    if !variant_owner_effects.match_arm_reachable(scrutinee, &arm.pattern) {
                        continue;
                    }
                    collect_variant_consumed_owner_parameters_from_path(
                        index_out,
                        source_out,
                        condition_out,
                        payload_condition_out,
                        &engine,
                        &owners,
                        &raw_aliases,
                        &raw_views,
                        &storage_origins,
                        &function_aliases,
                        &pending_reallocs,
                        &variant_owner_effects,
                        parameter_storage_sources,
                        &arm.ops,
                        &arm.value,
                        None,
                        Some((scrutinee, arm, *span)),
                        return_out,
                    );
                }
            }
            _ => {}
        }
        engine.check_ops(
            &mut owners,
            &mut function_aliases,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &mut pending_reallocs,
            &mut variant_owner_effects,
            &ops[index..=index],
        );
    }
    variant_owner_effects.collect_returned_result_effects(
        &owners,
        &raw_aliases,
        return_value,
        parameter_storage_sources,
        index_out,
        source_out,
        return_out,
        payload_condition_out,
    );
}

fn collect_variant_consumed_owner_parameters_from_path(
    index_out: &mut Vec<OwnerVariantParameterIndex>,
    source_out: &mut Vec<OwnerVariantProjectionSource>,
    condition_out: &mut Vec<OwnerVariantCondition>,
    payload_condition_out: &mut Vec<OwnerVariantPayloadCondition>,
    engine: &ResourceOwnerCheckEngine<'_>,
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    raw_views: &RawAddressViewTable,
    storage_origins: &StorageOriginTable,
    function_aliases: &FunctionAliasTable,
    pending_reallocs: &PendingRawReallocs,
    variant_owner_effects: &PendingVariantOwnerEffects,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    path_ops: &[ResourceOp],
    path_value: &Place,
    branch_condition: Option<(&ResourceConditionFact, bool)>,
    match_arm: Option<(&Place, &ResourceMatchArm, Span)>,
    return_out: &mut Vec<OwnerVariantProjectionReturn>,
) {
    let mut path_engine = ResourceOwnerCheckEngine {
        function: engine.function,
        types: engine.types,
        raw_alias_summaries: engine.raw_alias_summaries,
        summaries: engine.summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
    };
    let mut path_owners = owners.clone();
    let mut path_raw_aliases = raw_aliases.clone();
    let mut path_raw_views = raw_views.clone();
    let mut path_storage_origins = storage_origins.clone();
    let mut path_function_aliases = function_aliases.clone();
    let mut path_pending_reallocs = pending_reallocs.clone();
    let mut path_variant_owner_effects = variant_owner_effects.clone();
    apply_match_arm_entry(
        &mut path_engine,
        &mut path_owners,
        &mut path_raw_aliases,
        &mut path_raw_views,
        &mut path_storage_origins,
        &mut path_function_aliases,
        &mut path_pending_reallocs,
        &mut path_variant_owner_effects,
        match_arm,
    );
    let Some(constructed_variant) = construct_variant_for_value(path_ops, path_value) else {
        collect_variant_consumed_owner_parameters_from_nested_return(
            index_out,
            source_out,
            condition_out,
            payload_condition_out,
            &path_engine,
            &path_owners,
            &path_raw_aliases,
            &path_raw_views,
            &path_storage_origins,
            &path_function_aliases,
            &path_pending_reallocs,
            &path_variant_owner_effects,
            parameter_storage_sources,
            path_ops,
            path_value,
            return_out,
        );
        path_engine.check_ops(
            &mut path_owners,
            &mut path_function_aliases,
            &mut path_raw_aliases,
            &mut path_raw_views,
            &mut path_storage_origins,
            &mut path_pending_reallocs,
            &mut path_variant_owner_effects,
            path_ops,
        );
        path_variant_owner_effects.collect_returned_result_effects(
            &path_owners,
            &path_raw_aliases,
            path_value,
            parameter_storage_sources,
            index_out,
            source_out,
            return_out,
            payload_condition_out,
        );
        return;
    };
    if let Some((condition_fact, truthy_path)) = branch_condition {
        collect_owner_variant_condition(
            condition_out,
            &constructed_variant.variant,
            condition_fact,
            truthy_path,
            &path_raw_aliases,
            parameter_storage_sources,
        );
    }
    path_engine.check_ops(
        &mut path_owners,
        &mut path_function_aliases,
        &mut path_raw_aliases,
        &mut path_raw_views,
        &mut path_storage_origins,
        &mut path_pending_reallocs,
        &mut path_variant_owner_effects,
        path_ops,
    );
    if let Some((condition_fact, truthy_path)) = branch_condition {
        collect_owner_variant_payload_conditions(
            payload_condition_out,
            &constructed_variant,
            path_value,
            condition_fact,
            truthy_path,
            &path_raw_aliases,
        );
    }

    let (projection_returns, returned_sources) = returned_owner_returns_for_value(
        &path_owners,
        &path_raw_aliases,
        path_value,
        parameter_storage_sources,
    );
    record_variant_projection_returns(
        return_out,
        &constructed_variant.variant,
        &projection_returns,
        parameter_storage_sources,
    );
    let (indices, sources) =
        consumed_owner_parameters(&path_owners, parameter_storage_sources, &returned_sources);
    let variant = normalize_variant_name(&constructed_variant.variant);
    for parameter_index in indices {
        push_unique_variant_parameter_index(
            index_out,
            OwnerVariantParameterIndex {
                variant: variant.clone(),
                parameter_index,
            },
        );
    }
    for source in sources {
        push_unique_variant_projection_source(
            source_out,
            OwnerVariantProjectionSource {
                variant: variant.clone(),
                source,
            },
        );
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_match_arm_entry(
    path_engine: &mut ResourceOwnerCheckEngine<'_>,
    path_owners: &mut OwnerTable,
    path_raw_aliases: &mut RawCellAddressAliases,
    path_raw_views: &mut RawAddressViewTable,
    path_storage_origins: &mut StorageOriginTable,
    path_function_aliases: &mut FunctionAliasTable,
    path_pending_reallocs: &mut PendingRawReallocs,
    path_variant_owner_effects: &mut PendingVariantOwnerEffects,
    match_arm: Option<(&Place, &ResourceMatchArm, Span)>,
) {
    let Some((scrutinee, arm, span)) = match_arm else {
        return;
    };
    if !path_variant_owner_effects.match_arm_reachable(scrutinee, &arm.pattern) {
        return;
    }
    path_variant_owner_effects.apply_match_arm_returns(
        path_engine,
        path_owners,
        path_raw_aliases,
        path_raw_views,
        path_storage_origins,
        scrutinee,
        &arm.pattern,
        span,
    );
    if let Some(bind_local) = &arm.bind_local {
        if let Some(source) = match_bind_payload_place(scrutinee, arm, bind_local) {
            if path_engine.initializer_is_non_owning_raw_alias_view(
                path_owners,
                path_raw_aliases,
                &source,
                bind_local,
            ) {
                path_engine.copy_non_owning_owner_markers(path_owners, &source, bind_local);
                path_raw_aliases.copy_alias_or_seed(&source, bind_local);
                path_storage_origins.copy_origin(&source, bind_local);
            } else {
                path_engine.transfer_owner(
                    path_owners,
                    path_raw_aliases,
                    path_storage_origins,
                    &source,
                    bind_local,
                    ResourceOwnerOperation::MatchValue,
                    span,
                );
            }
            path_function_aliases.copy_alias(&source, bind_local);
            path_raw_views.copy(&source, bind_local);
            path_pending_reallocs.copy_result(&source, bind_local);
            path_variant_owner_effects.copy_result(&source, bind_local);
            path_variant_owner_effects.apply_match_arm_payload_conditions(
                path_raw_aliases,
                scrutinee,
                &arm.pattern,
                Some(bind_local),
            );
        } else {
            path_raw_aliases.clear(bind_local);
            path_raw_views.clear(bind_local);
            path_storage_origins.clear(bind_local);
            path_pending_reallocs.clear_result(bind_local);
            path_variant_owner_effects.clear_result(bind_local);
        }
    } else {
        path_variant_owner_effects.apply_match_arm_payload_conditions(
            path_raw_aliases,
            scrutinee,
            &arm.pattern,
            None,
        );
    }
    path_variant_owner_effects.apply_match_arm(
        path_engine,
        path_owners,
        path_raw_aliases,
        path_raw_views,
        path_storage_origins,
        scrutinee,
        &arm.pattern,
        span,
    );
}

#[derive(Debug, Clone)]
struct ConstructedVariant {
    variant: String,
    payloads: Vec<ConstructedVariantPayload>,
}

#[derive(Debug, Clone)]
struct ConstructedVariantPayload {
    suffix: Vec<PlaceProjection>,
    ty: crate::types::TypeId,
}

fn construct_variant_for_value(ops: &[ResourceOp], value: &Place) -> Option<ConstructedVariant> {
    for op in ops.iter().rev() {
        let ResourceOp::Construct {
            output,
            kind,
            inputs,
            ..
        } = op
        else {
            continue;
        };
        let AggregateKind::Enum { variant, .. } = kind else {
            continue;
        };
        if output != value {
            continue;
        }
        let mut payloads = Vec::new();
        for (index, input) in inputs.iter().enumerate() {
            let payload = construct_aggregate_field_place(output, kind, index, input);
            let suffix = place_suffix_after_prefix(&payload, output).unwrap_or_default();
            payloads.push(ConstructedVariantPayload {
                suffix,
                ty: input.ty,
            });
        }
        return Some(ConstructedVariant {
            variant: variant.clone(),
            payloads,
        });
    }
    None
}

fn collect_owner_variant_condition(
    out: &mut Vec<OwnerVariantCondition>,
    variant: &str,
    condition_fact: &ResourceConditionFact,
    truthy_path: bool,
    raw_aliases: &RawCellAddressAliases,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) {
    let Some(condition) = owner_value_condition(
        condition_fact,
        truthy_path,
        raw_aliases,
        parameter_storage_sources,
    ) else {
        return;
    };
    push_unique_variant_condition(
        out,
        OwnerVariantCondition {
            variant: normalize_variant_name(variant),
            condition,
        },
    );
}

fn owner_value_condition(
    condition_fact: &ResourceConditionFact,
    truthy_path: bool,
    raw_aliases: &RawCellAddressAliases,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) -> Option<OwnerValueCondition> {
    if let Some((place, condition)) = simple_condition_value_constraint(condition_fact, truthy_path)
    {
        return owner_param_value_condition(
            place,
            condition,
            raw_aliases,
            parameter_storage_sources,
        );
    }
    match (condition_fact, truthy_path) {
        (ResourceConditionFact::Any(facts), true) => {
            let mut conditions = Vec::new();
            for fact in facts {
                conditions.push(owner_value_condition(
                    fact,
                    truthy_path,
                    raw_aliases,
                    parameter_storage_sources,
                )?);
            }
            Some(OwnerValueCondition::Any(conditions))
        }
        (ResourceConditionFact::All(facts), true) => {
            let mut conditions = Vec::new();
            for fact in facts {
                conditions.push(owner_value_condition(
                    fact,
                    truthy_path,
                    raw_aliases,
                    parameter_storage_sources,
                )?);
            }
            Some(OwnerValueCondition::All(conditions))
        }
        (ResourceConditionFact::Any(facts), false) => {
            let mut conditions = Vec::new();
            for fact in facts {
                conditions.push(owner_value_condition(
                    fact,
                    truthy_path,
                    raw_aliases,
                    parameter_storage_sources,
                )?);
            }
            Some(OwnerValueCondition::All(conditions))
        }
        (ResourceConditionFact::All(facts), false) => {
            let mut conditions = Vec::new();
            for fact in facts {
                conditions.push(owner_value_condition(
                    fact,
                    truthy_path,
                    raw_aliases,
                    parameter_storage_sources,
                )?);
            }
            Some(OwnerValueCondition::Any(conditions))
        }
        (ResourceConditionFact::EqZero { .. }, _)
        | (ResourceConditionFact::NeZero { .. }, _)
        | (ResourceConditionFact::Positive { .. }, _)
        | (ResourceConditionFact::NonPositive { .. }, _)
        | (ResourceConditionFact::Negative { .. }, _)
        | (ResourceConditionFact::NonNegative { .. }, _) => None,
    }
}

fn owner_param_value_condition(
    place: &Place,
    condition: I32ValueCondition,
    raw_aliases: &RawCellAddressAliases,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) -> Option<OwnerValueCondition> {
    for place_alias in raw_aliases.aliases_for(place) {
        for source in parameter_storage_sources {
            for param_alias in raw_aliases.aliases_for(&source.place) {
                let Some(suffix) = place_suffix_after_prefix(&place_alias, &param_alias) else {
                    continue;
                };
                return Some(OwnerValueCondition::Param {
                    source: OwnerProjectionSource {
                        parameter_index: source.source.parameter_index,
                        suffix,
                        ty: place_alias.ty,
                    },
                    condition,
                });
            }
        }
    }
    None
}

fn collect_owner_variant_payload_conditions(
    out: &mut Vec<OwnerVariantPayloadCondition>,
    constructed_variant: &ConstructedVariant,
    value: &Place,
    condition_fact: &ResourceConditionFact,
    truthy_path: bool,
    raw_aliases: &RawCellAddressAliases,
) {
    let Some((_place, condition)) = simple_condition_value_constraint(condition_fact, truthy_path)
    else {
        return;
    };
    for payload in &constructed_variant.payloads {
        let payload_place = place_with_suffix(value, &payload.suffix, payload.ty);
        if raw_aliases.i32_condition_truth(&payload_place, condition) == Some(true) {
            push_unique_variant_payload_condition(
                out,
                OwnerVariantPayloadCondition {
                    variant: normalize_variant_name(&constructed_variant.variant),
                    suffix: payload.suffix.clone(),
                    ty: payload.ty,
                    condition,
                },
            );
        }
    }
}

fn normalize_variant_name(variant: &str) -> String {
    String::from(variant.rsplit("::").next().unwrap_or(variant))
}

fn push_unique_variant_parameter_index(
    out: &mut Vec<OwnerVariantParameterIndex>,
    entry: OwnerVariantParameterIndex,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

fn push_unique_variant_projection_source(
    out: &mut Vec<OwnerVariantProjectionSource>,
    entry: OwnerVariantProjectionSource,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

fn push_unique_variant_condition(
    out: &mut Vec<OwnerVariantCondition>,
    entry: OwnerVariantCondition,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

fn push_unique_variant_payload_condition(
    out: &mut Vec<OwnerVariantPayloadCondition>,
    entry: OwnerVariantPayloadCondition,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}
