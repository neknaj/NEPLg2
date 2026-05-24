extern crate alloc;

use alloc::{string::ToString, vec, vec::Vec};

use crate::ast::Effect;
use crate::span::Span;
use crate::types::{TypeCtx, TypeId, TypeKind};

use super::collection_slot_summary_build_range_preserve::body_preserves_place;
use super::collection_slot_summary_model::CollectionSlotLifecycleFunctionSummaryIndex;
use super::initialized::ResourceCheckEngine;
use super::initialized_alias::RawCellAddressAliases;
use super::initialized_alias_flow::RawCellAddressReturnSummaryIndex;
use super::initialized_scalar_flow::I32ScalarReturnSummaryIndex;
use super::initialized_summary::RawCellInitializationFunctionSummaryIndex;
use super::model::{AggregateKind, EffectOp, Place, ResourceCallTarget, ResourceId, ResourceOp};
use super::report::ResourceCheckDeferred;

#[test]
fn body_preserve_rejects_noncopy_assign_source_consumption() {
    let (types, owned_ty) = preserve_test_types();
    let protected = Place::local("owner".to_string(), owned_ty);
    let sink = Place::temporary(ResourceId(100), owned_ty);
    let op = ResourceOp::Assign {
        target: sink,
        value: protected.clone(),
        span: Span::dummy(),
    };

    with_preserve_test_engine(&types, |engine| {
        assert!(
            !body_preserves_place(engine, &RawCellAddressAliases::default(), &[op], &protected,),
            "Assign must not preserve a protected non-Copy anchor consumed as the value"
        );
    });
}

#[test]
fn body_preserve_rejects_noncopy_construct_input_consumption() {
    let (types, owned_ty) = preserve_test_types();
    let protected = Place::local("owner".to_string(), owned_ty);
    let output = Place::temporary(ResourceId(101), owned_ty);
    let op = ResourceOp::Construct {
        output,
        kind: AggregateKind::Struct {
            name: "OwnerPair".to_string(),
            field_offsets: vec![0],
        },
        inputs: vec![protected.clone()],
        span: Span::dummy(),
    };

    with_preserve_test_engine(&types, |engine| {
        assert!(
            !body_preserves_place(engine, &RawCellAddressAliases::default(), &[op], &protected,),
            "Construct must not preserve a protected non-Copy anchor consumed as an input"
        );
    });
}

#[test]
fn body_preserve_rejects_opaque_pure_call_anchor_argument() {
    let (types, owned_ty) = preserve_test_types();
    let protected = Place::local("owner".to_string(), owned_ty);
    let output = Place::temporary(ResourceId(102), types.unit());
    let op = ResourceOp::Call {
        output,
        target: ResourceCallTarget::User {
            name: "opaque".to_string(),
            type_args: Vec::new(),
        },
        args: vec![protected.clone()],
        effect: EffectOp::UserCall {
            name: "opaque".to_string(),
            effect: Effect::Pure,
        },
        span: Span::dummy(),
    };

    with_preserve_test_engine(&types, |engine| {
        assert!(
            !body_preserves_place(engine, &RawCellAddressAliases::default(), &[op], &protected,),
            "pure user calls are not preservation proof when they receive the protected anchor"
        );
    });
}

fn preserve_test_types() -> (TypeCtx, TypeId) {
    let mut types = TypeCtx::new();
    types.set_copy_trait_enabled(true);
    types.register_copy_impl_target(types.unit());
    types.register_copy_impl_target(types.i32());
    types.register_copy_impl_target(types.bool());
    let i32_ty = types.i32();
    let owned_ty = types.register_named(
        "OwnedAnchor".to_string(),
        TypeKind::Struct {
            name: "OwnedAnchor".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["value".to_string()],
        },
    );
    (types, owned_ty)
}

fn with_preserve_test_engine<R>(
    types: &TypeCtx,
    test: impl FnOnce(&ResourceCheckEngine<'_>) -> R,
) -> R {
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
    let engine = ResourceCheckEngine {
        function: "preserve_test",
        types,
        raw_alias_summaries: &raw_alias_summary_index,
        i32_scalar_summaries: &i32_scalar_summary_index,
        raw_init_summaries: &raw_init_summary_index,
        collection_slot_summaries: &collection_slot_summary_index,
        transform_range_certificates: None,
        diagnostics: Vec::new(),
        auto_drop_points: Vec::new(),
        deferred: ResourceCheckDeferred::default(),
        path_alternatives: Default::default(),
    };
    test(&engine)
}
