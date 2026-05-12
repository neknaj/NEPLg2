use alloc::vec::Vec;

use crate::types::TypeCtx;
use crate::types::TypeId;

use super::condition_fact::simple_condition_value_constraint;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{I32ValueCondition, Place, PlaceProjection, ResourceConditionFact};
use super::owner_summary_i32_leaf::i32_leaf_places;
use super::owner_summary_record::OwnerParameterConditionSource;
use super::owner_summary_variant_construct::{normalize_variant_name, ConstructedVariant};
use super::place_utils::{place_suffix_after_prefix, place_with_suffix};
use super::summary::{
    OwnerProjectionSource, OwnerValueCondition, OwnerVariantCondition, OwnerVariantPayloadCondition,
};

const SUMMARY_I32_CONDITIONS: [I32ValueCondition; 6] = [
    I32ValueCondition::EqZero,
    I32ValueCondition::NeZero,
    I32ValueCondition::Positive,
    I32ValueCondition::NonPositive,
    I32ValueCondition::Negative,
    I32ValueCondition::NonNegative,
];

pub(super) fn collect_owner_variant_condition(
    out: &mut Vec<OwnerVariantCondition>,
    variant: &str,
    condition_fact: &ResourceConditionFact,
    truthy_path: bool,
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
) {
    let Some(condition) = owner_value_condition(
        condition_fact,
        truthy_path,
        raw_aliases,
        parameter_condition_sources,
    ) else {
        return;
    };
    push_unique_variant_condition(
        out,
        OwnerVariantCondition {
            variant: normalize_variant_name(variant),
            condition,
        },
    );
}

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

pub(super) fn collect_owner_variant_known_conditions(
    out: &mut Vec<OwnerVariantCondition>,
    variant: &str,
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
) {
    for source in parameter_condition_sources {
        for condition in SUMMARY_I32_CONDITIONS {
            if !raw_aliases.i32_condition_is_known_true(&source.place, condition) {
                continue;
            }
            push_unique_variant_condition(
                out,
                OwnerVariantCondition {
                    variant: normalize_variant_name(variant),
                    condition: OwnerValueCondition::Param {
                        source: source.source.clone(),
                        condition,
                    },
                },
            );
        }
    }
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

fn owner_value_condition(
    condition_fact: &ResourceConditionFact,
    truthy_path: bool,
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
) -> Option<OwnerValueCondition> {
    if let Some((place, condition)) = simple_condition_value_constraint(condition_fact, truthy_path)
    {
        return owner_param_value_condition(
            place,
            condition,
            raw_aliases,
            parameter_condition_sources,
        );
    }
    match (condition_fact, truthy_path) {
        (ResourceConditionFact::Any(facts), true) => {
            let mut conditions = Vec::new();
            for fact in facts {
                conditions.push(owner_value_condition(
                    fact,
                    truthy_path,
                    raw_aliases,
                    parameter_condition_sources,
                )?);
            }
            Some(OwnerValueCondition::Any(conditions))
        }
        (ResourceConditionFact::All(facts), true) => {
            let mut conditions = Vec::new();
            for fact in facts {
                conditions.push(owner_value_condition(
                    fact,
                    truthy_path,
                    raw_aliases,
                    parameter_condition_sources,
                )?);
            }
            Some(OwnerValueCondition::All(conditions))
        }
        (ResourceConditionFact::Any(facts), false) => {
            let mut conditions = Vec::new();
            for fact in facts {
                conditions.push(owner_value_condition(
                    fact,
                    truthy_path,
                    raw_aliases,
                    parameter_condition_sources,
                )?);
            }
            Some(OwnerValueCondition::All(conditions))
        }
        (ResourceConditionFact::All(facts), false) => {
            let mut conditions = Vec::new();
            for fact in facts {
                conditions.push(owner_value_condition(
                    fact,
                    truthy_path,
                    raw_aliases,
                    parameter_condition_sources,
                )?);
            }
            Some(OwnerValueCondition::Any(conditions))
        }
        (ResourceConditionFact::EqZero { .. }, _)
        | (ResourceConditionFact::NeZero { .. }, _)
        | (ResourceConditionFact::Positive { .. }, _)
        | (ResourceConditionFact::NonPositive { .. }, _)
        | (ResourceConditionFact::Negative { .. }, _)
        | (ResourceConditionFact::NonNegative { .. }, _)
        | (ResourceConditionFact::I32Relation { .. }, _) => None,
    }
}

fn owner_param_value_condition(
    place: &Place,
    condition: I32ValueCondition,
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
) -> Option<OwnerValueCondition> {
    for place_alias in raw_aliases.aliases_for(place) {
        for source in parameter_condition_sources {
            for param_alias in raw_aliases.aliases_for(&source.place) {
                let Some(suffix) = place_suffix_after_prefix(&place_alias, &param_alias) else {
                    continue;
                };
                return Some(OwnerValueCondition::Param {
                    source: extend_owner_projection_source(&source.source, suffix, place_alias.ty),
                    condition,
                });
            }
        }
    }
    None
}

pub(super) fn extend_owner_projection_source(
    source: &OwnerProjectionSource,
    suffix: Vec<PlaceProjection>,
    ty: TypeId,
) -> OwnerProjectionSource {
    let mut combined_suffix = source.suffix.clone();
    combined_suffix.extend(suffix);
    OwnerProjectionSource {
        parameter_index: source.parameter_index,
        suffix: combined_suffix,
        ty,
    }
}

fn push_unique_variant_condition(
    out: &mut Vec<OwnerVariantCondition>,
    entry: OwnerVariantCondition,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

fn push_unique_variant_payload_condition(
    out: &mut Vec<OwnerVariantPayloadCondition>,
    entry: OwnerVariantPayloadCondition,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}
