use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::function_alias::FunctionAliasTable;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{
    OwnerState, Place, PlaceProjection, ResourceFunction, ResourceModule, ResourceTerminator,
    StorageId,
};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_state::OwnerTable;
use super::owner_summary_leaf::owner_leaf_places;
use super::place_utils::{place_suffix_after_prefix, push_unique_usize};
use super::report::ResourceOwnerCheckDeferred;
use super::storage_origin::StorageOriginTable;
use super::summary::{
    OwnerProjectionMarker, OwnerProjectionReturnSummary, OwnerProjectionSource, OwnerReturnSummary,
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
    let mut returned_sources = Vec::new();
    let mut function_aliases = FunctionAliasTable::default();
    for block in &function.blocks {
        engine.check_ops(
            &mut owners,
            &mut function_aliases,
            &mut raw_aliases,
            &mut storage_origins,
            &block.ops,
        );
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
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
            Some(OwnerState::Moved | OwnerState::Freed | OwnerState::MaybeFreed { .. }) => {
                if source.suffix.is_empty() {
                    push_unique_usize(&mut indices, source.parameter_index);
                } else {
                    push_unique_owner_projection_source(&mut sources, source);
                }
            }
            Some(OwnerState::Live { .. } | OwnerState::NoFreeObligation) | None => {}
        }
    }
    (indices, sources)
}

fn record_projection_owner_return(
    projection_returns: &mut Vec<OwnerProjectionReturnSummary>,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
    storage: StorageId,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    returned_sources: &mut Vec<OwnerProjectionSource>,
) {
    let entry_index = projection_returns
        .iter()
        .position(|entry| entry.suffix == suffix && entry.ty == ty)
        .unwrap_or_else(|| {
            projection_returns.push(OwnerProjectionReturnSummary {
                suffix: suffix.clone(),
                ty,
                parameter_indices: Vec::new(),
                parameter_sources: Vec::new(),
                returns_fresh_owner: false,
                returns_maybe_owner: false,
            });
            projection_returns.len() - 1
        });
    if let Some(source) = owner_source_for_storage(storage, parameter_storage_sources) {
        record_projection_owner_source(
            &mut projection_returns[entry_index],
            returned_sources,
            source,
        );
    } else {
        projection_returns[entry_index].returns_fresh_owner = true;
    }
}

fn record_projection_maybe_owner_return(
    projection_returns: &mut Vec<OwnerProjectionReturnSummary>,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
) {
    let entry_index = projection_returns
        .iter()
        .position(|entry| entry.suffix == suffix && entry.ty == ty)
        .unwrap_or_else(|| {
            projection_returns.push(OwnerProjectionReturnSummary {
                suffix: suffix.clone(),
                ty,
                parameter_indices: Vec::new(),
                parameter_sources: Vec::new(),
                returns_fresh_owner: false,
                returns_maybe_owner: false,
            });
            projection_returns.len() - 1
        });
    projection_returns[entry_index].returns_maybe_owner = true;
}

fn record_projection_marker(
    projection_markers: &mut Vec<OwnerProjectionMarker>,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
) {
    if !projection_markers
        .iter()
        .any(|marker| marker.suffix == suffix && marker.ty == ty)
    {
        projection_markers.push(OwnerProjectionMarker { suffix, ty });
    }
}

fn record_root_owner_return(
    parameter_indices: &mut Vec<usize>,
    parameter_sources: &mut Vec<OwnerProjectionSource>,
    returned_sources: &mut Vec<OwnerProjectionSource>,
    source: &OwnerProjectionSource,
) {
    if source.suffix.is_empty() {
        push_unique_usize(parameter_indices, source.parameter_index);
    } else {
        push_unique_owner_projection_source(parameter_sources, source);
    }
    push_unique_owner_projection_source(returned_sources, source);
}

fn record_projection_owner_source(
    summary: &mut OwnerProjectionReturnSummary,
    returned_sources: &mut Vec<OwnerProjectionSource>,
    source: &OwnerProjectionSource,
) {
    if source.suffix.is_empty() {
        push_unique_usize(&mut summary.parameter_indices, source.parameter_index);
    } else {
        push_unique_owner_projection_source(&mut summary.parameter_sources, source);
    }
    push_unique_owner_projection_source(returned_sources, source);
}

fn owner_source_for_storage<'a>(
    storage: StorageId,
    parameter_storage_sources: &'a [OwnerParameterStorageSource],
) -> Option<&'a OwnerProjectionSource> {
    parameter_storage_sources
        .iter()
        .find_map(|source| (source.storage == storage).then_some(&source.source))
}

fn push_unique_owner_projection_source(
    sources: &mut Vec<OwnerProjectionSource>,
    source: &OwnerProjectionSource,
) {
    if !sources.iter().any(|existing| existing == source) {
        sources.push(source.clone());
    }
}

struct OwnerParameterStorageSource {
    storage: StorageId,
    source: OwnerProjectionSource,
    place: Place,
}
