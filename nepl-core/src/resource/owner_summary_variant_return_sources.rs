use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place};
use super::owner_alias::{aliased_owner_descendant_entries, resolve_owner_alias_place};
use super::owner_extent::summarize_owner_storage_extent;
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
    match owners.state(&resolved_value) {
        Some(OwnerState::Live { storage, extent }) => {
            if let Some(source) = owner_source_for_storage(storage, parameter_storage_sources) {
                record_root_owner_return(
                    &mut unused_indices,
                    &mut unused_sources,
                    &mut unused_return_extents,
                    &mut returned_sources,
                    source,
                    summarize_owner_storage_extent(
                        raw_aliases,
                        parameter_condition_sources,
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
                    &mut unused_indices,
                    &mut unused_sources,
                    &mut unused_return_extents,
                    &mut returned_sources,
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
    for entry in owners.descendant_entries(&resolved_value) {
        if let Some(suffix) = place_suffix_after_prefix(&entry.place, &resolved_value) {
            match &entry.state {
                OwnerState::Live { storage, extent } => {
                    record_projection_owner_return(
                        &mut projection_returns,
                        suffix,
                        entry.place.ty,
                        *storage,
                        summarize_owner_storage_extent(
                            raw_aliases,
                            parameter_condition_sources,
                            extent,
                        ),
                        parameter_storage_sources,
                        &mut returned_sources,
                    );
                }
                OwnerState::MaybeFreed {
                    storage: Some(storage),
                } => {
                    record_projection_owner_return(
                        &mut projection_returns,
                        suffix,
                        entry.place.ty,
                        *storage,
                        OwnerExtentSummary::Unknown,
                        parameter_storage_sources,
                        &mut returned_sources,
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
    for aliased in aliased_owner_descendant_entries(owners, raw_aliases, &resolved_value) {
        match &aliased.entry.state {
            OwnerState::Live { storage, extent } => {
                record_projection_owner_return(
                    &mut projection_returns,
                    aliased.suffix,
                    aliased.entry.place.ty,
                    *storage,
                    summarize_owner_storage_extent(
                        raw_aliases,
                        parameter_condition_sources,
                        extent,
                    ),
                    parameter_storage_sources,
                    &mut returned_sources,
                );
            }
            OwnerState::MaybeFreed {
                storage: Some(storage),
            } => {
                record_projection_owner_return(
                    &mut projection_returns,
                    aliased.suffix,
                    aliased.entry.place.ty,
                    *storage,
                    OwnerExtentSummary::Unknown,
                    parameter_storage_sources,
                    &mut returned_sources,
                );
            }
            OwnerState::NoFreeObligation
            | OwnerState::Reserved { .. }
            | OwnerState::Moved
            | OwnerState::Freed
            | OwnerState::MaybeFreed { storage: None } => {}
        }
    }
    (projection_returns, returned_sources)
}
