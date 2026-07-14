use alloc::vec::Vec;

use crate::types::TypeCtx;

use super::condition_fact::simple_condition_value_constraint;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{I32ValueCondition, Place, PlaceProjection, ResourceConditionFact};
use super::owner_summary_i32_leaf::i32_leaf_places;
use super::owner_summary_variant_construct::ConstructedVariant;
use super::owner_summary_variant_i32_conditions::SUMMARY_I32_CONDITIONS;
use super::place_utils::{place_suffix_after_prefix, place_with_suffix};
use super::summary::OwnerVariantPayloadCondition;
use super::variant_name::normalize_variant_name;

#[derive(Default)]
pub(super) struct OwnerVariantPayloadConditionAccumulator {
    conditions: Vec<OwnerVariantPayloadCondition>,
    seen_variants: Vec<alloc::string::String>,
    observations: usize,
    unknown_path_seen: bool,
}

impl OwnerVariantPayloadConditionAccumulator {
    pub(super) fn merge_path(
        &mut self,
        variant: alloc::string::String,
        path_conditions: Vec<OwnerVariantPayloadCondition>,
    ) {
        self.observations += 1;
        if self.unknown_path_seen {
            if !self.seen_variants.iter().any(|seen| seen == &variant) {
                self.seen_variants.push(variant);
            }
            return;
        }
        if self.seen_variants.iter().any(|seen| seen == &variant) {
            self.conditions.retain(|existing| {
                existing.variant != variant || path_conditions.contains(existing)
            });
            return;
        }
        self.seen_variants.push(variant);
        for condition in path_conditions {
            push_unique_variant_payload_condition(&mut self.conditions, condition);
        }
    }

    pub(super) fn observation_count(&self) -> usize {
        self.observations
    }

    pub(super) fn observe_unknown_path(&mut self) {
        self.observations += 1;
        self.unknown_path_seen = true;
        self.conditions.clear();
    }

    pub(super) fn into_conditions(self) -> Vec<OwnerVariantPayloadCondition> {
        self.conditions
    }
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
        let leaves = i32_leaf_places(types, &payload_place);
        let input_leaves = i32_leaf_places(types, &payload.input);
        for leaf in leaves {
            let input_leaf = input_payload_leaf(&input_leaves, &leaf.suffix);
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
    input_leaves: &[super::owner_summary_leaf::OwnerLeafPlace],
    leaf_suffix: &[PlaceProjection],
) -> Option<Place> {
    input_leaves
        .iter()
        .find(|input_leaf| input_leaf.suffix == leaf_suffix)
        .map(|input_leaf| input_leaf.place.clone())
}

fn push_unique_variant_payload_condition(
    out: &mut Vec<OwnerVariantPayloadCondition>,
    entry: OwnerVariantPayloadCondition,
) {
    if !out.iter().any(|existing| existing == &entry) {
        out.push(entry);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TypeId;

    fn condition(variant: &str, condition: I32ValueCondition) -> OwnerVariantPayloadCondition {
        OwnerVariantPayloadCondition {
            variant: variant.into(),
            suffix: alloc::vec![PlaceProjection::EnumPayload {
                variant: variant.into(),
            }],
            ty: TypeId(1),
            condition,
        }
    }

    #[test]
    fn payload_condition_accumulator_intersects_same_variant_paths() {
        let mut accumulator = OwnerVariantPayloadConditionAccumulator::default();
        accumulator.merge_path(
            "Ok".into(),
            alloc::vec![
                condition("Ok", I32ValueCondition::EqZero),
                condition("Ok", I32ValueCondition::NonNegative),
                condition("Ok", I32ValueCondition::NonPositive),
            ],
        );
        accumulator.merge_path(
            "Ok".into(),
            alloc::vec![
                condition("Ok", I32ValueCondition::NeZero),
                condition("Ok", I32ValueCondition::NonNegative),
                condition("Ok", I32ValueCondition::Positive),
            ],
        );

        assert_eq!(
            accumulator.into_conditions(),
            alloc::vec![condition("Ok", I32ValueCondition::NonNegative)]
        );
    }

    #[test]
    fn payload_condition_accumulator_keeps_variants_independent_and_observes_empty_paths() {
        let mut accumulator = OwnerVariantPayloadConditionAccumulator::default();
        accumulator.merge_path(
            "Ok".into(),
            alloc::vec![condition("Ok", I32ValueCondition::EqZero)],
        );
        accumulator.merge_path(
            "Err".into(),
            alloc::vec![condition("Err", I32ValueCondition::Negative)],
        );
        accumulator.merge_path("Ok".into(), Vec::new());

        assert_eq!(
            accumulator.into_conditions(),
            alloc::vec![condition("Err", I32ValueCondition::Negative)]
        );
    }

    #[test]
    fn payload_condition_accumulator_clears_facts_across_unknown_paths_in_any_order() {
        for unknown_first in [false, true] {
            let mut accumulator = OwnerVariantPayloadConditionAccumulator::default();
            if unknown_first {
                accumulator.observe_unknown_path();
            }
            accumulator.merge_path(
                "Ok".into(),
                alloc::vec![condition("Ok", I32ValueCondition::EqZero)],
            );
            if !unknown_first {
                accumulator.observe_unknown_path();
            }
            assert!(accumulator.into_conditions().is_empty());
        }
    }
}
