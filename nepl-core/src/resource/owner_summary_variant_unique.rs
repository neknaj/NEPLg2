use alloc::vec::Vec;

use super::summary::{
    OwnerVariantCondition, OwnerVariantParameterIndex, OwnerVariantPayloadCondition,
    OwnerVariantProjectionSource,
};

pub(super) fn push_unique_variant_parameter_index(
    out: &mut Vec<OwnerVariantParameterIndex>,
    entry: OwnerVariantParameterIndex,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

pub(super) fn push_unique_variant_projection_source(
    out: &mut Vec<OwnerVariantProjectionSource>,
    entry: OwnerVariantProjectionSource,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

pub(super) fn push_unique_variant_condition(
    out: &mut Vec<OwnerVariantCondition>,
    entry: OwnerVariantCondition,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

pub(super) fn push_unique_variant_payload_condition(
    out: &mut Vec<OwnerVariantPayloadCondition>,
    entry: OwnerVariantPayloadCondition,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}
