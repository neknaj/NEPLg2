extern crate alloc;

use alloc::{string::ToString, vec, vec::Vec};

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::collection_slot_drop_proof::CollectionSlotDropObligation;
use super::collection_slot_lifecycle::{CollectionSlotLifecycleOp, CollectionSlotState};
use super::collection_slot_owner_transfer::CollectionSlotOwnerTransferObligation;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::{
    CollectionSlotInitializedRangeDropTraversalCertificate,
    CollectionSlotInitializedRangeDropTraversalProof, CollectionSlotLifecycleFunctionSummaryIndex,
    CollectionSlotTransformRangeCertificate, CollectionSlotTransformRangeDiscardProof,
    CollectionSlotTransformRangeOutputProof, CollectionSlotTransformRangeSourceProof,
};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_scalar_flow::I32ScalarReturnSummaryIndex;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::model::{Place, PlaceProjection, ResourceOffset};
use super::report::ResourceCheckDeferred;

#[test]
fn collection_slot_summary_forall_replay_drops_every_initialized_slot_in_range() {
    let (types, owned_ty) = types_with_sized_droppable_owned_for_summary();
    let i32_ty = types.i32();
    let storage = Place::local("storage".to_string(), i32_ty);
    let initialized_count = Place::i32_constant(2, i32_ty);
    let slot_0 = slot_at(storage.clone(), 0, i32_ty, owned_ty);
    let slot_1 = slot_at(storage.clone(), 4, i32_ty, owned_ty);
    let raw_alias_summaries = [];
    let i32_scalar_summaries = [];
    let raw_init_summaries = [];
    let collection_slot_summaries = [];
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(&raw_alias_summaries);
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(&i32_scalar_summaries);
    let raw_init_summary_index =
        RawCellInitializationFunctionSummaryIndex::new(&raw_init_summaries);
    let collection_slot_summary_index =
        CollectionSlotLifecycleFunctionSummaryIndex::new(&collection_slot_summaries);
    let mut engine = summary_test_engine(
        &types,
        &raw_alias_summary_index,
        &i32_scalar_summary_index,
        &raw_init_summary_index,
        &collection_slot_summary_index,
    );
    let mut cells = CellTable::default();
    let mut collection_slots = CollectionSlotStateTable::new();
    let raw_aliases = RawCellAddressAliases::default();
    collection_slots.set_slot_state(&slot_0, CollectionSlotState::Initialized(owned_ty));
    collection_slots.set_slot_state(&slot_1, CollectionSlotState::Initialized(owned_ty));

    engine.apply_certified_collection_slot_drop_traversal_range_with_aliases(
        &mut cells,
        &mut collection_slots,
        &raw_aliases,
        &storage,
        &initialized_count,
        owned_ty,
        CollectionSlotInitializedRangeDropTraversalCertificate {
            element_stride: 4,
            drop_proof: CollectionSlotInitializedRangeDropTraversalProof::LoadedValueDrop(
                CollectionSlotDropObligation::DropLoadedValue {
                    operation: CollectionSlotLifecycleOp::DropInitialized,
                    value_ty: owned_ty,
                },
            ),
        },
        crate::span::Span::dummy(),
    );

    assert!(
        engine.diagnostics.is_empty(),
        "full-range summary replay must accept all caller initialized slots in range: {:#?}",
        engine.diagnostics
    );
    assert!(collection_slots.entries().iter().all(|entry| {
        matches!(entry.state, CollectionSlotState::Dropped(slot_ty) if slot_ty == owned_ty)
    }));
}

#[test]
fn collection_slot_summary_transform_replay_moves_source_and_initializes_output_range() {
    let (types, owned_ty) = types_with_sized_droppable_owned_for_summary();
    let i32_ty = types.i32();
    let source_storage = Place::local("source_storage".to_string(), i32_ty);
    let output_storage = Place::local("output_storage".to_string(), i32_ty);
    let source_initialized_count = Place::i32_constant(2, i32_ty);
    let output_initialized_count = Place::i32_constant(1, i32_ty);
    let source_slot_0 = slot_at(source_storage.clone(), 0, i32_ty, owned_ty);
    let source_slot_1 = slot_at(source_storage.clone(), 4, i32_ty, owned_ty);
    let output_slot_0 = slot_at(output_storage.clone(), 0, i32_ty, owned_ty);
    let output_slot_1 = slot_at(output_storage.clone(), 4, i32_ty, owned_ty);
    let raw_alias_summaries = [];
    let i32_scalar_summaries = [];
    let raw_init_summaries = [];
    let collection_slot_summaries = [];
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(&raw_alias_summaries);
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(&i32_scalar_summaries);
    let raw_init_summary_index =
        RawCellInitializationFunctionSummaryIndex::new(&raw_init_summaries);
    let collection_slot_summary_index =
        CollectionSlotLifecycleFunctionSummaryIndex::new(&collection_slot_summaries);
    let mut engine = summary_test_engine(
        &types,
        &raw_alias_summary_index,
        &i32_scalar_summary_index,
        &raw_init_summary_index,
        &collection_slot_summary_index,
    );
    let mut cells = CellTable::default();
    let mut collection_slots = CollectionSlotStateTable::new();
    let raw_aliases = RawCellAddressAliases::default();
    collection_slots.set_slot_state(&source_slot_0, CollectionSlotState::Initialized(owned_ty));
    collection_slots.set_slot_state(&source_slot_1, CollectionSlotState::Initialized(owned_ty));

    engine.apply_certified_collection_slot_transform_range_with_aliases(
        &mut cells,
        &mut collection_slots,
        &raw_aliases,
        &source_storage,
        &source_initialized_count,
        &output_storage,
        &output_initialized_count,
        owned_ty,
        transform_range_certificate(
            owned_ty,
            CollectionSlotTransformRangeDiscardProof::LoadedValueDrop(
                CollectionSlotDropObligation::DropLoadedValue {
                    operation: CollectionSlotLifecycleOp::DropInitialized,
                    value_ty: owned_ty,
                },
            ),
        ),
        crate::span::Span::dummy(),
    );

    assert!(
        engine.diagnostics.is_empty(),
        "transform range replay must accept source move and output prefix initialization: {:#?}",
        engine.diagnostics
    );
    assert_eq!(
        collection_slots.state_with_aliases(&source_slot_0, &raw_aliases),
        CollectionSlotState::Uninitialized
    );
    assert_eq!(
        collection_slots.state_with_aliases(&source_slot_1, &raw_aliases),
        CollectionSlotState::Uninitialized
    );
    assert_eq!(
        collection_slots.state_with_aliases_and_ranges(&types, &output_slot_0, &raw_aliases),
        CollectionSlotState::Initialized(owned_ty)
    );
    assert_eq!(
        collection_slots.state_with_aliases_and_ranges(&types, &output_slot_1, &raw_aliases),
        CollectionSlotState::Uninitialized
    );
    assert!(
        collection_slots
            .release_storage_with_aliases(&source_storage, &raw_aliases)
            .is_ok(),
        "moved-out source slots must not block source storage release"
    );
    assert!(
        collection_slots
            .release_storage_with_aliases(&output_storage, &raw_aliases)
            .is_err(),
        "initialized output prefix must block storage release until cleanup traversal runs"
    );
    engine.apply_certified_collection_slot_drop_traversal_range_with_aliases(
        &mut cells,
        &mut collection_slots,
        &raw_aliases,
        &output_storage,
        &output_initialized_count,
        owned_ty,
        CollectionSlotInitializedRangeDropTraversalCertificate {
            element_stride: 4,
            drop_proof: CollectionSlotInitializedRangeDropTraversalProof::LoadedValueDrop(
                CollectionSlotDropObligation::DropLoadedValue {
                    operation: CollectionSlotLifecycleOp::DropInitialized,
                    value_ty: owned_ty,
                },
            ),
        },
        crate::span::Span::dummy(),
    );
    assert!(
        collection_slots
            .release_storage_with_aliases(&output_storage, &raw_aliases)
            .is_ok(),
        "drop traversal certificate must clear output initialized range before release"
    );
}

#[test]
fn collection_slot_summary_transform_replay_rejects_missing_discard_drop_proof() {
    let (types, owned_ty) = types_with_sized_droppable_owned_for_summary();
    let i32_ty = types.i32();
    let source_storage = Place::local("source_storage".to_string(), i32_ty);
    let output_storage = Place::local("output_storage".to_string(), i32_ty);
    let source_initialized_count = Place::i32_constant(2, i32_ty);
    let output_initialized_count = Place::i32_constant(1, i32_ty);
    let source_slot_0 = slot_at(source_storage.clone(), 0, i32_ty, owned_ty);
    let raw_alias_summaries = [];
    let i32_scalar_summaries = [];
    let raw_init_summaries = [];
    let collection_slot_summaries = [];
    let raw_alias_summary_index = RawCellAddressReturnSummaryIndex::new(&raw_alias_summaries);
    let i32_scalar_summary_index = I32ScalarReturnSummaryIndex::new(&i32_scalar_summaries);
    let raw_init_summary_index =
        RawCellInitializationFunctionSummaryIndex::new(&raw_init_summaries);
    let collection_slot_summary_index =
        CollectionSlotLifecycleFunctionSummaryIndex::new(&collection_slot_summaries);
    let mut engine = summary_test_engine(
        &types,
        &raw_alias_summary_index,
        &i32_scalar_summary_index,
        &raw_init_summary_index,
        &collection_slot_summary_index,
    );
    let mut cells = CellTable::default();
    let mut collection_slots = CollectionSlotStateTable::new();
    let raw_aliases = RawCellAddressAliases::default();
    collection_slots.set_slot_state(&source_slot_0, CollectionSlotState::Initialized(owned_ty));

    engine.apply_certified_collection_slot_transform_range_with_aliases(
        &mut cells,
        &mut collection_slots,
        &raw_aliases,
        &source_storage,
        &source_initialized_count,
        &output_storage,
        &output_initialized_count,
        owned_ty,
        transform_range_certificate(
            owned_ty,
            CollectionSlotTransformRangeDiscardProof::NoDiscard,
        ),
        crate::span::Span::dummy(),
    );

    assert!(
        !engine.diagnostics.is_empty(),
        "non-Copy transform replay must reject missing discard drop proof"
    );
    assert_eq!(
        collection_slots.state_with_aliases(&source_slot_0, &raw_aliases),
        CollectionSlotState::Initialized(owned_ty)
    );
}

fn slot_at(storage: Place, offset: usize, i32_ty: TypeId, owned_ty: TypeId) -> Place {
    storage
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::Known(offset)),
            i32_ty,
        )
        .with_projection(PlaceProjection::Deref, owned_ty)
}

fn summary_test_engine<'a>(
    types: &'a TypeCtx,
    raw_alias_summaries: &'a RawCellAddressReturnSummaryIndex<'a>,
    i32_scalar_summaries: &'a I32ScalarReturnSummaryIndex<'a>,
    raw_init_summaries: &'a RawCellInitializationFunctionSummaryIndex<'a>,
    collection_slot_summaries: &'a CollectionSlotLifecycleFunctionSummaryIndex<'a>,
) -> ResourceCheckEngine<'a> {
    ResourceCheckEngine {
        function: "caller",
        types,
        raw_alias_summaries,
        i32_scalar_summaries,
        raw_init_summaries,
        collection_slot_summaries,
        transform_range_certificates: None,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
    }
}

fn transform_range_certificate(
    owned_ty: TypeId,
    discard_drop_proof: CollectionSlotTransformRangeDiscardProof,
) -> CollectionSlotTransformRangeCertificate {
    CollectionSlotTransformRangeCertificate {
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
        discard_drop_proof,
    }
}

fn types_with_sized_droppable_owned_for_summary() -> (TypeCtx, TypeId) {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.bool());
    let i32_ty = types.i32();
    let owned_ty = types.register_named(
        "SizedOwned".to_string(),
        TypeKind::Struct {
            name: "SizedOwned".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["value".to_string()],
        },
    );
    types.register_drop_impl_target(owned_ty);
    (types, owned_ty)
}
