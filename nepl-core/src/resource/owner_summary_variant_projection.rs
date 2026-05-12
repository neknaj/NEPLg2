use alloc::vec::Vec;

use super::owner_summary_record::push_unique_owner_projection_source;
use super::summary::{
    OwnerProjectionReturnOwner, OwnerProjectionReturnSummary, OwnerProjectionSource,
    OwnerVariantProjectionReturn,
};

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
                OwnerProjectionReturnOwner::Parameter(source) if source.suffix.is_empty() => {
                    projection
                        .parameter_indices
                        .retain(|index| *index != source.parameter_index);
                }
                OwnerProjectionReturnOwner::Parameter(source) => {
                    projection
                        .parameter_sources
                        .retain(|projection_source| projection_source != source);
                }
                OwnerProjectionReturnOwner::Fresh => {
                    projection.returns_fresh_owner = false;
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
    });
}

pub(super) fn record_variant_projection_return_sources(
    returned_sources: &mut Vec<OwnerProjectionSource>,
    variant_returns: &[OwnerVariantProjectionReturn],
) {
    for variant_return in variant_returns {
        if let OwnerProjectionReturnOwner::Parameter(source) = &variant_return.owner {
            push_unique_owner_projection_source(returned_sources, source);
        }
    }
}
