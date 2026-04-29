use alloc::vec::Vec;

use crate::types::TypeId;

use super::model::{Place, PlaceProjection, StorageId};
use super::place_utils::push_unique_usize;
use super::summary::{OwnerProjectionMarker, OwnerProjectionReturnSummary, OwnerProjectionSource};

pub(super) fn record_projection_owner_return(
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

pub(super) fn record_projection_maybe_owner_return(
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

pub(super) fn record_projection_marker(
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

pub(super) fn record_root_owner_return(
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

pub(super) fn owner_source_for_storage<'a>(
    storage: StorageId,
    parameter_storage_sources: &'a [OwnerParameterStorageSource],
) -> Option<&'a OwnerProjectionSource> {
    parameter_storage_sources
        .iter()
        .find_map(|source| (source.storage == storage).then_some(&source.source))
}

pub(super) fn push_unique_owner_projection_source(
    sources: &mut Vec<OwnerProjectionSource>,
    source: &OwnerProjectionSource,
) {
    if !sources.iter().any(|existing| existing == source) {
        sources.push(source.clone());
    }
}

pub(super) struct OwnerParameterStorageSource {
    pub(super) storage: StorageId,
    pub(super) source: OwnerProjectionSource,
    pub(super) place: Place,
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
