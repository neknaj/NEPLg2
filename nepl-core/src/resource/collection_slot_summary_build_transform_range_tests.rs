extern crate alloc;

use alloc::{boxed::Box, string::ToString, vec, vec::Vec};

use crate::ast::Effect;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::collection_slot_summary_build_ops::collect_summary_ops_from_ops;
use super::collection_slot_summary_build_state::CollectionSlotSummaryBuildState;
use super::collection_slot_summary_model::{
    CollectionSlotLifecycleFunctionSummaryIndex, CollectionSlotLifecycleSummaryOp,
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
    ResourceI32RelationOp, ResourceId, ResourceLocal, ResourceOffset, ResourceOp,
    ResourceTerminator,
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

#[derive(Clone, Copy)]
enum TransformRangeLoopShape {
    ValidStoreAndDiscard,
    MissingDiscardDrop,
    MissingOutputStore,
    UnknownOutputStart,
}

fn collect_transform_range_summary(
    shape: TransformRangeLoopShape,
) -> Vec<CollectionSlotLifecycleSummaryOp> {
    let (types, owned_ty) = types_with_sized_droppable_owned_for_summary();
    let i32_ty = types.i32();
    let bool_ty = types.bool();
    let unit_ty = types.unit();
    let span = Span::dummy();
    let source_storage = Place::local("source_storage".to_string(), i32_ty);
    let source_count = Place::local("source_count".to_string(), i32_ty);
    let output_storage = Place::local("output_storage".to_string(), i32_ty);
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
    ops.extend([
        ResourceOp::Loop {
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
        },
        ResourceOp::CollectionSlotTransformRange {
            source_storage,
            source_initialized_count: source_count,
            output_storage,
            output_initialized_count: output_count,
            expected_ty: owned_ty,
            span,
        },
    ]);

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
