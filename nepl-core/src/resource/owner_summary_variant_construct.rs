use alloc::string::String;
use alloc::vec::Vec;

use crate::types::TypeId;

use super::model::{AggregateKind, Place, PlaceProjection, ResourceOp};
use super::place_utils::{construct_aggregate_field_place, place_suffix_after_prefix};

#[derive(Debug, Clone)]
pub(super) struct ConstructedVariant {
    pub(super) variant: String,
    pub(super) payloads: Vec<ConstructedVariantPayload>,
}

#[derive(Debug, Clone)]
pub(super) struct ConstructedVariantPayload {
    pub(super) suffix: Vec<PlaceProjection>,
    pub(super) ty: TypeId,
}

pub(super) fn construct_variant_for_value(
    ops: &[ResourceOp],
    value: &Place,
) -> Option<ConstructedVariant> {
    for op in ops.iter().rev() {
        let ResourceOp::Construct {
            output,
            kind,
            inputs,
            ..
        } = op
        else {
            continue;
        };
        let AggregateKind::Enum { variant, .. } = kind else {
            continue;
        };
        if output != value {
            continue;
        }
        let mut payloads = Vec::new();
        for (index, input) in inputs.iter().enumerate() {
            let payload = construct_aggregate_field_place(output, kind, index, input);
            let suffix = place_suffix_after_prefix(&payload, output).unwrap_or_default();
            payloads.push(ConstructedVariantPayload {
                suffix,
                ty: input.ty,
            });
        }
        return Some(ConstructedVariant {
            variant: variant.clone(),
            payloads,
        });
    }
    None
}

pub(super) fn normalize_variant_name(variant: &str) -> String {
    String::from(variant.rsplit("::").next().unwrap_or(variant))
}
