use super::collection_slot_summary_build_range_lifetime_test_support::{
    collect_loop_induction_summary_ops, has_forall_range_summary, PostLoopCertificateInterference,
};

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
