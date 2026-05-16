use alloc::vec::Vec;

use super::effect_summary_seed_alias::ParameterSeedAliases;
use super::effect_summary_seed_walk::collect_parameter_descendant_places;
use super::model::{Place, ResourceFunction, ResourceTerminator};
use super::place_utils::push_unique_place;

pub(super) fn parameter_summary_seed_places(
    function: &ResourceFunction,
    parameter: &Place,
) -> Vec<Place> {
    let mut places = Vec::new();
    let mut aliases = ParameterSeedAliases::default();
    push_unique_place(&mut places, parameter);
    for block in &function.blocks {
        collect_parameter_descendant_places(&block.ops, parameter, &mut aliases, &mut places);
        if let ResourceTerminator::Return {
            value: Some(value), ..
        } = &block.terminator
        {
            if let Some(seed) = aliases.derived_place(parameter, value) {
                push_unique_place(&mut places, &seed);
            }
        }
    }
    places.sort();
    places
}
