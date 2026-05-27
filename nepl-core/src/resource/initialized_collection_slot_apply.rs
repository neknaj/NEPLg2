extern crate alloc;

use crate::span::Span;
use alloc::string::ToString;

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::CollectionSlotDropProof;
use super::collection_slot_event_target::collection_slot_event_target;
use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_owner_transfer_proof::CollectionSlotOwnerTransferProof;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_storage_release_proof::{
    consume_collection_slot_storage_release_proof, storage_release_refutation,
    CollectionSlotStorageReleaseProof,
};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDiagnostic;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_lifecycle_with_proofs(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: Option<&RawCellAddressAliases>,
        pending_raw_storage: Option<&mut PendingRawReallocs>,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
        owner_transfer_proof: CollectionSlotOwnerTransferProof,
        drop_proof: CollectionSlotDropProof,
        storage_release_proof: CollectionSlotStorageReleaseProof,
        span: Span,
    ) {
        let canonical_target = raw_aliases
            .map(|raw_aliases| collection_slot_event_target(target, event, raw_aliases))
            .unwrap_or_else(|| target.clone());
        let target = &canonical_target;
        let result = match event {
            CollectionSlotLifecycleEvent::StorageDealloc { .. } => raw_aliases
                .map(|raw_aliases| {
                    collection_slots.storage_release_precondition_with_aliases(target, raw_aliases)
                })
                .unwrap_or_else(|| collection_slots.storage_release_precondition(target))
                .and_then(|()| {
                    if consume_collection_slot_storage_release_proof(
                        pending_raw_storage,
                        raw_aliases,
                        target,
                        storage_release_proof,
                    ) {
                        Ok(())
                    } else {
                        Err(storage_release_refutation(target))
                    }
                })
                .and_then(|()| {
                    raw_aliases
                        .map(|raw_aliases| {
                            collection_slots.release_storage_with_aliases(target, raw_aliases)
                        })
                        .unwrap_or_else(|| collection_slots.release_storage(target))
                        .map(|()| ())
                }),
            CollectionSlotLifecycleEvent::InitializeEmpty { .. }
            | CollectionSlotLifecycleEvent::BorrowRead { .. }
            | CollectionSlotLifecycleEvent::MoveOut { .. }
            | CollectionSlotLifecycleEvent::ReplaceInitialized { .. }
            | CollectionSlotLifecycleEvent::DropInitialized { .. } => self
                .collection_slot_lifecycle_proof_plan(
                    cells,
                    collection_slots,
                    raw_aliases,
                    target,
                    event,
                    drop_proof,
                    owner_transfer_proof,
                )
                .and_then(|plan| {
                    self.consume_collection_slot_lifecycle_proof_plan(
                        cells,
                        raw_aliases,
                        target,
                        plan,
                        drop_proof,
                        owner_transfer_proof,
                    )
                })
                .and_then(|()| {
                    raw_aliases
                        .map(|raw_aliases| {
                            collection_slots.apply_slot_event_with_aliases(
                                self.types,
                                target,
                                raw_aliases,
                                event,
                            )
                        })
                        .unwrap_or_else(|| {
                            collection_slots.apply_slot_event(self.types, target, event)
                        })
                        .map(|_| ())
                }),
        };
        if let Err(refutation) = result {
            let diagnostic_target = raw_aliases
                .map(|raw_aliases| {
                    collection_slot_event_target(&refutation.slot, event, raw_aliases)
                })
                .unwrap_or(refutation.slot);
            self.diagnostics
                .push(ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: self.function.to_string(),
                    target: diagnostic_target,
                    reason: refutation.reason,
                    span,
                });
        }
    }
}
