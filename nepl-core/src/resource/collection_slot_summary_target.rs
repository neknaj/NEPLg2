extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryPlace;
use super::collection_slot_summary_projection;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceLocal};

pub(super) fn instantiate_summary_target(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    target: &CollectionSlotLifecycleSummaryPlace,
) -> Option<Place> {
    collection_slot_summary_projection::instantiate_summary_place(engine, args, target)
}

pub(super) fn summary_place_for_params(
    params: &[ResourceLocal],
    target: &Place,
) -> Option<CollectionSlotLifecycleSummaryPlace> {
    collection_slot_summary_projection::summary_place_for_params(params, target)
}

pub(super) fn summary_place_for_params_with_aliases(
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    target: &Place,
) -> Option<CollectionSlotLifecycleSummaryPlace> {
    let mut candidates = Vec::new();
    push_unique_place(&mut candidates, target);
    for alias in raw_aliases.raw_address_aliases_for_value(target) {
        push_unique_place(&mut candidates, &alias);
    }
    for alias in raw_aliases.scalar_aliases_for_value(target) {
        push_unique_place(&mut candidates, &alias);
    }
    candidates
        .iter()
        .find_map(|candidate| summary_place_for_params(params, candidate))
}

fn push_unique_place(out: &mut Vec<Place>, place: &Place) {
    if !out.iter().any(|existing| existing == place) {
        out.push(place.clone());
    }
}
