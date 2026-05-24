extern crate alloc;

use alloc::{boxed::Box, string::ToString, vec, vec::Vec};

use crate::ast::Effect;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::collection_slot_lifecycle::CollectionSlotLifecycleEvent;
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

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum PostLoopCertificateInterference {
    UnrelatedLiteral,
    AnchorRead,
    AssignStorage,
    AssignInitializedCount,
    TouchSlot,
}

pub(super) fn collect_loop_induction_summary_ops(
    post_loop: PostLoopCertificateInterference,
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
    let post_loop_storage = Place::temporary(ResourceId(936), i32_ty);
    let post_loop_count = Place::temporary(ResourceId(937), i32_ty);
    let unrelated = Place::temporary(ResourceId(938), i32_ty);
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
    let body_ops = vec![
        ResourceOp::RawMemory {
            operation: RawMemoryOp::Load,
            output: loaded.clone(),
            args: vec![slot_address.clone()],
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
    let mut ops = vec![
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
                left: index,
                op: ResourceI32RelationOp::Lt,
                right: initialized_count.clone(),
            }),
            body_ops,
            span,
        },
    ];
    append_post_loop_interference(
        &mut ops,
        post_loop,
        span,
        i32_ty,
        owned_ty,
        &storage,
        &initialized_count,
        &slot_address,
        post_loop_storage,
        post_loop_count,
        unrelated,
    );
    ops.push(ResourceOp::CollectionSlotDropTraversal {
        storage,
        initialized_count,
        expected_ty: owned_ty,
        span,
    });

    collect_summary_ops_from_ops(
        &mut out,
        &mut engine,
        &mut state,
        &function.params,
        &collection_slot_summary_index,
        &ops,
    );
    out
}

fn append_post_loop_interference(
    ops: &mut Vec<ResourceOp>,
    post_loop: PostLoopCertificateInterference,
    span: Span,
    i32_ty: TypeId,
    owned_ty: TypeId,
    storage: &Place,
    initialized_count: &Place,
    slot_address: &Place,
    post_loop_storage: Place,
    post_loop_count: Place,
    unrelated: Place,
) {
    match post_loop {
        PostLoopCertificateInterference::UnrelatedLiteral => {
            ops.push(literal_i32_op(123, unrelated, i32_ty, span));
        }
        PostLoopCertificateInterference::AnchorRead => {
            ops.push(ResourceOp::Read {
                source: storage.clone(),
                output: post_loop_storage.clone(),
                span,
            });
            ops.push(ResourceOp::Expr {
                kind: ResourceExprKind::LocalRead,
                output: post_loop_storage,
                ty: i32_ty,
                span,
            });
        }
        PostLoopCertificateInterference::AssignStorage => {
            ops.push(literal_i32_op(7, post_loop_storage.clone(), i32_ty, span));
            ops.push(ResourceOp::Assign {
                target: storage.clone(),
                value: post_loop_storage,
                span,
            });
        }
        PostLoopCertificateInterference::AssignInitializedCount => {
            ops.push(literal_i32_op(1, post_loop_count.clone(), i32_ty, span));
            ops.push(ResourceOp::Assign {
                target: initialized_count.clone(),
                value: post_loop_count,
                span,
            });
        }
        PostLoopCertificateInterference::TouchSlot => {
            ops.push(ResourceOp::CollectionSlotLifecycle {
                target: slot_address.clone(),
                event: CollectionSlotLifecycleEvent::InitializeEmpty { value_ty: owned_ty },
                span,
            });
        }
    }
}

pub(super) fn has_forall_range_summary(out: &[CollectionSlotLifecycleSummaryOp]) -> bool {
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
        transform_range_certificates: None,
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
        name: "drop_guarded_lifetime".to_string(),
        origin_name: "drop_guarded_lifetime".to_string(),
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
