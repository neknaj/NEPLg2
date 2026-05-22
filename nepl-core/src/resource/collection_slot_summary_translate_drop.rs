extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryDropTraversalCoverage, CollectionSlotLifecycleSummaryOp,
    CollectionSlotLifecycleSummaryPlace,
};
use super::collection_slot_summary_target::{
    instantiate_summary_target_with_aliases, summary_place_for_params_with_aliases_and_types,
    translate_summary_target_for_params_with_aliases,
};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, ResourceLocal};

pub(super) fn translate_drop_traversal_summary_op(
    out: &mut Vec<CollectionSlotLifecycleSummaryOp>,
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    storage: &CollectionSlotLifecycleSummaryPlace,
    initialized_count: &CollectionSlotLifecycleSummaryPlace,
    expected_ty: TypeId,
    coverage: &CollectionSlotLifecycleSummaryDropTraversalCoverage,
) {
    let Some(storage) = translate_summary_place(engine, args, params, raw_aliases, storage) else {
        return;
    };
    let Some(initialized_count) =
        translate_summary_place(engine, args, params, raw_aliases, initialized_count)
    else {
        return;
    };
    let Some(coverage) =
        translate_drop_traversal_coverage(engine, args, params, raw_aliases, coverage)
    else {
        return;
    };
    out.push(CollectionSlotLifecycleSummaryOp::DropTraversal {
        storage,
        initialized_count,
        expected_ty,
        coverage,
    });
}

fn translate_drop_traversal_coverage(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    coverage: &CollectionSlotLifecycleSummaryDropTraversalCoverage,
) -> Option<CollectionSlotLifecycleSummaryDropTraversalCoverage> {
    match coverage {
        CollectionSlotLifecycleSummaryDropTraversalCoverage::CertifiedSlots(certified_slots) => {
            Some(
                CollectionSlotLifecycleSummaryDropTraversalCoverage::CertifiedSlots(
                    translate_summary_places(engine, args, params, raw_aliases, certified_slots)?,
                ),
            )
        }
        CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
            certificate,
        ) => Some(
            CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                *certificate,
            ),
        ),
    }
}

fn translate_summary_places(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    places: &[CollectionSlotLifecycleSummaryPlace],
) -> Option<Vec<CollectionSlotLifecycleSummaryPlace>> {
    let mut translated = Vec::new();
    for place in places {
        translated.push(translate_summary_place(
            engine,
            args,
            params,
            raw_aliases,
            place,
        )?);
    }
    Some(translated)
}

fn translate_summary_place(
    engine: &ResourceCheckEngine<'_>,
    args: &[Place],
    params: &[ResourceLocal],
    raw_aliases: &RawCellAddressAliases,
    place: &CollectionSlotLifecycleSummaryPlace,
) -> Option<CollectionSlotLifecycleSummaryPlace> {
    if let Some(place) =
        translate_summary_target_for_params_with_aliases(engine, args, params, raw_aliases, place)
    {
        return Some(place);
    }
    let actual = instantiate_summary_target_with_aliases(engine, args, raw_aliases, place)?;
    let actual = raw_aliases.canonicalize_owner_cell_address(&actual);
    summary_place_for_params_with_aliases_and_types(
        params,
        Some(engine.types),
        raw_aliases,
        &actual,
    )
}
