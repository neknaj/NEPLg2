# NEPLG2 self-host owner proof

## owner_obligation_uses_generic_proof

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
    ##: [4] ok
    ##: [5] ok
    ##: [6] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/math" as *
#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/proof" as *
#import "neplg2/core/resource/owner" as *
#import "std/test" as *

fn check_owned %fn Result SelfhostOwnerState SelfhostProofRefutation fn SelfhostOwnerStorageId Result unit str \result\expected:
    match result:
        Result::Ok state:
            match state:
                SelfhostOwnerState::Owned storage:
                    if selfhost_owner_storage_id_eq storage expected Result::Ok unit Result::Err "expected same owned storage"
                SelfhostOwnerState::NoOwner:
                    Result::Err "expected owned state"
                SelfhostOwnerState::Moved _storage:
                    Result::Err "expected owned state"
                SelfhostOwnerState::Released _storage:
                    Result::Err "expected owned state"
        Result::Err _refutation:
            Result::Err "expected owner proof"

fn check_moved %fn Result SelfhostOwnerState SelfhostProofRefutation fn SelfhostOwnerStorageId Result unit str \result\expected:
    match result:
        Result::Ok state:
            match state:
                SelfhostOwnerState::Moved storage:
                    if selfhost_owner_storage_id_eq storage expected Result::Ok unit Result::Err "expected same moved storage"
                SelfhostOwnerState::NoOwner:
                    Result::Err "expected moved state"
                SelfhostOwnerState::Owned _storage:
                    Result::Err "expected moved state"
                SelfhostOwnerState::Released _storage:
                    Result::Err "expected moved state"
        Result::Err _refutation:
            Result::Err "expected owner move proof"

fn check_released %fn Result SelfhostOwnerState SelfhostProofRefutation fn SelfhostOwnerStorageId Result unit str \result\expected:
    match result:
        Result::Ok state:
            match state:
                SelfhostOwnerState::Released storage:
                    if selfhost_owner_storage_id_eq storage expected Result::Ok unit Result::Err "expected same released storage"
                SelfhostOwnerState::NoOwner:
                    Result::Err "expected released state"
                SelfhostOwnerState::Owned _storage:
                    Result::Err "expected released state"
                SelfhostOwnerState::Moved _storage:
                    Result::Err "expected released state"
        Result::Err _refutation:
            Result::Err "expected owner release proof"

fn check_invalid_storage_id %fn SelfhostOwnerTransitionError Result unit str \actual:
    match actual:
        SelfhostOwnerTransitionError::InvalidStorageId:
            Result::Ok unit
        SelfhostOwnerTransitionError::StorageIdMismatch:
            Result::Err "expected invalid storage id"
        SelfhostOwnerTransitionError::AcquireWhileOwned:
            Result::Err "expected invalid storage id"
        SelfhostOwnerTransitionError::AcquireAfterMove:
            Result::Err "expected invalid storage id"
        SelfhostOwnerTransitionError::AcquireAfterRelease:
            Result::Err "expected invalid storage id"
        SelfhostOwnerTransitionError::MoveWithoutOwner:
            Result::Err "expected invalid storage id"
        SelfhostOwnerTransitionError::MoveAfterMove:
            Result::Err "expected invalid storage id"
        SelfhostOwnerTransitionError::MoveAfterRelease:
            Result::Err "expected invalid storage id"
        SelfhostOwnerTransitionError::ReleaseWithoutOwner:
            Result::Err "expected invalid storage id"
        SelfhostOwnerTransitionError::ReleaseAfterMove:
            Result::Err "expected invalid storage id"
        SelfhostOwnerTransitionError::ReleaseAfterRelease:
            Result::Err "expected invalid storage id"
        SelfhostOwnerTransitionError::ViewWithoutOwner:
            Result::Err "expected invalid storage id"
        SelfhostOwnerTransitionError::ViewAfterMove:
            Result::Err "expected invalid storage id"
        SelfhostOwnerTransitionError::ViewAfterRelease:
            Result::Err "expected invalid storage id"

fn check_storage_id_mismatch %fn SelfhostOwnerTransitionError Result unit str \actual:
    match actual:
        SelfhostOwnerTransitionError::StorageIdMismatch:
            Result::Ok unit
        SelfhostOwnerTransitionError::InvalidStorageId:
            Result::Err "expected storage id mismatch"
        SelfhostOwnerTransitionError::AcquireWhileOwned:
            Result::Err "expected storage id mismatch"
        SelfhostOwnerTransitionError::AcquireAfterMove:
            Result::Err "expected storage id mismatch"
        SelfhostOwnerTransitionError::AcquireAfterRelease:
            Result::Err "expected storage id mismatch"
        SelfhostOwnerTransitionError::MoveWithoutOwner:
            Result::Err "expected storage id mismatch"
        SelfhostOwnerTransitionError::MoveAfterMove:
            Result::Err "expected storage id mismatch"
        SelfhostOwnerTransitionError::MoveAfterRelease:
            Result::Err "expected storage id mismatch"
        SelfhostOwnerTransitionError::ReleaseWithoutOwner:
            Result::Err "expected storage id mismatch"
        SelfhostOwnerTransitionError::ReleaseAfterMove:
            Result::Err "expected storage id mismatch"
        SelfhostOwnerTransitionError::ReleaseAfterRelease:
            Result::Err "expected storage id mismatch"
        SelfhostOwnerTransitionError::ViewWithoutOwner:
            Result::Err "expected storage id mismatch"
        SelfhostOwnerTransitionError::ViewAfterMove:
            Result::Err "expected storage id mismatch"
        SelfhostOwnerTransitionError::ViewAfterRelease:
            Result::Err "expected storage id mismatch"

fn check_release_after_release %fn SelfhostOwnerTransitionError Result unit str \actual:
    match actual:
        SelfhostOwnerTransitionError::ReleaseAfterRelease:
            Result::Ok unit
        SelfhostOwnerTransitionError::InvalidStorageId:
            Result::Err "expected release after release"
        SelfhostOwnerTransitionError::StorageIdMismatch:
            Result::Err "expected release after release"
        SelfhostOwnerTransitionError::AcquireWhileOwned:
            Result::Err "expected release after release"
        SelfhostOwnerTransitionError::AcquireAfterMove:
            Result::Err "expected release after release"
        SelfhostOwnerTransitionError::AcquireAfterRelease:
            Result::Err "expected release after release"
        SelfhostOwnerTransitionError::MoveWithoutOwner:
            Result::Err "expected release after release"
        SelfhostOwnerTransitionError::MoveAfterMove:
            Result::Err "expected release after release"
        SelfhostOwnerTransitionError::MoveAfterRelease:
            Result::Err "expected release after release"
        SelfhostOwnerTransitionError::ReleaseWithoutOwner:
            Result::Err "expected release after release"
        SelfhostOwnerTransitionError::ReleaseAfterMove:
            Result::Err "expected release after release"
        SelfhostOwnerTransitionError::ViewWithoutOwner:
            Result::Err "expected release after release"
        SelfhostOwnerTransitionError::ViewAfterMove:
            Result::Err "expected release after release"
        SelfhostOwnerTransitionError::ViewAfterRelease:
            Result::Err "expected release after release"

fn check_owner_refutation %fn SelfhostProofRefutation fn fn SelfhostOwnerTransitionError Result unit str Result unit str \refutation\checker:
    match refutation:
        SelfhostProofRefutation::OwnerTransitionInvalid issue:
            checker issue.reason
        SelfhostProofRefutation::FactObligationMismatch _mismatch:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::UnexpectedEvidence _issue:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::TypeKindMismatch _issue:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::BorrowAccessInvalid _issue:
            Result::Err "expected owner refutation"
        SelfhostProofRefutation::EffectBoundaryInvalid _issue:
            Result::Err "expected owner refutation"

fn check_owner_error %fn Result SelfhostOwnerState SelfhostProofRefutation fn fn SelfhostOwnerTransitionError Result unit str Result unit str \result\checker:
    match result:
        Result::Err refutation:
            check_owner_refutation refutation checker
        Result::Ok _state:
            Result::Err "invalid owner transition was accepted"

fn main %impure fn unit i32 \unit:
    let span %SelfhostSourceSpan source_span_new 0 0 4
    let storage %SelfhostOwnerStorageId selfhost_owner_storage_id_new 7
    let other %SelfhostOwnerStorageId selfhost_owner_storage_id_new 9
    let invalid %SelfhostOwnerStorageId selfhost_owner_storage_id_new (sub 0 1)
    let acquire %SelfhostOwnerEventFact selfhost_owner_event_fact_new SelfhostOwnerEventKind::Acquire storage span
    let move_out %SelfhostOwnerEventFact selfhost_owner_event_fact_new SelfhostOwnerEventKind::MoveOut storage span
    let release %SelfhostOwnerEventFact selfhost_owner_event_fact_new SelfhostOwnerEventKind::Release storage span
    let view %SelfhostOwnerEventFact selfhost_owner_event_fact_new SelfhostOwnerEventKind::BorrowView storage span
    let release_other %SelfhostOwnerEventFact selfhost_owner_event_fact_new SelfhostOwnerEventKind::Release other span
    let acquire_invalid %SelfhostOwnerEventFact selfhost_owner_event_fact_new SelfhostOwnerEventKind::Acquire invalid span
    let checks0 checks_new
    let checks1 checks_push checks0 check_owned (selfhost_proof_owner_transition SelfhostOwnerState::NoOwner acquire) storage
    let checks2 checks_push checks1 check_owned (selfhost_proof_owner_transition (SelfhostOwnerState::Owned storage) view) storage
    let checks3 checks_push checks2 check_moved (selfhost_proof_owner_transition (SelfhostOwnerState::Owned storage) move_out) storage
    let checks4 checks_push checks3 check_released (selfhost_proof_owner_transition (SelfhostOwnerState::Owned storage) release) storage
    let checks5 checks_push checks4 check_owner_error (selfhost_proof_owner_transition (SelfhostOwnerState::Released storage) release) check_release_after_release
    let checks6 checks_push checks5 check_owner_error (selfhost_proof_owner_transition (SelfhostOwnerState::Owned storage) release_other) check_storage_id_mismatch
    let checks7 checks_push checks6 check_owner_error (selfhost_proof_owner_transition SelfhostOwnerState::NoOwner acquire_invalid) check_invalid_storage_id
    let shown checks_print_report checks7
    checks_exit_code shown
```
