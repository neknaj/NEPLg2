extern crate alloc;

use crate::layout::storage_size_bytes;
use crate::span::Span;
use crate::types::TypeId;

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::CollectionSlotDropObligation;
use super::collection_slot_drop_proof::CollectionSlotDropProof;
use super::collection_slot_drop_traversal_range::collection_slot_offset_is_inside_initialized_count;
use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    CollectionSlotState,
};
use super::collection_slot_owner_transfer::CollectionSlotOwnerTransferObligation;
use super::collection_slot_owner_transfer_proof::CollectionSlotOwnerTransferProof;
use super::collection_slot_state_alias::storage_alias_covering_slot;
use super::collection_slot_state_table::{CollectionSlotStateTable, CollectionSlotTableRefutation};
use super::collection_slot_summary_model::{
    CollectionSlotTransformRangeCertificate, CollectionSlotTransformRangeDiscardProof,
    CollectionSlotTransformRangeOutputProof, CollectionSlotTransformRangeSourceProof,
};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::model::Place;
use super::report::ResourceCheckDiagnostic;

impl ResourceCheckEngine<'_> {
    pub(super) fn apply_certified_collection_slot_transform_range_with_aliases(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        source_storage: &Place,
        source_initialized_count: &Place,
        output_storage: &Place,
        output_initialized_count: &Place,
        expected_ty: TypeId,
        certificate: CollectionSlotTransformRangeCertificate,
        span: Span,
    ) {
        let source_storage = raw_aliases.canonicalize_owner_cell_address(source_storage);
        let source_initialized_count = raw_aliases.canonicalize_scalar(source_initialized_count);
        let output_storage = raw_aliases.canonicalize_owner_cell_address(output_storage);
        let output_initialized_count = raw_aliases.canonicalize_scalar(output_initialized_count);
        let result = self.certified_collection_slot_transform_range_result(
            cells,
            collection_slots,
            raw_aliases,
            &source_storage,
            &source_initialized_count,
            &output_storage,
            &output_initialized_count,
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

    fn certified_collection_slot_transform_range_result(
        &self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        source_storage: &Place,
        source_initialized_count: &Place,
        output_storage: &Place,
        output_initialized_count: &Place,
        expected_ty: TypeId,
        certificate: CollectionSlotTransformRangeCertificate,
    ) -> Result<(), CollectionSlotTableRefutation> {
        if certificate.element_stride != storage_size_bytes(self.types, expected_ty) {
            return Err(range_proof_refutation(
                source_storage,
                CollectionSlotLifecycleOp::MoveOut,
                expected_ty,
            ));
        }
        let source_proof = self.transform_source_move_proof(
            source_storage,
            expected_ty,
            certificate.source_move_proof,
        )?;
        let output_proof = self.transform_output_store_proof(
            output_storage,
            expected_ty,
            certificate.output_store_proof,
        )?;
        self.transform_discard_drop_proof(
            source_storage,
            source_initialized_count,
            output_initialized_count,
            expected_ty,
            certificate.discard_drop_proof,
        )?;
        let mut committed_cells = cells.clone();
        let mut committed_slots = collection_slots.clone();
        self.move_out_source_slots_in_transform_range(
            &mut committed_cells,
            &mut committed_slots,
            raw_aliases,
            source_storage,
            source_initialized_count,
            expected_ty,
            source_proof,
        )?;
        self.initialize_tracked_output_slots_in_transform_range(
            &mut committed_cells,
            &mut committed_slots,
            raw_aliases,
            output_storage,
            output_initialized_count,
            expected_ty,
            output_proof,
        )?;
        committed_slots.clear_initialized_range_with_aliases(
            source_storage,
            source_initialized_count,
            expected_ty,
            certificate.element_stride,
            raw_aliases,
        );
        committed_slots.mark_initialized_range_with_aliases(
            output_storage,
            output_initialized_count,
            expected_ty,
            certificate.element_stride,
            raw_aliases,
        );
        *cells = committed_cells;
        *collection_slots = committed_slots;
        Ok(())
    }

    fn move_out_source_slots_in_transform_range(
        &self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        initialized_count: &Place,
        expected_ty: TypeId,
        owner_transfer_proof: CollectionSlotOwnerTransferProof,
    ) -> Result<(), CollectionSlotTableRefutation> {
        let entries =
            collection_slots.entries_covered_by_storage_with_aliases(storage, raw_aliases);
        for entry in entries {
            if !collection_slot_offset_is_inside_initialized_count(
                self.types,
                raw_aliases,
                &entry.slot,
                storage_alias_covering_slot(&entry.slot, storage, raw_aliases)
                    .as_ref()
                    .unwrap_or(storage),
                initialized_count,
                expected_ty,
            ) {
                continue;
            }
            match entry.state {
                CollectionSlotState::Initialized(_) => {
                    let event = CollectionSlotLifecycleEvent::MoveOut { expected_ty };
                    let plan = self.collection_slot_lifecycle_proof_plan(
                        cells,
                        collection_slots,
                        Some(raw_aliases),
                        &entry.slot,
                        event,
                        CollectionSlotDropProof::LocalLoadedValueDrop,
                        owner_transfer_proof,
                    )?;
                    self.consume_collection_slot_lifecycle_proof_plan(
                        cells,
                        Some(raw_aliases),
                        &entry.slot,
                        plan,
                        CollectionSlotDropProof::LocalLoadedValueDrop,
                        owner_transfer_proof,
                    )?;
                    collection_slots.apply_slot_event_with_aliases(
                        self.types,
                        &entry.slot,
                        raw_aliases,
                        event,
                    )?;
                }
                CollectionSlotState::MaybeInitialized(slot_ty) => {
                    return Err(CollectionSlotTableRefutation {
                        slot: entry.slot,
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
        Ok(())
    }

    fn initialize_tracked_output_slots_in_transform_range(
        &self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        storage: &Place,
        initialized_count: &Place,
        expected_ty: TypeId,
        owner_transfer_proof: CollectionSlotOwnerTransferProof,
    ) -> Result<(), CollectionSlotTableRefutation> {
        let entries =
            collection_slots.entries_covered_by_storage_with_aliases(storage, raw_aliases);
        for entry in entries {
            if !collection_slot_offset_is_inside_initialized_count(
                self.types,
                raw_aliases,
                &entry.slot,
                storage_alias_covering_slot(&entry.slot, storage, raw_aliases)
                    .as_ref()
                    .unwrap_or(storage),
                initialized_count,
                expected_ty,
            ) {
                continue;
            }
            let event = CollectionSlotLifecycleEvent::InitializeEmpty {
                value_ty: expected_ty,
            };
            let plan = self.collection_slot_lifecycle_proof_plan(
                cells,
                collection_slots,
                Some(raw_aliases),
                &entry.slot,
                event,
                CollectionSlotDropProof::LocalLoadedValueDrop,
                owner_transfer_proof,
            )?;
            self.consume_collection_slot_lifecycle_proof_plan(
                cells,
                Some(raw_aliases),
                &entry.slot,
                plan,
                CollectionSlotDropProof::LocalLoadedValueDrop,
                owner_transfer_proof,
            )?;
            collection_slots.apply_slot_event_with_aliases(
                self.types,
                &entry.slot,
                raw_aliases,
                event,
            )?;
        }
        Ok(())
    }

    fn transform_source_move_proof(
        &self,
        storage: &Place,
        expected_ty: TypeId,
        proof: CollectionSlotTransformRangeSourceProof,
    ) -> Result<CollectionSlotOwnerTransferProof, CollectionSlotTableRefutation> {
        match proof {
            CollectionSlotTransformRangeSourceProof::StateOnly
                if self.types.is_copy(expected_ty) =>
            {
                Ok(CollectionSlotOwnerTransferProof::SummaryStateOnly)
            }
            CollectionSlotTransformRangeSourceProof::LoadedValueMove(obligation)
                if obligation
                    == (CollectionSlotOwnerTransferObligation::MoveOutValue {
                        operation: CollectionSlotLifecycleOp::MoveOut,
                        value_ty: expected_ty,
                    }) =>
            {
                Ok(CollectionSlotOwnerTransferProof::SummaryCertified(
                    obligation,
                ))
            }
            _ => Err(CollectionSlotTableRefutation {
                slot: storage.clone(),
                reason: CollectionSlotLifecycleRefutation::OwnerTransferRequiresValueProof {
                    operation: CollectionSlotLifecycleOp::MoveOut,
                    slot_ty: expected_ty,
                },
            }),
        }
    }

    fn transform_output_store_proof(
        &self,
        storage: &Place,
        expected_ty: TypeId,
        proof: CollectionSlotTransformRangeOutputProof,
    ) -> Result<CollectionSlotOwnerTransferProof, CollectionSlotTableRefutation> {
        match proof {
            CollectionSlotTransformRangeOutputProof::StateOnly
                if self.types.is_copy(expected_ty) =>
            {
                Ok(CollectionSlotOwnerTransferProof::SummaryStateOnly)
            }
            CollectionSlotTransformRangeOutputProof::StoredValue(obligation)
                if obligation
                    == (CollectionSlotOwnerTransferObligation::StoreValue {
                        operation: CollectionSlotLifecycleOp::InitializeEmpty,
                        value_ty: expected_ty,
                    }) =>
            {
                Ok(CollectionSlotOwnerTransferProof::SummaryCertified(
                    obligation,
                ))
            }
            _ => Err(CollectionSlotTableRefutation {
                slot: storage.clone(),
                reason: CollectionSlotLifecycleRefutation::OwnerTransferRequiresValueProof {
                    operation: CollectionSlotLifecycleOp::InitializeEmpty,
                    slot_ty: expected_ty,
                },
            }),
        }
    }

    fn transform_discard_drop_proof(
        &self,
        storage: &Place,
        source_initialized_count: &Place,
        output_initialized_count: &Place,
        expected_ty: TypeId,
        proof: CollectionSlotTransformRangeDiscardProof,
    ) -> Result<(), CollectionSlotTableRefutation> {
        match proof {
            CollectionSlotTransformRangeDiscardProof::NoDiscard
                if self.types.is_copy(expected_ty)
                    || source_initialized_count == output_initialized_count =>
            {
                Ok(())
            }
            CollectionSlotTransformRangeDiscardProof::LoadedValueDrop(obligation)
                if obligation
                    == (CollectionSlotDropObligation::DropLoadedValue {
                        operation: CollectionSlotLifecycleOp::DropInitialized,
                        value_ty: expected_ty,
                    }) =>
            {
                Ok(())
            }
            _ => Err(CollectionSlotTableRefutation {
                slot: storage.clone(),
                reason: CollectionSlotLifecycleRefutation::DropRequiresElaboration {
                    operation: CollectionSlotLifecycleOp::DropInitialized,
                    slot_ty: expected_ty,
                },
            }),
        }
    }
}

fn range_proof_refutation(
    slot: &Place,
    operation: CollectionSlotLifecycleOp,
    expected_ty: TypeId,
) -> CollectionSlotTableRefutation {
    CollectionSlotTableRefutation {
        slot: slot.clone(),
        reason: CollectionSlotLifecycleRefutation::RangeProofRequired {
            operation,
            slot_ty: Some(expected_ty),
        },
    }
}
