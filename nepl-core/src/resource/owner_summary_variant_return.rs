use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place, PlaceProjection};
use super::owner_alias::{aliased_owner_descendant_entries, resolve_owner_alias_place};
use super::owner_extent::summarize_owner_storage_extent;
use super::owner_state::OwnerTable;
use super::owner_summary_record::{
    owner_source_for_storage, parameter_return_extent_for_source, record_projection_owner_return,
    record_root_owner_return, OwnerParameterConditionSource, OwnerParameterStorageSource,
};
use super::place_utils::place_suffix_after_prefix;
use super::summary::{
    OwnerExtentSummary, OwnerProjectionReturnOwner, OwnerProjectionReturnSummary,
    OwnerProjectionSource, OwnerVariantProjectionReturn,
};
use super::variant_name::normalize_variant_name;

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
                    owner: OwnerProjectionReturnOwner::Parameter {
                        returned_extent: parameter_return_extent_for_source(
                            &projection.parameter_return_extents,
                            &source,
                        )
                        .cloned()
                        .unwrap_or(OwnerExtentSummary::Unknown),
                        source,
                    },
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
                    owner: OwnerProjectionReturnOwner::Parameter {
                        returned_extent: parameter_return_extent_for_source(
                            &projection.parameter_return_extents,
                            source,
                        )
                        .cloned()
                        .unwrap_or(OwnerExtentSummary::Unknown),
                        source: source.clone(),
                    },
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
                    owner: OwnerProjectionReturnOwner::Fresh {
                        extent: projection.returns_fresh_owner_extent.clone(),
                    },
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

fn push_unique_variant_projection_return(
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
    match (&mut existing.owner, entry.owner) {
        (
            OwnerProjectionReturnOwner::Parameter {
                source: existing_source,
                returned_extent,
            },
            OwnerProjectionReturnOwner::Parameter {
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
            OwnerProjectionReturnOwner::Fresh { extent },
            OwnerProjectionReturnOwner::Fresh {
                extent: next_extent,
            },
        ) => {
            *extent =
                super::owner_extent::merge_owner_extent_summaries(extent.clone(), next_extent);
        }
        (OwnerProjectionReturnOwner::Maybe, _) => {}
        (_, OwnerProjectionReturnOwner::Maybe) => {
            existing.owner = OwnerProjectionReturnOwner::Maybe;
        }
        _ => {
            existing.owner = OwnerProjectionReturnOwner::Maybe;
        }
    }
}
