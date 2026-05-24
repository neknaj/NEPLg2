extern crate alloc;

use crate::layout::storage_size_bytes;
use crate::span::Span;
use crate::types::TypeId;

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::CollectionSlotDropObligation;
use super::collection_slot_drop_proof::CollectionSlotDropProof;
use super::collection_slot_drop_traversal_range::{
    collection_slot_offset_is_definitely_outside_initialized_count,
    collection_slot_offset_is_inside_initialized_count,
};
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

    pub(super) fn apply_certified_collection_slot_transform_source_drain_with_aliases(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        source_storage: &Place,
        source_initialized_count: &Place,
        expected_ty: TypeId,
        certificate: CollectionSlotTransformRangeCertificate,
        span: Span,
    ) {
        let source_storage = raw_aliases.canonicalize_owner_cell_address(source_storage);
        let source_initialized_count = raw_aliases.canonicalize_scalar(source_initialized_count);
        let result = self.certified_collection_slot_transform_source_drain_result(
            cells,
            collection_slots,
            raw_aliases,
            &source_storage,
            &source_initialized_count,
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

    pub(super) fn apply_local_collection_slot_transform_range_with_aliases(
        &mut self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        source_storage: &Place,
        source_initialized_count: &Place,
        output_storage: &Place,
        output_initialized_count: &Place,
        expected_ty: TypeId,
        span: Span,
    ) {
        let source_storage = raw_aliases.canonicalize_owner_cell_address(source_storage);
        let source_initialized_count = raw_aliases.canonicalize_scalar(source_initialized_count);
        let output_storage = raw_aliases.canonicalize_owner_cell_address(output_storage);
        let output_initialized_count = raw_aliases.canonicalize_scalar(output_initialized_count);
        let Some(certificate) = self.take_local_collection_slot_transform_range_certificate(
            raw_aliases,
            &source_storage,
            &source_initialized_count,
            &output_storage,
            &output_initialized_count,
            expected_ty,
        ) else {
            self.diagnostics
                .push(ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: self.function.into(),
                    target: source_storage,
                    reason: CollectionSlotLifecycleRefutation::RangeProofRequired {
                        operation: CollectionSlotLifecycleOp::MoveOut,
                        slot_ty: Some(expected_ty),
                    },
                    span,
                });
            return;
        };
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
        match result {
            Ok(()) => {
                self.clear_local_transform_source_entries_with_aliases(
                    collection_slots,
                    raw_aliases,
                    &source_storage,
                    &source_initialized_count,
                    expected_ty,
                );
            }
            Err(refutation) => {
                self.diagnostics
                    .push(ResourceCheckDiagnostic::CollectionSlotRefuted {
                        function: self.function.into(),
                        target: refutation.slot,
                        reason: refutation.reason,
                        span,
                    });
            }
        }
    }

    fn take_local_collection_slot_transform_range_certificate(
        &mut self,
        raw_aliases: &RawCellAddressAliases,
        source_storage: &Place,
        source_initialized_count: &Place,
        output_storage: &Place,
        output_initialized_count: &Place,
        expected_ty: TypeId,
    ) -> Option<CollectionSlotTransformRangeCertificate> {
        let candidates = self.transform_range_certificates.as_mut()?;
        let source_storage = raw_aliases.canonicalize_owner_cell_address(source_storage);
        let source_initialized_count = raw_aliases.canonicalize_scalar(source_initialized_count);
        let output_storage = raw_aliases.canonicalize_owner_cell_address(output_storage);
        let output_initialized_count = raw_aliases.canonicalize_scalar(output_initialized_count);
        let index = candidates.iter().rposition(|candidate| {
            raw_aliases.canonicalize_owner_cell_address(&candidate.source_storage) == source_storage
                && raw_aliases.canonicalize_scalar(&candidate.source_initialized_count)
                    == source_initialized_count
                && raw_aliases.canonicalize_owner_cell_address(&candidate.output_storage)
                    == output_storage
                && raw_aliases.canonicalize_scalar(&candidate.output_initialized_count)
                    == output_initialized_count
                && candidate.expected_ty == expected_ty
        })?;
        Some(candidates.remove(index).certificate)
    }

    fn clear_local_transform_source_entries_with_aliases(
        &self,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        source_storage: &Place,
        source_initialized_count: &Place,
        expected_ty: TypeId,
    ) {
        let entries =
            collection_slots.entries_covered_by_storage_with_aliases(source_storage, raw_aliases);
        for entry in entries {
            let storage_alias =
                storage_alias_covering_slot(&entry.slot, source_storage, raw_aliases)
                    .unwrap_or_else(|| source_storage.clone());
            if !collection_slot_offset_is_inside_initialized_count(
                self.types,
                raw_aliases,
                &entry.slot,
                &storage_alias,
                source_initialized_count,
                expected_ty,
            ) && collection_slot_offset_is_definitely_outside_initialized_count(
                self.types,
                raw_aliases,
                &entry.slot,
                &storage_alias,
                source_initialized_count,
                expected_ty,
            ) {
                continue;
            }
            match entry.state {
                CollectionSlotState::Initialized(slot_ty) if slot_ty == expected_ty => {
                    collection_slots
                        .set_slot_state(&entry.slot, CollectionSlotState::Moved(expected_ty));
                }
                CollectionSlotState::MaybeInitialized(slot_ty)
                    if slot_ty.is_none() || slot_ty == Some(expected_ty) =>
                {
                    collection_slots
                        .set_slot_state(&entry.slot, CollectionSlotState::Moved(expected_ty));
                }
                CollectionSlotState::Uninitialized
                | CollectionSlotState::Initialized(_)
                | CollectionSlotState::MaybeInitialized(_)
                | CollectionSlotState::Moved(_)
                | CollectionSlotState::Dropped(_)
                | CollectionSlotState::Released
                | CollectionSlotState::MaybeReleased => {}
            }
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

    fn certified_collection_slot_transform_source_drain_result(
        &self,
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        source_storage: &Place,
        source_initialized_count: &Place,
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
        self.transform_discard_drop_proof_source_only(
            source_storage,
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
        committed_slots.clear_initialized_range_with_aliases(
            source_storage,
            source_initialized_count,
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

    fn transform_discard_drop_proof_source_only(
        &self,
        storage: &Place,
        expected_ty: TypeId,
        proof: CollectionSlotTransformRangeDiscardProof,
    ) -> Result<(), CollectionSlotTableRefutation> {
        match proof {
            CollectionSlotTransformRangeDiscardProof::NoDiscard
                if self.types.is_copy(expected_ty) =>
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

#[cfg(test)]
mod tests {
    use alloc::boxed::Box;
    use alloc::string::ToString;
    use alloc::{vec, vec::Vec};

    use crate::types::{TypeCtx, TypeKind};

    use super::super::collection_slot_summary_build_state::CollectionSlotTransformRangeCertificateCandidate;
    use super::super::collection_slot_summary_model::CollectionSlotLifecycleFunctionSummaryIndex;
    use super::super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
    use super::super::initialized_scalar_flow::I32ScalarReturnSummaryIndex;
    use super::super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
    use super::super::model::{PlaceProjection, ResourceOffset};
    use super::super::report::ResourceCheckDeferred;
    use super::*;
    use crate::span::Span;

    #[test]
    fn local_transform_range_marker_requires_generated_certificate() {
        let (types, owned_ty) = types_with_owned_payload();
        let i32_ty = types.i32();
        let span = Span::dummy();
        let mut engine = test_engine(&types, Vec::new());
        let mut cells = CellTable::default();
        let mut collection_slots = CollectionSlotStateTable::new();
        let raw_aliases = RawCellAddressAliases::default();
        let source_storage = Place::local("source_storage".to_string(), i32_ty);
        let source_count = Place::local("source_count".to_string(), i32_ty);
        let output_storage = Place::local("output_storage".to_string(), i32_ty);
        let output_count = Place::local("output_count".to_string(), i32_ty);

        engine.apply_local_collection_slot_transform_range_with_aliases(
            &mut cells,
            &mut collection_slots,
            &raw_aliases,
            &source_storage,
            &source_count,
            &output_storage,
            &output_count,
            owned_ty,
            span,
        );

        assert!(
            engine.diagnostics.iter().any(|diagnostic| matches!(
                diagnostic,
                ResourceCheckDiagnostic::CollectionSlotRefuted {
                    reason:
                        CollectionSlotLifecycleRefutation::RangeProofRequired {
                            operation: CollectionSlotLifecycleOp::MoveOut,
                            slot_ty: Some(slot_ty),
                        },
                    ..
                } if *slot_ty == owned_ty
            )),
            "a local marker without the loop-derived certificate must not manufacture a non-Copy proof: {:#?}",
            engine.diagnostics
        );
    }

    #[test]
    fn local_transform_range_source_cleanup_preserves_out_of_range_slots() {
        let (types, owned_ty) = types_with_owned_payload();
        let i32_ty = types.i32();
        let span = Span::dummy();
        let source_storage = Place::local("source_storage".to_string(), i32_ty);
        let source_count = Place::local("source_count".to_string(), i32_ty);
        let output_storage = Place::local("output_storage".to_string(), i32_ty);
        let output_count = Place::local("output_count".to_string(), i32_ty);
        let source_slot0 = collection_slot_at(&source_storage, 0, i32_ty, owned_ty);
        let source_slot1 = collection_slot_at(&source_storage, 4, i32_ty, owned_ty);
        let mut engine = test_engine(
            &types,
            vec![transform_candidate(
                &source_storage,
                &source_count,
                &output_storage,
                &output_count,
                owned_ty,
            )],
        );
        let mut cells = CellTable::default();
        let mut collection_slots = CollectionSlotStateTable::new();
        collection_slots.set_slot_state(&source_slot0, CollectionSlotState::Initialized(owned_ty));
        collection_slots.set_slot_state(&source_slot1, CollectionSlotState::Initialized(owned_ty));
        let mut raw_aliases = RawCellAddressAliases::default();
        raw_aliases.set_i32_value(&source_count, 1);
        raw_aliases.set_i32_value(&output_count, 0);

        engine.apply_local_collection_slot_transform_range_with_aliases(
            &mut cells,
            &mut collection_slots,
            &raw_aliases,
            &source_storage,
            &source_count,
            &output_storage,
            &output_count,
            owned_ty,
            span,
        );

        assert_eq!(
            engine.diagnostics,
            Vec::<ResourceCheckDiagnostic>::new(),
            "the certified in-range transform should apply cleanly"
        );
        assert_eq!(
            collection_slots.state_with_aliases(&source_slot1, &raw_aliases),
            CollectionSlotState::Initialized(owned_ty),
            "source cleanup must not hide a live slot outside source_initialized_count"
        );
    }

    fn test_engine<'a>(
        types: &'a TypeCtx,
        transform_range_certificates: Vec<CollectionSlotTransformRangeCertificateCandidate>,
    ) -> ResourceCheckEngine<'a> {
        let raw_alias_summaries = RawCellAddressReturnSummaryIndex::new(&[]);
        let i32_scalar_summaries = I32ScalarReturnSummaryIndex::new(&[]);
        let raw_init_summaries = RawCellInitializationFunctionSummaryIndex::new(&[]);
        let collection_slot_summaries = CollectionSlotLifecycleFunctionSummaryIndex::new(&[]);
        ResourceCheckEngine {
            function: "transform_range_test",
            types,
            raw_alias_summaries: Box::leak(Box::new(raw_alias_summaries)),
            i32_scalar_summaries: Box::leak(Box::new(i32_scalar_summaries)),
            raw_init_summaries: Box::leak(Box::new(raw_init_summaries)),
            collection_slot_summaries: Box::leak(Box::new(collection_slot_summaries)),
            transform_range_certificates: Some(transform_range_certificates),
            diagnostics: Vec::new(),
            auto_drop_points: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
            path_alternatives: Default::default(),
        }
    }

    fn transform_candidate(
        source_storage: &Place,
        source_count: &Place,
        output_storage: &Place,
        output_count: &Place,
        owned_ty: TypeId,
    ) -> CollectionSlotTransformRangeCertificateCandidate {
        CollectionSlotTransformRangeCertificateCandidate {
            source_storage: source_storage.clone(),
            source_initialized_count: source_count.clone(),
            output_storage: output_storage.clone(),
            output_initialized_count: output_count.clone(),
            expected_ty: owned_ty,
            certificate: CollectionSlotTransformRangeCertificate {
                element_stride: 4,
                source_move_proof: CollectionSlotTransformRangeSourceProof::LoadedValueMove(
                    CollectionSlotOwnerTransferObligation::MoveOutValue {
                        operation: CollectionSlotLifecycleOp::MoveOut,
                        value_ty: owned_ty,
                    },
                ),
                output_store_proof: CollectionSlotTransformRangeOutputProof::StoredValue(
                    CollectionSlotOwnerTransferObligation::StoreValue {
                        operation: CollectionSlotLifecycleOp::InitializeEmpty,
                        value_ty: owned_ty,
                    },
                ),
                discard_drop_proof: CollectionSlotTransformRangeDiscardProof::LoadedValueDrop(
                    CollectionSlotDropObligation::DropLoadedValue {
                        operation: CollectionSlotLifecycleOp::DropInitialized,
                        value_ty: owned_ty,
                    },
                ),
            },
        }
    }

    fn collection_slot_at(
        storage: &Place,
        offset: usize,
        i32_ty: TypeId,
        owned_ty: TypeId,
    ) -> Place {
        storage
            .clone()
            .with_projection(
                PlaceProjection::StorageOffset(ResourceOffset::Known(offset)),
                i32_ty,
            )
            .with_projection(PlaceProjection::Deref, owned_ty)
    }

    fn types_with_owned_payload() -> (TypeCtx, TypeId) {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        types.register_copy_impl_target(types.i32());
        types.register_copy_impl_target(types.bool());
        let i32_ty = types.i32();
        let owned_ty = types.register_named(
            "OwnedPayload".to_string(),
            TypeKind::Struct {
                name: "OwnedPayload".to_string(),
                type_params: Vec::new(),
                fields: vec![i32_ty],
                field_names: vec!["value".to_string()],
            },
        );
        types.register_drop_impl_target(owned_ty);
        (types, owned_ty)
    }
}
