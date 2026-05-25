extern crate alloc;

use alloc::string::ToString;

use crate::layout::storage_size_bytes;
use crate::span::Span;
use crate::types::TypeId;

use super::cell_state::{raw_cell_address_prefix, CellTable};
use super::collection_slot_drop_proof::CollectionSlotDropProof;
use super::collection_slot_drop_traversal_range::{
    collection_slot_offset_is_definitely_outside_initialized_count,
    collection_slot_offset_is_inside_initialized_count,
};
use super::collection_slot_drop_traversal_slots::collection_slot_drop_traversal_slots;
use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    CollectionSlotState,
};
use super::collection_slot_owner_transfer_proof::CollectionSlotOwnerTransferProof;
use super::collection_slot_state_alias::storage_alias_covering_slot;
use super::collection_slot_state_identity::slot_requires_range_proof;
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::report::ResourceCheckDiagnostic;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_collection_slot_drop_traversal_with_aliases(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        initialized_count: &Place,
        expected_ty: TypeId,
        span: Span,
    ) {
        let storage = raw_aliases.canonicalize_owner_cell_address(storage);
        let initialized_count = raw_aliases.canonicalize_scalar(initialized_count);
        let result = self.collection_slot_drop_traversal_result(
            cells,
            collection_slots,
            raw_aliases,
            &storage,
            &initialized_count,
            expected_ty,
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
        initialized_count: &Place,
        expected_ty: TypeId,
        span: Span,
    ) {
        self.apply_collection_slot_drop_traversal_with_aliases(
            cells,
            collection_slots,
            raw_aliases,
            storage,
            initialized_count,
            expected_ty,
            span,
        );
    }

    pub(super) fn collection_slot_drop_traversal_result(
        &self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        initialized_count: &Place,
        expected_ty: TypeId,
    ) -> Result<(), CollectionSlotTableRefutation> {
        self.collection_slot_drop_traversal_result_with_drop_proof(
            cells,
            collection_slots,
            raw_aliases,
            storage,
            initialized_count,
            expected_ty,
            CollectionSlotDropProof::LocalLoadedValueDrop,
        )
    }

    pub(super) fn collection_slot_drop_traversal_result_with_drop_proof(
        &self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        initialized_count: &Place,
        expected_ty: TypeId,
        drop_proof: CollectionSlotDropProof,
    ) -> Result<(), CollectionSlotTableRefutation> {
        let slots = collection_slot_drop_traversal_slots(collection_slots, raw_aliases, storage);
        let mut committed_cells = cells.clone();
        let mut committed_slots = collection_slots.clone();
        for (slot, state) in slots {
            match state {
                CollectionSlotState::Initialized(slot_ty) => {
                    let storage_alias = storage_alias_covering_slot(&slot, storage, raw_aliases)
                        .unwrap_or_else(|| storage.clone());
                    let symbolic_range_slot = slot_requires_range_proof(&slot, &storage_alias);
                    if !collection_slot_offset_is_inside_initialized_count(
                        self.types,
                        raw_aliases,
                        &slot,
                        &storage_alias,
                        initialized_count,
                        expected_ty,
                    ) {
                        return Err(CollectionSlotTableRefutation {
                            slot,
                            reason: CollectionSlotLifecycleRefutation::RangeProofRequired {
                                operation: CollectionSlotLifecycleOp::DropTraversal,
                                slot_ty: Some(slot_ty),
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
                    if symbolic_range_slot {
                        committed_slots.set_slot_state(&slot, CollectionSlotState::Uninitialized);
                    }
                }
                CollectionSlotState::MaybeInitialized(slot_ty) => {
                    let storage_alias = storage_alias_covering_slot(&slot, storage, raw_aliases)
                        .unwrap_or_else(|| storage.clone());
                    if collection_slot_offset_is_definitely_outside_initialized_count(
                        self.types,
                        raw_aliases,
                        &slot,
                        &storage_alias,
                        initialized_count,
                        expected_ty,
                    ) {
                        continue;
                    }
                    if slot_ty.is_none() || slot_ty == Some(expected_ty) {
                        committed_slots.set_slot_state_with_aliases(
                            &slot,
                            raw_aliases,
                            CollectionSlotState::Uninitialized,
                        );
                        continue;
                    }
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
        committed_slots.clear_initialized_range_with_aliases(
            self.types,
            storage,
            initialized_count,
            expected_ty,
            storage_size_bytes(self.types, expected_ty),
            raw_aliases,
        );
        *cells = committed_cells;
        *collection_slots = committed_slots;
        Ok(())
    }

    pub(super) fn drop_collection_slot_in_traversal(
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
                cells.mark_raw_cell_moved_with_aliases(raw_aliases, &address, expected_ty);
            }
        }
        collection_slots.apply_slot_event_with_aliases(self.types, slot, raw_aliases, event)?;
        Ok(())
    }
}
