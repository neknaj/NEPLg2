use alloc::vec::Vec;

use super::condition_fact::simple_condition_value_constraint;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{I32ValueCondition, Place, ResourceConditionFact};
use super::owner_summary_record::OwnerParameterStorageSource;
use super::owner_summary_variant_construct::{normalize_variant_name, ConstructedVariant};
use super::place_utils::{place_suffix_after_prefix, place_with_suffix};
use super::summary::{
    OwnerProjectionSource, OwnerValueCondition, OwnerVariantCondition, OwnerVariantPayloadCondition,
};

pub(super) fn collect_owner_variant_condition(
    out: &mut Vec<OwnerVariantCondition>,
    variant: &str,
    condition_fact: &ResourceConditionFact,
    truthy_path: bool,
    raw_aliases: &RawCellAddressAliases,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) {
    let Some(condition) = owner_value_condition(
        condition_fact,
        truthy_path,
        raw_aliases,
        parameter_storage_sources,
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
    for payload in &constructed_variant.payloads {
        let payload_place = place_with_suffix(value, &payload.suffix, payload.ty);
        if raw_aliases.i32_condition_truth(&payload_place, condition) == Some(true) {
            push_unique_variant_payload_condition(
                out,
                OwnerVariantPayloadCondition {
                    variant: normalize_variant_name(&constructed_variant.variant),
                    suffix: payload.suffix.clone(),
                    ty: payload.ty,
                    condition,
                },
            );
        }
    }
}

fn owner_value_condition(
    condition_fact: &ResourceConditionFact,
    truthy_path: bool,
    raw_aliases: &RawCellAddressAliases,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) -> Option<OwnerValueCondition> {
    if let Some((place, condition)) = simple_condition_value_constraint(condition_fact, truthy_path)
    {
        return owner_param_value_condition(
            place,
            condition,
            raw_aliases,
            parameter_storage_sources,
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
                    parameter_storage_sources,
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
                    parameter_storage_sources,
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
                    parameter_storage_sources,
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
                    parameter_storage_sources,
                )?);
            }
            Some(OwnerValueCondition::Any(conditions))
        }
        (ResourceConditionFact::EqZero { .. }, _)
        | (ResourceConditionFact::NeZero { .. }, _)
        | (ResourceConditionFact::Positive { .. }, _)
        | (ResourceConditionFact::NonPositive { .. }, _)
        | (ResourceConditionFact::Negative { .. }, _)
        | (ResourceConditionFact::NonNegative { .. }, _) => None,
    }
}

fn owner_param_value_condition(
    place: &Place,
    condition: I32ValueCondition,
    raw_aliases: &RawCellAddressAliases,
    parameter_storage_sources: &[OwnerParameterStorageSource],
) -> Option<OwnerValueCondition> {
    for place_alias in raw_aliases.aliases_for(place) {
        for source in parameter_storage_sources {
            for param_alias in raw_aliases.aliases_for(&source.place) {
                let Some(suffix) = place_suffix_after_prefix(&place_alias, &param_alias) else {
                    continue;
                };
                return Some(OwnerValueCondition::Param {
                    source: OwnerProjectionSource {
                        parameter_index: source.source.parameter_index,
                        suffix,
                        ty: place_alias.ty,
                    },
                    condition,
                });
            }
        }
    }
    None
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
