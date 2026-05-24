extern crate alloc;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

use crate::ast::Effect;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::cell_state::CellTable;
use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
use super::collection_slot_state_table::CollectionSlotStateTable;
use super::collection_slot_summary_build_nested::apply_summary_condition_fact;
use super::collection_slot_summary_build_ops::collect_summary_ops_from_ops;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummaryIndex, CollectionSlotLifecycleSummaryOp,
};
use super::function_alias::FunctionAliasTable;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_scalar_flow::I32ScalarReturnSummaryIndex;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::initialized_variant::PendingVariantRawCellInitializations;
use super::model::{
    Place, PlaceProjection, RawMemoryOp, ResourceBlock, ResourceBlockId, ResourceConditionFact,
    ResourceExprKind, ResourceFunction, ResourceI32RelationOp, ResourceId, ResourceLocal,
    ResourceOffset, ResourceOp, ResourceTerminator,
};
use super::raw_realloc::PendingRawReallocs;
use super::report::ResourceCheckDeferred;

fn local(name: &str) -> Place {
    Place::local(String::from(name), TypeId(1))
}

fn empty_summary_state() -> CollectionSlotSummaryBuildState {
    CollectionSlotSummaryBuildState {
        cells: CellTable::default(),
        collection_slots: CollectionSlotStateTable::new(),
        raw_aliases: RawCellAddressAliases::default(),
        function_aliases: FunctionAliasTable::default(),
        pending_reallocs: PendingRawReallocs::default(),
        variant_initializations: PendingVariantRawCellInitializations::default(),
        drop_traversal_range_certificates: Vec::new(),
        transform_range_certificates: Vec::new(),
    }
}

#[test]
fn collection_slot_summary_branch_condition_fact_records_then_relation() {
    let index = local("i");
    let count = local("initialized_count");
    let fact = ResourceConditionFact::I32Relation {
        left: index.clone(),
        op: ResourceI32RelationOp::Lt,
        right: count.clone(),
    };
    let mut state = empty_summary_state();

    apply_summary_condition_fact(&mut state, Some(&fact), true);

    assert_eq!(
        state
            .raw_aliases
            .i32_relation_truth(&index, ResourceI32RelationOp::Lt, &count),
        Some(true)
    );
    assert_eq!(
        state
            .raw_aliases
            .i32_relation_truth(&index, ResourceI32RelationOp::Ge, &count),
        Some(false)
    );
}

#[test]
fn collection_slot_summary_branch_condition_fact_records_else_negation() {
    let index = local("i");
    let count = local("initialized_count");
    let fact = ResourceConditionFact::I32Relation {
        left: index.clone(),
        op: ResourceI32RelationOp::Lt,
        right: count.clone(),
    };
    let mut state = empty_summary_state();

    apply_summary_condition_fact(&mut state, Some(&fact), false);

    assert_eq!(
        state
            .raw_aliases
            .i32_relation_truth(&index, ResourceI32RelationOp::Ge, &count),
        Some(true)
    );
    assert_eq!(
        state
            .raw_aliases
            .i32_relation_truth(&index, ResourceI32RelationOp::Lt, &count),
        Some(false)
    );
}

#[test]
fn collection_slot_summary_branch_condition_fact_does_not_certify_forall_drop_traversal() {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.bool());
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let unit_ty = types.unit();
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
    let span = Span::dummy();
    let storage = Place::local("storage".to_string(), i32_ty);
    let initialized_count = Place::local("initialized_count".to_string(), i32_ty);
    let index = Place::local("i".to_string(), i32_ty);
    let condition = Place::temporary(ResourceId(900), bool_ty);
    let branch_output = Place::temporary(ResourceId(901), unit_ty);
    let then_value = Place::temporary(ResourceId(902), unit_ty);
    let else_value = Place::temporary(ResourceId(903), unit_ty);
    let slot_address = storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
            place: Box::new(index.clone()),
            scale: 4,
        }),
        i32_ty,
    );
    let slot = slot_address
        .clone()
        .with_projection(PlaceProjection::Deref, owned_ty);
    let initial = Place::temporary(ResourceId(904), owned_ty);
    let loaded = Place::temporary(ResourceId(905), owned_ty);
    let then_ops = vec![
        ResourceOp::Expr {
            kind: ResourceExprKind::Literal,
            output: initial.clone(),
            ty: owned_ty,
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Store,
            output: Place::temporary(ResourceId(906), unit_ty),
            args: vec![slot_address.clone(), initial],
            span,
        },
        ResourceOp::CollectionSlotLifecycle {
            target: slot,
            event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned_ty },
            span,
        },
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            output: loaded.clone(),
            args: vec![slot_address],
            span,
        },
        ResourceOp::Drop {
            place: loaded,
            span,
        },
        ResourceOp::CollectionSlotDropTraversal {
            storage: storage.clone(),
            initialized_count: initialized_count.clone(),
            expected_ty: owned_ty,
            span,
        },
        ResourceOp::Expr {
            kind: ResourceExprKind::Literal,
            output: then_value.clone(),
            ty: unit_ty,
            span,
        },
    ];
    let function = summary_test_function(unit_ty, i32_ty, span, storage, initialized_count.clone());
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
        function: function.name.as_str(),
        types: &types,
        raw_alias_summaries: &raw_alias_summary_index,
        i32_scalar_summaries: &i32_scalar_summary_index,
        raw_init_summaries: &raw_init_summary_index,
        collection_slot_summaries: &collection_slot_summary_index,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
    };
    let mut state = CollectionSlotSummaryBuildState::new(&types, &function);
    let mut out = Vec::new();

    collect_summary_ops_from_ops(
        &mut out,
        &mut engine,
        &mut state,
        &function.params,
        &collection_slot_summary_index,
        &[
            ResourceOp::Expr {
                kind: ResourceExprKind::Literal,
                output: condition.clone(),
                ty: bool_ty,
                span,
            },
            ResourceOp::Branch {
                output: branch_output,
                condition,
                condition_fact: Some(ResourceConditionFact::All(vec![
                    ResourceConditionFact::NonNegative {
                        place: index.clone(),
                    },
                    ResourceConditionFact::I32Relation {
                        left: index,
                        op: ResourceI32RelationOp::Lt,
                        right: initialized_count,
                    },
                ])),
                then_ops,
                then_value,
                else_ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::Literal,
                    output: else_value.clone(),
                    ty: unit_ty,
                    span,
                }],
                else_value,
                span,
            },
        ],
    );

    assert!(
        !out.iter().any(|op| summary_contains_drop_traversal(op)),
        "a branch-local symbolic slot range proof is not a full initialized-range traversal certificate: {out:#?}"
    );
}

fn summary_contains_drop_traversal(op: &CollectionSlotLifecycleSummaryOp) -> bool {
    match op {
        CollectionSlotLifecycleSummaryOp::DropTraversal { .. } => true,
        CollectionSlotLifecycleSummaryOp::Merge { paths } => paths
            .iter()
            .any(|path| path.iter().any(summary_contains_drop_traversal)),
        CollectionSlotLifecycleSummaryOp::Loop {
            condition_ops,
            body_ops,
        } => condition_ops
            .iter()
            .chain(body_ops)
            .any(summary_contains_drop_traversal),
        CollectionSlotLifecycleSummaryOp::Event { .. }
        | CollectionSlotLifecycleSummaryOp::Relocate { .. }
        | CollectionSlotLifecycleSummaryOp::TransformRange { .. } => false,
    }
}

fn summary_test_function(
    unit_ty: TypeId,
    i32_ty: TypeId,
    span: Span,
    storage: Place,
    initialized_count: Place,
) -> ResourceFunction {
    ResourceFunction {
        name: "drop_guarded".to_string(),
        origin_name: "drop_guarded".to_string(),
        type_params: Vec::new(),
        params: vec![
            ResourceLocal {
                name: "storage".to_string(),
                ty: i32_ty,
                mutable: false,
                place: storage,
            },
            ResourceLocal {
                name: "initialized_count".to_string(),
                ty: i32_ty,
                mutable: false,
                place: initialized_count,
            },
        ],
        result: unit_ty,
        effect: Effect::Pure,
        entry_block: ResourceBlockId(0),
        blocks: vec![ResourceBlock {
            id: ResourceBlockId(0),
            ops: Vec::new(),
            terminator: ResourceTerminator::Return { value: None, span },
            span,
        }],
        span,
    }
}
