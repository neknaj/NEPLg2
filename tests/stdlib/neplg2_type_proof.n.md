# NEPLg2 self-host type proof

## type_kind_compatibility_uses_generic_proof

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
```neplg2
#entry main
#target std
#indent 4

#import "core/result" as *
#import "neplg2/core/infra/span" as *
#import "neplg2/core/proof" as *
#import "neplg2/core/ty/ty" as *
#import "std/test" as *

fn check_i32_kind <(Result<SelfhostTypeKind,SelfhostProofRefutation>)->Result<(),str>> (result):
    match result:
        Result::Ok kind:
            match kind:
                SelfhostTypeKind::I32:
                    Result<(),str>::Ok ()
                SelfhostTypeKind::Error:
                    Result<(),str>::Err "expected i32 kind proof"
                SelfhostTypeKind::Unit:
                    Result<(),str>::Err "expected i32 kind proof"
                SelfhostTypeKind::Bool:
                    Result<(),str>::Err "expected i32 kind proof"
                SelfhostTypeKind::I64:
                    Result<(),str>::Err "expected i32 kind proof"
                SelfhostTypeKind::U8:
                    Result<(),str>::Err "expected i32 kind proof"
                SelfhostTypeKind::Char:
                    Result<(),str>::Err "expected i32 kind proof"
                SelfhostTypeKind::Str:
                    Result<(),str>::Err "expected i32 kind proof"
                SelfhostTypeKind::F32:
                    Result<(),str>::Err "expected i32 kind proof"
                SelfhostTypeKind::F64:
                    Result<(),str>::Err "expected i32 kind proof"
                SelfhostTypeKind::Never:
                    Result<(),str>::Err "expected i32 kind proof"
                SelfhostTypeKind::Function:
                    Result<(),str>::Err "expected i32 kind proof"
        Result::Err _refutation:
            Result<(),str>::Err "expected type kind proof"

fn check_bool_i32_mismatch <(Result<SelfhostTypeKind,SelfhostProofRefutation>)->Result<(),str>> (result):
    match result:
        Result::Err refutation:
            match refutation:
                SelfhostProofRefutation::TypeKindMismatch issue:
                    match issue.expected:
                        SelfhostTypeKind::I32:
                            match issue.actual:
                                SelfhostTypeKind::Bool:
                                    Result<(),str>::Ok ()
                                SelfhostTypeKind::Error:
                                    Result<(),str>::Err "expected bool actual kind"
                                SelfhostTypeKind::Unit:
                                    Result<(),str>::Err "expected bool actual kind"
                                SelfhostTypeKind::I32:
                                    Result<(),str>::Err "expected bool actual kind"
                                SelfhostTypeKind::I64:
                                    Result<(),str>::Err "expected bool actual kind"
                                SelfhostTypeKind::U8:
                                    Result<(),str>::Err "expected bool actual kind"
                                SelfhostTypeKind::Char:
                                    Result<(),str>::Err "expected bool actual kind"
                                SelfhostTypeKind::Str:
                                    Result<(),str>::Err "expected bool actual kind"
                                SelfhostTypeKind::F32:
                                    Result<(),str>::Err "expected bool actual kind"
                                SelfhostTypeKind::F64:
                                    Result<(),str>::Err "expected bool actual kind"
                                SelfhostTypeKind::Never:
                                    Result<(),str>::Err "expected bool actual kind"
                                SelfhostTypeKind::Function:
                                    Result<(),str>::Err "expected bool actual kind"
                        SelfhostTypeKind::Error:
                            Result<(),str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::Unit:
                            Result<(),str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::Bool:
                            Result<(),str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::I64:
                            Result<(),str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::U8:
                            Result<(),str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::Char:
                            Result<(),str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::Str:
                            Result<(),str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::F32:
                            Result<(),str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::F64:
                            Result<(),str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::Never:
                            Result<(),str>::Err "expected i32 expected kind"
                        SelfhostTypeKind::Function:
                            Result<(),str>::Err "expected i32 expected kind"
                SelfhostProofRefutation::FactObligationMismatch _mismatch:
                    Result<(),str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::SourceSpanInvalid _span:
                    Result<(),str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::RawBackendTextWithoutBlock _item:
                    Result<(),str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::RawBackendBlockEmpty _open_block:
                    Result<(),str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::ModuleDirectiveDuplicate _duplicate:
                    Result<(),str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::ModuleDeclarationHeaderMissing _issue:
                    Result<(),str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::ModuleDeclarationHeaderInvalid _issue:
                    Result<(),str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::TraitImplCoherenceInvalid _issue:
                    Result<(),str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::LifetimeOutlivesInvalid _issue:
                    Result<(),str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::ResourceCellTransitionInvalid _issue:
                    Result<(),str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::OwnerTransitionInvalid _issue:
                    Result<(),str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::BorrowAccessInvalid _issue:
                    Result<(),str>::Err "expected type kind mismatch"
                SelfhostProofRefutation::EffectBoundaryInvalid _issue:
                    Result<(),str>::Err "expected type kind mismatch"
        Result::Ok _kind:
            Result<(),str>::Err "bool kind was accepted as i32"

fn main <()*>i32> ():
    let span <SelfhostSourceSpan> source_span_new 0 0 3
    let i32_fact <SelfhostTypeKindFact> selfhost_type_kind_fact_new SelfhostTypeKind::I32 span
    let bool_fact <SelfhostTypeKindFact> selfhost_type_kind_fact_new SelfhostTypeKind::Bool span
    let checks0 checks_new
    let checks1 checks_push checks0 check_i32_kind selfhost_proof_type_kind_compatible SelfhostTypeKind::I32 i32_fact
    let checks2 checks_push checks1 check_bool_i32_mismatch selfhost_proof_type_kind_compatible SelfhostTypeKind::I32 bool_fact
    let shown checks_print_report checks2
    checks_exit_code shown
```
