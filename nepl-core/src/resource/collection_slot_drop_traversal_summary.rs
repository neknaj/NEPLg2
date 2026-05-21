extern crate alloc;

use alloc::vec::Vec;

use crate::span::Span;
use crate::types::TypeId;

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::{CollectionSlotDropObligation, CollectionSlotDropProof};
use super::collection_slot_drop_traversal::collection_slot_drop_traversal_slots;
use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use super::collection_slot_state_identity::place_covers_slot;
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::report::ResourceCheckDiagnostic;

impl ResourceCheckEngine<'_> {
    pub(super) fn collection_slot_drop_traversal_certified_slots(
        &self,
        cells: &CellTable,
        collection_slots: &CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        expected_ty: TypeId,
    ) -> Option<Vec<Place>> {
        let storage = raw_aliases.canonicalize_owner_cell_address(storage);
        let slots = collection_slot_drop_traversal_slots(collection_slots, &storage);
        let certified_slots: Vec<_> = slots
            .iter()
            .filter_map(|(slot, state)| match state {
                CollectionSlotState::Initialized(_) => Some(slot.clone()),
                CollectionSlotState::Uninitialized
                | CollectionSlotState::MaybeInitialized(_)
                | CollectionSlotState::Moved(_)
                | CollectionSlotState::Dropped(_)
                | CollectionSlotState::Released
                | CollectionSlotState::MaybeReleased => None,
            })
            .collect();
        if certified_slots.is_empty() {
            return None;
        }
        self.collection_slot_drop_traversal_result(
            &mut cells.clone(),
            &mut collection_slots.clone(),
            raw_aliases,
            &storage,
            expected_ty,
        )
        .is_ok()
        .then_some(certified_slots)
    }

    pub(super) fn apply_certified_collection_slot_drop_traversal_slots_with_aliases(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        expected_ty: TypeId,
        certified_slots: &[Place],
        span: Span,
    ) {
        let storage = raw_aliases.canonicalize_owner_cell_address(storage);
        let result = self.certified_collection_slot_drop_traversal_slots_result(
            cells,
            collection_slots,
            raw_aliases,
            &storage,
            expected_ty,
            certified_slots,
        );
        if let Err(refutation) = result {
            self.diagnostics
                .push(ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: self.function.into(),
                    target: refutation.slot,
                    reason: refutation.reason,
                    span,
                });
        }
    }

    fn certified_collection_slot_drop_traversal_slots_result(
        &self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        expected_ty: TypeId,
        certified_slots: &[Place],
    ) -> Result<(), CollectionSlotTableRefutation> {
        let mut committed_cells = cells.clone();
        let mut committed_slots = collection_slots.clone();
        let drop_proof = CollectionSlotDropProof::SummaryCertified(
            CollectionSlotDropObligation::DropLoadedValue {
                operation: CollectionSlotLifecycleOp::DropInitialized,
                value_ty: expected_ty,
            },
        );
        for slot in certified_slots {
            let slot = raw_aliases.canonicalize_owner_cell_address(slot);
            if !place_covers_slot(&slot, storage) {
                return Err(CollectionSlotTableRefutation {
                    slot: slot.clone(),
                    reason: CollectionSlotLifecycleRefutation::Unavailable {
                        operation: CollectionSlotLifecycleOp::DropTraversal,
                        state: committed_slots.state(&slot),
                    },
                });
            }
            self.drop_collection_slot_in_traversal(
                &mut committed_cells,
                &mut committed_slots,
                raw_aliases,
                &slot,
                expected_ty,
                drop_proof,
            )?;
        }
        *cells = committed_cells;
        *collection_slots = committed_slots;
        Ok(())
    }
}
