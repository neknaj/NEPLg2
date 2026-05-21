use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_projection::instantiate_summary_suffix_on_base;
use super::collection_slot_summary_return_model::CollectionSlotLifecycleReturnSlot;
use super::initialized::ResourceCheckEngine;
use super::model::Place;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_return_slots(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        args: &[Place],
        output: &Place,
        slots: &[CollectionSlotLifecycleReturnSlot],
    ) {
        for slot in slots {
            let Some(target) =
                instantiate_summary_suffix_on_base(self, args, output, &slot.suffix, slot.ty)
            else {
                continue;
            };
            collection_slots.set_return_slot_state(&target, slot.state);
        }
    }
}
