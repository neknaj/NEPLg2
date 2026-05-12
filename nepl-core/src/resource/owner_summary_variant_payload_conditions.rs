use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::condition_fact::simple_condition_value_constraint;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{I32ValueCondition, Place, PlaceProjection, ResourceConditionFact};
use super::owner_summary_i32_leaf::i32_leaf_places;
use super::owner_summary_variant_construct::{normalize_variant_name, ConstructedVariant};
use super::owner_summary_variant_i32_conditions::SUMMARY_I32_CONDITIONS;
use super::place_utils::{place_suffix_after_prefix, place_with_suffix};
use super::summary::OwnerVariantPayloadCondition;

pub(super) fn collect_owner_variant_payload_conditions(
    out: &mut Vec<OwnerVariantPayloadCondition>,
    types: &TypeCtx,
    constructed_variant: &ConstructedVariant,
    value: &Place,
    condition_fact: &ResourceConditionFact,
    truthy_path: bool,
    raw_aliases: &RawCellAddressAliases,
) {
    let Some((_place, condition)) = simple_condition_value_constraint(condition_fact, truthy_path)
    else {
        return;
    };
    collect_owner_variant_payload_condition(
        out,
        types,
        constructed_variant,
        value,
        condition,
        raw_aliases,
    );
}

pub(super) fn collect_owner_variant_known_payload_conditions(
    out: &mut Vec<OwnerVariantPayloadCondition>,
    types: &TypeCtx,
    constructed_variant: &ConstructedVariant,
    value: &Place,
    raw_aliases: &RawCellAddressAliases,
) {
    for condition in SUMMARY_I32_CONDITIONS {
        collect_owner_variant_payload_condition(
            out,
            types,
            constructed_variant,
            value,
            condition,
            raw_aliases,
        );
    }
}

fn collect_owner_variant_payload_condition(
    out: &mut Vec<OwnerVariantPayloadCondition>,
    types: &TypeCtx,
    constructed_variant: &ConstructedVariant,
    value: &Place,
    condition: I32ValueCondition,
    raw_aliases: &RawCellAddressAliases,
) {
    for payload in &constructed_variant.payloads {
        let payload_place = place_with_suffix(value, &payload.suffix, payload.ty);
        for leaf in i32_leaf_places(types, &payload_place) {
            let input_leaf = input_payload_leaf(types, payload, &leaf.suffix);
            let output_has_condition =
                raw_aliases.i32_condition_is_known_true(&leaf.place, condition);
            let input_has_condition = input_leaf
                .as_ref()
                .is_some_and(|input| raw_aliases.i32_condition_is_known_true(input, condition));
            if !output_has_condition && !input_has_condition {
                continue;
            }
            let suffix = place_suffix_after_prefix(&leaf.place, value).unwrap_or_else(|| {
                let mut suffix = payload.suffix.clone();
                suffix.extend(leaf.suffix.clone());
                suffix
            });
            push_unique_variant_payload_condition(
                out,
                OwnerVariantPayloadCondition {
                    variant: normalize_variant_name(&constructed_variant.variant),
                    suffix,
                    ty: leaf.place.ty,
                    condition,
                },
            );
        }
    }
}

fn input_payload_leaf(
    types: &TypeCtx,
    payload: &super::owner_summary_variant_construct::ConstructedVariantPayload,
    leaf_suffix: &[PlaceProjection],
) -> Option<Place> {
    let input_leaves = i32_leaf_places(types, &payload.input);
    input_leaves
        .into_iter()
        .find(|input_leaf| input_leaf.suffix == leaf_suffix)
        .map(|input_leaf| input_leaf.place)
}

fn push_unique_variant_payload_condition(
    out: &mut Vec<OwnerVariantPayloadCondition>,
    entry: OwnerVariantPayloadCondition,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}
