extern crate alloc;

use alloc::{boxed::Box, string::ToString, vec, vec::Vec};

use crate::ast::Effect;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::collection_slot_summary_build_ops::collect_summary_ops_from_ops;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::{
    CollectionSlotInitializedRangeDropTraversalProof, CollectionSlotLifecycleFunctionSummaryIndex,
    CollectionSlotLifecycleSummaryDropTraversalCoverage, CollectionSlotLifecycleSummaryOp,
    CollectionSlotTransformRangeDiscardProof, CollectionSlotTransformRangeOutputProof,
    CollectionSlotTransformRangeSourceProof,
};
use super::initialized::ResourceCheckEngine;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_scalar_flow::I32ScalarReturnSummaryIndex;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::model::{
    EffectOp, Place, PlaceProjection, RawMemoryOp, ResourceBlock, ResourceBlockId,
    ResourceCallTarget, ResourceConditionFact, ResourceExprKind, ResourceFunction,
    ResourceI32RelationOp, ResourceId, ResourceLocal, ResourceMatchArm, ResourceMatchPattern,
    ResourceOffset, ResourceOp, ResourceTerminator,
};
use super::report::ResourceCheckDeferred;

#[test]
fn collection_slot_summary_loop_induction_certifies_transform_range() {
    let out = collect_transform_range_summary(TransformRangeLoopShape::ValidStoreAndDiscard);

    assert!(
        out.iter().any(|op| matches!(
            op,
            CollectionSlotLifecycleSummaryOp::TransformRange { certificate, .. }
                if certificate.element_stride == 4
                    && matches!(
                        certificate.source_move_proof,
                        CollectionSlotTransformRangeSourceProof::LoadedValueMove(_)
                    )
                    && matches!(
                        certificate.output_store_proof,
                        CollectionSlotTransformRangeOutputProof::StoredValue(_)
                    )
                    && matches!(
                        certificate.discard_drop_proof,
                        CollectionSlotTransformRangeDiscardProof::LoadedValueDrop(_)
                    )
        )),
        "a source drain / output prefix loop must produce a transform range summary: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_induction_rejects_missing_discard_drop() {
    let out = collect_transform_range_summary(TransformRangeLoopShape::MissingDiscardDrop);

    assert!(
        !out.iter()
            .any(|op| matches!(op, CollectionSlotLifecycleSummaryOp::TransformRange { .. })),
        "a transform range summary must require actual drop coverage for discarded non-Copy values: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_induction_rejects_missing_output_store() {
    let out = collect_transform_range_summary(TransformRangeLoopShape::MissingOutputStore);

    assert!(
        !out.iter()
            .any(|op| matches!(op, CollectionSlotLifecycleSummaryOp::TransformRange { .. })),
        "a transform range summary must require output slot initialization coverage: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_induction_rejects_unknown_output_start() {
    let out = collect_transform_range_summary(TransformRangeLoopShape::UnknownOutputStart);

    assert!(
        !out.iter()
            .any(|op| matches!(op, CollectionSlotLifecycleSummaryOp::TransformRange { .. })),
        "a transform range summary must prove output prefix construction starts at zero: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_transform_marker_leaves_output_range_live_in_build_state() {
    let (_out, mut state, output_storage, _owned_ty) =
        collect_transform_range_summary_and_state(TransformRangeLoopShape::ValidStoreAndDiscard);

    assert!(
        state
            .collection_slots
            .release_storage_with_aliases(&output_storage, &state.raw_aliases)
            .is_err(),
        "summary build state must apply the certified transform marker so the output prefix stays live until cleanup"
    );
}

#[test]
fn collection_slot_summary_transform_rollback_cleanup_clears_output_range_state() {
    let (_out, mut state, output_storage, _owned_ty) =
        collect_transform_range_summary_and_state(TransformRangeLoopShape::ValidRollbackCleanup);

    assert!(
        state
            .collection_slots
            .release_storage_with_aliases(&output_storage, &state.raw_aliases)
            .is_ok(),
        "rollback cleanup traversal must clear the output prefix left by the transform marker"
    );
}

#[test]
fn collection_slot_summary_branch_transform_marker_leaves_output_range_live_in_build_state() {
    let (_out, mut state, output_storage, _owned_ty) =
        collect_transform_range_summary_and_state(TransformRangeLoopShape::BranchWrappedMarker);

    assert!(
        state
            .collection_slots
            .release_storage_with_aliases(&output_storage, &state.raw_aliases)
            .is_err(),
        "a transform marker inside a branch must update the outer summary state before later cleanup/release"
    );
}

#[test]
fn collection_slot_summary_loop_transform_marker_leaves_output_range_live_in_build_state() {
    let (_out, mut state, output_storage, _owned_ty) =
        collect_transform_range_summary_and_state(TransformRangeLoopShape::LoopWrappedMarker);

    assert!(
        state
            .collection_slots
            .release_storage_with_aliases(&output_storage, &state.raw_aliases)
            .is_err(),
        "a transform marker inside a loop must update the outer summary state before later cleanup/release"
    );
}

#[test]
fn collection_slot_summary_match_transform_marker_leaves_output_range_live_in_build_state() {
    let (_out, mut state, output_storage, _owned_ty) =
        collect_transform_range_summary_and_state(TransformRangeLoopShape::MatchWrappedMarker);

    assert!(
        state
            .collection_slots
            .release_storage_with_aliases(&output_storage, &state.raw_aliases)
            .is_err(),
        "a transform marker inside a match must update the outer summary state before later cleanup/release"
    );
}

#[test]
fn collection_slot_summary_loop_transform_marker_consumes_pending_certificate() {
    let (out, _state, _output_storage, _owned_ty) = collect_transform_range_summary_and_state(
        TransformRangeLoopShape::LoopWrappedMarkerThenDuplicateMarker,
    );

    assert_eq!(
        1,
        count_transform_range_summaries(&out),
        "a transform marker inside a loop must consume the pending certificate before later markers can reuse it: {out:#?}"
    );
    assert!(
        has_loop_body_transform_range_summary(&out),
        "the consumed certificate must be used by the loop body marker, not by the later duplicate marker: {out:#?}"
    );
    assert_eq!(
        0,
        count_top_level_transform_range_summaries(&out),
        "a later top-level duplicate marker must not reuse the loop body marker certificate: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_induction_certifies_transform_rollback_cleanup() {
    let (out, _state, _output_storage, owned_ty) =
        collect_transform_range_summary_and_state(TransformRangeLoopShape::ValidRollbackCleanup);

    assert!(
        out.iter()
            .any(|op| matches!(op, CollectionSlotLifecycleSummaryOp::TransformRange { .. })),
        "rollback path must keep the transform range summary before cleanup: {out:#?}"
    );
    assert!(
        has_output_cleanup_forall_range_summary(&out, owned_ty),
        "rollback cleanup must be represented as a full-range drop traversal summary: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_induction_rejects_incomplete_rollback_cleanup() {
    let out = collect_transform_range_summary(TransformRangeLoopShape::RollbackCleanupNonZeroStart);

    assert!(
        out.iter()
            .any(|op| matches!(op, CollectionSlotLifecycleSummaryOp::TransformRange { .. })),
        "incomplete rollback cleanup must not erase the transform summary itself: {out:#?}"
    );
    assert!(
        !out.iter().any(|op| matches!(
            op,
            CollectionSlotLifecycleSummaryOp::DropTraversal {
                coverage:
                    CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(_),
                ..
            }
        )),
        "rollback cleanup starting past zero must not certify the whole output prefix: {out:#?}"
    );
}

#[derive(Clone, Copy)]
enum TransformRangeLoopShape {
    ValidStoreAndDiscard,
    MissingDiscardDrop,
    MissingOutputStore,
    UnknownOutputStart,
    ValidRollbackCleanup,
    RollbackCleanupNonZeroStart,
    BranchWrappedMarker,
    LoopWrappedMarker,
    MatchWrappedMarker,
    LoopWrappedMarkerThenDuplicateMarker,
}

fn collect_transform_range_summary(
    shape: TransformRangeLoopShape,
) -> Vec<CollectionSlotLifecycleSummaryOp> {
    collect_transform_range_summary_and_state(shape).0
}

fn collect_transform_range_summary_and_state(
    shape: TransformRangeLoopShape,
) -> (
    Vec<CollectionSlotLifecycleSummaryOp>,
    CollectionSlotSummaryBuildState,
    Place,
    TypeId,
) {
    let (types, owned_ty) = types_with_sized_droppable_owned_for_summary();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let source_storage = Place::local("source_storage".to_string(), i32_ty);
    let source_count = Place::local("source_count".to_string(), i32_ty);
    let output_storage = Place::local("output_storage".to_string(), i32_ty);
    let output_storage_for_assert = output_storage.clone();
    let output_count = Place::local("output_count".to_string(), i32_ty);
    let read_index = Place::local("read_i".to_string(), i32_ty);
    let zero = Place::temporary(ResourceId(970), i32_ty);
    let output_zero = Place::temporary(ResourceId(982), i32_ty);
    let condition = Place::temporary(ResourceId(971), bool_ty);
    let loaded = Place::temporary(ResourceId(972), owned_ty);
    let source_slot_address = source_storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
            place: Box::new(read_index.clone()),
            scale: 4,
        }),
        i32_ty,
    );
    let output_slot_address = output_storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
            place: Box::new(output_count.clone()),
            scale: 4,
        }),
        i32_ty,
    );
    let write_one = Place::temporary(ResourceId(973), i32_ty);
    let next_write = Place::temporary(ResourceId(974), i32_ty);
    let read_one = Place::temporary(ResourceId(975), i32_ty);
    let next_read = Place::temporary(ResourceId(976), i32_ty);
    let store_unit = Place::temporary(ResourceId(977), unit_ty);
    let cleanup_index = Place::local("cleanup_i".to_string(), i32_ty);
    let cleanup_zero = Place::temporary(ResourceId(983), i32_ty);
    let cleanup_condition = Place::temporary(ResourceId(984), bool_ty);
    let cleanup_slot_address = output_storage.clone().with_projection(
        PlaceProjection::StorageOffset(ResourceOffset::ScaledSymbolic {
            place: Box::new(cleanup_index.clone()),
            scale: 4,
        }),
        i32_ty,
    );
    let cleanup_loaded = Place::temporary(ResourceId(985), owned_ty);
    let cleanup_one = Place::temporary(ResourceId(986), i32_ty);
    let cleanup_next = Place::temporary(ResourceId(987), i32_ty);
    let branch_marker_condition = Place::temporary(ResourceId(988), bool_ty);
    let branch_marker_output = Place::temporary(ResourceId(989), unit_ty);
    let branch_marker_then_value = Place::temporary(ResourceId(990), unit_ty);
    let branch_marker_else_value = Place::temporary(ResourceId(991), unit_ty);
    let loop_marker_condition = Place::temporary(ResourceId(992), bool_ty);
    let match_marker_scrutinee = Place::temporary(ResourceId(993), bool_ty);
    let match_marker_output = Place::temporary(ResourceId(994), unit_ty);
    let match_marker_then_value = Place::temporary(ResourceId(995), unit_ty);
    let match_marker_else_value = Place::temporary(ResourceId(996), unit_ty);
    let function = summary_test_function(
        unit_ty,
        i32_ty,
        span,
        source_storage.clone(),
        source_count.clone(),
        output_storage.clone(),
        output_count.clone(),
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
    let mut then_ops = Vec::new();
    if !matches!(shape, TransformRangeLoopShape::MissingOutputStore) {
        then_ops.push(ResourceOp::RawMemory {
            operation: RawMemoryOp::Store,
            output: store_unit,
            args: vec![output_slot_address, loaded.clone()],
            span,
        });
    }
    then_ops.extend([
        literal_i32_op(1, write_one.clone(), i32_ty, span),
        add_i32_op(output_count.clone(), write_one, next_write.clone(), span),
        ResourceOp::Assign {
            target: output_count.clone(),
            value: next_write,
            span,
        },
    ]);
    let else_ops = match shape {
        TransformRangeLoopShape::ValidStoreAndDiscard
        | TransformRangeLoopShape::ValidRollbackCleanup
        | TransformRangeLoopShape::RollbackCleanupNonZeroStart
        | TransformRangeLoopShape::BranchWrappedMarker
        | TransformRangeLoopShape::LoopWrappedMarker
        | TransformRangeLoopShape::MatchWrappedMarker
        | TransformRangeLoopShape::LoopWrappedMarkerThenDuplicateMarker
        | TransformRangeLoopShape::MissingOutputStore
        | TransformRangeLoopShape::UnknownOutputStart => vec![ResourceOp::Drop {
            place: loaded.clone(),
            span,
        }],
        TransformRangeLoopShape::MissingDiscardDrop => Vec::new(),
    };

    let mut ops = vec![
        literal_i32_op(0, zero.clone(), i32_ty, span),
        ResourceOp::DeclareLocal {
            place: read_index.clone(),
            source_name: "read_i".to_string(),
            mutable: true,
            initializer: Some(zero),
            span,
        },
    ];
    if matches!(shape, TransformRangeLoopShape::BranchWrappedMarker) {
        ops.push(ResourceOp::Expr {
            kind: ResourceExprKind::Literal,
            output: branch_marker_condition.clone(),
            ty: bool_ty,
            span,
        });
    }
    if matches!(shape, TransformRangeLoopShape::MatchWrappedMarker) {
        ops.push(ResourceOp::Expr {
            kind: ResourceExprKind::Literal,
            output: match_marker_scrutinee.clone(),
            ty: bool_ty,
            span,
        });
    }
    if !matches!(shape, TransformRangeLoopShape::UnknownOutputStart) {
        ops.extend([
            literal_i32_op(0, output_zero.clone(), i32_ty, span),
            ResourceOp::Assign {
                target: output_count.clone(),
                value: output_zero,
                span,
            },
        ]);
    }
    ops.extend([ResourceOp::Loop {
        condition_ops: vec![ResourceOp::Expr {
            kind: ResourceExprKind::Literal,
            output: condition.clone(),
            ty: bool_ty,
            span,
        }],
        condition,
        condition_fact: Some(ResourceConditionFact::I32Relation {
            left: read_index.clone(),
            op: ResourceI32RelationOp::Lt,
            right: source_count.clone(),
        }),
        body_ops: vec![
            ResourceOp::RawMemory {
                operation: RawMemoryOp::Load,
                output: loaded.clone(),
                args: vec![source_slot_address],
                span,
            },
            ResourceOp::Branch {
                output: Place::temporary(ResourceId(978), unit_ty),
                condition: Place::temporary(ResourceId(979), bool_ty),
                condition_fact: None,
                then_ops,
                then_value: Place::temporary(ResourceId(980), unit_ty),
                else_ops,
                else_value: Place::temporary(ResourceId(981), unit_ty),
                span,
            },
            literal_i32_op(1, read_one.clone(), i32_ty, span),
            add_i32_op(read_index.clone(), read_one, next_read.clone(), span),
            ResourceOp::Assign {
                target: read_index.clone(),
                value: next_read,
                span,
            },
        ],
        span,
    }]);
    let transform_marker = ResourceOp::CollectionSlotTransformRange {
        source_storage: source_storage.clone(),
        source_initialized_count: source_count.clone(),
        output_storage: output_storage.clone(),
        output_initialized_count: output_count.clone(),
        expected_ty: owned_ty,
        span,
    };
    match shape {
        TransformRangeLoopShape::BranchWrappedMarker => {
            ops.push(ResourceOp::Branch {
                output: branch_marker_output,
                condition: branch_marker_condition,
                condition_fact: None,
                then_ops: vec![
                    transform_marker,
                    ResourceOp::Expr {
                        kind: ResourceExprKind::Literal,
                        output: branch_marker_then_value.clone(),
                        ty: unit_ty,
                        span,
                    },
                ],
                then_value: branch_marker_then_value,
                else_ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::Literal,
                    output: branch_marker_else_value.clone(),
                    ty: unit_ty,
                    span,
                }],
                else_value: branch_marker_else_value,
                span,
            });
        }
        TransformRangeLoopShape::LoopWrappedMarker
        | TransformRangeLoopShape::LoopWrappedMarkerThenDuplicateMarker => {
            ops.push(ResourceOp::Loop {
                condition_ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::Literal,
                    output: loop_marker_condition.clone(),
                    ty: bool_ty,
                    span,
                }],
                condition: loop_marker_condition,
                condition_fact: None,
                body_ops: vec![transform_marker.clone()],
                span,
            });
            if matches!(
                shape,
                TransformRangeLoopShape::LoopWrappedMarkerThenDuplicateMarker
            ) {
                ops.push(transform_marker);
            }
        }
        TransformRangeLoopShape::MatchWrappedMarker => {
            ops.push(ResourceOp::Match {
                output: match_marker_output,
                scrutinee: match_marker_scrutinee,
                scrutinee_is_borrow_target: false,
                arms: vec![
                    ResourceMatchArm {
                        pattern: ResourceMatchPattern::BoolLiteral(true),
                        bind_local: None,
                        bind_source_name: None,
                        bind_mode: None,
                        ops: vec![
                            transform_marker,
                            ResourceOp::Expr {
                                kind: ResourceExprKind::Literal,
                                output: match_marker_then_value.clone(),
                                ty: unit_ty,
                                span,
                            },
                        ],
                        value: match_marker_then_value,
                        span,
                    },
                    ResourceMatchArm {
                        pattern: ResourceMatchPattern::BoolLiteral(false),
                        bind_local: None,
                        bind_source_name: None,
                        bind_mode: None,
                        ops: vec![ResourceOp::Expr {
                            kind: ResourceExprKind::Literal,
                            output: match_marker_else_value.clone(),
                            ty: unit_ty,
                            span,
                        }],
                        value: match_marker_else_value,
                        span,
                    },
                ],
                span,
            });
        }
        _ => {
            ops.push(transform_marker);
        }
    }
    if matches!(
        shape,
        TransformRangeLoopShape::ValidRollbackCleanup
            | TransformRangeLoopShape::RollbackCleanupNonZeroStart
    ) {
        let cleanup_start = match shape {
            TransformRangeLoopShape::RollbackCleanupNonZeroStart => 1,
            _ => 0,
        };
        ops.extend([
            literal_i32_op(cleanup_start, cleanup_zero.clone(), i32_ty, span),
            ResourceOp::DeclareLocal {
                place: cleanup_index.clone(),
                source_name: "cleanup_i".to_string(),
                mutable: true,
                initializer: Some(cleanup_zero),
                span,
            },
            ResourceOp::Loop {
                condition_ops: vec![ResourceOp::Expr {
                    kind: ResourceExprKind::Literal,
                    output: cleanup_condition.clone(),
                    ty: bool_ty,
                    span,
                }],
                condition: cleanup_condition,
                condition_fact: Some(ResourceConditionFact::I32Relation {
                    left: cleanup_index.clone(),
                    op: ResourceI32RelationOp::Lt,
                    right: output_count.clone(),
                }),
                body_ops: vec![
                    ResourceOp::RawMemory {
                        operation: RawMemoryOp::Load,
                        output: cleanup_loaded.clone(),
                        args: vec![cleanup_slot_address],
                        span,
                    },
                    ResourceOp::Drop {
                        place: cleanup_loaded,
                        span,
                    },
                    literal_i32_op(1, cleanup_one.clone(), i32_ty, span),
                    add_i32_op(
                        cleanup_index.clone(),
                        cleanup_one,
                        cleanup_next.clone(),
                        span,
                    ),
                    ResourceOp::Assign {
                        target: cleanup_index,
                        value: cleanup_next,
                        span,
                    },
                ],
                span,
            },
            ResourceOp::CollectionSlotDropTraversal {
                storage: output_storage,
                initialized_count: output_count,
                expected_ty: owned_ty,
                span,
            },
        ]);
    }

    collect_summary_ops_from_ops(
        &mut out,
        &mut engine,
        &mut state,
        &function.params,
        &collection_slot_summary_index,
        &ops,
    );
    (out, state, output_storage_for_assert, owned_ty)
}

fn has_output_cleanup_forall_range_summary(
    out: &[CollectionSlotLifecycleSummaryOp],
    owned_ty: TypeId,
) -> bool {
    out.iter().any(|op| {
        matches!(
            op,
            CollectionSlotLifecycleSummaryOp::DropTraversal {
                storage,
                initialized_count,
                expected_ty,
                coverage:
                    CollectionSlotLifecycleSummaryDropTraversalCoverage::ForallInitializedRange(
                        certificate
                    ),
            } if storage.parameter_index == 2
                && initialized_count.parameter_index == 3
                && *expected_ty == owned_ty
                && certificate.element_stride == 4
                && matches!(
                    certificate.drop_proof,
                    CollectionSlotInitializedRangeDropTraversalProof::LoadedValueDrop(_)
                )
        )
    })
}

fn count_transform_range_summaries(out: &[CollectionSlotLifecycleSummaryOp]) -> usize {
    out.iter()
        .map(|op| match op {
            CollectionSlotLifecycleSummaryOp::TransformRange { .. }
            | CollectionSlotLifecycleSummaryOp::TransformRangeSourceDrain { .. } => 1,
            CollectionSlotLifecycleSummaryOp::Merge { paths } => paths
                .iter()
                .map(|path| count_transform_range_summaries(path))
                .sum(),
            CollectionSlotLifecycleSummaryOp::Loop {
                condition_ops,
                body_ops,
            } => {
                count_transform_range_summaries(condition_ops)
                    + count_transform_range_summaries(body_ops)
            }
            CollectionSlotLifecycleSummaryOp::Event { .. }
            | CollectionSlotLifecycleSummaryOp::Relocate { .. }
            | CollectionSlotLifecycleSummaryOp::DropTraversal { .. } => 0,
        })
        .sum()
}

fn count_top_level_transform_range_summaries(out: &[CollectionSlotLifecycleSummaryOp]) -> usize {
    out.iter()
        .filter(|op| matches!(op, CollectionSlotLifecycleSummaryOp::TransformRange { .. }))
        .count()
}

fn has_loop_body_transform_range_summary(out: &[CollectionSlotLifecycleSummaryOp]) -> bool {
    out.iter().any(|op| match op {
        CollectionSlotLifecycleSummaryOp::Loop { body_ops, .. } => {
            count_transform_range_summaries(body_ops) > 0
        }
        CollectionSlotLifecycleSummaryOp::Merge { paths } => paths
            .iter()
            .any(|path| has_loop_body_transform_range_summary(path)),
        CollectionSlotLifecycleSummaryOp::Event { .. }
        | CollectionSlotLifecycleSummaryOp::Relocate { .. }
        | CollectionSlotLifecycleSummaryOp::DropTraversal { .. }
        | CollectionSlotLifecycleSummaryOp::TransformRange { .. }
        | CollectionSlotLifecycleSummaryOp::TransformRangeSourceDrain { .. } => false,
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

fn add_i32_op(left: Place, right: Place, output: Place, span: Span) -> ResourceOp {
    ResourceOp::Call {
        output,
        target: ResourceCallTarget::User {
            name: "add".to_string(),
            type_args: Vec::new(),
        },
        args: vec![left, right],
        effect: EffectOp::Pure,
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
    source_storage: Place,
    source_count: Place,
    output_storage: Place,
    output_count: Place,
) -> ResourceFunction {
    ResourceFunction {
        name: "transform_guarded".to_string(),
        origin_name: "transform_guarded".to_string(),
        type_params: Vec::new(),
        params: vec![
            ResourceLocal {
                name: "source_storage".to_string(),
                ty: i32_ty,
                mutable: false,
                place: source_storage,
            },
            ResourceLocal {
                name: "source_count".to_string(),
                ty: i32_ty,
                mutable: false,
                place: source_count,
            },
            ResourceLocal {
                name: "output_storage".to_string(),
                ty: i32_ty,
                mutable: false,
                place: output_storage,
            },
            ResourceLocal {
                name: "output_count".to_string(),
                ty: i32_ty,
                mutable: true,
                place: output_count,
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
