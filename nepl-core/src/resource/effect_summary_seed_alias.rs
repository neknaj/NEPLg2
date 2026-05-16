extern crate alloc;

use alloc::vec::Vec;

use super::model::{AggregateKind, Place};
use super::place_utils::{
    construct_aggregate_field_place, place_suffix_after_prefix, place_with_suffix,
    reference_target_place,
};

#[derive(Debug, Clone, Default)]
pub(super) struct ParameterSeedAliases {
    entries: Vec<ParameterSeedAlias>,
}

#[derive(Debug, Clone)]
struct ParameterSeedAlias {
    alias: Place,
    source: Place,
}

impl ParameterSeedAliases {
    pub(super) fn derived_place(&self, parameter: &Place, place: &Place) -> Option<Place> {
        if place_suffix_after_prefix(place, parameter).is_some() {
            return Some(place.clone());
        }
        for entry in self.entries.iter().rev() {
            let Some(suffix) = place_suffix_after_prefix(place, &entry.alias) else {
                continue;
            };
            return Some(place_with_suffix(&entry.source, &suffix, place.ty));
        }
        None
    }

    pub(super) fn record_copy(&mut self, parameter: &Place, source: &Place, target: &Place) {
        let source = self.derived_place(parameter, source);
        self.clear(target);
        if let Some(source) = source {
            self.entries.push(ParameterSeedAlias {
                alias: target.clone(),
                source,
            });
        }
    }

    pub(super) fn record_borrow(&mut self, parameter: &Place, source: &Place, output: &Place) {
        let Some(source) = self.derived_place(parameter, source) else {
            return;
        };
        let target = reference_target_place(output, source.ty);
        self.clear(&target);
        self.entries.push(ParameterSeedAlias {
            alias: target,
            source,
        });
    }

    pub(super) fn record_construct(
        &mut self,
        parameter: &Place,
        output: &Place,
        kind: &AggregateKind,
        inputs: &[Place],
    ) {
        self.clear(output);
        for (index, input) in inputs.iter().enumerate() {
            let Some(source) = self.derived_place(parameter, input) else {
                continue;
            };
            let alias = construct_aggregate_field_place(output, kind, index, input);
            self.entries.push(ParameterSeedAlias { alias, source });
        }
    }

    fn clear(&mut self, target: &Place) {
        self.entries.retain(|entry| {
            place_suffix_after_prefix(&entry.alias, target).is_none()
                && place_suffix_after_prefix(target, &entry.alias).is_none()
        });
    }
}
