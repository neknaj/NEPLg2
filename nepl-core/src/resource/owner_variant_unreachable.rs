use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::owner_variant::{PendingUnreachableVariant, PendingVariantOwnerEffects};
use super::owner_variant_condition_truth::owner_value_condition_truth;
use super::summary::OwnerVariantCondition;
use super::variant_name::normalize_variant_name;

impl PendingVariantOwnerEffects {
    pub(super) fn record_unreachable_variants(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        output: &Place,
        args: &[Place],
        conditions: &[OwnerVariantCondition],
    ) {
        let mut variants = Vec::new();
        for condition in conditions {
            if !variants.iter().any(|variant| variant == &condition.variant) {
                variants.push(condition.variant.clone());
            }
        }
        for variant in variants {
            let mut saw_condition = false;
            let mut all_conditions_false = true;
            for condition in conditions
                .iter()
                .filter(|condition| condition.variant == variant)
            {
                saw_condition = true;
                match owner_value_condition_truth(raw_aliases, args, &condition.condition) {
                    Some(false) => {}
                    Some(true) | None => {
                        all_conditions_false = false;
                        break;
                    }
                }
            }
            if saw_condition && all_conditions_false {
                self.push_unique_unreachable(PendingUnreachableVariant {
                    result: output.clone(),
                    variant: normalize_variant_name(&variant),
                });
            }
        }
    }
}
