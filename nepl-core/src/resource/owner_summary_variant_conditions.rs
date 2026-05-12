use alloc::vec::Vec;

use crate::types::TypeId;

use super::condition_fact::simple_condition_value_constraint;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{I32ValueCondition, Place, PlaceProjection, ResourceConditionFact};
use super::owner_summary_record::OwnerParameterConditionSource;
use super::owner_summary_variant_i32_conditions::SUMMARY_I32_CONDITIONS;
use super::place_utils::place_suffix_after_prefix;
use super::summary::{OwnerProjectionSource, OwnerValueCondition, OwnerVariantCondition};
use super::variant_name::normalize_variant_name;

pub(super) use super::owner_summary_variant_payload_conditions::{
    collect_owner_variant_known_payload_conditions, collect_owner_variant_payload_conditions,
};

pub(super) fn owner_variant_condition_from_fact(
    condition_fact: &ResourceConditionFact,
    truthy_path: bool,
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
) -> Option<OwnerValueCondition> {
    owner_value_condition(
        condition_fact,
        truthy_path,
        raw_aliases,
        parameter_condition_sources,
    )
}

pub(super) fn push_owner_variant_path_condition(
    out: &mut Vec<OwnerVariantCondition>,
    variant: &str,
    conditions: Vec<OwnerValueCondition>,
) {
    let condition = combined_path_condition(conditions);
    push_unique_variant_condition(
        out,
        OwnerVariantCondition {
            variant: normalize_variant_name(variant),
            condition,
        },
    );
}

pub(super) fn owner_variant_known_conditions(
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
) -> Vec<OwnerValueCondition> {
    let mut out = Vec::new();
    for source in parameter_condition_sources {
        for condition in SUMMARY_I32_CONDITIONS {
            if !raw_aliases.i32_condition_is_known_true(&source.place, condition) {
                continue;
            }
            push_unique_owner_value_condition(
                &mut out,
                OwnerValueCondition::Param {
                    source: source.source.clone(),
                    condition,
                },
            );
        }
    }
    out
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

fn combined_path_condition(conditions: Vec<OwnerValueCondition>) -> OwnerValueCondition {
    let mut out = Vec::new();
    for condition in conditions {
        match condition {
            OwnerValueCondition::Always => {}
            OwnerValueCondition::All(conditions) => {
                for condition in conditions {
                    push_unique_owner_value_condition(&mut out, condition);
                }
            }
            condition => push_unique_owner_value_condition(&mut out, condition),
        }
    }
    match out.len() {
        0 => OwnerValueCondition::Always,
        1 => out.pop().unwrap_or(OwnerValueCondition::Always),
        _ => OwnerValueCondition::All(out),
    }
}

fn push_unique_owner_value_condition(
    out: &mut Vec<OwnerValueCondition>,
    condition: OwnerValueCondition,
) {
    if !out.iter().any(|existing| existing == &condition) {
        out.push(condition);
    }
}
