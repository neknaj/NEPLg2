extern crate alloc;

use alloc::string::ToString;

use crate::span::Span;

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::CollectionSlotDropProof;
use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_owner_transfer_proof::CollectionSlotOwnerTransferProof;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized::ResourceCheckEngine;
use super::model::Place;
use super::report::ResourceCheckDiagnostic;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_lifecycle_with_proofs(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
        owner_transfer_proof: CollectionSlotOwnerTransferProof,
        drop_proof: CollectionSlotDropProof,
        span: Span,
    ) {
        let result = match event {
            CollectionSlotLifecycleEvent::StorageDealloc => {
                collection_slots.release_storage(target).map(|()| ())
            }
            CollectionSlotLifecycleEvent::InitializeEmpty { .. }
            | CollectionSlotLifecycleEvent::BorrowRead { .. }
            | CollectionSlotLifecycleEvent::MoveOut { .. }
            | CollectionSlotLifecycleEvent::ReplaceInitialized { .. }
            | CollectionSlotLifecycleEvent::DropInitialized { .. } => self
                .reject_unproven_collection_slot_drop(
                    cells,
                    collection_slots,
                    target,
                    event,
                    drop_proof,
                )
                .and_then(|()| {
                    self.reject_unproven_collection_slot_owner_transfer(
                        cells,
                        collection_slots,
                        target,
                        event,
                        owner_transfer_proof,
                    )
                })
                .and_then(|()| collection_slots.apply_slot_event(target, event).map(|_| ())),
        };
        if let Err(refutation) = result {
            self.diagnostics
                .push(ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: self.function.to_string(),
                    target: refutation.slot,
                    reason: refutation.reason,
                    span,
                });
        }
    }
}
