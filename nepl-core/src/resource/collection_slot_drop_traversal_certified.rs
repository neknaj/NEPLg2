extern crate alloc;

use crate::layout::storage_size_bytes;
use crate::span::Span;
use crate::types::TypeId;

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::CollectionSlotDropProof;
use super::collection_slot_drop_traversal_range::collection_slot_offset_is_inside_initialized_count;
use super::collection_slot_drop_traversal_summary_proof::summary_certified_drop_traversal_proof;
use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use super::collection_slot_state_alias::{
    place_covers_slot_with_aliases, storage_alias_covering_slot,
};
use super::collection_slot_state_identity::slot_requires_range_proof;
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::collection_slot_summary_model::CollectionSlotInitializedRangeDropTraversalCertificate;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::report::ResourceCheckDiagnostic;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_certified_collection_slot_drop_traversal_slots_with_aliases(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        initialized_count: &Place,
        expected_ty: TypeId,
        certified_slots: &[Place],
        span: Span,
    ) {
        let storage = raw_aliases.canonicalize_owner_cell_address(storage);
        let initialized_count = raw_aliases.canonicalize_scalar(initialized_count);
        let result = self.certified_collection_slot_drop_traversal_slots_result(
            cells,
            collection_slots,
            raw_aliases,
            &storage,
            &initialized_count,
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

    pub(super) fn apply_certified_collection_slot_drop_traversal_range_with_aliases(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        initialized_count: &Place,
        expected_ty: TypeId,
        certificate: CollectionSlotInitializedRangeDropTraversalCertificate,
        span: Span,
    ) {
        let storage = raw_aliases.canonicalize_owner_cell_address(storage);
        let initialized_count = raw_aliases.canonicalize_scalar(initialized_count);
        let result = self.certified_collection_slot_drop_traversal_range_result(
            cells,
            collection_slots,
            raw_aliases,
            &storage,
            &initialized_count,
            expected_ty,
            certificate,
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
        initialized_count: &Place,
        expected_ty: TypeId,
        certified_slots: &[Place],
    ) -> Result<(), CollectionSlotTableRefutation> {
        let mut committed_cells = cells.clone();
        let mut committed_slots = collection_slots.clone();
        let drop_proof = summary_certified_drop_traversal_proof(expected_ty);
        for slot in certified_slots {
            let slot = raw_aliases.canonicalize_owner_cell_address(slot);
            let storage_alias = storage_alias_covering_slot(&slot, storage, raw_aliases)
                .unwrap_or_else(|| storage.clone());
            if !place_covers_slot_with_aliases(&slot, storage, raw_aliases) {
                return Err(CollectionSlotTableRefutation {
                    slot: slot.clone(),
                    reason: CollectionSlotLifecycleRefutation::Unavailable {
                        operation: CollectionSlotLifecycleOp::DropTraversal,
                        state: committed_slots.state(&slot),
                    },
                });
            }
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
                    slot: slot.clone(),
                    reason: CollectionSlotLifecycleRefutation::RangeProofRequired {
                        operation: CollectionSlotLifecycleOp::DropTraversal,
                        slot_ty: Some(expected_ty),
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
        *cells = committed_cells;
        *collection_slots = committed_slots;
        Ok(())
    }

    fn certified_collection_slot_drop_traversal_range_result(
        &self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        initialized_count: &Place,
        expected_ty: TypeId,
        certificate: CollectionSlotInitializedRangeDropTraversalCertificate,
    ) -> Result<(), CollectionSlotTableRefutation> {
        if certificate.element_stride != storage_size_bytes(self.types, expected_ty) {
            return Err(CollectionSlotTableRefutation {
                slot: storage.clone(),
                reason: CollectionSlotLifecycleRefutation::RangeProofRequired {
                    operation: CollectionSlotLifecycleOp::DropTraversal,
                    slot_ty: Some(expected_ty),
                },
            });
        }
        self.collection_slot_drop_traversal_result_with_drop_proof(
            cells,
            collection_slots,
            raw_aliases,
            storage,
            initialized_count,
            expected_ty,
            CollectionSlotDropProof::SummaryCertified(certificate.drop_obligation),
        )
    }
}
