extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeId;

use super::cell_state::CellTable;
use super::collection_slot_lifecycle::CollectionSlotState;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized::ResourceCheckEngine;
use super::model::Place;
use super::report::ResourceCheckOperation;
use super::type_pattern::type_pattern_matches;

impl ResourceCheckEngine<'_> {
    pub(super) fn certified_collection_managed_non_copy_raw_cells_for_realloc(
        &mut self,
        cells: &CellTable,
        collection_slots: &CollectionSlotStateTable,
        address: &Place,
        span: Span,
    ) -> Option<Vec<Place>> {
        let conflicts = cells.live_non_copy_raw_cells_under(address, self.types);
        let mut certified = Vec::new();
        let mut accepted = true;
        for conflict in conflicts {
            if collection_slot_state_covers_live_raw_cell(
                collection_slots.state(&conflict.place),
                conflict.place.ty,
                self.types,
            ) {
                certified.push(conflict.place);
            } else {
                accepted = false;
                self.push_unavailable(
                    ResourceCheckOperation::RawMemoryReallocCell,
                    &conflict.place,
                    conflict.state,
                    span,
                );
            }
        }
        accepted.then_some(certified)
    }
}

fn collection_slot_state_covers_live_raw_cell(
    state: CollectionSlotState,
    cell_ty: TypeId,
    types: &crate::types::TypeCtx,
) -> bool {
    match state {
        CollectionSlotState::Initialized(slot_ty) => {
            collection_slot_payload_types_match(slot_ty, cell_ty, types)
        }
        CollectionSlotState::MaybeInitialized(Some(slot_ty)) => {
            collection_slot_payload_types_match(slot_ty, cell_ty, types)
        }
        CollectionSlotState::MaybeInitialized(None) => true,
        CollectionSlotState::Uninitialized
        | CollectionSlotState::Moved(_)
        | CollectionSlotState::Dropped(_)
        | CollectionSlotState::Released
        | CollectionSlotState::MaybeReleased => false,
    }
}

fn collection_slot_payload_types_match(
    slot_ty: TypeId,
    cell_ty: TypeId,
    types: &crate::types::TypeCtx,
) -> bool {
    slot_ty == cell_ty
        || type_pattern_matches(types, slot_ty, cell_ty)
        || type_pattern_matches(types, cell_ty, slot_ty)
}
