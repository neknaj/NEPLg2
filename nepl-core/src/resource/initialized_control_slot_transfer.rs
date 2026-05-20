use crate::span::Span;

use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized::ResourceCheckEngine;
use super::model::Place;

pub(super) fn transfer_control_value_slots(
    engine: &mut ResourceCheckEngine<'_>,
    collection_slots: &mut CollectionSlotStateTable,
    source: &Place,
    target: &Place,
    span: Span,
) {
    engine.transfer_slot_state_if_moved(collection_slots, source, target, span);
}
