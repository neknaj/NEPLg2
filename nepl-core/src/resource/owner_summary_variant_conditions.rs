use alloc::vec::Vec;

use super::condition_fact::simple_condition_value_constraint;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{I32ValueCondition, Place, ResourceConditionFact};
use super::owner_summary_record::{OwnerParameterStorageSource, OwnerParameterValueSource};
use super::owner_summary_variant_construct::{normalize_variant_name, ConstructedVariant};
use super::owner_summary_variant_unique::{
    push_unique_variant_condition, push_unique_variant_payload_condition,
};
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
    parameter_value_sources: &[OwnerParameterValueSource],
) -> bool {
    let Some(condition) = owner_value_condition(
        condition_fact,
        truthy_path,
        raw_aliases,
        parameter_storage_sources,
        parameter_value_sources,
    ) else {
        return false;
    };
    push_unique_variant_condition(
        out,
        OwnerVariantCondition {
            variant: normalize_variant_name(variant),
            condition,
        },
    );
    true
}

pub(super) fn collect_owner_variant_unconditional_reachability(
    out: &mut Vec<OwnerVariantCondition>,
    variant: &str,
) {
    push_unique_variant_condition(
        out,
        OwnerVariantCondition {
            variant: normalize_variant_name(variant),
            condition: OwnerValueCondition::All(Vec::new()),
        },
    );
}

fn owner_value_condition(
    condition_fact: &ResourceConditionFact,
    truthy_path: bool,
    raw_aliases: &RawCellAddressAliases,
    parameter_storage_sources: &[OwnerParameterStorageSource],
    parameter_value_sources: &[OwnerParameterValueSource],
) -> Option<OwnerValueCondition> {
    if let Some((place, condition)) = simple_condition_value_constraint(condition_fact, truthy_path)
    {
        return owner_param_value_condition(
            place,
            condition,
            raw_aliases,
            parameter_storage_sources,
            parameter_value_sources,
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
                    parameter_value_sources,
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
                    parameter_value_sources,
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
                    parameter_value_sources,
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
                    parameter_value_sources,
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
    parameter_value_sources: &[OwnerParameterValueSource],
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
    for place_alias in raw_aliases.aliases_for(place) {
        for value_source in parameter_value_sources {
            for param_alias in raw_aliases.aliases_for(&value_source.place) {
                let Some(suffix) = place_suffix_after_prefix(&place_alias, &param_alias) else {
                    continue;
                };
                return Some(OwnerValueCondition::Param {
                    source: OwnerProjectionSource {
                        parameter_index: value_source.source.parameter_index,
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
