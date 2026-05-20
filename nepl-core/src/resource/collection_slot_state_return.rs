use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::model::Place;
use super::place_utils::push_unique_place;

impl CollectionSlotStateTable {
    pub(super) fn set_return_slot_state(&mut self, target: &Place, state: CollectionSlotState) {
        match state {
            CollectionSlotState::Released => self.set_storage_release_marker(target),
            CollectionSlotState::MaybeReleased => self.set_storage_maybe_release_marker(target),
            _ => self.set_slot_state(target, state),
        }
    }

    fn set_storage_release_marker(&mut self, storage: &Place) {
        self.clear_storage_prefix(storage);
        push_unique_place(&mut self.released_storage, storage);
    }

    fn set_storage_maybe_release_marker(&mut self, storage: &Place) {
        self.clear_storage_prefix(storage);
        push_unique_place(&mut self.maybe_released_storage, storage);
    }
}
