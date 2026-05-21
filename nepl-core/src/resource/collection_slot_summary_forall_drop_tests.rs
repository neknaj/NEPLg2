extern crate alloc;

use alloc::string::ToString;
use alloc::vec;
use alloc::vec::Vec;

use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::collection_slot_lifecycle::{
    CollectionSlotLifecycleOp, CollectionSlotLifecycleRefutation, CollectionSlotState,
};
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummaryIndex,
    CollectionSlotLifecycleSummaryDropTraversalCoverage, CollectionSlotLifecycleSummaryOp,
    CollectionSlotLifecycleSummaryPlace,
};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_scalar_flow::I32ScalarReturnSummaryIndex;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::model::{Place, PlaceProjection, ResourceOffset};
use super::report::{ResourceCheckDeferred, ResourceCheckDiagnostic};
use crate::span::Span;

#[test]
fn forall_drop_summary_replay_drops_every_caller_slot_inside_count() {
    let (types, owned_ty) = types_with_sized_droppable_owned();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let storage = Place::local("buffer".to_string(), i32_ty);
    let initialized_count = Place::i32_constant(2, i32_ty);
    let slot_0 = collection_slot(&storage, 0, i32_ty, owned_ty);
    let slot_1 = collection_slot(&storage, 4, i32_ty, owned_ty);
    let mut cells = CellTable::default();
    let mut slots = CollectionSlotStateTable::new();
    slots.set_slot_state(&slot_0, CollectionSlotState::Initialized(owned_ty));
    slots.set_slot_state(&slot_1, CollectionSlotState::Initialized(owned_ty));
    let raw_aliases = RawCellAddressAliases::default();

    let diagnostics = apply_forall_drop_summary(
        "main",
        &types,
        &mut cells,
        &mut slots,
        &raw_aliases,
        &storage,
        &initialized_count,
        owned_ty,
        span,
    );

    assert!(diagnostics.is_empty(), "{diagnostics:#?}");
    assert_eq!(slots.state(&slot_0), CollectionSlotState::Dropped(owned_ty));
    assert_eq!(slots.state(&slot_1), CollectionSlotState::Dropped(owned_ty));
}

#[test]
fn forall_drop_summary_replay_rejects_caller_slot_outside_count() {
    let (types, owned_ty) = types_with_sized_droppable_owned();
    let i32_ty = types.i32();
    let span = Span::dummy();
    let storage = Place::local("buffer".to_string(), i32_ty);
    let initialized_count = Place::i32_constant(2, i32_ty);
    let slot_0 = collection_slot(&storage, 0, i32_ty, owned_ty);
    let slot_2 = collection_slot(&storage, 8, i32_ty, owned_ty);
    let mut cells = CellTable::default();
    let mut slots = CollectionSlotStateTable::new();
    slots.set_slot_state(&slot_0, CollectionSlotState::Initialized(owned_ty));
    slots.set_slot_state(&slot_2, CollectionSlotState::Initialized(owned_ty));
    let raw_aliases = RawCellAddressAliases::default();

    let diagnostics = apply_forall_drop_summary(
        "main",
        &types,
        &mut cells,
        &mut slots,
        &raw_aliases,
        &storage,
        &initialized_count,
        owned_ty,
        span,
    );

    assert_eq!(
        diagnostics,
        vec![ResourceCheckDiagnostic::CollectionSlotRefuted {
            function: "main".to_string(),
            target: slot_2.clone(),
            reason: CollectionSlotLifecycleRefutation::RangeProofRequired {
                operation: CollectionSlotLifecycleOp::DropTraversal,
                slot_ty: Some(owned_ty),
            },
            span,
        }]
    );
    assert_eq!(
        slots.state(&slot_0),
        CollectionSlotState::Initialized(owned_ty)
    );
    assert_eq!(
        slots.state(&slot_2),
        CollectionSlotState::Initialized(owned_ty)
    );
}

#[test]
fn drop_traversal_summary_coverage_is_exhaustively_typed() {
    let i32_ty = TypeId(1);
    let summary_place = CollectionSlotLifecycleSummaryPlace {
        parameter_index: 0,
        suffix: Vec::new(),
        ty: i32_ty,
    };
    let op = CollectionSlotLifecycleSummaryOp::DropTraversal {
        storage: summary_place.clone(),
        initialized_count: summary_place,
        expected_ty: i32_ty,
        coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange,
    };

    match op {
        CollectionSlotLifecycleSummaryOp::DropTraversal {
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange,
            ..
        } => {}
        CollectionSlotLifecycleSummaryOp::DropTraversal {
            coverage: CollectionSlotLifecycleSummaryDropTraversalCoverage::CertifiedSlots(_),
            ..
        } => panic!("test constructs a forall coverage summary"),
        CollectionSlotLifecycleSummaryOp::Event { .. }
        | CollectionSlotLifecycleSummaryOp::Relocate { .. }
        | CollectionSlotLifecycleSummaryOp::Merge { .. }
        | CollectionSlotLifecycleSummaryOp::Loop { .. } => {
            panic!("test constructs a drop traversal summary")
        }
    }
}

fn types_with_sized_droppable_owned() -> (TypeCtx, TypeId) {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    let i32_ty = types.i32();
    let owned_ty = types.register_named(
        "SizedOwned".to_string(),
        TypeKind::Struct {
            name: "SizedOwned".to_string(),
            type_params: Vec::new(),
            fields: vec![i32_ty],
            field_names: vec!["value".to_string()],
        },
    );
    types.register_drop_impl_target(owned_ty);
    (types, owned_ty)
}

fn collection_slot(storage: &Place, offset: usize, address_ty: TypeId, owned_ty: TypeId) -> Place {
    storage
        .clone()
        .with_projection(
            PlaceProjection::StorageOffset(ResourceOffset::Known(offset)),
            address_ty,
        )
        .with_projection(PlaceProjection::Deref, owned_ty)
}

fn apply_forall_drop_summary(
    function: &str,
    types: &TypeCtx,
    cells: &mut CellTable,
    slots: &mut CollectionSlotStateTable,
    raw_aliases: &RawCellAddressAliases,
    storage: &Place,
    initialized_count: &Place,
    owned_ty: TypeId,
    span: Span,
) -> Vec<ResourceCheckDiagnostic> {
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
    let mut engine = ResourceCheckEngine {
        function,
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
    engine.apply_certified_collection_slot_drop_traversal_forall_with_aliases(
        cells,
        slots,
        raw_aliases,
        storage,
        initialized_count,
        owned_ty,
        span,
    );
    engine.diagnostics
}
