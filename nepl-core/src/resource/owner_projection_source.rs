use super::model::PlaceProjection;
use super::summary::{OwnerProjectionReturnOwner, OwnerProjectionSource, OwnerReturnSummary};

pub(super) fn owner_projection_sources_overlap(
    left: &OwnerProjectionSource,
    right: &OwnerProjectionSource,
) -> bool {
    left.parameter_index == right.parameter_index
        && projection_suffixes_overlap(&left.suffix, &right.suffix)
}

pub(super) fn owner_projection_source_returned_by_variant(
    summary: &OwnerReturnSummary,
    source: &OwnerProjectionSource,
) -> bool {
    summary.variant_projection_returns.iter().any(|entry| {
        matches!(
            &entry.owner,
            OwnerProjectionReturnOwner::Parameter {
                source: returned,
                ..
            } if owner_projection_sources_overlap(returned, source)
        )
    })
}

pub(super) fn owner_projection_source_consumed_unconditionally(
    summary: &OwnerReturnSummary,
    source: &OwnerProjectionSource,
) -> bool {
    summary
        .consumed_parameter_indices
        .iter()
        .any(|index| *index == source.parameter_index)
        || summary
            .consumed_parameter_sources
            .iter()
            .any(|consumed| owner_projection_sources_overlap(consumed, source))
}

pub(super) fn projection_suffixes_overlap(
    left: &[PlaceProjection],
    right: &[PlaceProjection],
) -> bool {
    left.starts_with(right) || right.starts_with(left)
}
