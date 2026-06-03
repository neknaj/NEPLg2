use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId};

use super::model::PlaceProjection;
use super::owner_summary_record::OwnerParameterStorageSource;
use super::owner_summary_variant_return::record_variant_projection_returns;
use super::summary::{
    OwnerProjectionReturnSummary, OwnerVariantProjectionReturn, OwnerVariantProjectionSource,
};
use super::variant_name::variant_names_match;

pub(super) fn record_ambiguous_enum_projection_returns_as_variant_returns(
    variant_returns: &mut Vec<OwnerVariantProjectionReturn>,
    types: &TypeCtx,
    result_ty: TypeId,
    projection_returns: &[OwnerProjectionReturnSummary],
    parameter_storage_sources: &[OwnerParameterStorageSource],
) -> Vec<OwnerVariantProjectionSource> {
    let variants = ambiguous_projection_variants(projection_returns);
    if variants.len() < 2 {
        return Vec::new();
    }
    let mut consumed_sources = Vec::new();
    for variant in variants {
        for source in record_variant_projection_returns(
            variant_returns,
            types,
            result_ty,
            &variant,
            projection_returns,
            parameter_storage_sources,
        ) {
            push_unique_variant_projection_source(
                &mut consumed_sources,
                OwnerVariantProjectionSource {
                    variant: super::variant_name::normalize_variant_name(variant),
                    source,
                },
            );
        }
    }
    consumed_sources
}

fn push_unique_variant_projection_source(
    out: &mut Vec<OwnerVariantProjectionSource>,
    entry: OwnerVariantProjectionSource,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

fn ambiguous_projection_variants<'a>(
    projection_returns: &'a [OwnerProjectionReturnSummary],
) -> Vec<&'a str> {
    let mut variants: Vec<&str> = Vec::new();
    for projection in projection_returns {
        let Some(PlaceProjection::EnumPayload { variant }) = projection.suffix.first() else {
            continue;
        };
        if !variants
            .iter()
            .any(|existing| variant_names_match(existing, variant))
        {
            variants.push(variant.as_str());
        }
    }
    variants
}
