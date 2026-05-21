use super::collection_slot_summary_build_range_certificate_test_support::{
    collect_loop_induction_summary_ops, has_forall_range_summary, LoopBodyInterference,
};

#[test]
fn collection_slot_summary_loop_induction_certifies_forall_drop_traversal() {
    let out = collect_loop_induction_summary_ops(LoopBodyInterference::None);

    assert!(
        has_forall_range_summary(&out),
        "a zero-based one-step loop with a loaded-value drop must produce a typed full-range certificate: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_induction_rejects_tail_storage_mutation() {
    let out = collect_loop_induction_summary_ops(LoopBodyInterference::AssignStorageAfterStep);

    assert!(
        !has_forall_range_summary(&out),
        "a loop that mutates storage after the induction step must not produce a full-range certificate: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_induction_rejects_move_output_storage_mutation() {
    let out = collect_loop_induction_summary_ops(LoopBodyInterference::MoveToStorageAfterStep);

    assert!(
        !has_forall_range_summary(&out),
        "a Move whose output overwrites storage must invalidate the full-range certificate: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_induction_rejects_move_output_count_mutation() {
    let out =
        collect_loop_induction_summary_ops(LoopBodyInterference::MoveToInitializedCountAfterStep);

    assert!(
        !has_forall_range_summary(&out),
        "a Move whose output overwrites initialized_count must invalidate the full-range certificate: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_induction_rejects_user_call_storage_argument() {
    let out = collect_loop_induction_summary_ops(LoopBodyInterference::UserCallStorageAfterStep);

    assert!(
        !has_forall_range_summary(&out),
        "an opaque pure user call that receives storage must not be treated as preserving the full-range certificate without a preservation proof: {out:#?}"
    );
}

#[test]
fn collection_slot_summary_loop_induction_rejects_user_call_storage_alias_argument() {
    let out =
        collect_loop_induction_summary_ops(LoopBodyInterference::UserCallStorageAliasAfterStep);

    assert!(
        !has_forall_range_summary(&out),
        "an opaque pure user call that receives a storage alias must not bypass the full-range certificate preservation check: {out:#?}"
    );
}
