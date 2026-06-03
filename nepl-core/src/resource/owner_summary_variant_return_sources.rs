use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_alias::{aliased_owner_descendant_entries, resolve_owner_alias_place};
use super::owner_extent::summarize_owner_storage_extent_for_owner;
use super::owner_state::OwnerTable;
use super::owner_summary_record::{
    owner_source_for_storage, record_projection_owner_return, record_root_owner_return,
    OwnerParameterConditionSource, OwnerParameterStorageSource,
};
use super::place_utils::place_suffix_after_prefix;
use super::summary::{OwnerExtentSummary, OwnerProjectionReturnSummary, OwnerProjectionSource};

pub(super) fn returned_owner_returns_for_value(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    parameter_condition_sources: &[OwnerParameterConditionSource],
) -> (
    Vec<OwnerProjectionReturnSummary>,
    Vec<OwnerProjectionSource>,
) {
    let mut projection_returns = Vec::new();
    let mut returned_sources = Vec::new();
    let mut unused_indices = Vec::new();
    let mut unused_sources = Vec::new();
    let mut unused_return_extents = Vec::new();
    let resolved_value = resolve_owner_alias_place(owners, raw_aliases, value);
    record_returned_owner_returns_under_root(
        owners,
        raw_aliases,
        value,
        parameter_storage_sources,
        parameter_condition_sources,
        &mut projection_returns,
        &mut returned_sources,
        &mut unused_indices,
        &mut unused_sources,
        &mut unused_return_extents,
    );
    if resolved_value != *value {
        record_returned_owner_returns_under_root(
            owners,
            raw_aliases,
            &resolved_value,
            parameter_storage_sources,
            parameter_condition_sources,
            &mut projection_returns,
            &mut returned_sources,
            &mut unused_indices,
            &mut unused_sources,
            &mut unused_return_extents,
        );
    }
    (projection_returns, returned_sources)
}

#[allow(clippy::too_many_arguments)]
fn record_returned_owner_returns_under_root(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    root: &Place,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    parameter_condition_sources: &[OwnerParameterConditionSource],
    projection_returns: &mut Vec<OwnerProjectionReturnSummary>,
    returned_sources: &mut Vec<OwnerProjectionSource>,
    unused_indices: &mut Vec<usize>,
    unused_sources: &mut Vec<OwnerProjectionSource>,
    unused_return_extents: &mut Vec<super::summary::OwnerParameterReturnExtent>,
) {
    match owners.state(root) {
        Some(OwnerState::Live { storage, extent }) => {
            if let Some(source) = owner_source_for_storage(storage, parameter_storage_sources) {
                record_root_owner_return(
                    unused_indices,
                    unused_sources,
                    unused_return_extents,
                    returned_sources,
                    source,
                    summarize_owner_storage_extent_for_owner(
                        raw_aliases,
                        parameter_condition_sources,
                        root,
                        &extent,
                    ),
                );
            }
        }
        Some(OwnerState::MaybeFreed {
            storage: Some(storage),
        }) => {
            if let Some(source) = owner_source_for_storage(storage, parameter_storage_sources) {
                record_root_owner_return(
                    unused_indices,
                    unused_sources,
                    unused_return_extents,
                    returned_sources,
                    source,
                    OwnerExtentSummary::Unknown,
                );
            }
        }
        Some(OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed)
        | Some(OwnerState::Reserved { .. })
        | Some(OwnerState::MaybeFreed { storage: None })
        | None => {}
    }
    for entry in owners.descendant_entries(root) {
        if let Some(suffix) = place_suffix_after_prefix(&entry.place, root) {
            match &entry.state {
                OwnerState::Live { storage, extent } => {
                    record_projection_owner_return(
                        projection_returns,
                        suffix,
                        entry.place.ty,
                        *storage,
                        summarize_owner_storage_extent_for_owner(
                            raw_aliases,
                            parameter_condition_sources,
                            &entry.place,
                            extent,
                        ),
                        parameter_storage_sources,
                        returned_sources,
                    );
                }
                OwnerState::MaybeFreed {
                    storage: Some(storage),
                } => {
                    record_projection_owner_return(
                        projection_returns,
                        suffix,
                        entry.place.ty,
                        *storage,
                        OwnerExtentSummary::Unknown,
                        parameter_storage_sources,
                        returned_sources,
                    );
                }
                OwnerState::NoFreeObligation
                | OwnerState::Reserved { .. }
                | OwnerState::Moved
                | OwnerState::Freed
                | OwnerState::MaybeFreed { storage: None } => {}
            }
        }
    }
    for aliased in aliased_owner_descendant_entries(owners, raw_aliases, root) {
        match &aliased.entry.state {
            OwnerState::Live { storage, extent } => {
                record_projection_owner_return(
                    projection_returns,
                    aliased.suffix,
                    aliased.entry.place.ty,
                    *storage,
                    summarize_owner_storage_extent_for_owner(
                        raw_aliases,
                        parameter_condition_sources,
                        &aliased.entry.place,
                        extent,
                    ),
                    parameter_storage_sources,
                    returned_sources,
                );
            }
            OwnerState::MaybeFreed {
                storage: Some(storage),
            } => {
                record_projection_owner_return(
                    projection_returns,
                    aliased.suffix,
                    aliased.entry.place.ty,
                    *storage,
                    OwnerExtentSummary::Unknown,
                    parameter_storage_sources,
                    returned_sources,
                );
            }
            OwnerState::NoFreeObligation
            | OwnerState::Reserved { .. }
            | OwnerState::Moved
            | OwnerState::Freed
            | OwnerState::MaybeFreed { storage: None } => {}
        }
    }
}
