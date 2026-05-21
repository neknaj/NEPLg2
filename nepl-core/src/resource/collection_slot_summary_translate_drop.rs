extern crate alloc;

use alloc::vec::Vec;

use crate::types::TypeId;

use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryDropTraversalProof, CollectionSlotLifecycleSummaryOp,
    CollectionSlotLifecycleSummaryPlace,
};
use super::collection_slot_summary_target::{instantiate_summary_target, summary_place_for_params};
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
    certified_slots: &[CollectionSlotLifecycleSummaryPlace],
    proof: CollectionSlotLifecycleSummaryDropTraversalProof,
) {
    let Some(storage) = translate_summary_place(engine, args, params, raw_aliases, storage) else {
        return;
    };
    let Some(initialized_count) =
        translate_summary_place(engine, args, params, raw_aliases, initialized_count)
    else {
        return;
    };
    let Some(certified_slots) =
        translate_summary_places(engine, args, params, raw_aliases, certified_slots)
    else {
        return;
    };
    out.push(CollectionSlotLifecycleSummaryOp::DropTraversal {
        storage,
        initialized_count,
        expected_ty,
        certified_slots,
        proof,
    });
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
    let actual = instantiate_summary_target(engine, args, place)?;
    let actual = raw_aliases.canonicalize_owner_cell_address(&actual);
    summary_place_for_params(params, &actual)
}
