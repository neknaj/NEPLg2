# NEPLg2 self-host borrow proof

## borrow_access_uses_generic_resource_proof

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
#import "neplg2/core/resource/borrow_state" as *
#import "std/test" as *

fn check_shared_one %fn Result SelfhostBorrowState SelfhostProofRefutation Result () str \result:
    match result:
        Result::Ok state:
            match state:
                SelfhostBorrowState::Shared shared_count:
                    if eq shared_count 1 Result<(),str>::Ok () Result<(),str>::Err "expected one shared borrow"
                SelfhostBorrowState::Unborrowed:
                    Result<(),str>::Err "expected one shared borrow"
                SelfhostBorrowState::Mutable:
                    Result<(),str>::Err "expected one shared borrow"
        Result::Err _refutation:
            Result<(),str>::Err "expected borrow access proof"

fn check_unborrowed %fn Result SelfhostBorrowState SelfhostProofRefutation Result () str \result:
    match result:
        Result::Ok state:
            match state:
                SelfhostBorrowState::Unborrowed:
                    Result<(),str>::Ok ()
                SelfhostBorrowState::Shared _count:
                    Result<(),str>::Err "expected unborrowed state"
                SelfhostBorrowState::Mutable:
                    Result<(),str>::Err "expected unborrowed state"
        Result::Err _refutation:
            Result<(),str>::Err "expected borrow access proof"

fn check_mutable_while_shared %fn SelfhostBorrowAccessError Result () str \reason:
    match reason:
        SelfhostBorrowAccessError::MutableBorrowWhileShared:
            Result<(),str>::Ok ()
        SelfhostBorrowAccessError::InvalidSharedBorrowCount:
            Result<(),str>::Err "expected mutable while shared"
        SelfhostBorrowAccessError::SharedBorrowWhileMutable:
            Result<(),str>::Err "expected mutable while shared"
        SelfhostBorrowAccessError::MutableBorrowWhileMutable:
            Result<(),str>::Err "expected mutable while shared"
        SelfhostBorrowAccessError::EndSharedWithoutSharedBorrow:
            Result<(),str>::Err "expected mutable while shared"
        SelfhostBorrowAccessError::EndMutableWithoutMutableBorrow:
            Result<(),str>::Err "expected mutable while shared"

fn check_shared_while_mutable %fn SelfhostBorrowAccessError Result () str \reason:
    match reason:
        SelfhostBorrowAccessError::SharedBorrowWhileMutable:
            Result<(),str>::Ok ()
        SelfhostBorrowAccessError::InvalidSharedBorrowCount:
            Result<(),str>::Err "expected shared while mutable"
        SelfhostBorrowAccessError::MutableBorrowWhileShared:
            Result<(),str>::Err "expected shared while mutable"
        SelfhostBorrowAccessError::MutableBorrowWhileMutable:
            Result<(),str>::Err "expected shared while mutable"
        SelfhostBorrowAccessError::EndSharedWithoutSharedBorrow:
            Result<(),str>::Err "expected shared while mutable"
        SelfhostBorrowAccessError::EndMutableWithoutMutableBorrow:
            Result<(),str>::Err "expected shared while mutable"

fn check_invalid_shared_count %fn SelfhostBorrowAccessError Result () str \reason:
    match reason:
        SelfhostBorrowAccessError::InvalidSharedBorrowCount:
            Result<(),str>::Ok ()
        SelfhostBorrowAccessError::SharedBorrowWhileMutable:
            Result<(),str>::Err "expected invalid shared count"
        SelfhostBorrowAccessError::MutableBorrowWhileShared:
            Result<(),str>::Err "expected invalid shared count"
        SelfhostBorrowAccessError::MutableBorrowWhileMutable:
            Result<(),str>::Err "expected invalid shared count"
        SelfhostBorrowAccessError::EndSharedWithoutSharedBorrow:
            Result<(),str>::Err "expected invalid shared count"
        SelfhostBorrowAccessError::EndMutableWithoutMutableBorrow:
            Result<(),str>::Err "expected invalid shared count"

fn check_borrow_refutation %fn SelfhostProofRefutation fn fn SelfhostBorrowAccessError Result () str Result () str \refutation\checker:
    match refutation:
        SelfhostProofRefutation::BorrowAccessInvalid issue:
            checker issue.reason
        SelfhostProofRefutation::FactObligationMismatch _mismatch:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::UnexpectedEvidence _issue:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::SourceSpanInvalid _span:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::TypeKindMismatch _issue:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::OwnerTransitionInvalid _issue:
            Result<(),str>::Err "expected borrow access refutation"
        SelfhostProofRefutation::EffectBoundaryInvalid _issue:
            Result<(),str>::Err "expected borrow access refutation"

fn check_borrow_error %fn Result SelfhostBorrowState SelfhostProofRefutation fn fn SelfhostBorrowAccessError Result () str Result () str \result\checker:
    match result:
        Result::Err refutation:
            check_borrow_refutation refutation checker
        Result::Ok _state:
            Result<(),str>::Err "borrow conflict was accepted"

fn main %impure fn () i32 \():
    let span %SelfhostSourceSpan source_span_new 0 0 4
    let start_shared %SelfhostBorrowAccessFact selfhost_borrow_access_fact_new SelfhostBorrowRequestKind::StartShared span
    let start_mut %SelfhostBorrowAccessFact selfhost_borrow_access_fact_new SelfhostBorrowRequestKind::StartMutable span
    let end_shared %SelfhostBorrowAccessFact selfhost_borrow_access_fact_new SelfhostBorrowRequestKind::EndShared span
    let checks0 checks_new
    let checks1 checks_push checks0 check_shared_one selfhost_proof_borrow_access SelfhostBorrowState::Unborrowed start_shared
    let checks2 checks_push checks1 check_borrow_error (selfhost_proof_borrow_access (SelfhostBorrowState::Shared 1) start_mut) check_mutable_while_shared
    let checks3 checks_push checks2 check_borrow_error (selfhost_proof_borrow_access SelfhostBorrowState::Mutable start_shared) check_shared_while_mutable
    let checks4 checks_push checks3 check_unborrowed selfhost_proof_borrow_access (SelfhostBorrowState::Shared 1) end_shared
    let checks5 checks_push checks4 check_borrow_error (selfhost_proof_borrow_access (SelfhostBorrowState::Shared 0) start_shared) check_invalid_shared_count
    let shown checks_print_report checks5
    checks_exit_code shown
```
