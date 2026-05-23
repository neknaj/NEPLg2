use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::raw_cell_value_flow_alias::{
    canonical_raw_cell_place_with_aliases, place_with_canonical_symbolic_offsets,
};

pub(super) fn collection_slot_event_target(
    target: &Place,
    event: CollectionSlotLifecycleEvent,
    raw_aliases: &RawCellAddressAliases,
) -> Place {
    match event {
        CollectionSlotLifecycleEvent::StorageDealloc => {
            place_with_canonical_symbolic_offsets(target, raw_aliases)
        }
        CollectionSlotLifecycleEvent::InitializeEmpty { .. }
        | CollectionSlotLifecycleEvent::BorrowRead { .. }
        | CollectionSlotLifecycleEvent::MoveOut { .. }
        | CollectionSlotLifecycleEvent::ReplaceInitialized { .. }
        | CollectionSlotLifecycleEvent::DropInitialized { .. } => {
            canonical_raw_cell_place_with_aliases(target, raw_aliases)
        }
    }
}
