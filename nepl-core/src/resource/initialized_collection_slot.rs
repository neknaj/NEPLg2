extern crate alloc;

use alloc::string::ToString;

use crate::span::Span;

use super::cell_state::CellTable;
use super::collection_slot_lifecycle::{
    apply_collection_slot_lifecycle_event, CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp,
    CollectionSlotLifecycleRefutation, CollectionSlotReplacement,
};
use super::collection_slot_owner_transfer::{
    collection_slot_owner_transfer_obligation, consume_collection_slot_owner_transfer_proof,
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
        cells: &mut CellTable,
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
                .and_then(|()| {
                    self.reject_unproven_collection_slot_owner_transfer(
                        cells,
                        collection_slots,
                        target,
                        event,
                    )
                })
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
        cells: &mut CellTable,
        collection_slots: &mut CollectionSlotStateTable,
        raw_aliases: &RawCellAddressAliases,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
        span: Span,
    ) {
        let target = raw_aliases.canonicalize_owner_cell_address(target);
        self.apply_collection_slot_lifecycle(cells, collection_slots, &target, event, span);
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
        let old_storage = raw_aliases.canonicalize_owner_cell_address(old_storage);
        let new_storage = raw_aliases.canonicalize_owner_cell_address(new_storage);
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

    fn reject_unproven_collection_slot_owner_transfer(
        &self,
        cells: &mut CellTable,
        collection_slots: &CollectionSlotStateTable,
        target: &Place,
        event: CollectionSlotLifecycleEvent,
    ) -> Result<(), CollectionSlotTableRefutation> {
        let Some(obligation) = collection_slot_owner_transfer_obligation(self.types, event) else {
            return Ok(());
        };
        let state = collection_slots.state(target);
        apply_collection_slot_lifecycle_event(state, event).map_err(|reason| {
            CollectionSlotTableRefutation {
                slot: target.clone(),
                reason,
            }
        })?;
        if consume_collection_slot_owner_transfer_proof(cells, target, obligation, self.types) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::{vec, vec::Vec};

    use crate::types::{TypeCtx, TypeId, TypeKind};

    use super::super::collection_slot_lifecycle::CollectionSlotState;
    use super::super::collection_slot_summary_model::{
        CollectionSlotLifecycleFunctionSummary, CollectionSlotLifecycleFunctionSummaryIndex,
    };
    use super::super::initialized_alias_flow::{
        RawCellAddressReturnSummary, RawCellAddressReturnSummaryIndex,
    };
    use super::super::initialized_scalar_flow::{
        I32ScalarReturnSummary, I32ScalarReturnSummaryIndex,
    };
    use super::super::initialized_summary::{
        RawCellInitializationFunctionSummary, RawCellInitializationFunctionSummaryIndex,
    };
    use super::super::model::{PlaceProjection, ResourceOffset};
    use super::super::report::ResourceCheckDeferred;

    #[test]
    fn non_copy_initialize_requires_value_flow_proof() {
        let (types, owned_ty) = types_with_non_copy_owned();
        let span = Span::dummy();
        let slot = slot_place(owned_ty);
        let mut cells = CellTable::default();
        let mut collection_slots = CollectionSlotStateTable::new();

        with_engine(&types, |engine| {
            engine.apply_collection_slot_lifecycle(
                &mut cells,
                &mut collection_slots,
                &slot,
                CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned_ty },
                span,
            );

            assert_eq!(
                engine.diagnostics,
                vec![ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: "main".to_string(),
                    target: slot.clone(),
                    reason: CollectionSlotLifecycleRefutation::OwnerTransferRequiresValueProof {
                        operation: CollectionSlotLifecycleOp::InitializeEmpty,
                        slot_ty: owned_ty,
                    },
                    span,
                }]
            );
            assert_eq!(
                collection_slots.state(&slot),
                CollectionSlotState::Uninitialized
            );
        });
    }

    #[test]
    fn non_copy_move_out_requires_value_flow_proof_for_proven_slot_state() {
        let (types, owned_ty) = types_with_non_copy_owned();
        let span = Span::dummy();
        let slot = slot_place(owned_ty);
        let mut cells = CellTable::default();
        let mut collection_slots = CollectionSlotStateTable::new();
        collection_slots.set_slot_state(&slot, CollectionSlotState::Initialized(owned_ty));

        with_engine(&types, |engine| {
            engine.apply_collection_slot_lifecycle(
                &mut cells,
                &mut collection_slots,
                &slot,
                CollectionSlotLifecycleEvent::MoveOut {
                    expected_ty: owned_ty,
                },
                span,
            );

            assert_eq!(
                engine.diagnostics,
                vec![ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: "main".to_string(),
                    target: slot.clone(),
                    reason: CollectionSlotLifecycleRefutation::OwnerTransferRequiresValueProof {
                        operation: CollectionSlotLifecycleOp::MoveOut,
                        slot_ty: owned_ty,
                    },
                    span,
                }]
            );
            assert_eq!(
                collection_slots.state(&slot),
                CollectionSlotState::Initialized(owned_ty)
            );
        });
    }

    #[test]
    fn non_copy_replace_return_old_requires_value_flow_proof_for_proven_slot_state() {
        let (types, owned_ty) = types_with_non_copy_owned();
        let replacement_ty = types.i32();
        let span = Span::dummy();
        let slot = slot_place(owned_ty);
        let mut cells = CellTable::default();
        let mut collection_slots = CollectionSlotStateTable::new();
        collection_slots.set_slot_state(&slot, CollectionSlotState::Initialized(owned_ty));

        with_engine(&types, |engine| {
            engine.apply_collection_slot_lifecycle(
                &mut cells,
                &mut collection_slots,
                &slot,
                CollectionSlotLifecycleEvent::ReplaceInitialized {
                    old_ty: owned_ty,
                    new_ty: replacement_ty,
                    old_owner: CollectionSlotReplacement::ReturnOldOwner,
                },
                span,
            );

            assert_eq!(
                engine.diagnostics,
                vec![ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: "main".to_string(),
                    target: slot.clone(),
                    reason: CollectionSlotLifecycleRefutation::OwnerTransferRequiresValueProof {
                        operation: CollectionSlotLifecycleOp::ReplaceInitialized,
                        slot_ty: owned_ty,
                    },
                    span,
                }]
            );
            assert_eq!(
                collection_slots.state(&slot),
                CollectionSlotState::Initialized(owned_ty)
            );
        });
    }

    #[test]
    fn droppable_drop_initialized_requires_elaboration_for_proven_slot_state() {
        let (types, owned_ty) = types_with_droppable_owned();
        let span = Span::dummy();
        let slot = slot_place(owned_ty);
        let mut cells = CellTable::default();
        let mut collection_slots = CollectionSlotStateTable::new();
        collection_slots.set_slot_state(&slot, CollectionSlotState::Initialized(owned_ty));

        with_engine(&types, |engine| {
            engine.apply_collection_slot_lifecycle(
                &mut cells,
                &mut collection_slots,
                &slot,
                CollectionSlotLifecycleEvent::DropInitialized {
                    expected_ty: owned_ty,
                },
                span,
            );

            assert_eq!(
                engine.diagnostics,
                vec![ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: "main".to_string(),
                    target: slot.clone(),
                    reason: CollectionSlotLifecycleRefutation::DropRequiresElaboration {
                        operation: CollectionSlotLifecycleOp::DropInitialized,
                        slot_ty: owned_ty,
                    },
                    span,
                }]
            );
            assert_eq!(
                collection_slots.state(&slot),
                CollectionSlotState::Initialized(owned_ty)
            );
        });
    }

    #[test]
    fn droppable_replace_drop_old_requires_elaboration_for_proven_slot_state() {
        let (types, owned_ty) = types_with_droppable_owned();
        let replacement_ty = types.i32();
        let span = Span::dummy();
        let slot = slot_place(owned_ty);
        let mut cells = CellTable::default();
        let mut collection_slots = CollectionSlotStateTable::new();
        collection_slots.set_slot_state(&slot, CollectionSlotState::Initialized(owned_ty));

        with_engine(&types, |engine| {
            engine.apply_collection_slot_lifecycle(
                &mut cells,
                &mut collection_slots,
                &slot,
                CollectionSlotLifecycleEvent::ReplaceInitialized {
                    old_ty: owned_ty,
                    new_ty: replacement_ty,
                    old_owner: CollectionSlotReplacement::DropOldOwner,
                },
                span,
            );

            assert_eq!(
                engine.diagnostics,
                vec![ResourceCheckDiagnostic::CollectionSlotRefuted {
                    function: "main".to_string(),
                    target: slot.clone(),
                    reason: CollectionSlotLifecycleRefutation::DropRequiresElaboration {
                        operation: CollectionSlotLifecycleOp::ReplaceInitialized,
                        slot_ty: owned_ty,
                    },
                    span,
                }]
            );
            assert_eq!(
                collection_slots.state(&slot),
                CollectionSlotState::Initialized(owned_ty)
            );
        });
    }

    fn with_engine(types: &TypeCtx, run: impl FnOnce(&mut ResourceCheckEngine<'_>)) {
        let raw_alias_summaries: Vec<RawCellAddressReturnSummary> = Vec::new();
        let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(&raw_alias_summaries);
        let i32_scalar_summaries: Vec<I32ScalarReturnSummary> = Vec::new();
        let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(&i32_scalar_summaries);
        let raw_init_summaries: Vec<RawCellInitializationFunctionSummary> = Vec::new();
        let raw_init_summary_index =
            RawCellInitializationFunctionSummaryIndex::new(&raw_init_summaries);
        let collection_slot_summaries: Vec<CollectionSlotLifecycleFunctionSummary> = Vec::new();
        let collection_slot_summary_index =
            CollectionSlotLifecycleFunctionSummaryIndex::new(&collection_slot_summaries);
        let mut engine = ResourceCheckEngine {
            function: "main",
            types,
            raw_alias_summaries: &raw_alias_summary_index,
            i32_scalar_summaries: &i32_scalar_summary_index,
            raw_init_summaries: &raw_init_summary_index,
            collection_slot_summaries: &collection_slot_summary_index,
            diagnostics: Vec::new(),
            auto_drop_points: Vec::new(),
            deferred: ResourceCheckDeferred::default(),
        };
        run(&mut engine);
    }

    fn types_with_non_copy_owned() -> (TypeCtx, TypeId) {
        let mut types = TypeCtx::new();
        types.set_copy_trait_enabled(true);
        types.register_copy_impl_target(types.unit());
        types.register_copy_impl_target(types.i32());
        let owned_ty = types.register_named(
            "Owned".to_string(),
            TypeKind::Struct {
                name: "Owned".to_string(),
                type_params: Vec::new(),
                fields: Vec::new(),
                field_names: Vec::new(),
            },
        );
        (types, owned_ty)
    }

    fn types_with_droppable_owned() -> (TypeCtx, TypeId) {
        let (mut types, owned_ty) = types_with_non_copy_owned();
        types.register_drop_impl_target(owned_ty);
        (types, owned_ty)
    }

    fn slot_place(owned_ty: TypeId) -> Place {
        Place::local("buffer".to_string(), owned_ty).with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
            owned_ty,
        )
    }
}
