extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeId;

use super::cell_state::CellTable;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleSummaryDropTraversalProof, CollectionSlotLifecycleSummaryPlace,
};
use super::collection_slot_summary_target::instantiate_summary_target;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

pub(super) fn apply_drop_traversal_summary_op(
    engine: &mut ResourceCheckEngine<'_>,
    cells: &mut CellTable,
    collection_slots: &mut CollectionSlotStateTable,
    raw_aliases: &RawCellAddressAliases,
    args: &[Place],
    storage: &CollectionSlotLifecycleSummaryPlace,
    initialized_count: &CollectionSlotLifecycleSummaryPlace,
    expected_ty: TypeId,
    certified_slots: &[CollectionSlotLifecycleSummaryPlace],
    proof: CollectionSlotLifecycleSummaryDropTraversalProof,
    span: Span,
) {
    match proof {
        CollectionSlotLifecycleSummaryDropTraversalProof::CertifiedLoadedValueDrops => {}
    }
    let Some(storage) = instantiate_summary_target(engine, args, storage) else {
        return;
    };
    let Some(initialized_count) = instantiate_summary_target(engine, args, initialized_count)
    else {
        return;
    };
    let mut slots = Vec::new();
    for slot in certified_slots {
        if let Some(slot) = instantiate_summary_target(engine, args, slot) {
            slots.push(slot);
        }
    }
    if slots.len() != certified_slots.len() {
        return;
    }
    engine.apply_certified_collection_slot_drop_traversal_slots_with_aliases(
        cells,
        collection_slots,
        raw_aliases,
        &storage,
        &initialized_count,
        expected_ty,
        &slots,
        span,
    );
}
