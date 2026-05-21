extern crate alloc;

use alloc::string::ToString;
use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeId;

use super::cell_state::{raw_cell_address_prefix, CellTable};
use super::collection_slot_drop_proof::CollectionSlotDropProof;
use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    CollectionSlotState,
};
use super::collection_slot_owner_transfer_proof::CollectionSlotOwnerTransferProof;
use super::collection_slot_state_table::{
    place_covers_slot, CollectionSlotStateTable, CollectionSlotTableRefutation,
};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::report::ResourceCheckDiagnostic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CollectionSlotDropTraversalProof {
    LocalLoadedValueDrop,
    SummaryCertified,
}

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_drop_traversal_with_aliases(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        expected_ty: TypeId,
        proof: CollectionSlotDropTraversalProof,
        span: Span,
    ) {
        let storage = raw_aliases.canonicalize_owner_cell_address(storage);
        let result = self.collection_slot_drop_traversal_result(
            cells,
            collection_slots,
            raw_aliases,
            &storage,
            expected_ty,
            proof,
        );
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

    pub(super) fn apply_local_collection_slot_drop_traversal(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        expected_ty: TypeId,
        span: Span,
    ) {
        self.apply_collection_slot_drop_traversal_with_aliases(
            cells,
            collection_slots,
            raw_aliases,
            storage,
            expected_ty,
            CollectionSlotDropTraversalProof::LocalLoadedValueDrop,
            span,
        );
    }

    pub(super) fn collection_slot_drop_traversal_available(
        &self,
        cells: &CellTable,
        collection_slots: &CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        expected_ty: TypeId,
    ) -> bool {
        let storage = raw_aliases.canonicalize_owner_cell_address(storage);
        self.collection_slot_drop_traversal_result(
            &mut cells.clone(),
            &mut collection_slots.clone(),
            raw_aliases,
            &storage,
            expected_ty,
            CollectionSlotDropTraversalProof::LocalLoadedValueDrop,
        )
        .is_ok()
    }

    fn collection_slot_drop_traversal_result(
        &self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        expected_ty: TypeId,
        proof: CollectionSlotDropTraversalProof,
    ) -> Result<(), CollectionSlotTableRefutation> {
        let slots = collection_slot_drop_traversal_slots(collection_slots, storage);
        let mut committed_cells = cells.clone();
        let mut committed_slots = collection_slots.clone();
        let drop_proof = match proof {
            CollectionSlotDropTraversalProof::LocalLoadedValueDrop => {
                CollectionSlotDropProof::LocalLoadedValueDrop
            }
            CollectionSlotDropTraversalProof::SummaryCertified => {
                CollectionSlotDropProof::SummaryCertified(
                    super::collection_slot_drop_proof::CollectionSlotDropObligation::DropLoadedValue {
                        operation: CollectionSlotLifecycleOp::DropInitialized,
                        value_ty: expected_ty,
                    },
                )
            }
        };
        for (slot, state) in slots {
            match state {
                CollectionSlotState::Initialized(_) => {
                    self.drop_collection_slot_in_traversal(
                        &mut committed_cells,
                        &mut committed_slots,
                        raw_aliases,
                        &slot,
                        expected_ty,
                        drop_proof,
                    )?;
                }
                CollectionSlotState::MaybeInitialized(slot_ty) => {
                    return Err(CollectionSlotTableRefutation {
                        slot,
                        reason:
                            CollectionSlotLifecycleRefutation::MaybeLiveSlotDuringStorageDealloc {
                                slot_ty,
                            },
                    });
                }
                CollectionSlotState::Uninitialized
                | CollectionSlotState::Moved(_)
                | CollectionSlotState::Dropped(_)
                | CollectionSlotState::Released
                | CollectionSlotState::MaybeReleased => {}
            }
        }
        *cells = committed_cells;
        *collection_slots = committed_slots;
        Ok(())
    }

    fn drop_collection_slot_in_traversal(
        &self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        slot: &Place,
        expected_ty: TypeId,
        drop_proof: CollectionSlotDropProof,
    ) -> Result<(), CollectionSlotTableRefutation> {
        let event = CollectionSlotLifecycleEvent::DropInitialized { expected_ty };
        let plan = self.collection_slot_lifecycle_proof_plan(
            cells,
            collection_slots,
            Some(raw_aliases),
            slot,
            event,
            drop_proof,
            CollectionSlotOwnerTransferProof::LocalRawValueFlow,
        )?;
        self.consume_collection_slot_lifecycle_proof_plan(
            cells,
            Some(raw_aliases),
            slot,
            plan,
            drop_proof,
            CollectionSlotOwnerTransferProof::LocalRawValueFlow,
        )?;
        if matches!(drop_proof, CollectionSlotDropProof::SummaryCertified(_))
            && !self.types.is_copy(expected_ty)
        {
            if let Some(address) = raw_cell_address_prefix(slot) {
                cells.mark_raw_cell_moved(&address, expected_ty);
            }
        }
        collection_slots.apply_slot_event(slot, event)?;
        Ok(())
    }
}

fn collection_slot_drop_traversal_slots(
    collection_slots: &CollectionSlotStateTable,
    storage: &Place,
) -> Vec<(Place, CollectionSlotState)> {
    collection_slots
        .entries()
        .iter()
        .filter_map(|entry| {
            if !place_covers_slot(&entry.slot, storage) {
                return None;
            }
            Some((entry.slot.clone(), entry.state))
        })
        .collect()
}
