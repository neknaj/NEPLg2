use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::CollectionSlotLifecycleReturnSlot;
use super::initialized::ResourceCheckEngine;
use super::model::Place;
use super::place_utils::place_with_suffix;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_return_slots(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        output: &Place,
        slots: &[CollectionSlotLifecycleReturnSlot],
    ) {
        collection_slots.clear_storage_prefix(output);
        for slot in slots {
            let target = place_with_suffix(output, &slot.suffix, slot.ty);
            collection_slots.set_return_slot_state(&target, slot.state);
        }
    }
}
