use alloc::vec::Vec;

use super::summary::{
    OwnerProjectionReturnSummary, OwnerProjectionSource, OwnerVariantParameterIndex,
    OwnerVariantProjectionReturn, OwnerVariantProjectionReturnKind, OwnerVariantProjectionSource,
};

pub(super) fn remove_variant_consumptions_returned_by_same_variant(
    variant_indices: &mut Vec<OwnerVariantParameterIndex>,
    variant_sources: &mut Vec<OwnerVariantProjectionSource>,
    variant_returns: &[OwnerVariantProjectionReturn],
) {
    for variant_return in variant_returns {
        match &variant_return.kind {
            OwnerVariantProjectionReturnKind::Parameter(source) => {
                if source.suffix.is_empty() {
                    variant_indices.retain(|consumed| {
                        consumed.variant != variant_return.variant
                            || consumed.parameter_index != source.parameter_index
                    });
                } else {
                    variant_sources.retain(|consumed| {
                        consumed.variant != variant_return.variant || consumed.source != *source
                    });
                }
            }
            OwnerVariantProjectionReturnKind::FreshOwner
            | OwnerVariantProjectionReturnKind::MaybeOwner => {}
        }
    }
}

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
