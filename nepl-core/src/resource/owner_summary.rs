use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, ResourceFunction, ResourceModule, ResourceTerminator};
use super::owner_alias::{aliased_owner_descendant_entries, resolve_owner_alias_place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_extent::{
    merge_owner_extent_summaries, summarize_consumed_extent_requirements,
    summarize_owner_storage_extent_for_owner,
};
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_summary_consumed::consumed_owner_parameters;
use super::owner_summary_parameters::seed_owner_summary_parameters;
use super::owner_summary_raw_view_return::{
    record_non_owning_raw_view_returns, returned_projection_is_non_owning_raw_view,
};
use super::owner_summary_record::{
    owner_source_for_storage, record_projection_marker, record_projection_maybe_owner_return,
    record_projection_owner_return, record_root_owner_return,
};
use super::owner_summary_resolved_variant::collect_resolved_parameter_variants_from_return;
use super::owner_summary_size_return::record_size_returns;
use super::owner_summary_storage_origin::record_storage_origin_marker;
use super::owner_summary_type_params::owner_summary_type_params;
use super::owner_summary_update::update_owner_return_summary;
use super::owner_summary_variant_build::collect_variant_consumed_owner_parameters_from_return;
use super::owner_summary_variant_projection::finalize_variant_projection_returns;
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::place_suffix_after_prefix;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceOwnerCheckDeferred;
use super::storage_origin::StorageOriginTable;
use super::summary::{OwnerExtentSummary, OwnerReturnSummary, OwnerReturnSummaryIndex};
use super::summary_worklist::SummaryWorklist;

pub(super) fn compute_owner_return_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
) -> Vec<OwnerReturnSummary> {
    let mut worklist = SummaryWorklist::new(module);
    let mut summaries = Vec::new();
    while let Some(function_index) = worklist.pop() {
        let summary_index = OwnerReturnSummaryIndex::new(&summaries);
        let summary =
            function_owner_return_summary(&module.functions[function_index], types, &summary_index);
        if update_owner_return_summary(&mut summaries, summary) {
            worklist.notify_changed(function_index);
        }
    }
    #[cfg(all(not(target_os = "none"), not(target_arch = "wasm32")))]
    if std::env::var_os("NEPL_COMPILE_STAGE_TIMING").is_some() {
        std::eprintln!(
            "[compile-stage] resource_owner_summary_recomputations={} summaries={}",
            worklist.recomputations(),
            summaries.len()
        );
    }
    summaries
}

fn function_owner_return_summary(
    function: &ResourceFunction,
    types: &TypeCtx,
    summaries: &OwnerReturnSummaryIndex<'_>,
) -> OwnerReturnSummary {
    let mut engine = ResourceOwnerCheckEngine {
        function: function.name.as_str(),
        types,
        summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
        owner_extent_requirements: Vec::new(),
        memory_span_requirements: Vec::new(),
        params: &function.params,
    };
    let mut owners = OwnerTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut raw_views = RawAddressViewTable::default();
    let mut storage_origins = StorageOriginTable::default();
    let (parameter_storage_sources, parameter_condition_sources) = seed_owner_summary_parameters(
        types,
        function,
        summaries,
        &mut owners,
        &mut raw_aliases,
        &mut storage_origins,
    );

    let mut parameter_indices = Vec::new();
    let mut parameter_sources = Vec::new();
    let mut parameter_return_extents = Vec::new();
    let mut returns_fresh_owner = false;
    let mut returns_maybe_owner = false;
    let mut non_owning_raw_view_returns = Vec::new();
    let mut projection_returns = Vec::new();
    let mut projection_markers = Vec::new();
    let mut storage_origin_markers = Vec::new();
    let mut host_size_returns = Vec::new();
    let mut type_size_returns = Vec::new();
    let mut variant_consumed_parameter_indices = Vec::new();
    let mut variant_consumed_parameter_sources = Vec::new();
    let mut variant_consumed_extent_requirements = Vec::new();
    let mut variant_projection_returns = Vec::new();
    let mut resolved_parameter_variants = Vec::new();
    let mut variant_conditions = Vec::new();
    let mut variant_payload_conditions = Vec::new();
    let mut returned_sources = Vec::new();
    let mut returns_fresh_owner_extent = OwnerExtentSummary::Unknown;
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
                &mut variant_consumed_extent_requirements,
                &mut variant_conditions,
                &mut variant_payload_conditions,
                function,
                types,
                summaries,
                &parameter_storage_sources,
                &parameter_condition_sources,
                &block.ops,
                value,
                &mut host_size_returns,
                &mut type_size_returns,
                &mut variant_projection_returns,
            );
            collect_resolved_parameter_variants_from_return(
                &mut resolved_parameter_variants,
                function,
                types,
                &block.ops,
                value,
            );

            let resolved_value = resolve_owner_alias_place(&owners, &raw_aliases, value);
            if !returned_projection_is_non_owning_raw_view(&raw_views, value, &[], value.ty) {
                match owners.state(&resolved_value) {
                    Some(OwnerState::Live { storage, extent }) => {
                        if let Some(source) =
                            owner_source_for_storage(storage, &parameter_storage_sources)
                        {
                            record_root_owner_return(
                                &mut parameter_indices,
                                &mut parameter_sources,
                                &mut parameter_return_extents,
                                &mut returned_sources,
                                source,
                                summarize_owner_storage_extent_for_owner(
                                    &raw_aliases,
                                    &parameter_condition_sources,
                                    &resolved_value,
                                    &extent,
                                ),
                            );
                        } else {
                            let extent_summary = summarize_owner_storage_extent_for_owner(
                                &raw_aliases,
                                &parameter_condition_sources,
                                &resolved_value,
                                &extent,
                            );
                            if returns_fresh_owner {
                                returns_fresh_owner_extent = merge_owner_extent_summaries(
                                    returns_fresh_owner_extent,
                                    extent_summary,
                                );
                            } else {
                                returns_fresh_owner = true;
                                returns_fresh_owner_extent = extent_summary;
                            }
                        }
                    }
                    Some(OwnerState::MaybeFreed { storage }) => {
                        if let Some(source) = storage.and_then(|storage| {
                            owner_source_for_storage(storage, &parameter_storage_sources)
                        }) {
                            record_root_owner_return(
                                &mut parameter_indices,
                                &mut parameter_sources,
                                &mut parameter_return_extents,
                                &mut returned_sources,
                                source,
                                OwnerExtentSummary::Unknown,
                            );
                        } else {
                            returns_maybe_owner = true;
                        }
                    }
                    Some(
                        OwnerState::NoFreeObligation
                        | OwnerState::Reserved { .. }
                        | OwnerState::Moved
                        | OwnerState::Freed,
                    )
                    | None => {}
                }
            }
            for entry in owners.descendant_entries(&resolved_value) {
                if let Some(suffix) = place_suffix_after_prefix(&entry.place, &resolved_value) {
                    if returned_projection_is_non_owning_raw_view(
                        &raw_views,
                        value,
                        &suffix,
                        entry.place.ty,
                    ) {
                        continue;
                    }
                    match entry.state {
                        OwnerState::Live { storage, extent } => {
                            record_projection_owner_return(
                                &mut projection_returns,
                                suffix,
                                entry.place.ty,
                                storage,
                                summarize_owner_storage_extent_for_owner(
                                    &raw_aliases,
                                    &parameter_condition_sources,
                                    &entry.place,
                                    &extent,
                                ),
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
                                    OwnerExtentSummary::Unknown,
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
                        OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed => {}
                    }
                }
            }
            for aliased in aliased_owner_descendant_entries(&owners, &raw_aliases, &resolved_value)
            {
                if returned_projection_is_non_owning_raw_view(
                    &raw_views,
                    value,
                    &aliased.suffix,
                    aliased.entry.place.ty,
                ) {
                    continue;
                }
                match aliased.entry.state {
                    OwnerState::Live { storage, extent } => {
                        record_projection_owner_return(
                            &mut projection_returns,
                            aliased.suffix,
                            aliased.entry.place.ty,
                            storage,
                            summarize_owner_storage_extent_for_owner(
                                &raw_aliases,
                                &parameter_condition_sources,
                                &aliased.entry.place,
                                &extent,
                            ),
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
                                OwnerExtentSummary::Unknown,
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
                    OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed => {}
                }
            }
            record_non_owning_raw_view_returns(&raw_views, value, &mut non_owning_raw_view_returns);
            if resolved_value != *value {
                record_non_owning_raw_view_returns(
                    &raw_views,
                    &resolved_value,
                    &mut non_owning_raw_view_returns,
                );
            }
            record_size_returns(
                &mut host_size_returns,
                &mut type_size_returns,
                &raw_aliases,
                value,
                &resolved_value,
            );
            for entry in storage_origins.entries_under(&resolved_value) {
                if let Some(suffix) = place_suffix_after_prefix(&entry.place, &resolved_value) {
                    if returned_projection_is_non_owning_raw_view(
                        &raw_views,
                        value,
                        &suffix,
                        entry.place.ty,
                    ) {
                        continue;
                    }
                    let origin_place = storage_origins
                        .origin_source(&entry.place)
                        .unwrap_or_else(|| entry.place.clone());
                    let resolved_origin_place =
                        resolve_owner_alias_place(&owners, &raw_aliases, &origin_place);
                    match owners.state(&resolved_origin_place) {
                        Some(OwnerState::Live { storage, extent }) => {
                            record_projection_owner_return(
                                &mut projection_returns,
                                suffix,
                                entry.place.ty,
                                storage,
                                summarize_owner_storage_extent_for_owner(
                                    &raw_aliases,
                                    &parameter_condition_sources,
                                    &resolved_origin_place,
                                    &extent,
                                ),
                                &parameter_storage_sources,
                                &mut returned_sources,
                            );
                            continue;
                        }
                        Some(OwnerState::MaybeFreed {
                            storage: Some(storage),
                        }) => {
                            record_projection_owner_return(
                                &mut projection_returns,
                                suffix,
                                entry.place.ty,
                                storage,
                                OwnerExtentSummary::Unknown,
                                &parameter_storage_sources,
                                &mut returned_sources,
                            );
                            continue;
                        }
                        Some(OwnerState::MaybeFreed { storage: None }) => {
                            record_projection_maybe_owner_return(
                                &mut projection_returns,
                                suffix,
                                entry.place.ty,
                            );
                            continue;
                        }
                        Some(
                            OwnerState::NoFreeObligation
                            | OwnerState::Reserved { .. }
                            | OwnerState::Moved
                            | OwnerState::Freed,
                        )
                        | None => {}
                    }
                    record_storage_origin_marker(
                        &mut storage_origin_markers,
                        suffix,
                        entry.place.ty,
                        entry.origin,
                    );
                }
            }
        }
    }

    finalize_variant_projection_returns(
        &mut projection_returns,
        &mut returned_sources,
        &mut variant_projection_returns,
        &parameter_storage_sources,
    );

    let (consumed_parameter_indices, consumed_parameter_sources) =
        consumed_owner_parameters(&owners, &parameter_storage_sources, &returned_sources);
    let consumed_extent_requirements = summarize_consumed_extent_requirements(
        &raw_aliases,
        &parameter_storage_sources,
        &parameter_condition_sources,
        &engine.owner_extent_requirements,
        &consumed_parameter_indices,
        &consumed_parameter_sources,
    );
    let memory_span_requirements = engine.memory_span_requirements.clone();
    OwnerReturnSummary {
        function: function.name.clone(),
        type_params: owner_summary_type_params(types, function),
        parameter_indices,
        parameter_sources,
        parameter_return_extents,
        consumed_parameter_indices,
        consumed_parameter_sources,
        consumed_extent_requirements,
        memory_span_requirements,
        host_size_returns,
        type_size_returns,
        variant_consumed_parameter_indices,
        variant_consumed_parameter_sources,
        variant_consumed_extent_requirements,
        variant_projection_returns,
        resolved_parameter_variants,
        variant_conditions,
        variant_payload_conditions,
        non_owning_raw_view_returns,
        returns_fresh_owner,
        returns_fresh_owner_extent,
        returns_maybe_owner,
        projection_returns,
        projection_markers,
        storage_origin_markers,
    }
}
