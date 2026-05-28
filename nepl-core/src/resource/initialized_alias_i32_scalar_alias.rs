extern crate alloc;

use alloc::vec::Vec;

use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_i32_condition_context::I32ConditionQueryContext;
use super::initialized_alias_i32_offset::{place_has_raw_address_projection, scalar_alias_rank};
use super::model::Place;
use super::place_utils::push_unique_place;

impl RawCellAddressAliases {
    pub(super) fn scalar_aliases_for_value(&self, place: &Place) -> Vec<Place> {
        self.scalar_aliases_for(place)
    }

    pub(super) fn scalar_aliases_for_value_with_context(
        &self,
        place: &Place,
        context: &mut I32ConditionQueryContext,
    ) -> Vec<Place> {
        if let Some(aliases) = context.scalar_aliases(place) {
            return aliases;
        }
        let aliases = self.scalar_aliases_for_value(place);
        context.memoize_scalar_aliases(place, aliases.clone());
        aliases
    }

    pub(super) fn canonicalize_scalar_with_context(
        &self,
        place: &Place,
        context: &mut I32ConditionQueryContext,
    ) -> Place {
        self.scalar_aliases_for_value_with_context(place, context)
            .into_iter()
            .filter(|alias| !place_has_raw_address_projection(alias))
            .min_by_key(scalar_alias_rank)
            .unwrap_or_else(|| place.clone())
    }

    pub(super) fn scalar_fact_recording_sources(&self, place: &Place) -> Vec<Place> {
        let mut sources = Vec::new();
        push_unique_place(&mut sources, &self.canonicalize_scalar(place));
        for alias in self.scalar_aliases_for(place) {
            push_unique_place(&mut sources, &alias);
        }
        sources
    }
}
