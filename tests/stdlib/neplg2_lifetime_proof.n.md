# NEPLg2 self-host lifetime proof

## lifetime_outlives_uses_generic_proof

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "core/math" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/proof" as *
#import "neplg2/core/resource/lifetime" as *
#import "std/test" as *

fn check_relation_proven %fn Result SelfhostLifetimeRelation SelfhostProofRefutation fn SelfhostLifetimeRelation Result unit str \result\expected:
    match result:
        Result::Ok relation:
            if selfhost_lifetime_relation_proves_outlives relation Result::Ok unit Result::Err "expected outlives relation"
        Result::Err _refutation:
            Result::Err "expected lifetime proof"

fn check_required_mismatch %fn SelfhostLifetimeOutlivesError Result unit str \reason:
    match reason:
        SelfhostLifetimeOutlivesError::RequiredLifetimeMismatch:
            Result::Ok unit
        SelfhostLifetimeOutlivesError::InvalidSubjectLifetime:
            Result::Err "expected required lifetime mismatch"
        SelfhostLifetimeOutlivesError::InvalidRequiredLifetime:
            Result::Err "expected required lifetime mismatch"
        SelfhostLifetimeOutlivesError::SubjectDoesNotOutliveRequired:
            Result::Err "expected required lifetime mismatch"
        SelfhostLifetimeOutlivesError::UnrelatedLifetimes:
            Result::Err "expected required lifetime mismatch"

fn check_subject_shorter %fn SelfhostLifetimeOutlivesError Result unit str \reason:
    match reason:
        SelfhostLifetimeOutlivesError::SubjectDoesNotOutliveRequired:
            Result::Ok unit
        SelfhostLifetimeOutlivesError::RequiredLifetimeMismatch:
            Result::Err "expected shorter subject lifetime"
        SelfhostLifetimeOutlivesError::InvalidSubjectLifetime:
            Result::Err "expected shorter subject lifetime"
        SelfhostLifetimeOutlivesError::InvalidRequiredLifetime:
            Result::Err "expected shorter subject lifetime"
        SelfhostLifetimeOutlivesError::UnrelatedLifetimes:
            Result::Err "expected shorter subject lifetime"

fn check_invalid_subject %fn SelfhostLifetimeOutlivesError Result unit str \reason:
    match reason:
        SelfhostLifetimeOutlivesError::InvalidSubjectLifetime:
            Result::Ok unit
        SelfhostLifetimeOutlivesError::RequiredLifetimeMismatch:
            Result::Err "expected invalid subject lifetime"
        SelfhostLifetimeOutlivesError::InvalidRequiredLifetime:
            Result::Err "expected invalid subject lifetime"
        SelfhostLifetimeOutlivesError::SubjectDoesNotOutliveRequired:
            Result::Err "expected invalid subject lifetime"
        SelfhostLifetimeOutlivesError::UnrelatedLifetimes:
            Result::Err "expected invalid subject lifetime"

fn check_lifetime_refutation %fn SelfhostProofRefutation fn fn SelfhostLifetimeOutlivesError Result unit str Result unit str \refutation\checker:
    match refutation:
        SelfhostProofRefutation::LifetimeOutlivesInvalid issue:
            checker issue.reason
        SelfhostProofRefutation::FactObligationMismatch _mismatch:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::UnexpectedEvidence _issue:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::TypeKindMismatch _issue:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::OwnerTransitionInvalid _issue:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::BorrowAccessInvalid _issue:
            Result::Err "expected lifetime outlives refutation"
        SelfhostProofRefutation::EffectBoundaryInvalid _issue:
            Result::Err "expected lifetime outlives refutation"

fn check_lifetime_error %fn Result SelfhostLifetimeRelation SelfhostProofRefutation fn fn SelfhostLifetimeOutlivesError Result unit str Result unit str \result\checker:
    match result:
        Result::Err refutation:
            check_lifetime_refutation refutation checker
        Result::Ok _relation:
            Result::Err "invalid lifetime relation was accepted"

fn main %impure fn unit i32 \unit:
    let span %SelfhostSourceSpan source_span_new 0 0 4
    let root_id %SelfhostLifetimeId selfhost_lifetime_id_new 0
    let child_id %SelfhostLifetimeId selfhost_lifetime_id_new 1
    let sibling_id %SelfhostLifetimeId selfhost_lifetime_id_new 2
    let invalid_id %SelfhostLifetimeId selfhost_lifetime_id_new (sub 0 1)
    let root %SelfhostLifetimePosition selfhost_lifetime_position_new root_id 0
    let child %SelfhostLifetimePosition selfhost_lifetime_position_new child_id 2
    let invalid %SelfhostLifetimePosition selfhost_lifetime_position_new invalid_id 0
    let root_outlives_child %SelfhostLifetimeRelation selfhost_lifetime_relation_from_scope_path root child SelfhostLifetimeScopePathKind::SubjectAncestorOfRequired
    let same_lifetime %SelfhostLifetimeRelation selfhost_lifetime_relation_from_scope_path root root SelfhostLifetimeScopePathKind::SameNode
    let child_outlives_root %SelfhostLifetimeRelation selfhost_lifetime_relation_from_scope_path child root SelfhostLifetimeScopePathKind::RequiredAncestorOfSubject
    let invalid_subject %SelfhostLifetimeRelation selfhost_lifetime_relation_from_scope_path invalid root SelfhostLifetimeScopePathKind::SubjectAncestorOfRequired
    let fact_ok %SelfhostLifetimeOutlivesFact selfhost_lifetime_outlives_fact_new root_id child_id root_outlives_child SelfhostLifetimeUseKind::ReturnBorrow span
    let fact_same %SelfhostLifetimeOutlivesFact selfhost_lifetime_outlives_fact_new root_id root_id same_lifetime SelfhostLifetimeUseKind::ReferenceAssignment span
    let fact_short %SelfhostLifetimeOutlivesFact selfhost_lifetime_outlives_fact_new child_id root_id child_outlives_root SelfhostLifetimeUseKind::StoreInLongerStorage span
    let fact_invalid %SelfhostLifetimeOutlivesFact selfhost_lifetime_outlives_fact_new invalid_id root_id invalid_subject SelfhostLifetimeUseKind::ClosureCapture span
    let checks0 checks_new
    let checks1 checks_push checks0 check_relation_proven (selfhost_proof_lifetime_outlives child_id fact_ok) root_outlives_child
    let checks2 checks_push checks1 check_relation_proven (selfhost_proof_lifetime_outlives root_id fact_same) same_lifetime
    let checks3 checks_push checks2 check_lifetime_error (selfhost_proof_lifetime_outlives root_id fact_short) check_subject_shorter
    let checks4 checks_push checks3 check_lifetime_error (selfhost_proof_lifetime_outlives root_id fact_invalid) check_invalid_subject
    let checks5 checks_push checks4 check_lifetime_error (selfhost_proof_lifetime_outlives sibling_id fact_ok) check_required_mismatch
    let shown checks_print_report checks5
    checks_exit_code shown
```
