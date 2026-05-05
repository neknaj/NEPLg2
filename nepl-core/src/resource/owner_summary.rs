use crate::types::TypeCtx;
use alloc::vec::Vec;

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummary;
use super::model::{OwnerState, ResourceFunction, ResourceModule, ResourceTerminator};
use super::owner_alias::{aliased_owner_descendant_entries, resolve_owner_alias_place};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_state::OwnerTable;
use super::owner_summary_cleanup::{
    remove_variant_consumed_parameter_sources,
    remove_variant_consumptions_returned_by_same_variant, remove_variant_projection_return_sources,
};
use super::owner_summary_leaf::owner_parameter_leaf_places;
use super::owner_summary_record::{
    owner_source_for_storage, push_unique_owner_projection_source, record_projection_marker,
    record_projection_maybe_owner_return, record_projection_owner_return, record_root_owner_return,
    OwnerParameterStorageSource, OwnerParameterValueSource,
};
use super::owner_summary_variant_build::collect_variant_consumed_owner_parameters_from_return;
use super::owner_variant::PendingVariantOwnerEffects;
use super::place_utils::{place_suffix_after_prefix, push_unique_usize};
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceOwnerCheckDeferred;
use super::storage_origin::StorageOriginTable;
use super::summary::{OwnerProjectionSource, OwnerReturnSummary};

pub(super) fn compute_owner_return_summaries(
    module: &ResourceModule,
    types: &TypeCtx,
    raw_alias_summaries: &[RawCellAddressReturnSummary],
) -> Vec<OwnerReturnSummary> {
    let mut summaries = Vec::new();
    for _ in 0..=module.functions.len() {
        let mut next = Vec::new();
        for function in &module.functions {
            let summary =
                function_owner_return_summary(function, types, raw_alias_summaries, &summaries);
            if summary.returns_fresh_owner
                || summary.returns_maybe_owner
                || !summary.parameter_indices.is_empty()
                || !summary.parameter_sources.is_empty()
                || !summary.consumed_parameter_indices.is_empty()
                || !summary.consumed_parameter_sources.is_empty()
                || !summary.variant_consumed_parameter_indices.is_empty()
                || !summary.variant_consumed_parameter_sources.is_empty()
                || !summary.variant_projection_returns.is_empty()
                || !summary.variant_conditions.is_empty()
                || !summary.variant_payload_conditions.is_empty()
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
    raw_alias_summaries: &[RawCellAddressReturnSummary],
    summaries: &[OwnerReturnSummary],
) -> OwnerReturnSummary {
    let mut engine = ResourceOwnerCheckEngine {
        function: function.name.as_str(),
        types,
        raw_alias_summaries,
        summaries,
        diagnostics: Vec::new(),
        deferred: ResourceOwnerCheckDeferred::default(),
    };
    let mut owners = OwnerTable::default();
    let mut raw_aliases = RawCellAddressAliases::default();
    let mut raw_views = RawAddressViewTable::default();
    let mut storage_origins = StorageOriginTable::default();
    let mut parameter_storage_sources = Vec::new();
    let parameter_value_sources = function
        .params
        .iter()
        .enumerate()
        .map(|(parameter_index, param)| OwnerParameterValueSource {
            source: OwnerProjectionSource {
                parameter_index,
                suffix: Vec::new(),
                ty: param.place.ty,
            },
            place: param.place.clone(),
        })
        .collect::<Vec<_>>();
    for leaf in owner_parameter_leaf_places(types, function) {
        owners.allocate(&leaf.place);
        raw_aliases.mark(&leaf.place);
        storage_origins.mark_owned(&leaf.place);
        if let Some(OwnerState::Live { storage }) = owners.state(&leaf.place) {
            parameter_storage_sources.push(OwnerParameterStorageSource {
                storage,
                source: OwnerProjectionSource {
                    parameter_index: leaf.parameter_index,
                    suffix: leaf.suffix,
                    ty: leaf.place.ty,
                },
                place: leaf.place,
            });
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
    let mut variant_projection_returns = Vec::new();
    let mut variant_conditions = Vec::new();
    let mut variant_payload_conditions = Vec::new();
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
                &mut variant_conditions,
                &mut variant_payload_conditions,
                function,
                types,
                raw_alias_summaries,
                summaries,
                &parameter_storage_sources,
                &parameter_value_sources,
                &block.ops,
                value,
                &mut variant_projection_returns,
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
                Some(
                    OwnerState::NoFreeObligation
                    | OwnerState::Reserved { .. }
                    | OwnerState::Moved
                    | OwnerState::Freed,
                )
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
                        OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed => {}
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
                    OwnerState::Reserved { .. } | OwnerState::Moved | OwnerState::Freed => {}
                }
            }
        }
    }

    remove_variant_consumptions_returned_by_same_variant(
        &mut variant_consumed_parameter_indices,
        &mut variant_consumed_parameter_sources,
        &variant_projection_returns,
    );
    remove_variant_projection_return_sources(&mut projection_returns, &variant_projection_returns);

    let (mut consumed_parameter_indices, mut consumed_parameter_sources) =
        consumed_owner_parameters(&owners, &parameter_storage_sources, &returned_sources);
    remove_variant_consumed_parameter_sources(
        &mut consumed_parameter_indices,
        &mut consumed_parameter_sources,
        &variant_consumed_parameter_indices,
        &variant_consumed_parameter_sources,
    );
    OwnerReturnSummary {
        function: function.name.clone(),
        parameter_indices,
        parameter_sources,
        consumed_parameter_indices,
        consumed_parameter_sources,
        variant_consumed_parameter_indices,
        variant_consumed_parameter_sources,
        variant_projection_returns,
        variant_conditions,
        variant_payload_conditions,
        returns_fresh_owner,
        returns_maybe_owner,
        projection_returns,
        projection_markers,
    }
}

pub(super) fn consumed_owner_parameters(
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
            Some(
                OwnerState::Live { .. }
                | OwnerState::Reserved { .. }
                | OwnerState::MaybeFreed { .. },
            )
            | None => {}
        }
    }
    (indices, sources)
}
