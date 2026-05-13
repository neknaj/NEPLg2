use alloc::vec::Vec;

use super::model::PlaceProjection;
use super::owner_summary_record::{
    parameter_return_extent_for_source, OwnerParameterStorageSource,
};
use super::summary::{
    OwnerExtentSummary, OwnerProjectionReturnOwner, OwnerProjectionReturnSummary,
    OwnerProjectionSource, OwnerVariantProjectionReturn,
};
use super::variant_name::normalize_variant_name;

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
