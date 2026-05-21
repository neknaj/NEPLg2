extern crate alloc;

use alloc::string::ToString;
use alloc::{vec, vec::Vec};

use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::collection_slot_drop_obligation;
use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleEvent, CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation,
    CollectionSlotReplacement, CollectionSlotState,
};
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummary, CollectionSlotLifecycleFunctionSummaryIndex,
    CollectionSlotLifecycleSummaryDropProof, CollectionSlotLifecycleSummaryEventProof,
    CollectionSlotLifecycleSummaryOwnerTransferProof,
};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::{
    RawCellAddressReturnSummary, RawCellAddressReturnSummaryIndex,
};
use super::initialized_scalar_flow::{I32ScalarReturnSummary, I32ScalarReturnSummaryIndex};
use super::initialized_summary::{
    RawCellInitializationFunctionSummary, RawCellInitializationFunctionSummaryIndex,
};
use super::model::{Place, PlaceProjection, ResourceOffset};
use super::place_utils::raw_memory_cell_place;
use super::raw_cell_value_flow::RawCellValueFlowKind;
use super::report::{ResourceCheckDeferred, ResourceCheckDiagnostic};

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
fn droppable_drop_initialized_accepts_loaded_value_drop_proof() {
    let (types, owned_ty) = types_with_droppable_owned();
    let span = Span::dummy();
    let address = slot_address_place(owned_ty);
    let slot = raw_memory_cell_place(&address, owned_ty);
    let mut cells = CellTable::default();
    let mut collection_slots = CollectionSlotStateTable::new();
    collection_slots.set_slot_state(&slot, CollectionSlotState::Initialized(owned_ty));
    cells.record_raw_cell_value_flow(&address, owned_ty, RawCellValueFlowKind::DropLoadedCell);

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

        assert_eq!(engine.diagnostics, vec![]);
        assert_eq!(
            collection_slots.state(&slot),
            CollectionSlotState::Dropped(owned_ty)
        );
    });
}

#[test]
fn droppable_drop_initialized_rejects_loaded_value_without_drop() {
    let (types, owned_ty) = types_with_droppable_owned();
    let span = Span::dummy();
    let address = slot_address_place(owned_ty);
    let slot = raw_memory_cell_place(&address, owned_ty);
    let mut cells = CellTable::default();
    let mut collection_slots = CollectionSlotStateTable::new();
    collection_slots.set_slot_state(&slot, CollectionSlotState::Initialized(owned_ty));
    cells.record_raw_cell_value_flow(&address, owned_ty, RawCellValueFlowKind::MoveOutLoadedCell);

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

#[test]
fn droppable_replace_drop_old_accepts_drop_and_store_proofs() {
    let (types, owned_ty) = types_with_droppable_owned();
    let span = Span::dummy();
    let address = slot_address_place(owned_ty);
    let slot = raw_memory_cell_place(&address, owned_ty);
    let mut cells = CellTable::default();
    let mut collection_slots = CollectionSlotStateTable::new();
    collection_slots.set_slot_state(&slot, CollectionSlotState::Initialized(owned_ty));
    cells.record_raw_cell_value_flow(&address, owned_ty, RawCellValueFlowKind::DropLoadedCell);
    cells.record_raw_cell_value_flow(&address, owned_ty, RawCellValueFlowKind::StoreValue);

    with_engine(&types, |engine| {
        engine.apply_collection_slot_lifecycle(
            &mut cells,
            &mut collection_slots,
            &slot,
            CollectionSlotLifecycleEvent::ReplaceInitialized {
                old_ty: owned_ty,
                new_ty: owned_ty,
                old_owner: CollectionSlotReplacement::DropOldOwner,
            },
            span,
        );

        assert_eq!(engine.diagnostics, vec![]);
        assert_eq!(
            collection_slots.state(&slot),
            CollectionSlotState::Initialized(owned_ty)
        );
    });
}

#[test]
fn summary_certified_drop_proof_allows_caller_replay_without_local_drop_fact() {
    let (types, owned_ty) = types_with_droppable_owned();
    let span = Span::dummy();
    let slot = slot_place(owned_ty);
    let mut cells = CellTable::default();
    let raw_aliases = RawCellAddressAliases::default();
    let mut collection_slots = CollectionSlotStateTable::new();
    collection_slots.set_slot_state(&slot, CollectionSlotState::Initialized(owned_ty));
    let obligation = collection_slot_drop_obligation(
        &types,
        CollectionSlotLifecycleEvent::DropInitialized {
            expected_ty: owned_ty,
        },
    )
    .expect("droppable slot drop should require proof");
    let proof = CollectionSlotLifecycleSummaryEventProof {
        owner_transfer: CollectionSlotLifecycleSummaryOwnerTransferProof::StateOnly,
        slot_drop: CollectionSlotLifecycleSummaryDropProof::LoadedValueDrop(obligation),
    };

    with_engine(&types, |engine| {
        engine.apply_collection_slot_lifecycle_summary_event_with_aliases(
            &mut cells,
            &mut collection_slots,
            &raw_aliases,
            &slot,
            CollectionSlotLifecycleEvent::DropInitialized {
                expected_ty: owned_ty,
            },
            proof,
            span,
        );

        assert_eq!(engine.diagnostics, vec![]);
        assert_eq!(
            collection_slots.state(&slot),
            CollectionSlotState::Dropped(owned_ty)
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
        path_alternatives: Default::default(),
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

fn slot_address_place(owned_ty: TypeId) -> Place {
    Place::local("buffer".to_string(), owned_ty).with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::Known(0)),
        owned_ty,
    )
}

fn slot_place(owned_ty: TypeId) -> Place {
    raw_memory_cell_place(&slot_address_place(owned_ty), owned_ty)
}
