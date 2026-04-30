use alloc::vec::Vec;

use super::summary::{
    OwnerProjectionReturnSummary, OwnerProjectionSource, OwnerVariantParameterIndex,
    OwnerVariantProjectionReturn, OwnerVariantProjectionReturnKind, OwnerVariantProjectionSource,
};

pub(super) fn remove_variant_consumed_parameter_sources(
    consumed_indices: &mut Vec<usize>,
    consumed_sources: &mut Vec<OwnerProjectionSource>,
    variant_indices: &[OwnerVariantParameterIndex],
    variant_sources: &[OwnerVariantProjectionSource],
) {
    for variant in variant_indices {
        consumed_indices.retain(|index| *index != variant.parameter_index);
    }
    for variant in variant_sources {
        consumed_sources.retain(|source| source != &variant.source);
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
            match &variant_return.kind {
                OwnerVariantProjectionReturnKind::Parameter(source) => {
                    if source.suffix.is_empty() {
                        projection
                            .parameter_indices
                            .retain(|index| *index != source.parameter_index);
                    } else {
                        projection
                            .parameter_sources
                            .retain(|existing| existing != source);
                    }
                }
                OwnerVariantProjectionReturnKind::FreshOwner => {
                    projection.returns_fresh_owner = false;
                }
                OwnerVariantProjectionReturnKind::MaybeOwner => {
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
