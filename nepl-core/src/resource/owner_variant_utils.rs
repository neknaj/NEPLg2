extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place, PlaceProjection};
use super::owner_alias::resolve_owner_alias_place;
use super::owner_state::OwnerTable;
use super::owner_summary_record::{owner_source_for_storage, OwnerParameterStorageSource};
use super::summary::{
    OwnerProjectionSource, OwnerVariantCondition, OwnerVariantParameterIndex,
    OwnerVariantProjectionReturn, OwnerVariantProjectionSource,
};
use super::variant_name::normalize_variant_name;

pub(super) fn owner_projection_sources_for_place(
    owners: &OwnerTable,
    raw_aliases: &RawCellAddressAliases,
    place: &Place,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) -> Vec<OwnerProjectionSource> {
    let resolved = resolve_owner_alias_place(owners, raw_aliases, place);
    let mut out = Vec::new();
    if let Some(source) =
        owner_projection_source_for_owner_state(owners.state(&resolved), parameter_storage_sources)
    {
        push_unique_projection_source(&mut out, source);
    }
    for entry in owners.live_entries_under(&resolved) {
        if let Some(source) =
            owner_projection_source_for_owner_state(Some(entry.state), parameter_storage_sources)
        {
            push_unique_projection_source(&mut out, source);
        }
    }
    out
}

fn owner_projection_source_for_owner_state(
    state: Option<OwnerState>,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) -> Option<OwnerProjectionSource> {
    let storage = match state {
        Some(OwnerState::Live { storage, .. }) => storage,
        Some(OwnerState::MaybeFreed {
            storage: Some(storage),
        }) => storage,
        Some(
            OwnerState::NoFreeObligation
            | OwnerState::Reserved { .. }
            | OwnerState::Moved
            | OwnerState::Freed,
        )
        | Some(OwnerState::MaybeFreed { storage: None })
        | None => return None,
    };
    owner_source_for_storage(storage, parameter_storage_sources).cloned()
}

pub(super) fn push_unique_variant_consumed_source(
    index_out: &mut Vec<OwnerVariantParameterIndex>,
    source_out: &mut Vec<OwnerVariantProjectionSource>,
    variant: String,
    source: OwnerProjectionSource,
) {
    if source.suffix.is_empty() {
        let entry = OwnerVariantParameterIndex {
            variant,
            parameter_index: source.parameter_index,
        };
        if !index_out.iter().any(|existing| existing == &entry) {
            index_out.push(entry);
        }
    } else {
        let entry = OwnerVariantProjectionSource { variant, source };
        if !source_out.iter().any(|existing| existing == &entry) {
            source_out.push(entry);
        }
    }
}

pub(super) fn push_unique_variant_projection_return(
    out: &mut Vec<OwnerVariantProjectionReturn>,
    entry: OwnerVariantProjectionReturn,
) {
    let Some(existing) = out.iter_mut().find(|existing| {
        existing.variant == entry.variant
            && existing.suffix == entry.suffix
            && existing.ty == entry.ty
    }) else {
        out.push(entry);
        return;
    };
    if existing.owner == entry.owner {
        return;
    }
    let entry_owner_extent = projection_return_owner_extent(&entry.owner);
    match (&mut existing.owner, entry.owner) {
        (
            super::summary::OwnerProjectionReturnOwner::Parameter {
                source: existing_source,
                returned_extent,
            },
            super::summary::OwnerProjectionReturnOwner::Parameter {
                source,
                returned_extent: next_extent,
            },
        ) if existing_source == &source => {
            *returned_extent = super::owner_extent::merge_owner_extent_summaries(
                returned_extent.clone(),
                next_extent,
            );
        }
        (
            super::summary::OwnerProjectionReturnOwner::Fresh { extent },
            super::summary::OwnerProjectionReturnOwner::Fresh {
                extent: next_extent,
            },
        ) => {
            *extent =
                super::owner_extent::merge_owner_extent_summaries(extent.clone(), next_extent);
        }
        (
            super::summary::OwnerProjectionReturnOwner::UnknownSource { extent },
            super::summary::OwnerProjectionReturnOwner::UnknownSource {
                extent: next_extent,
            },
        ) => {
            *extent =
                super::owner_extent::merge_owner_extent_summaries(extent.clone(), next_extent);
        }
        (super::summary::OwnerProjectionReturnOwner::Maybe, _) => {}
        (_, super::summary::OwnerProjectionReturnOwner::Maybe) => {
            existing.owner = super::summary::OwnerProjectionReturnOwner::Maybe;
        }
        _ => {
            existing.owner = super::summary::OwnerProjectionReturnOwner::UnknownSource {
                extent: super::owner_extent::merge_owner_extent_summaries(
                    projection_return_owner_extent(&existing.owner),
                    entry_owner_extent,
                ),
            };
        }
    }
}

fn projection_return_owner_extent(
    owner: &super::summary::OwnerProjectionReturnOwner,
) -> super::summary::OwnerExtentSummary {
    match owner {
        super::summary::OwnerProjectionReturnOwner::Parameter {
            returned_extent, ..
        } => returned_extent.clone(),
        super::summary::OwnerProjectionReturnOwner::Fresh { extent }
        | super::summary::OwnerProjectionReturnOwner::UnknownSource { extent } => extent.clone(),
        super::summary::OwnerProjectionReturnOwner::Maybe => {
            super::summary::OwnerExtentSummary::Unknown
        }
    }
}

pub(super) fn push_unique_owner_variant_condition(
    out: &mut Vec<OwnerVariantCondition>,
    entry: OwnerVariantCondition,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

fn push_unique_projection_source(
    out: &mut Vec<OwnerProjectionSource>,
    source: OwnerProjectionSource,
) {
    if !out.iter().any(|existing| existing == &source) {
        out.push(source);
    }
}

pub(super) fn payload_bind_suffix<'a>(
    suffix: &'a [PlaceProjection],
    variant: &str,
) -> &'a [PlaceProjection] {
    let Some(PlaceProjection::EnumPayload {
        variant: suffix_variant,
    }) = suffix.first()
    else {
        return suffix;
    };
    if normalize_variant_name(suffix_variant) == variant {
        &suffix[1..]
    } else {
        suffix
    }
}
