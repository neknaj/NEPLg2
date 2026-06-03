use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::model::{Place, PlaceProjection};
use super::owner_summary_leaf::owner_leaf_places;
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
    types: &TypeCtx,
    result_ty: TypeId,
    variant: &str,
    projection_returns: &[OwnerProjectionReturnSummary],
    parameter_storage_sources: &[OwnerParameterStorageSource],
) -> Vec<OwnerProjectionSource> {
    let variant = normalize_variant_name(variant);
    let mut ambiguous_parameter_sources = Vec::new();
    for projection in projection_returns {
        if !projection_targets_variant(projection, &variant) {
            continue;
        }
        if !projection_can_carry_owner(types, result_ty, projection) {
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
                &mut ambiguous_parameter_sources,
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
                &mut ambiguous_parameter_sources,
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
                &mut ambiguous_parameter_sources,
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
                &mut ambiguous_parameter_sources,
            );
        }
    }
    ambiguous_parameter_sources
}

fn projection_can_carry_owner(
    types: &TypeCtx,
    result_ty: TypeId,
    projection: &OwnerProjectionReturnSummary,
) -> bool {
    let result = Place::unknown(result_ty);
    owner_leaf_places(types, &result)
        .iter()
        .any(|leaf| leaf.suffix == projection.suffix && leaf.place.ty == projection.ty)
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
    ambiguous_parameter_sources: &mut Vec<OwnerProjectionSource>,
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
    let existing_parameter_source = projection_return_owner_parameter_source(&existing.owner);
    let entry_parameter_source = projection_return_owner_parameter_source(&entry.owner);
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
        (
            OwnerProjectionReturnOwner::UnknownSource { extent },
            OwnerProjectionReturnOwner::UnknownSource {
                extent: next_extent,
            },
        ) => {
            *extent =
                super::owner_extent::merge_owner_extent_summaries(extent.clone(), next_extent);
        }
        (OwnerProjectionReturnOwner::Maybe, _) => {
            if let Some(source) = entry_parameter_source {
                push_unique_owner_projection_source(ambiguous_parameter_sources, &source);
            }
        }
        (_, OwnerProjectionReturnOwner::Maybe) => {
            if let Some(source) = existing_parameter_source {
                push_unique_owner_projection_source(ambiguous_parameter_sources, &source);
            }
            existing.owner = OwnerProjectionReturnOwner::Maybe;
        }
        _ => {
            if let Some(source) = existing_parameter_source {
                push_unique_owner_projection_source(ambiguous_parameter_sources, &source);
            }
            if let Some(source) = entry_parameter_source {
                push_unique_owner_projection_source(ambiguous_parameter_sources, &source);
            }
            existing.owner = OwnerProjectionReturnOwner::UnknownSource {
                extent: super::owner_extent::merge_owner_extent_summaries(
                    projection_return_owner_extent(&existing.owner),
                    entry_owner_extent,
                ),
            };
        }
    }
}

fn push_unique_owner_projection_source(
    out: &mut Vec<OwnerProjectionSource>,
    source: &OwnerProjectionSource,
) {
    if !out.iter().any(|existing| existing == source) {
        out.push(source.clone());
    }
}

fn projection_return_owner_parameter_source(
    owner: &OwnerProjectionReturnOwner,
) -> Option<OwnerProjectionSource> {
    match owner {
        OwnerProjectionReturnOwner::Parameter { source, .. } => Some(source.clone()),
        OwnerProjectionReturnOwner::Fresh { .. }
        | OwnerProjectionReturnOwner::UnknownSource { .. }
        | OwnerProjectionReturnOwner::Maybe => None,
    }
}

fn projection_return_owner_extent(owner: &OwnerProjectionReturnOwner) -> OwnerExtentSummary {
    match owner {
        OwnerProjectionReturnOwner::Parameter {
            returned_extent, ..
        } => returned_extent.clone(),
        OwnerProjectionReturnOwner::Fresh { extent }
        | OwnerProjectionReturnOwner::UnknownSource { extent } => extent.clone(),
        OwnerProjectionReturnOwner::Maybe => OwnerExtentSummary::Unknown,
    }
}
