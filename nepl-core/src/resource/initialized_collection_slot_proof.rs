use super::cell_state::CellTable;
use super::collection_slot_drop_proof::{
    collection_slot_drop_obligation, collection_slot_drop_proof_satisfied,
    consume_collection_slot_drop_proof, CollectionSlotDropObligation, CollectionSlotDropProof,
};
use super::collection_slot_lifecycle::{
    apply_collection_slot_lifecycle_event, CollectionSlotLifecycleEvent,
    CollectionSlotLifecycleRefutation,
};
use super::collection_slot_owner_transfer::{
    collection_slot_owner_transfer_obligation, CollectionSlotOwnerTransferObligation,
};
use super::collection_slot_owner_transfer_proof::{
    collection_slot_owner_transfer_proof_satisfied, consume_collection_slot_owner_transfer_proof,
    CollectionSlotOwnerTransferProof,
};
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use crate::types::TypeCtx;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CollectionSlotLifecycleProofPlan {
    drop: Option<CollectionSlotDropObligation>,
    owner_transfer: Option<CollectionSlotOwnerTransferObligation>,
}

impl ResourceCheckEngine<'_> {
    pub(super) fn collection_slot_lifecycle_proof_plan(
        &self,
        cells: &CellTable,
        collection_slots: &CollectionSlotStateTable,
        raw_aliases: Option<&RawCellAddressAliases>,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
        drop_proof: CollectionSlotDropProof,
        owner_transfer_proof: CollectionSlotOwnerTransferProof,
    ) -> Result<CollectionSlotLifecycleProofPlan, CollectionSlotTableRefutation> {
        apply_event_precondition(self.types, collection_slots, target, event)?;

        let drop = collection_slot_drop_obligation(self.types, event);
        if let Some(obligation) = drop {
            if !collection_slot_drop_proof_satisfied(
                cells,
                raw_aliases,
                target,
                obligation,
                drop_proof,
                self.types,
            ) {
                return Err(drop_refutation(target, obligation));
            }
        }

        let owner_transfer = collection_slot_owner_transfer_obligation(self.types, event);
        if let Some(obligation) = owner_transfer {
            if !collection_slot_owner_transfer_proof_satisfied(
                cells,
                raw_aliases,
                target,
                obligation,
                owner_transfer_proof,
                self.types,
            ) {
                return Err(owner_transfer_refutation(target, obligation));
            }
        }

        Ok(CollectionSlotLifecycleProofPlan {
            drop,
            owner_transfer,
        })
    }

    pub(super) fn consume_collection_slot_lifecycle_proof_plan(
        &self,
        cells: &mut CellTable,
        raw_aliases: Option<&RawCellAddressAliases>,
        target: &Place,
        plan: CollectionSlotLifecycleProofPlan,
        drop_proof: CollectionSlotDropProof,
        owner_transfer_proof: CollectionSlotOwnerTransferProof,
    ) -> Result<(), CollectionSlotTableRefutation> {
        let mut committed = cells.clone();
        if let Some(obligation) = plan.drop {
            if !consume_collection_slot_drop_proof(
                &mut committed,
                raw_aliases,
                target,
                obligation,
                drop_proof,
                self.types,
            ) {
                return Err(drop_refutation(target, obligation));
            }
        }
        if let Some(obligation) = plan.owner_transfer {
            if !consume_collection_slot_owner_transfer_proof(
                &mut committed,
                raw_aliases,
                target,
                obligation,
                owner_transfer_proof,
                self.types,
            ) {
                return Err(owner_transfer_refutation(target, obligation));
            }
        }
        *cells = committed;
        Ok(())
    }
}

fn apply_event_precondition(
    types: &TypeCtx,
    collection_slots: &CollectionSlotStateTable,
    target: &Place,
    event: CollectionSlotLifecycleEvent,
) -> Result<(), CollectionSlotTableRefutation> {
    apply_collection_slot_lifecycle_event(types, collection_slots.state(target), event)
        .map(|_| ())
        .map_err(|reason| CollectionSlotTableRefutation {
            slot: target.clone(),
            reason,
        })
}

fn drop_refutation(
    target: &Place,
    obligation: CollectionSlotDropObligation,
) -> CollectionSlotTableRefutation {
    let (operation, slot_ty) = obligation.primary_refutation();
    CollectionSlotTableRefutation {
        slot: target.clone(),
        reason: CollectionSlotLifecycleRefutation::DropRequiresElaboration { operation, slot_ty },
    }
}

fn owner_transfer_refutation(
    target: &Place,
    obligation: CollectionSlotOwnerTransferObligation,
) -> CollectionSlotTableRefutation {
    let (operation, slot_ty) = obligation.primary_refutation();
    CollectionSlotTableRefutation {
        slot: target.clone(),
        reason: CollectionSlotLifecycleRefutation::OwnerTransferRequiresValueProof {
            operation,
            slot_ty,
        },
    }
}
