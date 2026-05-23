extern crate alloc;

use alloc::vec::Vec;

use super::collection_slot_summary_model::CollectionSlotLifecycleSummaryPlace;
use super::collection_slot_summary_projection;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceLocal};
use super::raw_cell_value_flow_alias::raw_cell_place_alias_candidates;
use crate::types::TypeCtx;

#[cfg(test)]
pub(super) fn instantiate_summary_target(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    target: &CollectionSlotLifecycleSummaryPlace,
) -> Option<Place> {
    collection_slot_summary_projection::instantiate_summary_place(engine, args, target)
}

pub(super) fn instantiate_summary_target_with_aliases(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    raw_aliases: &RawCellAddressAliases,
    target: &CollectionSlotLifecycleSummaryPlace,
) -> Option<Place> {
    collection_slot_summary_projection::instantiate_summary_place_with_aliases(
        engine,
        args,
        raw_aliases,
        target,
    )
}

#[cfg(test)]
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
    summary_place_for_params_with_aliases_and_types(params, None, raw_aliases, target)
}

pub(super) fn summary_place_for_params_with_aliases_and_types(
    params: &[ResourceLocal],
    types: Option<&TypeCtx>,
    raw_aliases: &RawCellAddressAliases,
    target: &Place,
) -> Option<CollectionSlotLifecycleSummaryPlace> {
    let mut candidates = raw_cell_place_alias_candidates(target, raw_aliases);
    for alias in raw_aliases.scalar_aliases_for_value(target) {
        push_unique_place(&mut candidates, &alias);
    }
    candidates.iter().find_map(|candidate| {
        if let Some(types) = types {
            collection_slot_summary_projection::summary_place_for_params_with_scalar_aliases_and_types(
                params,
                types,
                raw_aliases,
                candidate,
            )
        } else {
            collection_slot_summary_projection::summary_place_for_params_with_scalar_aliases(
                params,
                raw_aliases,
                candidate,
            )
        }
    })
}

pub(super) fn translate_summary_target_for_params_with_aliases(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    target: &CollectionSlotLifecycleSummaryPlace,
) -> Option<CollectionSlotLifecycleSummaryPlace> {
    collection_slot_summary_projection::translate_summary_place_for_params_with_aliases(
        engine,
        args,
        params,
        raw_aliases,
        target,
    )
}

fn push_unique_place(out: &mut Vec<Place>, place: &Place) {
    if !out.iter().any(|existing| existing == place) {
        out.push(place.clone());
    }
}
