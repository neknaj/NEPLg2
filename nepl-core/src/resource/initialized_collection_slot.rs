extern crate alloc;

use alloc::string::ToString;

use crate::span::Span;

use super::collection_slot_lifecycle::{
    apply_collection_slot_lifecycle_event, CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp,
    CollectionSlotLifecycleRefutation, CollectionSlotReplacement,
};
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::drop_requirement::resource_type_needs_drop_code;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
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
            | CollectionSlotLifecycleEvent::DropInitialized { .. } => self
                .reject_unelaborated_collection_slot_drop(collection_slots, target, event)
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

    pub(super) fn apply_collection_slot_lifecycle_with_aliases(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
        span: Span,
    ) {
        let target = raw_aliases.canonicalize(target);
        self.apply_collection_slot_lifecycle(collection_slots, &target, event, span);
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

    pub(super) fn apply_collection_storage_relocate_with_aliases(
        &mut self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        old_storage: &Place,
        new_storage: &Place,
        span: Span,
    ) {
        let old_storage = raw_aliases.canonicalize(old_storage);
        let new_storage = raw_aliases.canonicalize(new_storage);
        self.apply_collection_storage_relocate(collection_slots, &old_storage, &new_storage, span);
    }

    fn reject_unelaborated_collection_slot_drop(
        &self,
        collection_slots: &CollectionSlotStateTable,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
    ) -> Result<(), CollectionSlotTableRefutation> {
        let Some((operation, slot_ty)) = collection_slot_drop_obligation(event) else {
            return Ok(());
        };
        let state = collection_slots.state(target);
        apply_collection_slot_lifecycle_event(state, event).map_err(|reason| {
            CollectionSlotTableRefutation {
                slot: target.clone(),
                reason,
            }
        })?;
        if resource_type_needs_drop_code(self.types, slot_ty) {
            Err(CollectionSlotTableRefutation {
                slot: target.clone(),
                reason: CollectionSlotLifecycleRefutation::DropRequiresElaboration {
                    operation,
                    slot_ty,
                },
            })
        } else {
            Ok(())
        }
    }
}

fn collection_slot_drop_obligation(
    event: CollectionSlotLifecycleEvent,
) -> Option<(CollectionSlotLifecycleOp, crate::types::TypeId)> {
    match event {
        CollectionSlotLifecycleEvent::DropInitialized { expected_ty } => {
            Some((CollectionSlotLifecycleOp::DropInitialized, expected_ty))
        }
        CollectionSlotLifecycleEvent::ReplaceInitialized {
            old_ty,
            new_ty: _,
            old_owner: CollectionSlotReplacement::DropOldOwner,
        } => Some((CollectionSlotLifecycleOp::ReplaceInitialized, old_ty)),
        CollectionSlotLifecycleEvent::ReplaceInitialized {
            old_ty: _,
            new_ty: _,
            old_owner: CollectionSlotReplacement::ReturnOldOwner,
        }
        | CollectionSlotLifecycleEvent::InitializeEmpty { .. }
        | CollectionSlotLifecycleEvent::BorrowRead { .. }
        | CollectionSlotLifecycleEvent::MoveOut { .. }
        | CollectionSlotLifecycleEvent::StorageDealloc => None,
    }
}
