extern crate alloc;

use alloc::{boxed::Box, string::ToString, vec, vec::Vec};

use crate::ast::Effect;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::collection_slot_summary_build_ops::collect_summary_ops_from_ops;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummaryIndex,
    CollectionSlotLifecycleSummaryDropTraversalCoverage, CollectionSlotLifecycleSummaryOp,
};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_scalar_flow::I32ScalarReturnSummaryIndex;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::model::{
    EffectOp, Place, PlaceProjection, RawMemoryOp, ResourceBlock, ResourceBlockId,
    ResourceCallTarget, ResourceConditionFact, ResourceExprKind, ResourceFunction,
    ResourceI32RelationOp, ResourceId, ResourceLocal, ResourceOffset, ResourceOp,
    ResourceTerminator,
};
use super::report::ResourceCheckDeferred;

#[test]
fn collection_slot_summary_loop_induction_certifies_forall_drop_traversal() {
    let out = collect_loop_induction_summary_ops(false);

    assert!(
        has_forall_range_summary(&out),
        "a zero-based one-step loop with a loaded-value drop must produce a typed full-range certificate: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_induction_rejects_tail_storage_mutation() {
    let out = collect_loop_induction_summary_ops(true);

    assert!(
        !has_forall_range_summary(&out),
        "a loop that mutates storage after the induction step must not produce a full-range certificate: {out:#?}"
    );
}

fn collect_loop_induction_summary_ops(
    mutate_storage_after_step: bool,
) -> Vec<CollectionSlotLifecycleSummaryOp> {
    let (types, owned_ty) = types_with_sized_droppable_owned_for_summary();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let storage = Place::local("storage".to_string(), i32_ty);
    let initialized_count = Place::local("initialized_count".to_string(), i32_ty);
    let index = Place::local("i".to_string(), i32_ty);
    let zero = Place::temporary(ResourceId(930), i32_ty);
    let condition = Place::temporary(ResourceId(931), bool_ty);
    let slot_address = storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
            place: Box::new(index.clone()),
            scale: 4,
        }),
        i32_ty,
    );
    let loaded = Place::temporary(ResourceId(932), owned_ty);
    let one = Place::temporary(ResourceId(933), i32_ty);
    let next = Place::temporary(ResourceId(934), i32_ty);
    let replacement_storage = Place::temporary(ResourceId(935), i32_ty);
    let function = summary_test_function(
        unit_ty,
        i32_ty,
        span,
        storage.clone(),
        initialized_count.clone(),
    );
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
        function.name.as_str(),
        &types,
        &raw_alias_summary_index,
        &i32_scalar_summary_index,
        &raw_init_summary_index,
        &collection_slot_summary_index,
    );
    let mut state = CollectionSlotSummaryBuildState::new(&types, &function);
    let mut out = Vec::new();
    let mut body_ops = vec![
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
        literal_i32_op(1, one.clone(), i32_ty, span),
        ResourceOp::Call {
            output: next.clone(),
            target: ResourceCallTarget::User {
                name: "add".to_string(),
                type_args: Vec::new(),
            },
            args: vec![index.clone(), one],
            effect: EffectOp::Pure,
            span,
        },
        ResourceOp::Assign {
            target: index.clone(),
            value: next,
            span,
        },
    ];
    if mutate_storage_after_step {
        body_ops.push(literal_i32_op(7, replacement_storage.clone(), i32_ty, span));
        body_ops.push(ResourceOp::Assign {
            target: storage.clone(),
            value: replacement_storage,
            span,
        });
    }

    collect_summary_ops_from_ops(
        &mut out,
        &mut engine,
        &mut state,
        &function.params,
        &collection_slot_summary_index,
        &[
            literal_i32_op(0, zero.clone(), i32_ty, span),
            ResourceOp::DeclareLocal {
                place: index.clone(),
                source_name: "i".to_string(),
                mutable: true,
                initializer: Some(zero),
                span,
            },
            ResourceOp::Loop {
                condition_ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::Literal,
                    output: condition.clone(),
                    ty: bool_ty,
                    span,
                }],
                condition,
                condition_fact: Some(ResourceConditionFact::I32Relation {
                    left: index.clone(),
                    op: ResourceI32RelationOp::Lt,
                    right: initialized_count.clone(),
                }),
                body_ops,
                span,
            },
            ResourceOp::CollectionSlotDropTraversal {
                storage,
                initialized_count,
                expected_ty: owned_ty,
                span,
            },
        ],
    );
    out
}

fn has_forall_range_summary(out: &[CollectionSlotLifecycleSummaryOp]) -> bool {
    out.iter().any(|op| {
        matches!(
            op,
            CollectionSlotLifecycleSummaryOp::DropTraversal {
                coverage:
                    CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                        certificate
                    ),
                ..
            } if certificate.element_stride == 4
        )
    })
}

fn literal_i32_op(value: i32, output: Place, ty: TypeId, span: Span) -> ResourceOp {
    ResourceOp::Expr {
        kind: ResourceExprKind::LiteralI32(value),
        output,
        ty,
        span,
    }
}

fn summary_test_engine<'a>(
    function: &'a str,
    types: &'a TypeCtx,
    raw_alias_summaries: &'a RawCellAddressReturnSummaryIndex<'a>,
    i32_scalar_summaries: &'a I32ScalarReturnSummaryIndex<'a>,
    raw_init_summaries: &'a RawCellInitializationFunctionSummaryIndex<'a>,
    collection_slot_summaries: &'a CollectionSlotLifecycleFunctionSummaryIndex<'a>,
) -> ResourceCheckEngine<'a> {
    ResourceCheckEngine {
        function,
        types,
        raw_alias_summaries,
        i32_scalar_summaries,
        raw_init_summaries,
        collection_slot_summaries,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
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
