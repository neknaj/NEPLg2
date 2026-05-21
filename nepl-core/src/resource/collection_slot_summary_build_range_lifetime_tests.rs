extern crate alloc;

use alloc::{string::ToString, vec};

use crate::span::Span;
use crate::types::{TypeCtx, TypeKind};

use super::collection_slot_drop_proof::CollectionSlotDropObligation;
use super::collection_slot_lifecycle::CollectionSlotLifecycleOp;
use super::collection_slot_summary_build_range_lifetime::drop_traversal_range_certificate_survives_op;
use super::collection_slot_summary_build_range_lifetime_test_support::{
    collect_loop_induction_summary_ops, has_forall_range_summary, PostLoopCertificateInterference,
};
use super::collection_slot_summary_build_state::CollectionSlotDropTraversalRangeCertificateCandidate;
use super::collection_slot_summary_model::CollectionSlotInitializedRangeDropTraversalCertificate;
use super::initialized_alias::RawCellAddressAliases;
use super::model::{Place, RawMemoryOp, ResourceId, ResourceOp};

#[test]
fn collection_slot_summary_loop_certificate_survives_unrelated_post_loop_op() {
    let out = collect_loop_induction_summary_ops(PostLoopCertificateInterference::UnrelatedLiteral);
    assert!(
        has_forall_range_summary(&out),
        "an unrelated scalar temporary after the loop must not invalidate the full-range certificate: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_certificate_survives_post_loop_anchor_read() {
    let out = collect_loop_induction_summary_ops(PostLoopCertificateInterference::AnchorRead);
    assert!(
        has_forall_range_summary(&out),
        "copying the storage anchor into a temporary before the traversal must not invalidate the full-range certificate: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_certificate_rejects_post_loop_storage_assignment() {
    let out = collect_loop_induction_summary_ops(PostLoopCertificateInterference::AssignStorage);
    assert!(
        !has_forall_range_summary(&out),
        "a storage assignment between the loop and traversal must invalidate the full-range certificate: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_certificate_rejects_post_loop_count_assignment() {
    let out =
        collect_loop_induction_summary_ops(PostLoopCertificateInterference::AssignInitializedCount);
    assert!(
        !has_forall_range_summary(&out),
        "an initialized_count assignment between the loop and traversal must invalidate the full-range certificate: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_certificate_rejects_post_loop_slot_lifecycle() {
    let out = collect_loop_induction_summary_ops(PostLoopCertificateInterference::TouchSlot);
    assert!(
        !has_forall_range_summary(&out),
        "a slot lifecycle event between the loop and traversal must invalidate the full-range certificate: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_certificate_rejects_post_loop_raw_load_storage() {
    let mut types = TypeCtx::new();
    let i32_ty = types.i32();
    let owned_ty = types.register_named(
        "OwnedPayload".to_string(),
        TypeKind::Struct {
            name: "OwnedPayload".to_string(),
            type_params: vec![],
            fields: vec![i32_ty],
            field_names: vec!["value".to_string()],
        },
    );
    types.register_drop_impl_target(owned_ty);
    let storage = Place::local("storage".to_string(), i32_ty);
    let initialized_count = Place::local("initialized_count".to_string(), i32_ty);
    let candidate = CollectionSlotDropTraversalRangeCertificateCandidate {
        storage: storage.clone(),
        initialized_count,
        expected_ty: owned_ty,
        certificate: CollectionSlotInitializedRangeDropTraversalCertificate {
            element_stride: 4,
            drop_obligation: CollectionSlotDropObligation::DropLoadedValue {
                operation: CollectionSlotLifecycleOp::DropInitialized,
                value_ty: owned_ty,
            },
        },
    };
    let op = ResourceOp::RawMemory {
        operation: RawMemoryOp::Load,
        output: Place::temporary(ResourceId(960), owned_ty),
        args: vec![storage],
        span: Span::dummy(),
    };

    assert!(
        !drop_traversal_range_certificate_survives_op(
            &types,
            &RawCellAddressAliases::default(),
            &candidate,
            &op,
        ),
        "a post-loop typed raw load from protected storage can move non-Copy slot state and must invalidate the certificate"
    );
}
