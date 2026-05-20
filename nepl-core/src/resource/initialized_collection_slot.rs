extern crate alloc;

use alloc::string::ToString;

use crate::span::Span;

use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::initialized::ResourceCheckEngine;
use super::model::Place;
use super::report::ResourceCheckDiagnostic;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_lifecycle(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
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
            | CollectionSlotLifecycleEvent::DropInitialized { .. } => {
                collection_slots.apply_slot_event(target, event).map(|_| ())
            }
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

    pub(super) fn apply_collection_storage_relocate(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        old_storage: &Place,
        new_storage: &Place,
        span: Span,
    ) {
        if let Err(refutation) = collection_slots.relocate_storage(old_storage, new_storage) {
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
