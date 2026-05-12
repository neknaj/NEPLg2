extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::{I32ValueCondition, Place};
use super::owner_return_apply_source::owner_projection_source_place_for_arg;
use super::owner_summary_record::OwnerParameterConditionSource;
use super::owner_summary_variant_conditions::extend_owner_projection_source;
use super::place_utils::place_suffix_after_prefix;
use super::summary::{OwnerValueCondition, OwnerVariantCondition};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PendingVariantValueCondition {
    pub(super) result: Place,
    pub(super) variant: String,
    condition: PendingValueCondition,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum PendingValueCondition {
    Fact {
        place: Place,
        condition: I32ValueCondition,
    },
    Any(Vec<PendingValueCondition>),
    All(Vec<PendingValueCondition>),
}

impl PendingVariantValueCondition {
    pub(super) fn from_summary_condition(
        raw_aliases: &RawCellAddressAliases,
        result: &Place,
        args: &[Place],
        variant: String,
        entry: &OwnerVariantCondition,
    ) -> Option<Self> {
        Some(Self {
            result: result.clone(),
            variant,
            condition: pending_value_condition(raw_aliases, args, &entry.condition)?,
        })
    }

    pub(super) fn apply_if_selected(
        &self,
        raw_aliases: &mut RawCellAddressAliases,
        scrutinee: &Place,
        variant: &str,
    ) {
        if self.result != *scrutinee || self.variant != variant {
            return;
        }
        self.condition.apply_definite_facts(raw_aliases);
    }

    pub(super) fn with_result(&self, result: Place) -> Self {
        Self {
            result,
            variant: self.variant.clone(),
            condition: self.condition.clone(),
        }
    }

    pub(super) fn selected_summary_condition(
        &self,
        raw_aliases: &RawCellAddressAliases,
        parameter_condition_sources: &[OwnerParameterConditionSource],
        scrutinee: &Place,
        selected_variant: &str,
        output_variant: String,
    ) -> Option<OwnerVariantCondition> {
        if self.result != *scrutinee || self.variant != selected_variant {
            return None;
        }
        Some(OwnerVariantCondition {
            variant: output_variant,
            condition: self
                .condition
                .to_owner_value_condition(raw_aliases, parameter_condition_sources)?,
        })
    }
}

impl PendingValueCondition {
    fn apply_definite_facts(&self, raw_aliases: &mut RawCellAddressAliases) {
        match self {
            PendingValueCondition::Fact { place, condition } => {
                raw_aliases.add_i32_condition(place, *condition);
            }
            PendingValueCondition::All(conditions) => {
                for condition in conditions {
                    condition.apply_definite_facts(raw_aliases);
                }
            }
            PendingValueCondition::Any(_) => {}
        }
    }

    fn to_owner_value_condition(
        &self,
        raw_aliases: &RawCellAddressAliases,
        parameter_condition_sources: &[OwnerParameterConditionSource],
    ) -> Option<OwnerValueCondition> {
        match self {
            PendingValueCondition::Fact { place, condition } => pending_fact_owner_value_condition(
                raw_aliases,
                parameter_condition_sources,
                place,
                *condition,
            ),
            PendingValueCondition::Any(conditions) => {
                Some(OwnerValueCondition::Any(pending_owner_value_conditions(
                    raw_aliases,
                    parameter_condition_sources,
                    conditions,
                )?))
            }
            PendingValueCondition::All(conditions) => {
                Some(OwnerValueCondition::All(pending_owner_value_conditions(
                    raw_aliases,
                    parameter_condition_sources,
                    conditions,
                )?))
            }
        }
    }
}

fn pending_value_condition(
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    condition: &OwnerValueCondition,
) -> Option<PendingValueCondition> {
    match condition {
        OwnerValueCondition::Param { source, condition } => {
            let arg = args.get(source.parameter_index)?;
            let place = owner_projection_source_place_for_arg(arg, source);
            Some(PendingValueCondition::Fact {
                place: raw_aliases.canonicalize(&place),
                condition: *condition,
            })
        }
        OwnerValueCondition::Any(conditions) => {
            let conditions = pending_value_conditions(raw_aliases, args, conditions)?;
            Some(PendingValueCondition::Any(conditions))
        }
        OwnerValueCondition::All(conditions) => {
            let conditions = pending_value_conditions(raw_aliases, args, conditions)?;
            Some(PendingValueCondition::All(conditions))
        }
    }
}

fn pending_value_conditions(
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    conditions: &[OwnerValueCondition],
) -> Option<Vec<PendingValueCondition>> {
    let mut out = Vec::new();
    for condition in conditions {
        out.push(pending_value_condition(raw_aliases, args, condition)?);
    }
    Some(out)
}

fn pending_fact_owner_value_condition(
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
    place: &Place,
    condition: I32ValueCondition,
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

fn pending_owner_value_conditions(
    raw_aliases: &RawCellAddressAliases,
    parameter_condition_sources: &[OwnerParameterConditionSource],
    conditions: &[PendingValueCondition],
) -> Option<Vec<OwnerValueCondition>> {
    let mut out = Vec::new();
    for condition in conditions {
        out.push(condition.to_owner_value_condition(raw_aliases, parameter_condition_sources)?);
    }
    Some(out)
}
