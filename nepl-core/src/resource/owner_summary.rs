use alloc::string::String;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeCtx;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    AggregateKind, OwnerState, Place, ResourceFunction, ResourceMatchPattern, ResourceModule,
    ResourceOp, ResourceTerminator,
};
use super::owner_alias::{aliased_owner_descendant_entries, resolve_owner_alias_place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_summary_leaf::owner_leaf_places;
use super::owner_summary_record::{
    owner_source_for_storage, push_unique_owner_projection_source, record_projection_marker,
    record_projection_maybe_owner_return, record_projection_owner_return, record_root_owner_return,
    OwnerParameterStorageSource,
};
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::{place_suffix_after_prefix, push_unique_usize};
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceOwnerCheckDeferred;
use super::storage_origin::StorageOriginTable;
use super::summary::{
    OwnerProjectionSource, OwnerReturnSummary, OwnerVariantParameterIndex,
    OwnerVariantProjectionSource,
};

pub(super) fn compute_owner_return_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
) -> Vec<OwnerReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let summary = function_owner_return_summary(function, types, &summaries);
            if summary.returns_fresh_owner
                || summary.returns_maybe_owner
                || !summary.parameter_indices.is_empty()
                || !summary.parameter_sources.is_empty()
                || !summary.consumed_parameter_indices.is_empty()
                || !summary.consumed_parameter_sources.is_empty()
                || !summary.variant_consumed_parameter_indices.is_empty()
                || !summary.variant_consumed_parameter_sources.is_empty()
                || !summary.projection_returns.is_empty()
                || !summary.projection_markers.is_empty()
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

fn function_owner_return_summary(
    function: &ResourceFunction,
    types: &TypeCtx,
    summaries: &[OwnerReturnSummary],
) -> OwnerReturnSummary {
    let mut engine = ResourceOwnerCheckEngine {
        function: function.name.as_str(),
        types,
        summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
    };
    let mut owners = OwnerTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut raw_views = RawAddressViewTable::default();
    let mut storage_origins = StorageOriginTable::default();
    let mut parameter_storage_sources = Vec::new();
    for (index, param) in function.params.iter().enumerate() {
        for leaf in owner_leaf_places(types, &param.place) {
            owners.allocate(&leaf.place);
            raw_aliases.mark(&leaf.place);
            storage_origins.mark_owned(&leaf.place);
            if let Some(OwnerState::Live { storage }) = owners.state(&leaf.place) {
                parameter_storage_sources.push(OwnerParameterStorageSource {
                    storage,
                    source: OwnerProjectionSource {
                        parameter_index: index,
                        suffix: leaf.suffix,
                        ty: leaf.place.ty,
                    },
                    place: leaf.place,
                });
            }
        }
    }

    let mut parameter_indices = Vec::new();
    let mut parameter_sources = Vec::new();
    let mut returns_fresh_owner = false;
    let mut returns_maybe_owner = false;
    let mut projection_returns = Vec::new();
    let mut projection_markers = Vec::new();
    let mut variant_consumed_parameter_indices = Vec::new();
    let mut variant_consumed_parameter_sources = Vec::new();
    let mut returned_sources = Vec::new();
    let mut function_aliases = FunctionAliasTable::default();
    let mut pending_reallocs = PendingRawReallocs::default();
    let mut variant_owner_effects = PendingVariantOwnerEffects::default();
    for block in &function.blocks {
        engine.check_ops(
            &mut owners,
            &mut function_aliases,
            &mut raw_aliases,
            &mut raw_views,
            &mut storage_origins,
            &mut pending_reallocs,
            &mut variant_owner_effects,
            &block.ops,
        );
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            collect_variant_consumed_owner_parameters_from_return(
                &mut variant_consumed_parameter_indices,
                &mut variant_consumed_parameter_sources,
                function,
                types,
                summaries,
                &parameter_storage_sources,
                &block.ops,
                value,
            );

            let resolved_value = resolve_owner_alias_place(&owners, &raw_aliases, value);
            match owners.state(&resolved_value) {
                Some(OwnerState::Live { storage }) => {
                    if let Some(source) =
                        owner_source_for_storage(storage, &parameter_storage_sources)
                    {
                        record_root_owner_return(
                            &mut parameter_indices,
                            &mut parameter_sources,
                            &mut returned_sources,
                            source,
                        );
                    } else {
                        returns_fresh_owner = true;
                    }
                }
                Some(OwnerState::MaybeFreed { storage }) => {
                    if let Some(source) = storage.and_then(|storage| {
                        owner_source_for_storage(storage, &parameter_storage_sources)
                    }) {
                        record_root_owner_return(
                            &mut parameter_indices,
                            &mut parameter_sources,
                            &mut returned_sources,
                            source,
                        );
                    } else {
                        returns_maybe_owner = true;
                    }
                }
                Some(OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed)
                | None => {}
            }
            for entry in owners.descendant_entries(&resolved_value) {
                if let Some(suffix) = place_suffix_after_prefix(&entry.place, &resolved_value) {
                    match entry.state {
                        OwnerState::Live { storage } => {
                            record_projection_owner_return(
                                &mut projection_returns,
                                suffix,
                                entry.place.ty,
                                storage,
                                &parameter_storage_sources,
                                &mut returned_sources,
                            );
                        }
                        OwnerState::NoFreeObligation => {
                            record_projection_marker(
                                &mut projection_markers,
                                suffix,
                                entry.place.ty,
                            );
                        }
                        OwnerState::MaybeFreed { storage } => {
                            if let Some(storage) = storage {
                                record_projection_owner_return(
                                    &mut projection_returns,
                                    suffix,
                                    entry.place.ty,
                                    storage,
                                    &parameter_storage_sources,
                                    &mut returned_sources,
                                );
                            } else {
                                record_projection_maybe_owner_return(
                                    &mut projection_returns,
                                    suffix,
                                    entry.place.ty,
                                );
                            }
                        }
                        OwnerState::Moved | OwnerState::Freed => {}
                    }
                }
            }
            for aliased in aliased_owner_descendant_entries(&owners, &raw_aliases, &resolved_value)
            {
                match aliased.entry.state {
                    OwnerState::Live { storage } => {
                        record_projection_owner_return(
                            &mut projection_returns,
                            aliased.suffix,
                            aliased.entry.place.ty,
                            storage,
                            &parameter_storage_sources,
                            &mut returned_sources,
                        );
                    }
                    OwnerState::NoFreeObligation => {
                        record_projection_marker(
                            &mut projection_markers,
                            aliased.suffix,
                            aliased.entry.place.ty,
                        );
                    }
                    OwnerState::MaybeFreed { storage } => {
                        if let Some(storage) = storage {
                            record_projection_owner_return(
                                &mut projection_returns,
                                aliased.suffix,
                                aliased.entry.place.ty,
                                storage,
                                &parameter_storage_sources,
                                &mut returned_sources,
                            );
                        } else {
                            record_projection_maybe_owner_return(
                                &mut projection_returns,
                                aliased.suffix,
                                aliased.entry.place.ty,
                            );
                        }
                    }
                    OwnerState::Moved | OwnerState::Freed => {}
                }
            }
        }
    }

    let (consumed_parameter_indices, consumed_parameter_sources) =
        consumed_owner_parameters(&owners, &parameter_storage_sources, &returned_sources);
    OwnerReturnSummary {
        function: function.name.clone(),
        parameter_indices,
        parameter_sources,
        consumed_parameter_indices,
        consumed_parameter_sources,
        variant_consumed_parameter_indices,
        variant_consumed_parameter_sources,
        returns_fresh_owner,
        returns_maybe_owner,
        projection_returns,
        projection_markers,
    }
}

fn consumed_owner_parameters(
    owners: &OwnerTable,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    returned_sources: &[OwnerProjectionSource],
) -> (Vec<usize>, Vec<OwnerProjectionSource>) {
    let mut indices = Vec::new();
    let mut sources = Vec::new();
    for entry in parameter_storage_sources {
        let source = &entry.source;
        if returned_sources.iter().any(|returned| returned == source) {
            continue;
        }
        match owners.state(&entry.place) {
            Some(OwnerState::Moved | OwnerState::Freed) => {
                if source.suffix.is_empty() {
                    push_unique_usize(&mut indices, source.parameter_index);
                } else {
                    push_unique_owner_projection_source(&mut sources, source);
                }
            }
            Some(OwnerState::NoFreeObligation) => {
                if source.suffix.is_empty() {
                    push_unique_usize(&mut indices, source.parameter_index);
                } else {
                    push_unique_owner_projection_source(&mut sources, source);
                }
            }
            Some(OwnerState::Live { .. } | OwnerState::MaybeFreed { .. }) | None => {}
        }
    }
    (indices, sources)
}

fn collect_variant_consumed_owner_parameters_from_return(
    index_out: &mut Vec<OwnerVariantParameterIndex>,
    source_out: &mut Vec<OwnerVariantProjectionSource>,
    function: &ResourceFunction,
    types: &TypeCtx,
    summaries: &[OwnerReturnSummary],
    parameter_storage_sources: &[OwnerParameterStorageSource],
    ops: &[ResourceOp],
    return_value: &super::model::Place,
) {
    let mut engine = ResourceOwnerCheckEngine {
        function: function.name.as_str(),
        types,
        summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
    };
    let mut owners = OwnerTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut raw_views = RawAddressViewTable::default();
    let mut storage_origins = StorageOriginTable::default();
    let mut function_aliases = FunctionAliasTable::default();
    let mut pending_reallocs = PendingRawReallocs::default();
    let mut variant_owner_effects = PendingVariantOwnerEffects::default();
    for param in &function.params {
        for leaf in owner_leaf_places(types, &param.place) {
            owners.allocate(&leaf.place);
            raw_aliases.mark(&leaf.place);
            storage_origins.mark_owned(&leaf.place);
        }
    }

    for (index, op) in ops.iter().enumerate() {
        match op {
            ResourceOp::Branch {
                output,
                then_ops,
                then_value,
                else_ops,
                else_value,
                ..
            } if output == return_value => {
                collect_variant_consumed_owner_parameters_from_path(
                    index_out,
                    source_out,
                    &engine,
                    &owners,
                    &raw_aliases,
                    &raw_views,
                    &storage_origins,
                    &function_aliases,
                    &pending_reallocs,
                    &variant_owner_effects,
                    parameter_storage_sources,
                    then_ops,
                    then_value,
                    None,
                );
                collect_variant_consumed_owner_parameters_from_path(
                    index_out,
                    source_out,
                    &engine,
                    &owners,
                    &raw_aliases,
                    &raw_views,
                    &storage_origins,
                    &function_aliases,
                    &pending_reallocs,
                    &variant_owner_effects,
                    parameter_storage_sources,
                    else_ops,
                    else_value,
                    None,
                );
            }
            ResourceOp::Match {
                output,
                scrutinee,
                arms,
                span,
            } if output == return_value => {
                for arm in arms {
                    collect_variant_consumed_owner_parameters_from_path(
                        index_out,
                        source_out,
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
                        Some((scrutinee, &arm.pattern, *span)),
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
}

fn collect_variant_consumed_owner_parameters_from_path(
    index_out: &mut Vec<OwnerVariantParameterIndex>,
    source_out: &mut Vec<OwnerVariantProjectionSource>,
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
    match_arm: Option<(&Place, &ResourceMatchPattern, Span)>,
) {
    let Some(variant) = construct_variant_for_value(path_ops, path_value) else {
        return;
    };
    let mut path_engine = ResourceOwnerCheckEngine {
        function: engine.function,
        types: engine.types,
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
    if let Some((scrutinee, pattern, span)) = match_arm {
        path_variant_owner_effects.apply_match_arm(
            &mut path_engine,
            &mut path_owners,
            &mut path_raw_aliases,
            &mut path_raw_views,
            &mut path_storage_origins,
            scrutinee,
            pattern,
            span,
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

    let returned_sources = returned_owner_sources_for_value(
        &path_owners,
        &path_raw_aliases,
        path_value,
        parameter_storage_sources,
    );
    let (indices, sources) =
        consumed_owner_parameters(&path_owners, parameter_storage_sources, &returned_sources);
    let variant = normalize_variant_name(&variant);
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

fn returned_owner_sources_for_value(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    value: &super::model::Place,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) -> Vec<OwnerProjectionSource> {
    let mut returned_sources = Vec::new();
    let mut unused_indices = Vec::new();
    let mut unused_sources = Vec::new();
    let mut unused_projection_returns = Vec::new();
    let resolved_value = resolve_owner_alias_place(owners, raw_aliases, value);
    match owners.state(&resolved_value) {
        Some(OwnerState::Live { storage })
        | Some(OwnerState::MaybeFreed {
            storage: Some(storage),
        }) => {
            if let Some(source) = owner_source_for_storage(storage, parameter_storage_sources) {
                record_root_owner_return(
                    &mut unused_indices,
                    &mut unused_sources,
                    &mut returned_sources,
                    source,
                );
            }
        }
        Some(OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed)
        | Some(OwnerState::MaybeFreed { storage: None })
        | None => {}
    }
    for entry in owners.descendant_entries(&resolved_value) {
        if let Some(suffix) = place_suffix_after_prefix(&entry.place, &resolved_value) {
            if let OwnerState::Live { storage }
            | OwnerState::MaybeFreed {
                storage: Some(storage),
            } = entry.state
            {
                record_projection_owner_return(
                    &mut unused_projection_returns,
                    suffix,
                    entry.place.ty,
                    storage,
                    parameter_storage_sources,
                    &mut returned_sources,
                );
            }
        }
    }
    for aliased in aliased_owner_descendant_entries(owners, raw_aliases, &resolved_value) {
        if let OwnerState::Live { storage }
        | OwnerState::MaybeFreed {
            storage: Some(storage),
        } = aliased.entry.state
        {
            record_projection_owner_return(
                &mut unused_projection_returns,
                aliased.suffix,
                aliased.entry.place.ty,
                storage,
                parameter_storage_sources,
                &mut returned_sources,
            );
        }
    }
    returned_sources
}

fn construct_variant_for_value(ops: &[ResourceOp], value: &super::model::Place) -> Option<String> {
    for op in ops.iter().rev() {
        let ResourceOp::Construct {
            output,
            kind: AggregateKind::Enum { variant, .. },
            ..
        } = op
        else {
            continue;
        };
        if output == value {
            return Some(variant.clone());
        }
    }
    None
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
