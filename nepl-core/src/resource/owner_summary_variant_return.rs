use alloc::string::String;
use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place, PlaceProjection};
use super::owner_alias::{aliased_owner_descendant_entries, resolve_owner_alias_place};
use super::owner_state::OwnerTable;
use super::owner_summary_record::{
    owner_source_for_storage, record_projection_owner_return, record_root_owner_return,
    OwnerParameterStorageSource,
};
use super::place_utils::place_suffix_after_prefix;
use super::summary::{
    OwnerProjectionReturnOwner, OwnerProjectionReturnSummary, OwnerProjectionSource,
    OwnerVariantProjectionReturn,
};

pub(super) fn returned_owner_returns_for_value(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    value: &Place,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) -> (
    Vec<OwnerProjectionReturnSummary>,
    Vec<OwnerProjectionSource>,
) {
    let mut projection_returns = Vec::new();
    let mut returned_sources = Vec::new();
    let mut unused_indices = Vec::new();
    let mut unused_sources = Vec::new();
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
        | Some(OwnerState::Reserved { .. })
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
                    &mut projection_returns,
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
                &mut projection_returns,
                aliased.suffix,
                aliased.entry.place.ty,
                storage,
                parameter_storage_sources,
                &mut returned_sources,
            );
        }
    }
    (projection_returns, returned_sources)
}

pub(super) fn record_variant_projection_returns(
    out: &mut Vec<OwnerVariantProjectionReturn>,
    variant: &str,
    projection_returns: &[OwnerProjectionReturnSummary],
    parameter_storage_sources: &[OwnerParameterStorageSource],
) {
    let variant = normalize_variant_name(variant);
    for projection in projection_returns {
        if !projection_targets_variant(projection, &variant) {
            continue;
        }
        for parameter_index in &projection.parameter_indices {
            let Some(source) = root_parameter_source(parameter_storage_sources, *parameter_index)
            else {
                continue;
            };
            push_unique_variant_projection_return(
                out,
                OwnerVariantProjectionReturn {
                    variant: variant.clone(),
                    suffix: projection.suffix.clone(),
                    ty: projection.ty,
                    owner: OwnerProjectionReturnOwner::Parameter(source),
                },
            );
        }
        for source in &projection.parameter_sources {
            push_unique_variant_projection_return(
                out,
                OwnerVariantProjectionReturn {
                    variant: variant.clone(),
                    suffix: projection.suffix.clone(),
                    ty: projection.ty,
                    owner: OwnerProjectionReturnOwner::Parameter(source.clone()),
                },
            );
        }
        if projection.returns_fresh_owner {
            push_unique_variant_projection_return(
                out,
                OwnerVariantProjectionReturn {
                    variant: variant.clone(),
                    suffix: projection.suffix.clone(),
                    ty: projection.ty,
                    owner: OwnerProjectionReturnOwner::Fresh,
                },
            );
        }
        if projection.returns_maybe_owner {
            push_unique_variant_projection_return(
                out,
                OwnerVariantProjectionReturn {
                    variant: variant.clone(),
                    suffix: projection.suffix.clone(),
                    ty: projection.ty,
                    owner: OwnerProjectionReturnOwner::Maybe,
                },
            );
        }
    }
}

fn projection_targets_variant(projection: &OwnerProjectionReturnSummary, variant: &str) -> bool {
    let Some(PlaceProjection::EnumPayload {
        variant: projection_variant,
    }) = projection.suffix.first()
    else {
        return false;
    };
    normalize_variant_name(projection_variant) == variant
}

fn root_parameter_source(
    parameter_storage_sources: &[OwnerParameterStorageSource],
    parameter_index: usize,
) -> Option<OwnerProjectionSource> {
    parameter_storage_sources
        .iter()
        .find(|entry| {
            entry.source.parameter_index == parameter_index && entry.source.suffix.is_empty()
        })
        .map(|entry| entry.source.clone())
}

fn normalize_variant_name(variant: &str) -> String {
    String::from(variant.rsplit("::").next().unwrap_or(variant))
}

fn push_unique_variant_projection_return(
    out: &mut Vec<OwnerVariantProjectionReturn>,
    entry: OwnerVariantProjectionReturn,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}
