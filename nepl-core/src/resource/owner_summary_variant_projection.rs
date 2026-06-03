use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::owner_summary_record::{
    push_unique_owner_projection_source, OwnerParameterStorageSource,
};
use super::owner_summary_variant_ambiguous::record_ambiguous_enum_projection_returns_as_variant_returns;
use super::summary::{
    OwnerProjectionReturnOwner, OwnerProjectionReturnSummary, OwnerProjectionSource,
    OwnerVariantProjectionReturn, OwnerVariantProjectionSource,
};

pub(super) fn finalize_variant_projection_returns(
    projection_returns: &mut Vec<OwnerProjectionReturnSummary>,
    returned_sources: &mut Vec<OwnerProjectionSource>,
    variant_returns: &mut Vec<OwnerVariantProjectionReturn>,
    variant_consumed_sources: &mut Vec<OwnerVariantProjectionSource>,
    types: &TypeCtx,
    result_ty: TypeId,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) {
    for source in record_ambiguous_enum_projection_returns_as_variant_returns(
        variant_returns,
        types,
        result_ty,
        projection_returns,
        parameter_storage_sources,
    ) {
        push_unique_variant_projection_source(variant_consumed_sources, source);
    }
    remove_non_parameter_variant_projection_return_sources(
        returned_sources,
        projection_returns,
        variant_returns,
    );
    remove_variant_projection_return_sources(projection_returns, variant_returns);
    record_variant_projection_return_sources(returned_sources, variant_returns);
}

fn push_unique_variant_projection_source(
    out: &mut Vec<OwnerVariantProjectionSource>,
    entry: OwnerVariantProjectionSource,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

pub(super) fn remove_non_parameter_variant_projection_return_sources(
    returned_sources: &mut Vec<OwnerProjectionSource>,
    projection_returns: &[OwnerProjectionReturnSummary],
    variant_returns: &[OwnerVariantProjectionReturn],
) {
    for projection in projection_returns {
        for variant_return in variant_returns
            .iter()
            .filter(|entry| entry.suffix == projection.suffix && entry.ty == projection.ty)
        {
            if matches!(
                variant_return.owner,
                OwnerProjectionReturnOwner::Parameter { .. }
            ) {
                continue;
            }
            remove_projection_return_sources(returned_sources, projection);
        }
    }
}

fn remove_projection_return_sources(
    returned_sources: &mut Vec<OwnerProjectionSource>,
    projection: &OwnerProjectionReturnSummary,
) {
    for parameter_index in &projection.parameter_indices {
        returned_sources.retain(|source| {
            source.parameter_index != *parameter_index || !source.suffix.is_empty()
        });
    }
    for projection_source in &projection.parameter_sources {
        returned_sources.retain(|source| source != projection_source);
    }
}

pub(super) fn remove_variant_projection_return_sources(
    projection_returns: &mut Vec<OwnerProjectionReturnSummary>,
    variant_returns: &[OwnerVariantProjectionReturn],
) {
    for projection in projection_returns.iter_mut() {
        for variant_return in variant_returns
            .iter()
            .filter(|entry| entry.suffix == projection.suffix && entry.ty == projection.ty)
        {
            match &variant_return.owner {
                OwnerProjectionReturnOwner::Parameter { source, .. }
                    if source.suffix.is_empty() =>
                {
                    projection
                        .parameter_indices
                        .retain(|index| *index != source.parameter_index);
                    projection
                        .parameter_return_extents
                        .retain(|extent| extent.source != *source);
                }
                OwnerProjectionReturnOwner::Parameter { source, .. } => {
                    projection
                        .parameter_sources
                        .retain(|projection_source| projection_source != source);
                    projection
                        .parameter_return_extents
                        .retain(|extent| extent.source != *source);
                }
                OwnerProjectionReturnOwner::Fresh { .. } => {
                    projection.returns_fresh_owner = false;
                    projection.returns_fresh_owner_extent =
                        super::summary::OwnerExtentSummary::Unknown;
                }
                OwnerProjectionReturnOwner::UnknownSource { .. } => {
                    projection.parameter_indices.clear();
                    projection.parameter_sources.clear();
                    projection.parameter_return_extents.clear();
                    projection.returns_fresh_owner = false;
                    projection.returns_fresh_owner_extent =
                        super::summary::OwnerExtentSummary::Unknown;
                    projection.returns_maybe_owner = false;
                }
                OwnerProjectionReturnOwner::Maybe => {
                    projection.returns_maybe_owner = false;
                }
            }
        }
    }
    projection_returns.retain(|projection| {
        projection.returns_fresh_owner
            || projection.returns_maybe_owner
            || !projection.parameter_indices.is_empty()
            || !projection.parameter_sources.is_empty()
            || !projection.parameter_return_extents.is_empty()
    });
}

pub(super) fn record_variant_projection_return_sources(
    returned_sources: &mut Vec<OwnerProjectionSource>,
    variant_returns: &[OwnerVariantProjectionReturn],
) {
    for variant_return in variant_returns {
        if let OwnerProjectionReturnOwner::Parameter { source, .. } = &variant_return.owner {
            push_unique_owner_projection_source(returned_sources, source);
        }
    }
}
