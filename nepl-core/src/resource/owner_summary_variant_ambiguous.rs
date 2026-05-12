use alloc::vec::Vec;

use super::model::PlaceProjection;
use super::owner_summary_record::OwnerParameterStorageSource;
use super::owner_summary_variant_return::record_variant_projection_returns;
use super::summary::{OwnerProjectionReturnSummary, OwnerVariantProjectionReturn};

pub(super) fn record_ambiguous_enum_projection_returns_as_variant_returns(
    variant_returns: &mut Vec<OwnerVariantProjectionReturn>,
    projection_returns: &[OwnerProjectionReturnSummary],
    parameter_storage_sources: &[OwnerParameterStorageSource],
) {
    let variants = ambiguous_projection_variants(projection_returns);
    if variants.len() < 2 {
        return;
    }
    for variant in variants {
        record_variant_projection_returns(
            variant_returns,
            &variant,
            projection_returns,
            parameter_storage_sources,
        );
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
            .any(|existing| same_variant(*existing, variant))
        {
            variants.push(variant.as_str());
        }
    }
    variants
}

fn same_variant(left: &str, right: &str) -> bool {
    left.rsplit("::").next().unwrap_or(left) == right.rsplit("::").next().unwrap_or(right)
}
