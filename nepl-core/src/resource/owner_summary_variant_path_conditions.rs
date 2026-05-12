use alloc::vec::Vec;

use crate::span::Span;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceConditionFact, ResourceMatchArm};
use super::owner_summary_record::OwnerParameterConditionSource;
use super::owner_summary_variant_conditions::{
    owner_variant_condition_from_fact, owner_variant_known_conditions,
    push_owner_variant_path_condition,
};
use super::owner_variant::PendingVariantOwnerEffects;
use super::summary::OwnerVariantCondition;

pub(super) fn record_owner_variant_path_condition(
    out: &mut Vec<OwnerVariantCondition>,
    variant_owner_effects: &PendingVariantOwnerEffects,
    parameter_condition_sources: &[OwnerParameterConditionSource],
    variant: &str,
    entry_raw_aliases: &RawCellAddressAliases,
    exit_raw_aliases: &RawCellAddressAliases,
    branch_condition: Option<(&ResourceConditionFact, bool)>,
    match_arm: Option<(&Place, &ResourceMatchArm, Span)>,
) {
    let mut path_conditions = Vec::new();
    if let Some((condition_fact, truthy_path)) = branch_condition {
        if let Some(condition) = owner_variant_condition_from_fact(
            condition_fact,
            truthy_path,
            entry_raw_aliases,
            parameter_condition_sources,
        ) {
            path_conditions.push(condition);
        }
    }
    if let Some((scrutinee, arm, _span)) = match_arm {
        let mut match_conditions = Vec::new();
        variant_owner_effects.collect_match_arm_value_condition_summaries(
            &mut match_conditions,
            entry_raw_aliases,
            parameter_condition_sources,
            variant,
            scrutinee,
            &arm.pattern,
        );
        path_conditions.extend(
            match_conditions
                .into_iter()
                .map(|condition| condition.condition),
        );
    }
    path_conditions.extend(owner_variant_known_conditions(
        exit_raw_aliases,
        parameter_condition_sources,
    ));
    push_owner_variant_path_condition(out, variant, path_conditions);
}
