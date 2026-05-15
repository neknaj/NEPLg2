extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{OwnerState, Place, StorageId};
use super::owner_check::ResourceOwnerCheckEngine;
use super::owner_raw_view::RawAddressViewTable;
use super::owner_return_apply_source::summary_projection_place;
use super::owner_state::OwnerTable;
use super::owner_variant::{
    PendingVariantOwnerConsumption, PendingVariantOwnerReturn, PendingVariantOwnerReturnSource,
};
use super::place_utils::places_overlap;
use super::report::ResourceOwnerOperation;
use super::storage_origin::StorageOriginTable;
use super::summary::{OwnerExtentSummary, OwnerVariantConsumedExtentRequirement};

pub(super) fn pending_consumption_source(
    entry: &PendingVariantOwnerConsumption,
    raw_aliases: &RawCellAddressAliases,
) -> Place {
    let arg = raw_aliases.canonicalize(&entry.arg);
    summary_projection_place(&arg, &entry.suffix, entry.ty)
}

pub(super) fn consume_pending_variant_owner(
    engine: &mut ResourceOwnerCheckEngine<'_>,
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    storage_origins: &mut StorageOriginTable,
    entry: &PendingVariantOwnerConsumption,
    source: &Place,
    span: Span,
) -> bool {
    if let Some(requirement) = &entry.extent {
        if !engine.ensure_owner_extent_matches_summary(
            owners,
            raw_aliases,
            source,
            &requirement.expected,
            requirement.operation,
            span,
        ) {
            engine.push_unavailable(
                requirement.operation,
                source,
                owners.state(source).unwrap_or(OwnerState::NoFreeObligation),
                span,
            );
            return false;
        }
    }
    engine.move_owner_out(
        owners,
        raw_aliases,
        storage_origins,
        source,
        ResourceOwnerOperation::CallArgument,
        span,
    );
    true
}

pub(super) fn pending_return_source(
    entry: &PendingVariantOwnerReturn,
    raw_aliases: &RawCellAddressAliases,
) -> Option<Place> {
    let PendingVariantOwnerReturnSource::Parameter {
        arg,
        source_suffix,
        source_ty,
        ..
    } = &entry.source
    else {
        return None;
    };
    let arg = raw_aliases.canonicalize(arg);
    Some(summary_projection_place(&arg, source_suffix, *source_ty))
}

pub(super) fn apply_pending_variant_owner_return(
    engine: &mut ResourceOwnerCheckEngine<'_>,
    owners: &mut OwnerTable,
    raw_aliases: &mut RawCellAddressAliases,
    raw_views: &mut RawAddressViewTable,
    storage_origins: &mut StorageOriginTable,
    entry: &PendingVariantOwnerReturn,
    result: &Place,
    span: Span,
) -> Option<Place> {
    let target = summary_projection_place(result, &entry.target_suffix, entry.target_ty);
    let source = match &entry.source {
        PendingVariantOwnerReturnSource::Parameter {
            extent_requirement,
            returned_extent,
            ..
        } => {
            let source = pending_return_source(entry, raw_aliases)?;
            if owners.has_transferable_owner(&target) {
                if !places_overlap(&source, &target)
                    && engine.has_transferable_owner(owners, raw_aliases, &source)
                {
                    engine.move_owner_out(
                        owners,
                        raw_aliases,
                        storage_origins,
                        &source,
                        ResourceOwnerOperation::ReturnValue,
                        span,
                    );
                    raw_views.clear(&source);
                }
                return Some(source);
            }
            if let Some(requirement) = extent_requirement {
                if !engine.ensure_owner_extent_matches_summary(
                    owners,
                    raw_aliases,
                    &source,
                    &requirement.expected,
                    requirement.operation,
                    span,
                ) {
                    engine.push_unavailable(
                        requirement.operation,
                        &source,
                        owners
                            .state(&source)
                            .unwrap_or(OwnerState::NoFreeObligation),
                        span,
                    );
                    return None;
                }
            }
            raw_aliases.copy_scalar_facts_if_tracked(&source, &target);
            engine.transfer_owner_from_summary_effect(
                owners,
                raw_aliases,
                raw_views,
                storage_origins,
                &source,
                &target,
                ResourceOwnerOperation::ReturnValue,
                span,
            );
            if !matches!(returned_extent, super::model::OwnerStorageExtent::Unknown) {
                owners.set_live_extent(&target, returned_extent.clone());
            }
            Some(source)
        }
        PendingVariantOwnerReturnSource::Fresh { extent } => {
            if owners.has_transferable_owner(&target) {
                return None;
            }
            owners.allocate_with_extent(&target, extent.clone());
            raw_aliases.mark(&target);
            storage_origins.mark_owned(&target);
            None
        }
        PendingVariantOwnerReturnSource::UnknownSource { extent } => {
            if owners.has_transferable_owner(&target) {
                return None;
            }
            owners.allocate_with_extent(&target, extent.clone());
            raw_aliases.mark(&target);
            storage_origins.mark_owned(&target);
            None
        }
        PendingVariantOwnerReturnSource::Maybe => {
            if owners.has_transferable_owner(&target) {
                return None;
            }
            owners.set_state(&target, OwnerState::MaybeFreed { storage: None });
            raw_aliases.mark(&target);
            storage_origins.mark_owned(&target);
            None
        }
    };
    raw_views.clear(&target);
    source
}

pub(super) fn reserved_owner_state(owners: &OwnerTable, source: &Place) -> OwnerState {
    let storage = match owners.state(source) {
        Some(OwnerState::Live { storage, .. }) => Some(storage),
        Some(OwnerState::MaybeFreed { storage } | OwnerState::Reserved { storage }) => storage,
        Some(OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed) | None => {
            first_storage_under(owners, source)
        }
    };
    OwnerState::Reserved { storage }
}

fn first_storage_under(owners: &OwnerTable, source: &Place) -> Option<StorageId> {
    owners
        .live_entries_under(source)
        .into_iter()
        .find_map(|entry| match entry.state {
            OwnerState::Live { storage, .. } => Some(storage),
            OwnerState::MaybeFreed { storage } | OwnerState::Reserved { storage } => storage,
            OwnerState::NoFreeObligation | OwnerState::Moved | OwnerState::Freed => None,
        })
}

pub(super) fn push_or_merge_variant_extent_requirement(
    out: &mut Vec<OwnerVariantConsumedExtentRequirement>,
    entry: OwnerVariantConsumedExtentRequirement,
) {
    if matches!(entry.extent, OwnerExtentSummary::Unknown) {
        return;
    }
    if let Some(existing) = out.iter_mut().find(|existing| {
        existing.variant == entry.variant
            && existing.owner == entry.owner
            && existing.operation == entry.operation
    }) {
        existing.extent = super::owner_extent::merge_owner_extent_summaries(
            existing.extent.clone(),
            entry.extent,
        );
        return;
    }
    out.push(entry);
}
