use super::cell_state::CellTable;
use super::collection_slot_drop_proof::{
    collection_slot_drop_obligation, consume_collection_slot_drop_proof, CollectionSlotDropProof,
};
use super::collection_slot_lifecycle::{
    apply_collection_slot_lifecycle_event, CollectionSlotLifecycleEvent,
    CollectionSlotLifecycleRefutation,
};
use super::collection_slot_owner_transfer::collection_slot_owner_transfer_obligation;
use super::collection_slot_owner_transfer_proof::{
    consume_collection_slot_owner_transfer_proof, CollectionSlotOwnerTransferProof,
};
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;

impl ResourceCheckEngine<'_> {
    pub(super) fn reject_unproven_collection_slot_drop(
        &self,
        cells: &mut CellTable,
        collection_slots: &CollectionSlotStateTable,
        raw_aliases: Option<&RawCellAddressAliases>,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
        proof: CollectionSlotDropProof,
    ) -> Result<(), CollectionSlotTableRefutation> {
        let Some(obligation) = collection_slot_drop_obligation(self.types, event) else {
            return Ok(());
        };
        apply_event_precondition(collection_slots, target, event)?;
        if consume_collection_slot_drop_proof(
            cells,
            raw_aliases,
            target,
            obligation,
            proof,
            self.types,
        ) {
            Ok(())
        } else {
            let (operation, slot_ty) = obligation.primary_refutation();
            Err(CollectionSlotTableRefutation {
                slot: target.clone(),
                reason: CollectionSlotLifecycleRefutation::DropRequiresElaboration {
                    operation,
                    slot_ty,
                },
            })
        }
    }

    pub(super) fn reject_unproven_collection_slot_owner_transfer(
        &self,
        cells: &mut CellTable,
        collection_slots: &CollectionSlotStateTable,
        raw_aliases: Option<&RawCellAddressAliases>,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
        proof: CollectionSlotOwnerTransferProof,
    ) -> Result<(), CollectionSlotTableRefutation> {
        let Some(obligation) = collection_slot_owner_transfer_obligation(self.types, event) else {
            return Ok(());
        };
        apply_event_precondition(collection_slots, target, event)?;
        if consume_collection_slot_owner_transfer_proof(
            cells,
            raw_aliases,
            target,
            obligation,
            proof,
            self.types,
        ) {
            Ok(())
        } else {
            let (operation, slot_ty) = obligation.primary_refutation();
            Err(CollectionSlotTableRefutation {
                slot: target.clone(),
                reason: CollectionSlotLifecycleRefutation::OwnerTransferRequiresValueProof {
                    operation,
                    slot_ty,
                },
            })
        }
    }
}

fn apply_event_precondition(
    collection_slots: &CollectionSlotStateTable,
    target: &Place,
    event: CollectionSlotLifecycleEvent,
) -> Result<(), CollectionSlotTableRefutation> {
    apply_collection_slot_lifecycle_event(collection_slots.state(target), event)
        .map(|_| ())
        .map_err(|reason| CollectionSlotTableRefutation {
            slot: target.clone(),
            reason,
        })
}
